# ADR 0044: One Router State With a `FromRef` Impl per Port

| Field | Value |
| ----- | ----- |
| Status | Proposed. The project lead confirms |
| Date | 2026-11-24 |
| Deciders | The project lead |
| Ticket | [STA-314](https://linear.app/state-machine/issue/STA-314) |

## Context

The `AuthenticatedAccount` extractor of the `api` crate (STA-314) calls a `TokenVerifier`. It gets
the verifier from the axum router state. The shape of that state binds later tickets:

- STA-581 adds an access middleware. Its ticket names more ports and caches: `ApiKeyRepository`,
  `AccountPlanRepository`, a `moka` cache of keys, and a `moka` cache of plans, next to
  `TokenVerifier`.
- STA-582 and STA-583 add the key endpoints. Their handlers take `AuthenticatedAccount`.
- `api::router` is public. On `2026b17`, 9 call sites pass it a state: `main` and 8 calls in
  `api/tests/`.

axum gives a handler shared values in two ways:

- **State.** `Router::with_state` holds one value of type `S`. An extractor asks for a part of it
  through `FromRef<S>`. If no `FromRef` impl exists, the code does not compile.
- **Extensions.** A layer inserts a value into each request. If the layer is missing, the handler
  fails at run time with a `500`.

## Decision

1. **The router holds one state struct, `AppState`.** `api::router(internal_docs, state)` takes it,
   and every route shares it.
2. **Each port in `AppState` gets its own `FromRef<AppState>` impl.** The first one gives
   `Arc<dyn TokenVerifier>`.
3. **An extractor asks for the port, not for `AppState`.** `AuthenticatedAccount` is generic over
   every `S` with `Arc<dyn TokenVerifier>: FromRef<S>`. A test can then pass a state of its own.

## Options Considered

Read on 2026-11-24 in the code of `api` at `2026b17` and in the axum 0.8.9 docs.

| Option | A missing port | Adding a port changes | Test with one port |
| ------ | -------------- | --------------------- | ------------------ |
| **A: `AppState` with `FromRef` per port (chosen)** | Does not compile | `AppState`, its constructor, and one `FromRef` impl | Pass a state that gives only that port |
| B: `Arc<dyn TokenVerifier>` as the whole state | Does not compile | The state type, the signature of `api::router`, every extractor bound, and the 9 call sites | Pass the port as the state |
| C: one `Extension` layer per port | `500` at run time | One `.layer(Extension(…))` in the router | Add the layer in the test |

- A keeps the check at compile time, and adding a port leaves the extractors unchanged.
- B is the smallest code with one port. STA-581 brings a second port, so B changes every caller
  then.
- C finds a missing port only at run time, as a `500` on the first request that needs it.

## Consequences

- STA-581 adds its ports as fields of `AppState`, each with a `FromRef` impl.
- `AppState::new` takes each port. Each new port changes `AppState::new`, so its callers change
  too: `main`, and the fixture `fake_app_state` that the integration tests use.
- `AppState` is `Clone`. Each port sits behind an `Arc`, so a clone copies pointers only.
- The decision does not cover where the access middleware of STA-581 keeps the
  `AuthenticatedAccount` it builds. That ticket decides it.

## Open Questions

- If `AppState::new` grows past a few ports, a builder can replace it. A later ticket with more
  ports decides this.

## Sources

- `api/src/routes/mod.rs` and `api/src/middleware/authentication/authenticated_account/mod.rs` at
  `2026b17`.
- `grep -rn "api::router(" api/src api/tests | wc -l` at `2026b17`, run on 2026-11-24: 9.
- [STA-581](https://linear.app/state-machine/issue/STA-581), description, read on 2026-11-24.
- axum 0.8.9, "Sharing state with handlers":
  <https://docs.rs/axum/0.8.9/axum/index.html#sharing-state-with-handlers>.
