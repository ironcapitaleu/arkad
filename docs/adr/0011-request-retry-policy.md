# ADR 0011: Retry Each SEC Request, Then Each Machine Once

| Field | Value |
| ----- | ----- |
| Status | Proposed. The project lead confirms |
| Date | 2026-11-14 |
| Deciders | The project lead |
| Ticket | [STA-505](https://linear.app/state-machine/issue/STA-505) |

## Context

A transient SEC failure is a failure for which `FailedSecRequest::is_transient` returns `true`: a
429, a 502, a 503, a 504, a failed or timed-out request, or a failed or timed-out read of the body.

Before STA-505, the only retry was per run, from STA-441. `with_retry` in
`sec/src/lib/implementations/extraction/retry.rs` ran a whole machine again after 10 s and after
60 s, so 3 runs of a machine in all. One cut connection on the 40th file of a filing sent the 39
files before it again.

STA-505 adds a retry per request in a wrapper client, `RetryingSecClient<C>`. The two levels
multiply: 3 attempts per request in each of 3 runs send one request up to 9 times.

Facts that bound the choice:

- The SEC answers 503 for a filing folder that does not exist, as observed on 2026-11-01
  (`FailedSecRequest::is_unavailable`). So a 503 can be permanent, and the attempts need a limit.
- A Notes file is up to 0.85 GB, about 35 s per download at 25 MB/s (STA-505 notes). Each extra
  send of it costs that time again.
- The rate limiter paces the sends to about 9 per second. No run on record got a 429
  (`sec/design/pipeline/ingestion_paths.md`, "Daily Run", measured runs).

The choice binds every binary and the run-level retry of the facade. Each binary builds its
client through `sec/src/bin/shared/sec_client.rs`.

## Decision

1. **Every binary retries each request.** The client of a binary is
   `RetryingSecClient<SecClient>` with the default `RetryPolicy`: 3 attempts, a first wait of 1 s,
   a factor of 2, and a cap of 4 s. So the waits are 1 s and 2 s before jitter.
2. **Jitter picks each wait between half and all of the backoff.** A wait never drops to zero, so
   a retry during a short outage still waits.
3. **Only a transient failure retries.** A 403, a 404, an invalid body, and every other permanent
   failure return after one attempt.
4. **Each attempt takes its own permit of the rate limiter.** A retry never raises the rate.
5. **The run-level retry stays, cut to one rerun after 60 s.** `DEFAULT_RETRY_DELAYS` is `[60 s]`.
   One request is sent at most 3 × 2 = 6 times. The waits add up to at most 66 s.
6. **The wrapper ignores `Retry-After`.** It waits by its own backoff on a 429 and a 503.

## Options Considered

Counted on 2026-11-14 from the code on `feat/sec-request-retry`. "Coverage" is the longest outage
that the waits of both levels span.

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
- `backfill` and `daily_filings` log each retry of a request as `batch_request_retried`, with the
  error, the URL, and the retry number. So a run log shows a 429 or a 503 even when a later attempt
  succeeds.
- A request of several files, such as a filing document set, retries each file on its own.
- A 503 for a folder that does not exist costs 3 sends in each of 2 runs, so 6 sends, before the
  machine fails.
- The `submissions` and Company Facts runs outside `daily_filings` get the retry per request. They
  have no run-level retry.
- A caller of the `Extraction` facade that passes a plain `SecClient` gets no retry per request.
  The facade sends through the client it gets.
- `DEFAULT_RETRY_DELAYS` changes from `[Duration; 2]` to `[Duration; 1]`. A caller that names the
  array type must change.

## Open Questions

- If a run gets a 429, or the SEC starts to send `Retry-After`, reopen point 6.
- If a run log shows outages longer than about one minute, reopen point 5.

## Sources

- `sec/src/lib/shared/http_client/implementations/sec_client/error.rs`: `is_transient` and
  `is_unavailable`, read on 2026-11-14.
- `sec/src/lib/implementations/extraction/retry.rs`: `DEFAULT_RETRY_DELAYS` and `with_retry`, read
  on 2026-11-14.
- [STA-505](https://linear.app/state-machine/issue/STA-505): the Notes file size and download
  time, and the observation that no run got a 429, read on 2026-11-14.
