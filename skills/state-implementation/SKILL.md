---
name: state-implementation
description: >
  Use when implementing state logic, adding domain concepts, working on
  compute_output_data, introducing domain errors, or designing the testing
  strategy for a specific state's business logic. Pair-programming guide.
version: 0.1.0
---

# State Implementation Skill

## Purpose

Guide the incremental implementation of domain logic for a state after its boilerplate scaffold exists. This is a pair-programming workflow — the skill prompts questions, then guides step-by-step implementation.

## Initial Questioning

Before implementation, clarify:

1. **What does this state compute?** (high-level: "validates CIK format", "executes HTTP request")
2. **What domain concepts does it introduce?** (e.g., Cik, SecClient, XbrlParser)
3. **What external dependencies are needed?** (HTTP clients, file I/O, parsers, crates)
4. **What can go wrong?** (error cases: network failure, invalid format, missing data)
5. **Downstream consumers?** (who uses this state's output? affects output design)

## Implementation Workflow

### Phase 1: Domain Concepts (test independently)

Design and implement domain structs/traits that the state uses but that exist independently of the state machine:

- Domain types (e.g., `Cik`, `AccessionNumber`, `Frame`)
- Domain errors (e.g., `CikError::InvalidFormat`)
- Validation logic on the types themselves
- Unit tests for each domain concept in isolation

These are the building blocks. Each should be fully tested before the state uses them.

### Phase 2: Compute Logic (happy path first)

Implement `compute_output_data_async`:

1. Start with the happy path only
2. Use domain concepts from Phase 1
3. Transform input + context into output
4. Set `self.output = Some(output)` on success
5. Write async test verifying happy path

### Phase 3: Error Handling (one error case at a time)

For each error scenario:

1. Define the domain error variant (if new)
2. Define how it maps into StateError (via `From` or explicit conversion)
3. Add the error path to `compute_output_data_async`
4. Write a test triggering that exact error
5. Verify the error chain format matches project conventions: `[StateError] ..., Caused by: [DomainError] ..., Reason: '...'`

Error hierarchy:

- **Domain errors** = "this concept is invalid" (e.g., `InvalidCikFormat`)
- **State errors** = "something went wrong in this state" (wraps domain errors)
- **SuperState errors** = top-level wrapper (wraps state errors)

### Phase 4: Dependency Inversion (if external deps)

When the state calls external systems:

1. Define a trait for the external dependency (e.g., `trait HttpClient`)
2. Context holds a concrete implementation (e.g., `SecClient`)
3. For tests: create a fixture/mock implementing the trait
4. Inject via Context — state only knows the trait interface

### Phase 5: Integration Tests

In `tests/` directory:

1. Test the state with real (or realistic mock) dependencies
2. End-to-end: construct state → compute → verify output
3. Test transition from source state → this state → verify
4. Error scenarios with realistic failure modes

### Phase 6: Binary Integration

Once the state's logic is implemented and tested, integrate it into the applicable binaries:

- **Identify which binaries need this state** — not all states belong in every binary, but most will appear in at least one
- **Add to the SuperState's async stream** — wire the state into the pipeline's sequential execution so it runs in the correct order
- **Wire transitions** — ensure the binary's run loop or stream handles the transition from the previous state into this one and from this one into the next
- **Respect Streamable trait** — if the SuperState implements streaming, ensure the new state integrates correctly (yields progress, can be polled, respects cancellation)
- **Test the full pipeline** — run the binary end-to-end to verify the new state executes in sequence and produces expected results downstream

### Phase 7: Documentation Updates

Update design docs and README to reflect the implemented logic:

- **Design docs** (e.g., `design/` mermaid diagrams): Update state machine diagrams with any new transitions, error paths, or domain concepts introduced
- **README.md**: Update architecture overviews, module descriptions, or feature lists if the state introduces new capabilities
- **Mermaid sequence/state diagrams**: Ensure the compute flow, error branching, and dependency interactions are accurately represented

Always verify existing diagrams still reflect reality after implementation changes.

### Phase 7: Verification

Final checks:

- All Display impls produce readable output
- All error variants are reachable (no dead code)
- Trait compliance tests pass (auto-traits)
- Documentation: docstrings on all public items
- Async and sync paths both work
- Design docs and README are consistent with the implementation

## Testing Strategy Matrix

| What | Where | Dependencies |
| --- | --- | --- |
| Domain concepts (Cik, etc.) | Same file, `#[cfg(test)]` | None |
| State compute (happy path) | Same file, `#[cfg(test)]` | Mock/fixture context |
| State compute (errors) | Same file, `#[cfg(test)]` | Mock/fixture context |
| Transitions | SuperState test module | Previous state fixture |
| Integration (end-to-end) | `tests/` directory | Real or mock external deps |

## Domain Error Pattern

```rust
#[derive(Debug, Error, Clone, PartialEq, PartialOrd, Hash, Eq, Ord)]
pub enum MyDomainError {
    #[error("[InvalidFoo] Foo validation failed, Reason: '{reason}'")]
    InvalidFoo { reason: String },

    #[error("[FailedBarRequest] Bar request failed, Caused by: {source}")]
    FailedBarRequest {
        #[source]
        source: InnerError,
    },
}

impl From<MyDomainError> for StateError {
    fn from(e: MyDomainError) -> Self {
        StateError::DomainSpecificError(e)
    }
}
```

## Dependency Inversion Pattern

```rust
// Trait in domain layer
pub trait HttpClient: Send + Sync {
    async fn get(&self, url: &str) -> Result<Response, ClientError>;
}

// Real implementation
pub struct SecClient { /* ... */ }
impl HttpClient for SecClient { /* ... */ }

// Test fixture
pub struct MockClient { pub response: Response }
impl HttpClient for MockClient {
    async fn get(&self, _url: &str) -> Result<Response, ClientError> {
        Ok(self.response.clone())
    }
}

// Context uses concrete type
pub struct MyStateContext {
    pub client: SecClient,
}
```
