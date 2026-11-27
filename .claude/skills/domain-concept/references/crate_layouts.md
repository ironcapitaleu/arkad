---
source: the source tree of each crate on `main`
last-verified: 2026-11-27
update-frequency: when a crate is added, or when a crate changes where it puts its shared concepts
---

# Crate Layouts: Examples

These rows show the steps of "Where a Concept Lives" applied to the crates of the workspace on the
date above. They are examples, not a full list. A crate can be missing, and a row can be out of
date. If a crate is missing or differs from its row, read the crate and follow the steps. The steps
decide, not this table.

| Crate | Value type | Trait and its implementations | Fakes |
| ----- | ---------- | ----------------------------- | ----- |
| `sec` | `src/lib/shared/{concept}/` | `src/lib/shared/{concept}/traits/` and `src/lib/shared/{concept}/implementations/`, per concept, as in `http_client`, `rate_limiter`, and `row_digest` | `src/lib/tests/fixtures/sample_{concept}/` |
| `storage` | `src/shared/{concept}/` | `src/traits/{concept}/` and `src/implementations/{concept}/`, at the crate root, as in `repository` | `src/tests/fixtures/fake_{type}/` |
| `api` | `src/shared/{concept}/` | `src/traits/{trait}/` and `src/implementations/{type}/`, at the crate root, one folder per concrete type, as in `token_verifier` and `supabase_token_verifier` | `src/tests/fixtures/fake_{type}/` |
| `xbrl` | `src/core/{concept}.rs`, or `src/core/{concept}/` with a `mod.rs` | In the module of the concept, as `Taxonomy` in `src/core/taxonomy/mod.rs`. `xbrl` has no `traits/` folder | `src/tests/fixtures/sample_{concept}/` |

## Examples by Part

| Part | Example |
| ---- | ------- |
| A trait in `traits/`, one file per trait | `sec/src/lib/shared/rate_limiter/traits/rate_limiter.rs` |
| A trait that wraps no external crate | `sec/src/lib/shared/row_digest/traits/digest_row.rs` |
| A concrete type in `implementations/` | `sec/src/lib/shared/rate_limiter/implementations/sec_rate_limiter/` |
| Constants next to their type | `sec/src/lib/shared/row_digest/implementations/byte_form/constants.rs`, `sec/src/lib/shared/cik/constants.rs` |
| The error of a value type | `sec/src/lib/shared/cik/cik_error.rs` |
| The error of a trait | `api/src/traits/token_verifier/token_error/` |
| The error of an implementation folder | `sec/src/lib/shared/http_client/implementations/sec_client/error.rs` |
| The error of a function at the root of the concept | `sec/src/lib/shared/row_digest/duplicate_digest_group/` |
