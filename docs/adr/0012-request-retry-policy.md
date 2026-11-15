# ADR 0012: Retry Each SEC Request in `SecClient`, Then Each Machine Once

| Field | Value |
| ----- | ----- |
| Status | Proposed. The project lead confirms |
| Date | 2026-11-15 |
| Deciders | The project lead |
| Ticket | [STA-505](https://linear.app/state-machine/issue/STA-505) |

## Context

A transient SEC failure is a failure for which `FailedSecRequest::is_transient` returns `true`: a
429, a 502, a 503, a 504, a failed or timed-out request, or a failed or timed-out read of the body.

Before STA-505, the only retry was per run, from STA-441. `with_retry` in
`sec/src/lib/implementations/extraction/retry.rs` ran a whole machine again after 10 s and after
60 s, so 3 runs of a machine in all. One cut connection on the 40th file of a filing sent the 39
files before it again.

STA-505 adds a retry per request. Two levels multiply: 3 attempts per request in each of 3 runs
send one request up to 9 times.

Facts that bound the choice:

- The SEC answers 503 for a filing folder that does not exist, as observed on 2026-11-01
  (`FailedSecRequest::is_unavailable`). So a 503 can be permanent, and the attempts need a limit.
- A Notes file is up to 0.85 GB, about 35 s per download at 25 MB/s (STA-505 notes). Each extra
  send of it costs that time again.
- The rate limiter paces the sends to about 9 per second. No run on record got a 429
  (`sec/design/pipeline/ingestion_paths.md`, "Daily Run", measured runs).
- `reqwest` 0.13.5 retries by default only an HTTP/2 `GOAWAY` with `NO_ERROR` and a
  `REFUSED_STREAM`, at once and without a wait. A custom `reqwest` policy has no backoff, sends
  inside one permit of the rate limiter, and does not see a failed read of the body
  (`reqwest-0.13.5/src/retry.rs`).

The choice binds the `SecClient` trait, every client that implements it, every binary, and the
run-level retry of the facade.

## Decision

1. **The request cycle of `SecClient` holds the retry.** The trait gets three methods with a
   default body:
   - `retry_policy()` returns the `RetryPolicy`. The default is `RetryPolicy::SEND_ONCE`.
   - `is_transient(&error)` says whether a new attempt can fix the error. The default is `false`.
   - `on_retry(&error, &url, retry)` runs before each wait. The default does nothing.

   The default `execute_sec_request` loops: it takes a permit, sends, and on a transient failure
   with attempts left calls `on_retry`, waits, and sends again.
2. **The concrete `SecClient` retries by default.** `SecClient::new` sets the default
   `RetryPolicy`: 3 attempts, a first wait of 1 s, a factor of 2, and a cap of 4 s. So the waits
   are 1 s and 2 s before jitter. `is_transient` calls `FailedSecRequest::is_transient`.
   `with_retry_policy` and `with_on_retry` change the policy and the callback.
3. **Jitter picks each wait between half and all of the backoff.** A wait never drops to zero, so
   a retry during a short outage still waits.
4. **Only a transient failure retries.** A 403, a 404, an invalid body, and every other permanent
   failure return after one attempt.
5. **Each attempt takes its own permit of the rate limiter.** A retry never raises the rate.
6. **`backfill` and `daily_filings` log each retry** as `batch_request_retried`, with the error,
   the URL, and the retry number.
7. **The run-level retry stays, cut to one rerun after 60 s.** `DEFAULT_RETRY_DELAYS` is `[60 s]`.
   One request is sent at most 3 × 2 = 6 times. The waits add up to at most 66 s.
8. **The retry ignores `Retry-After`.** It waits by its own backoff on a 429 and a 503.

### Change to the `SecClient` Trait

`AGENTS.md`, "Open-Closed Principle", asks for the trait, the reason, and the implementors and
callers that change.

- **Trait:** `SecClient` (`sec/src/lib/shared/http_client/traits/sec_client.rs`).
- **Why an extension cannot hold it:** the trait's default `execute_sec_request` owns the request
  cycle: the permit, the send, and the build of the response. The retry must sit around the
  permit, so each attempt takes one. A wrapper type holds the same logic, but it needs a second
  client type in every binary, forwarding of `inner`, `rate_limiter`, and `user_agent`, and
  stand-in `Eq`, `Ord`, and `Hash` impls for its callback. The rate limiter sits in the trait for
  the same reason.
- **Implementors that change:** none have to. Each new method has a default body, so the fakes
  `AlwaysRespondingSecClient`, `RoutingSecClient`, and `FakeSecClient` compile unchanged and
  send once. The concrete `SecClient` and the two fakes that test the retry override the methods.
- **Callers that change:** none. `execute_sec_request` keeps its signature.

## Options Considered

### Where the Retry Lives

Compared on 2026-11-15 on `feat/sec-request-retry`.

| Option | Trait change | Types in the binaries | Extra code |
| ------ | ------------ | --------------------- | ---------- |
| **A (chosen): request cycle of `SecClient`** | 3 methods with a default body | unchanged | none |
| B: wrapper `RetryingSecClient<C>` | none | `RetryingSecClient<SecClient>` | forwarding, stand-in impls for the callback |
| C: `reqwest` retry policy | none | unchanged | a status classifier |

- A keeps one client type, and every caller of a real `SecClient`, the facade included, gets the
  retry without a change.
- B leaves the trait as it is, but every binary names a second type, and the wrapper repeats the
  accessors of the trait.
- C has no wait between attempts, skips the rate limiter, and does not see a failed read of the
  body.

### How Often to Retry

Counted on 2026-11-14. "Coverage" is the longest outage that the waits of both levels span.

| Option | Sends of one request, at most | Coverage | Notes file downloads, at most |
| ------ | ----------------------------- | -------- | ----------------------------- |
| **A (chosen): per request 3, per run 2 (60 s)** | 6 | about 66 s | 6, about 3.5 min |
| B: per request 3, per run 3 (10 s, 60 s) | 9 | about 79 s | 9, about 5 min |
| C: per request 3, no run-level retry | 3 | about 3 s | 3, about 1.8 min |

- A keeps the coverage of an outage of about one minute that B has, with 3 fewer sends. The 10 s
  rerun of B overlaps the per-request waits, which already cover a blip of a few seconds.
- C covers only a blip. A 503 outage of 10 s would fail every machine of the run, and only the
  Cloud Run retry of the whole job would cover it.
- `Retry-After`: `FailedSecRequest` does not carry the response headers. Reading the header needs
  a new field on the error and a parse of the header. No run has seen a 429, so the header has no
  observed use.

## Consequences

- A cut connection costs one request again, not a rerun of the machine.
- A request of several files, such as a filing document set, retries each file on its own.
- Every caller of a real `SecClient` gets the retry, the facade and every binary included.
- A run log of `backfill` and `daily_filings` shows a 429 or a 503 even when a later attempt
  succeeds, with the URL that tells an outage from a missing folder.
- A 503 for a folder that does not exist costs 3 sends in each of 2 runs, so 6 sends, before the
  machine fails.
- A test fake sends once, unless the test gives it a policy.
- The `submissions` and Company Facts runs outside `daily_filings` get the retry per request. They
  have no run-level retry.
- `DEFAULT_RETRY_DELAYS` changes from `[Duration; 2]` to `[Duration; 1]`. A caller that names the
  array type must change.
- The callback is a function pointer, so it cannot capture state such as a run ID.

## Open Questions

- If a run gets a 429, or the SEC starts to send `Retry-After`, reopen point 8.
- If a run log shows outages longer than about one minute, reopen point 7.

## Sources

- `sec/src/lib/shared/http_client/implementations/sec_client/error.rs`: `is_transient` and
  `is_unavailable`, read on 2026-11-14.
- `sec/src/lib/implementations/extraction/retry.rs`: `DEFAULT_RETRY_DELAYS` and `with_retry`, read
  on 2026-11-14.
- `reqwest-0.13.5/src/retry.rs` in the Cargo registry: the default retry and its missing backoff,
  read on 2026-11-15.
- [STA-505](https://linear.app/state-machine/issue/STA-505): the Notes file size and download
  time, and the observation that no run got a 429, read on 2026-11-14.
