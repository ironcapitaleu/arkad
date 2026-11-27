# ADR 0045: The Access Enums Store as Lowercase Snake Case Names

| Field | Value |
| ----- | ----- |
| Status | Accepted by the project lead on 2026-11-26 ([STA-570 comment `8950aba5`](https://linear.app/state-machine/issue/STA-570#comment-8950aba5)) |
| Date | 2026-11-26 |
| Deciders | The project lead |
| Ticket | [STA-570](https://linear.app/state-machine/issue/STA-570) |

## Context

STA-570 adds the value types of the access ports to `storage::shared`. Its Definition of Done asks
that each type has a `Display` form that the Postgres adapters can store, and a parse back from it.

Three of the types are enums whose value Postgres stores as text:

| Type | Variants | Postgres column |
| ---- | -------- | --------------- |
| `KeyEnvironment` | `Live`, `Test` | `api_keys.environment` |
| `Channel` | `ApiKey`, `Login` | `api_usage.channel` |
| `AccountPlan` | `Free`, `ScreenerPro`, `ApiDeveloper`, `ApiBusiness` | the plan of an account |

A fourth enum, `Window` (`Day`, `Minute`), is not stored. Its name labels a window in the
`RateLimit` headers.

The planned tables check two of the columns. On 2026-11-26, the Linear document "Access Control for
`/v1`: Tokens, API Keys, and Quotas (Draft)" held these lines:

```sql
environment text NOT NULL CHECK (environment IN ('live', 'test')),
channel     text NOT NULL CHECK (channel IN ('api_key', 'login')),
```

The document fixes no text for a plan. The plan comes from the subscription data that the Paddle
webhooks write, in a later project.

The choice binds STA-575 (the migrations), STA-579 and STA-580 (the Postgres adapters), and the
later plan table. A change of a stored name after the first row needs a data migration.

## Decision

1. **Each access enum writes its variant as a lowercase snake case name.** `Display` and
   `as_str` write these names, and `FromStr` reads them:

   | Type | Names |
   | ---- | ----- |
   | `KeyEnvironment` | `live`, `test` |
   | `Channel` | `api_key`, `login` |
   | `AccountPlan` | `free`, `screener_pro`, `api_developer`, `api_business` |
   | `Window` | `day`, `minute` |

2. **`FromStr` accepts the exact name only.** It rejects every other case and every alias, such
   as `Live` or `Screener Pro`. A column then holds one spelling per value, and a `CHECK`
   constraint lists every allowed value.

## Options Considered

No option was measured. The facts follow from the SQL above and from the code of STA-570.

| Option | Matches the planned `CHECK` constraints | Changes when a plan gets a new display name | Readable in a SQL query |
| ------ | --------------------------------------- | ------------------------------------------- | ----------------------- |
| **A (chosen):** snake case names | Yes | No | Yes |
| B: display names, such as `Screener Pro` | No. `api_key` has no display name | Yes | Yes |
| C: integer codes, such as `1` | No | No | No. A reader needs the Rust code |

- A uses the form that the planned `CHECK` constraints already hold, so one rule covers all enums.
- B ties stored data to a product name. A rename of a plan then needs a data migration.
- C hides the meaning of a row, and a reorder of the variants changes the meaning of every row.

## Consequences

- The migrations of STA-575 use these names in their `CHECK` constraints or Postgres enum types.
- The adapters of STA-579 and STA-580 bind `as_str()` and read with `str::parse`. Neither keeps a
  second mapping.
- A new variant, such as a custom plan, adds one name, a match arm in `as_str` and `FromStr`, and a
  migration of the `CHECK` constraint.
- The decision does not cover the text form of `ApiKeyId` or `UsageKey`. Postgres stores an
  `ApiKeyId` as `uuid`, and a `UsageKey` as three columns.

## Open Questions

- Whether the plan column uses a `CHECK` constraint on `text` or a Postgres enum type. STA-565
  designs the Postgres backend and answers it. Both hold the names of this decision.

## Sources

- [STA-570](https://linear.app/state-machine/issue/STA-570), Definition of Done, read on
  2026-11-26.
- Linear document "Access Control for `/v1`: Tokens, API Keys, and Quotas (Draft)", the
  `CREATE TABLE` statements of `access.api_keys` and `access.api_usage`, read on 2026-11-26.
