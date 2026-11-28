# ADR 0055: Byte Form of `input_digest`

| Field | Value |
| ----- | ----- |
| Status | Proposed. The project owner confirms |
| Date | 2026-11-28 |
| Deciders | The project owner |
| Ticket | [STA-616](https://linear.app/state-machine/issue/STA-616) (accessed 2026-11-28) |

## Context

Each row of `unit_manifests` holds an `input_digest`: the SHA-256 over the sorted list of the input
keys and their SHA-256 values. Rule 2 of a write compares the digest of a new unit with the digest
of the current unit of its key. If both digests and the `transform_version` are equal, the writer
skips the unit.

The contract names the inputs of the digest, but not its bytes. The writer of each dataset, and
each later run, must compute the same bytes. Otherwise rule 2 never matches a stored unit.

Terms in this ADR:

- **Input:** one raw document or dataset file that a unit reads: its key and its SHA-256.
- **Byte form:** the bytes of `ByteForm` that ADR 0033 defines for `content_digest`. Each value
  starts with a marker byte. A text value then holds a 4-byte big-endian length and its UTF-8
  text. A list or a struct holds a 4-byte big-endian count, and its items follow.

## Decision

1. **`input_digest` is the SHA-256 over the byte form of one list of structs.** The list holds one
   struct per input. Each struct holds two text values: the key, then the lowercase hex text of
   the SHA-256.
2. **The inputs sort by key, then by the hex text of the SHA-256**, by the bytes of the text. So
   the order in which a caller hands over the inputs does not change the digest.
3. **A unit with no input** hashes the byte form of an empty list: the bytes `01 00 00 00 00`.

## Options Considered

Checked on 2026-11-28 against `main` at commit `f3aa37c`.

| Option | Boundaries between values | New code | Same form as `content_digest` |
| ------ | ------------------------- | -------- | ----------------------------- |
| **A (chosen). `ByteForm` of a list of structs** | Each value has a length | None, `ByteForm` exists | Yes |
| B. Text lines `<key> <sha256>`, joined by newlines | A key with a space or a newline is ambiguous | A printer | No |
| C. The concatenated bytes of the keys and the raw digests | No boundary: two key lists can give the same bytes | A printer | No |

- A has a length prefix on each value, so no two input lists give the same bytes.
- B depends on a key never holding a space or a newline. `DocumentKey` does not forbid either.
- C can give the same bytes for `ab` + `c` and `a` + `bc`.

## Consequences

- A change of the form makes rule 2 miss for every stored unit. The next run then writes every
  unit again, once. A change needs a new ADR that supersedes this one.
- The form changes when `ByteForm` changes. ADR 0033 already makes such a change a new
  `transform_version`, which writes every unit again anyway.
- The digest does not depend on the order of the inputs. Two inputs with the same key and the same
  SHA-256 enter twice. The writer does not remove a duplicate input.

## Open Questions

- None. The decision reopens if an input gains a field that rule 2 must compare, for example the
  URL of the request.

## Sources

- ADR 0033, the byte form of `content_digest`.
- `sec/src/lib/shared/row_digest/implementations/byte_form/mod.rs` at commit `f3aa37c`: the length
  prefixes of a text value, a list, and a struct.
- `printf '\x01\x00\x00\x00\x00' | sha256sum`, run on 2026-11-28: `957b88b1…11c8027`, the digest of
  a unit with no input.
