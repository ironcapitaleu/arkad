---
name: domain-concept
description: >
  This skill should be used when the user asks to "create a domain concept", "add a domain type",
  "design a domain concept", "implement a shared type", "add dependency injection", or needs to
  create a new domain struct, enum, or trait in the shared directory (e.g., Cik, SecClient,
  HttpClient) with validation, errors, fakes, and tests.
version: 0.1.0
argument-hint: "[concept-name]"
allowed-tools: [Read, Write, Edit, Bash, AskUserQuestion]
---

# Domain Concept Skill

## Purpose

Design and implement a new domain concept — a standalone type that lives in `sec/src/lib/shared/`
(or the equivalent shared directory), is independent of any specific state, and can be developed
and tested in isolation.

Domain concepts range from simple value types (like `Cik`) to trait-based abstractions with
dependency injection (like `SecClient` / `HttpClient`).

## Questionnaire

Use `AskUserQuestion` for ALL questions. NEVER fall back to plain text questions.
Free-form input uses the "Other" option — the user types their answer there.

Not everything needs to be known upfront. It's fine to leave some answers as "don't know yet"
and discover them together during implementation. The questionnaire is a starting point, not
a gate.

If the user provided context when invoking the skill (e.g., `/domain-concept RateLimiter`)
or the conversation already established what the concept is, skip questions you can already
answer. Do not guess names from existing codebase concepts — either infer from context or
let the user provide the name via "Other".

**First prompt:**

- "Does it interact with an external system (HTTP, file system, database, etc.)?" — options:
  - "Yes" — it calls or communicates with something outside the process
  - "No" — it's self-contained data/logic
  - "Don't know yet" — figure it out together
- "Does it wrap an external dependency (3rd-party crate)?" — options:
  - "Yes" — needs a trait abstraction + fake for testability
  - "No" — standalone, no 3rd-party wrapping needed
  - "Don't know yet" — figure it out together
- "Is creation fallible?" — options:
  - "Yes" — new() can fail, returns Result
  - "No" — new() always succeeds
  - "Don't know yet" — figure it out together

**Second prompt (if it wraps an external dependency):**

- "What 3rd-party crate does it wrap?" — use "Other" for free text (header: "Library")
- "What trait methods define the interface?" — use "Other" for free text (header: "Trait methods")
- "What associated types does the trait need? (e.g., Request, Response, Error)" — use "Other" for free text (header: "Types")

**Determining the implementation path:**

- If it wraps an external dependency → Path B (trait + fake + real impl + integration tests)
- If it interacts with an external system but doesn't wrap a dep → Path B (trait for testability)
- Otherwise → Path A (value type with optional validation)
- If answers are "don't know yet" → start with what's known, discover the rest together

## Implementation Workflow

### Path A: Value Type (e.g., Cik, EntityName, Url)

#### Phase 1: Type Definition

Create in `sec/src/lib/shared/{concept_name}/mod.rs`:

```rust
#[derive(Debug, Clone, PartialEq, PartialOrd, Hash, Eq, Ord, Serialize)]
pub struct ConceptName {
    value: InnerType,
}

impl ConceptName {
    pub fn new(input: &str) -> Result<Self, ConceptError> {
        // validate
        Ok(Self { value: validated })
    }

    pub fn value(&self) -> &InnerType {
        &self.value
    }
}

impl fmt::Display for ConceptName {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.value)
    }
}
```

#### Phase 2: Error Type

If construction is fallible:

```rust
#[derive(Debug, Error, Clone, PartialEq, PartialOrd, Hash, Eq, Ord)]
pub enum ConceptError {
    #[error("[InvalidConceptName] Validation failed, Reason: '{reason}'")]
    InvalidFormat { reason: String },
}
```

#### Phase 3: Tests

- Auto-trait tests (Send, Sync, Unpin, Sized, Clone, Debug, Hash, Eq, Ord, PartialEq, PartialOrd)
- `should_create_valid_concept_when_input_is_valid`
- `should_fail_when_input_violates_invariant` (one per invariant)
- `should_return_inner_value`
- `should_display_correctly`

---

### Path B: Trait-Based Abstraction with Dependency Injection

Use this path when the concept wraps an external dependency (HTTP client, file system, etc.)
that needs to be decoupled for testability.

#### Directory Structure

```text
{concept_name}/
├── mod.rs                    # Re-exports traits and implementations
├── traits/
│   ├── mod.rs                # pub use of trait modules
│   ├── inner.rs              # Low-level trait (e.g., InnerClient)
│   └── {domain}.rs           # Domain-level trait (e.g., SecClient)
└── implementations/
    ├── mod.rs                # pub use of implementation modules
    ├── {library}.rs          # Real implementation (e.g., reqwest_client.rs)
    └── {domain_impl}/
        ├── mod.rs            # Domain implementation struct
        └── error.rs          # Implementation-specific errors
```

#### Phase 1: Define the Trait(s)

Start with the trait interface — what methods does it expose? The trait is the contract.
Everything else (fakes, real impl) derives from it.

**Low-level trait** (abstracts the library):

```rust
#[async_trait]
pub trait InnerClient: Send + Sync + Debug + Clone {
    type Request;
    type Response;
    type Error;

    async fn execute_request(&self, request: Self::Request) -> Result<Self::Response, Self::Error>;
}
```

**Domain-level trait** (adds domain knowledge on top):

```rust
#[async_trait]
pub trait SecClient: Send + Sync + Debug {
    type Inner: InnerClient;
    type Request;
    type Response;
    type Error;

    fn inner(&self) -> &Self::Inner;
    async fn execute_sec_request(&self, request: Self::Request) -> Result<Self::Response, Self::Error>;
}
```

#### Phase 2: Fake Implementation(s) + Trait Unit Tests

Create Fakes in `sec/src/lib/tests/fixtures/sample_{concept_name}/` and write unit tests
against the trait using those Fakes. This validates the trait design before touching any
3rd-party dependency.

**Convention:** Each fake implements the trait and provides a fixed response. The naming
pattern is `Always{Behavior}{ConceptName}` (e.g., `AlwaysSucceedingHttpClient`,
`AlwaysFailingHttpClient`). By default, create at least two variants: one that always
succeeds and one that always fails. Suggest additional variants if the domain warrants
them (e.g., `AlwaysRateLimitedClient`, `AlwaysTimingOutClient`).

**Example 1: AlwaysSucceedingHttpClient (happy path fake)**

```rust
#[derive(Debug, Clone, PartialEq)]
pub struct AlwaysSucceedingHttpClient;

#[async_trait]
impl InnerClient for AlwaysSucceedingHttpClient {
    type Request = ();
    type Response = String;
    type Error = String;

    async fn execute_request(&self, request: Self::Request) -> Result<Self::Response, Self::Error> {
        Ok(format!(
            "Simulated success response for request: {:?}",
            request
        ))
    }
}
```

**Example 2: AlwaysFailingHttpClient (error path fake)**

```rust
#[derive(Debug, Clone)]
pub struct AlwaysFailingHttpClient;

#[async_trait]
impl InnerClient for AlwaysFailingHttpClient {
    type Request = ();
    type Response = String;
    type Error = String;

    async fn execute_request(&self, request: Self::Request) -> Result<Self::Response, Self::Error> {
        Err(format!(
            "Simulated network error for request: {:?}",
            request
        ))
    }
}
```

Write trait-level unit tests using these Fakes (in the trait file's `#[cfg(test)]` module):

- Happy path test using the always-succeeding fake
- Error path test using the always-failing fake
- Auto-trait tests on fake implementations

#### Phase 3: Real Implementation

Now wrap the actual 3rd-party dependency:

```rust
use async_trait::async_trait;
use reqwest::{Client, Error as ReqwestError, Request, Response};

#[async_trait]
impl InnerClient for Client {
    type Request = Request;
    type Response = Response;
    type Error = ReqwestError;

    async fn execute_request(&self, request: Self::Request) -> Result<Self::Response, Self::Error> {
        self.execute(request).await
    }
}
```

#### Phase 4: Integration Tests

Test the real implementation against the actual external system:

- Place in `tests/` directory
- Use the real 3rd-party wrapped implementation
- Test against a real or sandboxed endpoint
- Verify the contract holds end-to-end

```rust
use sec::shared::http_client::InnerClient;

fn test_client() -> reqwest::Client {
    reqwest::Client::builder()
        .pool_max_idle_per_host(0)
        .build()
        .expect("Building a reqwest Client with default settings should always succeed")
}

#[tokio::test]
async fn should_return_ok_status_code_when_request_is_valid() {
    let client = test_client();
    let url = "https://httpbin.org/get";
    let request_url = reqwest::Url::parse(url)
        .expect(&format!("The hardcoded URL `{url}` should always be valid"));
    let request = reqwest::Request::new(reqwest::Method::GET, request_url);

    let expected_result = reqwest::StatusCode::OK;

    let result = client
        .execute_request(request)
        .await
        .expect(&format!("A request to the URL `{url}` should always succeed"))
        .status();

    assert_eq!(result, expected_result);
}
```

---

## Full End-to-End Example: InnerClient (HTTP abstraction)

This shows all four phases working together as a cohesive unit:

### 1. Trait (the contract)

```rust
#[async_trait]
pub trait InnerClient: Send + Sync + Debug + Clone {
    type Request;
    type Response;
    type Error;

    async fn execute_request(&self, request: Self::Request) -> Result<Self::Response, Self::Error>;
}
```

### 2. Fakes (unit test the trait)

```rust
#[derive(Debug, Clone, PartialEq)]
pub struct AlwaysSucceedingHttpClient;

#[async_trait]
impl InnerClient for AlwaysSucceedingHttpClient {
    type Request = ();
    type Response = String;
    type Error = String;

    async fn execute_request(&self, request: Self::Request) -> Result<Self::Response, Self::Error> {
        Ok(format!("Simulated success response for request: {:?}", request))
    }
}

#[derive(Debug, Clone)]
pub struct AlwaysFailingHttpClient;

#[async_trait]
impl InnerClient for AlwaysFailingHttpClient {
    type Request = ();
    type Response = String;
    type Error = String;

    async fn execute_request(&self, request: Self::Request) -> Result<Self::Response, Self::Error> {
        Err(format!("Simulated network error for request: {:?}", request))
    }
}
```

### 3. Real implementation (wraps reqwest)

```rust
use reqwest::{Client, Error as ReqwestError, Request, Response};

#[async_trait]
impl InnerClient for Client {
    type Request = Request;
    type Response = Response;
    type Error = ReqwestError;

    async fn execute_request(&self, request: Self::Request) -> Result<Self::Response, Self::Error> {
        self.execute(request).await
    }
}
```

### 4. Integration test (verifies real impl against live endpoint)

```rust
use sec::shared::http_client::InnerClient;

fn test_client() -> reqwest::Client {
    reqwest::Client::builder()
        .pool_max_idle_per_host(0)
        .build()
        .expect("Building a reqwest Client with default settings should always succeed")
}

#[tokio::test]
async fn should_return_ok_status_code_when_request_is_valid() {
    let client = test_client();
    let url = "https://httpbin.org/get";
    let request_url = reqwest::Url::parse(url)
        .expect(&format!("The hardcoded URL `{url}` should always be valid"));
    let request = reqwest::Request::new(reqwest::Method::GET, request_url);

    let expected_result = reqwest::StatusCode::OK;

    let result = client
        .execute_request(request)
        .await
        .expect(&format!("A request to the URL `{url}` should always succeed"))
        .status();

    assert_eq!(result, expected_result);
}
```

---

## Value Type Examples

### Cik (validated, fallible)

```rust
pub struct Cik { value: String }

impl Cik {
    pub fn new(raw: &str) -> Result<Self, CikError> {
        let padded = format!("{:0>10}", raw.trim());
        if padded.len() != 10 || !padded.chars().all(|c| c.is_ascii_digit()) {
            return Err(CikError::InvalidFormat { input: raw.to_string() });
        }
        Ok(Self { value: padded })
    }
}
```

### EntityName (simple wrapper, infallible)

```rust
pub struct EntityName { value: String }

impl EntityName {
    pub fn new(name: impl Into<String>) -> Self {
        Self { value: name.into() }
    }
}
```

## Checklist

- [ ] Struct/trait defined with correct derives and bounds
- [ ] Constructor with validation (if value type)
- [ ] Error type (if fallible)
- [ ] Display impl
- [ ] Getter/accessor method(s)
- [ ] Real implementation (if trait-based)
- [ ] Fake implementation in `tests/fixtures/` (if trait-based)
- [ ] Auto-trait tests
- [ ] Happy path test(s)
- [ ] Error path test(s)
- [ ] Module registered in parent `mod.rs`
- [ ] Compiles and tests pass
