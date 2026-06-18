---
name: state-implementation
description: >
  Use when implementing state logic, adding domain concepts, working on
  compute_output_data, introducing domain errors, or designing the testing
  strategy for a specific state's business logic. Pair-programming guide.
version: 0.1.0
argument-hint: "[state-name]"
allowed-tools: [Read, Write, Edit, Bash, AskUserQuestion]
---

# State Implementation Skill

## Purpose

Guide the incremental implementation of domain logic for a state after its boilerplate scaffold
exists. This is a pair-programming workflow — step-by-step with the user.

This skill assumes:

- A state design exists (from `/state-design`)
- Boilerplate is scaffolded (from `/state-scaffold`)

If either is missing, direct the user to run those first.

## Initial Verification

Before starting implementation, verify:

1. Read the state design document for requirements
2. Confirm scaffold exists (directory structure, stub files)
3. Confirm it compiles (`cargo check`)

Then clarify any remaining questions with the user:

1. **What domain concepts does it introduce?** (e.g., Cik, SecClient, XbrlParser)
2. **What external dependencies are needed?** (HTTP clients, file I/O, parsers, crates)
3. **What can go wrong?** (error cases: network failure, invalid format, missing data)

## Implementation Workflow

### Phase 1: Domain Concepts (test independently)

Design and implement domain structs/traits that the state uses but that exist independently of the state machine:

- Domain types (e.g., `Cik`, `AccessionNumber`, `Frame`)
- Domain errors (e.g., `CikError::InvalidFormat`)
- Validation logic on the types themselves
- Unit tests for each domain concept in isolation

These are the building blocks. Each should be fully tested before the state uses them.

### Phase 2: Compute Logic

Implement `compute_output_data_async`:

1. Call the domain method/function that does the work (returns its own `Result`)
2. On success: wrap in output struct, set `self.output = Some(output)`
3. On error: convert to `StateError` via named error type and propagate
4. Write tests for both happy path and error path

### Phase 3: Error Handling

Error handling follows natural Result propagation — not one error at a time, but by calling
functions that return their own errors and converting at the state boundary:

```rust
async fn compute_output_data_async(&mut self) -> Result<(), StateError> {
    let result = self.input.client.do_something().await;

    match result {
        Ok(data) => {
            self.output = Some(MyStateOutput::new(data));
            Ok(())
        }
        Err(e) => {
            let e: StateError =
                NamedStateError::from_domain_error(self.state_name().to_string(), e).into();
            Err(e)
        }
    }
}
```

The pattern:

1. Call a domain function/method that returns its own error type
2. Convert to a **named state error** (e.g., `InvalidCikFormat`, `FailedRequestExecution`) using `from_domain_error` or `::new`
3. Convert that into `StateError` via `.into()`
4. Return `Err(e)`

Error hierarchy:

- **Domain errors** — returned by domain methods (e.g., `CikError`, `SecClientError`)
- **Named state errors** — wrap domain errors with state context (e.g., `InvalidCikFormat`)
- **StateError** — the enum all named state errors convert into
- **SuperStateError** — top-level wrapper (wraps StateError)

## Examples from the Codebase

### Example 1: Simple compute — no external call (PrepareSecRequest)

Assembles output from input fields, no async work, cannot fail:

```rust
async fn compute_output_data_async(&mut self) -> Result<(), StateError> {
    let sec_client = self.input.sec_client.clone();
    let sec_request = SecRequest::builder()
        .all_company_facts()
        .cik(self.input.validated_cik.clone())
        .build();

    self.output = Some(PrepareSecRequestOutput::new(sec_client, sec_request));

    Ok(())
}
```

### Example 2: Validation with domain error (ValidateCikFormat)

Calls a domain constructor that validates and returns its own error type:

```rust
async fn compute_output_data_async(&mut self) -> Result<(), StateError> {
    let cik = Cik::new(&self.input.raw_cik);

    match cik {
        Ok(cik) => {
            self.output = Some(ValidateCikFormatOutput { validated_cik: cik });
        }
        Err(e) => {
            let e: StateError =
                InvalidCikFormat::from_domain_error(self.state_name().to_string(), e).into();
            return Err(e);
        }
    }

    Ok(())
}
```

### Example 3: External async call (ExecuteSecRequest)

Calls an async method on a client, converts error at boundary:

```rust
async fn compute_output_data_async(&mut self) -> Result<(), StateError> {
    let client = &self.input.sec_client;
    let request = &self.input.sec_request;

    let result = client.execute_sec_request(request.clone()).await;

    match result {
        Ok(response) => {
            self.output = Some(ExecuteSecRequestOutput::new(response));
            Ok(())
        }
        Err(e) => {
            let e: StateError =
                FailedRequestExecution::new(self.state_name().to_string(), e).into();
            Err(e)
        }
    }
}
```

### Phase 4: Dependency Inversion (if external deps)

When the state calls external systems:

1. Define a trait for the external dependency (e.g., `trait HttpClient`)
2. Context holds a concrete implementation (e.g., `SecClient`)
3. For tests: create a fixture/fake implementing the trait
4. Inject via Context — state only knows the trait interface

### Phase 5: Integration Tests

In `tests/` directory:

1. Test the state with real (or realistic fake) dependencies
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

Always present updated mermaid diagrams to the user for visual verification — AI cannot render or confirm diagram correctness.

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
| State compute (happy path) | Same file, `#[cfg(test)]` | Fake/fixture context |
| State compute (errors) | Same file, `#[cfg(test)]` | Fake/fixture context |
| Transitions | SuperState test module | Previous state fixture |
| Integration (end-to-end) | `tests/` directory | Real or fake external deps |

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
pub struct FakeClient { pub response: Response }
impl HttpClient for FakeClient {
    async fn get(&self, _url: &str) -> Result<Response, ClientError> {
        Ok(self.response.clone())
    }
}

// Context uses concrete type
pub struct MyStateContext {
    pub client: SecClient,
}
```
