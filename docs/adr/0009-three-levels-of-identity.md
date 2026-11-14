# ADR 0009: Identity Has Three Levels: Company, Security, and Listing

| Field | Value |
| ----- | ----- |
| Status | Proposed. The project lead confirms. |
| Date | 2026-11-14 |
| Deciders | The project lead |
| Ticket | [STA-501](https://linear.app/state-machine/issue/STA-501) |

## Context

[ADR 0001](0001-mvp-database.md), "Design Rules" 1, makes identifiers rows with `company_id`,
scheme, value, valid from, valid to, and source. [ADR 0007](0007-gold-company-brain.md) puts them
in the metadata store of gold. `hybrid_data_model.md` §5.2 models a listing as a claim from a
company to an exchange, with the ticker as its payload.

Some identifiers do not name a company:

- An ISIN or a FIGI names a security, such as a share class or a bond. Apple has 949 ISINs, mostly
  debt (STA-501, "Description", from the fog log of the epic MVP Data Pt. 2, 2026-10-31).
- A ticker names a security on one exchange. Alphabet has two share classes, `GOOGL` and `GOOG`.
- A price belongs to a security on one exchange. The fog log of MVP Data Pt. 2 keys
  `prices_eod` on a FIGI, or on the ticker and the exchange, not on `company_id`.

The choice binds the gold tables, the resolver ports, the ticker search of `/v1`, and the price
store. A later change moves stored keys.

Terms:

- **Security**: one instrument that a company issues, such as a share class or a bond.
- **Listing**: one security that trades on one exchange.

## Decision

1. **Identity has three levels: company, security, and listing.** Each level has its own internal
   key: `company_id`, `security_id`, and `listing_id`. ADR 0001, "Design Rules" 4, applies to each
   key: the load assigns it once and never reuses or changes it.
2. **Each scheme belongs to one level.** `cik` and `lei` name a company. `figi_share_class` names a
   security. `us_ticker` and `figi` name a listing.
3. **The identifier rows of each level follow ADR 0001, "Design Rules" 1,** with the key of the
   level in place of `company_id`.
4. **A listing names one security and one exchange for its whole life.** The edge from a security
   to its company is a claim with a source and a validity.
5. **A classification code is a claim on a company, not an identifier.** Many companies share one
   code.
6. **Grouping uses exact keys at every level** (ADR 0001, "Design Rules" 3). A ticker change with
   no exact identifier that links the two tickers gives a new listing.

`sec/design/data_model/identifier_model.md` describes the rows, the links, and the reads.

## Options Considered

No option was measured.

| Option | GOOG and GOOGL | A bond with no listing | One security on two exchanges | Key of a price |
| ------ | -------------- | ---------------------- | ----------------------------- | -------------- |
| **A (chosen):** three levels | Two securities, two listings | A security with no listing | One security, two listings | `listing_id` |
| B: company only, tickers as company identifiers | Two identifiers of one company | No place | Two identifiers of one company | None. A price needs a ticker and an exchange outside the model |
| C: company and listing | Two listings | No place | Two listings with nothing to tie them | `listing_id` |

- A has a place for each identifier that the ticket names, at the level it names.
- B fits the SEC data alone, where a filing names a company and its tickers. It has no place for a
  FIGI or an ISIN, and no key for a price.
- C has no place for a bond, and no shared key for a security on two exchanges.

## Consequences

- The metadata store gets the nodes `Security` and `Listing`, and the edges `ISSUED_BY`,
  `OF_SECURITY`, and `ON_EXCHANGE`. `hybrid_data_model.md` §5.1 and §5.2 hold them.
- `EntityResolver` resolves company schemes only. Securities and listings get resolver ports of
  their own.
- The SEC filings give no key for a security: the share class of a ticker is not in silver. The
  security level stays empty until a FIGI source loads.
- With SEC data only, a ticker change gives a new listing. A `figi` claim joins the two listings
  later, with a dated `same_as` event.
- Prices and a ticker search key on `listing_id`.

## Open Questions

- The gold tables of the three levels. The gold design fixes them (ADR 0007, "Open Questions").
- The claim resolution spec of ADR 0001, "Open Questions", covers the claims of all three levels.
- Whether silver keeps the share class of each ticker, which the instance names as a member of
  `us-gaap:StatementClassOfStockAxis`. It needs a new `schema_version` of `filings`. The owner
  decides.

## Sources

- [STA-501](https://linear.app/state-machine/issue/STA-501), "Description" and "Definition of
  Done", read on 2026-11-14.
- The fog log of the Linear project MVP Data Pt. 2: "Non-filing data in silver" (`prices_eod`) and
  "Identifier licences", read on 2026-11-14.
- [ADR 0001](0001-mvp-database.md), "Design Rules", and [ADR 0007](0007-gold-company-brain.md),
  Decisions 3 and 6, on `main` at `13a10f7`.
- `xbrl/src/core/filing.rs`, `Ticker`: a symbol and an exchange only, on `main` at `13a10f7`.
