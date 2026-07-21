"""
design_patterns_demo.py — a runnable, dependency-free illustration of the design
patterns behind arkad's storage layer.

Background companion to `design_patterns_primer.md` and the normative design in
`storage_traits_design.md` (STA-130). It mirrors the Rust design in Python so the
patterns are easy to read at a glance:

  Ports & Adapters      abstract base classes (Storage, RawStore, ...) are *ports*;
                        concrete classes are *adapters* (an in-memory fake + a
                        "Postgres-like" stub).
  Repository            the composing facade the pipeline depends on.
  Unit of Work          Repository.ingest() persists a bundle atomically
                        (all tiers commit together, or none do).
  CQRS                  writes go through ingest(); reads go through query methods,
                        over a source-of-truth (raw) + projections (graph/facts) model.
  Dependency Injection  the pipeline is handed a Repository; it never names a backend.
  Data Mapper           domain types below carry no SQL; adapters do the mapping.

Run:  python3 design_patterns_demo.py
"""

from __future__ import annotations

from abc import ABC, abstractmethod
from dataclasses import dataclass
from typing import Iterator

# ---------------------------------------------------------------------------
# Domain types — persistence-ignorant (no SQL here). cf. Data Mapper.
# In Rust: `RawFiling`, `FactSet`, `GraphDelta`, `IngestionUnit`, `CanonicalFact`.
# ---------------------------------------------------------------------------


@dataclass(frozen=True)
class RawFiling:
    accession: str
    company: str
    payload: str  # verbatim filing text — the durable system of record


@dataclass(frozen=True)
class CanonicalFact:
    company: str
    element: str
    period: str
    value: int


@dataclass(frozen=True)
class GraphDelta:
    company: str
    filing: str  # e.g. a HAS_FILING edge to upsert


@dataclass(frozen=True)
class FactSet:
    company: str
    facts: tuple[CanonicalFact, ...]


@dataclass(frozen=True)
class IngestionUnit:
    """Everything derived from ONE filing, ready to persist as a unit.

    The pipeline builds this; it never knows where or how it lands.
    """

    raw: RawFiling
    graph: GraphDelta
    facts: FactSet


# ---------------------------------------------------------------------------
# Error currency — classify but keep the original (cf. `StorageError`, with
# `is_retryable()` and `From<Self::Error>` so backends convert into one type).
# ---------------------------------------------------------------------------


class StorageError(Exception):
    retryable = False


class Unavailable(StorageError):  # transient -> retry
    retryable = True


class Conflict(StorageError):  # already exists -> do not retry
    pass


# ---------------------------------------------------------------------------
# Ports (Rust: traits). The pipeline depends only on these abstractions.
# ---------------------------------------------------------------------------


class Storage(ABC):
    """Base port: identity/lifecycle, universal to any store."""

    @abstractmethod
    def backend(self) -> str: ...

    def migrate(self) -> None:  # default no-op; real adapters override
        return None

    def health(self) -> None:
        return None


class RawStore(Storage):
    @abstractmethod
    def append(self, raw: RawFiling) -> None: ...

    @abstractmethod
    def scan(self) -> Iterator[RawFiling]: ...  # replay source for backfill/rebuild


class GraphStore(Storage):
    @abstractmethod
    def upsert(self, delta: GraphDelta) -> None: ...


class FactStore(Storage):
    @abstractmethod
    def upsert(self, facts: FactSet) -> None: ...

    @abstractmethod
    def query(self, company: str) -> list[CanonicalFact]: ...  # CQRS read side


class Repository(ABC):
    """Composing facade: HAS-A a store per tier; owns the atomic write.

    In Rust this is `Repository` with `type Raw: RawStore`, `type Graph: GraphStore`,
    `type Facts: FactStore` + `ingest`.
    """

    raw: RawStore
    graph: GraphStore
    facts: FactStore

    @abstractmethod
    def ingest(self, unit: IngestionUnit) -> None: ...


# ---------------------------------------------------------------------------
# Adapters — in-memory fakes (test doubles) implementing the same ports.
# ---------------------------------------------------------------------------


class FakeRawStore(RawStore):
    def __init__(self) -> None:
        self._items: list[RawFiling] = []

    def backend(self) -> str:
        return "fake"

    def append(self, raw: RawFiling) -> None:
        self._items.append(raw)

    def scan(self) -> Iterator[RawFiling]:
        return iter(list(self._items))


class FakeGraphStore(GraphStore):
    def __init__(self) -> None:
        self._deltas: list[GraphDelta] = []

    def backend(self) -> str:
        return "fake"

    def upsert(self, delta: GraphDelta) -> None:
        self._deltas.append(delta)


class FakeFactStore(FactStore):
    def __init__(self) -> None:
        self._by_company: dict[str, list[CanonicalFact]] = {}

    def backend(self) -> str:
        return "fake"

    def upsert(self, facts: FactSet) -> None:
        self._by_company.setdefault(facts.company, []).extend(facts.facts)

    def query(self, company: str) -> list[CanonicalFact]:
        return list(self._by_company.get(company, []))


class FakeRepository(Repository):
    """Records what the pipeline handed us.

    Tests assert on `ingested` — not on a query round-trip — which keeps them
    honest to the *weak* ingest contract (raw is durable; graph/fact
    materializations may lag on a mixed-engine backend).
    """

    def __init__(self) -> None:
        self.raw = FakeRawStore()
        self.graph = FakeGraphStore()
        self.facts = FakeFactStore()
        self.ingested: list[IngestionUnit] = []

    def ingest(self, unit: IngestionUnit) -> None:
        self.raw.append(unit.raw)
        self.graph.upsert(unit.graph)
        self.facts.upsert(unit.facts)
        self.ingested.append(unit)


# ---------------------------------------------------------------------------
# Adapter — a "Postgres-like" repository whose tiers share one connection, so
# ingest() is ONE transaction (the strong contract). Swapping it in for the
# fake changes nothing for the pipeline: that is the Dependency-Injection payoff.
# ---------------------------------------------------------------------------


class PgLikeRepository(Repository):
    def __init__(self) -> None:
        self.raw = FakeRawStore()
        self.graph = FakeGraphStore()
        self.facts = FakeFactStore()

    def ingest(self, unit: IngestionUnit) -> None:
        # Unit of Work: begin -> write all tiers -> commit; roll back on failure.
        # (Atomicity lives INSIDE the adapter — never as a begin()/commit() seam
        # on the ports, which would leak transactions into every fake.)
        try:
            # self._begin()
            self.raw.append(unit.raw)
            self.graph.upsert(unit.graph)
            self.facts.upsert(unit.facts)
            # self._commit()
        except StorageError:
            # self._rollback()
            raise


# ---------------------------------------------------------------------------
# Service layer / pipeline — depends only on the Repository port (DI).
# ---------------------------------------------------------------------------


def load(unit: IngestionUnit, sink: Repository) -> None:
    """The 'Load' step of the ETL pipeline. Knows nothing about the backend."""
    sink.ingest(unit)


def sample_unit(company: str = "ACME", accession: str = "0001") -> IngestionUnit:
    facts = FactSet(company, (CanonicalFact(company, "Revenue", "FY2024", 1000),))
    return IngestionUnit(
        RawFiling(accession, company, "<xbrl/>"),
        GraphDelta(company, accession),
        facts,
    )


if __name__ == "__main__":
    # 1) Same pipeline, two different backends — the whole point of the ports.
    for make in (FakeRepository, PgLikeRepository):
        sink = make()
        load(sample_unit(), sink)
        got = sink.facts.query("ACME")  # CQRS read side
        assert got == [CanonicalFact("ACME", "Revenue", "FY2024", 1000)], got
        print(f"{make.__name__:18} -> query('ACME') = {got}")

    # 2) Test-style assertion on the RECORDED unit (weak-contract-honest).
    fake = FakeRepository()
    load(sample_unit(), fake)
    assert len(fake.ingested) == 1
    assert fake.ingested[0].raw.accession == "0001"

    # 3) Replay/backfill reads the raw system-of-record (the bulk path).
    fake.raw.append(RawFiling("0002", "ACME", "<xbrl/>"))
    print("replay scan (raw SoT):", [f.accession for f in fake.raw.scan()])

    # 4) Error currency classifies retry-vs-dead-letter.
    print("retryable? Unavailable:", Unavailable().retryable, "| Conflict:", Conflict().retryable)

    print("OK")
