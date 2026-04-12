---
name: Shared HTTP client across pipeline runs
description: Future optimization to share a single reqwest::Client across all concurrent pipeline runs for connection pooling and TLS reuse
type: project
---

Currently each pipeline creates its own `SecClient::default()` inside `PrepareSecRequest`'s compute, meaning each pipeline opens fresh TCP connections. A shared `reqwest::Client` would reuse connections from the pool.

**Why:** Connection pooling and TLS session reuse — faster requests, less load on SEC servers. Currently 99% of pipeline time is the HTTP call.

**How to apply:** Pass a shared `SecClient` (or `reqwest::Client`) into the state machine from the outside rather than constructing it inside `PrepareSecRequest`. This means `ExtractSuperState` or the builder needs to accept an external client dependency. The `Extraction` builder in the binary would create one client and pass it to all pipeline runs.
