---
name: testing
description: >
  This skill should be used when the user asks to "review tests", "add tests", "check test coverage",
  "write integration tests", "add boilerplate tests", "review testing strategy", or needs to write,
  review, or improve unit tests or integration tests for a module, component, package, or crate.
version: 0.1.0
argument-hint: "[module-or-file-path]"
allowed-tools: [Read, Write, Edit, Bash, AskUserQuestion]
---

# Testing Skill

## Purpose

Write, review, and improve tests for any part of the codebase. Covers unit tests (in-file),
integration tests (`tests/` directory), and boilerplate trait-compliance tests. Proactively
suggests meaningful tests and applies boilerplate tests wherever they're missing.

## Context Gathering

If no specific target is provided, gather context automatically:

1. Check recent conversation history — what was just implemented or discussed?
2. Check `git diff` — what files were recently modified?
3. From context, identify the target and suggest options to the user via `AskUserQuestion`

When a target is identified (either from context or user input), reason about:

- **What crate/module is it in?** — `sec`, `xbrl`, `state_machine`?
- **What kind of item is it?** — struct, trait, enum, state, domain concept?
- **What role does it play?**
  - A state in a state machine → needs state method tests, async compute tests, auto-trait tests
  - A domain concept (value type) → needs construction tests, validation tests, Display tests, auto-trait tests
  - A trait with behavior (async methods, associated types, invariants) → needs unit tests in the trait file using fakes to verify the trait's contract/design
  - A trait that's just bounds (marker traits, `Send + Sync + Debug`) → compile-time only, no fakes needed
  - A struct that implements a trait → tested as a struct; the trait impl is exercised through the struct's own unit tests
  - StateData/Context → needs update tests, reference tests, auto-trait tests
- **Is it Send + Sync?** → add thread-safety boilerplate tests
- **Does it implement Display?** → add Display format verification tests
- **Is it fallible?** → add error path tests for each error variant
- **Does it wrap an external dep?** → suggest integration tests with real impl

Based on this reasoning, suggest what tests to write/review and let the user confirm or adjust.

## Modes

- **Review** — audit existing tests for a module/component, find gaps
- **Write** — add missing tests (boilerplate + meaningful behavioral tests)
- **Integration** — design and implement integration tests for a component or pipeline
- **Compliance check** — verify all structs have the required boilerplate tests

## Conventions

- **Pattern:** Arrange, Define, Act, Assert
- **Exactly ONE `assert!` per test function**
- **Naming:** Behavioral tests use `should_..._when_...`; auto-trait compliance tests use `should_implement_...` or `should_be_...` (no `when` needed). Always snake_case, verbose is fine.
- **Location:** Unit tests in same file under `#[cfg(test)]`; integration tests in `tests/` directory
- **Assertions:** Use `pretty_assertions` (`assert_eq!`, `assert_ne!`)
- **`.expect()` messages:** Explain WHY the operation should not fail in that context
- **No comments:** No `// Arrange` / `// Act` — the structure speaks for itself

## Boilerplate Tests (apply to ALL domain types and state structs)

Every domain type and state struct should include compile-time trait compliance tests (auto traits like `Send`/`Sync`/`Unpin` and derived traits like `Debug`, `Clone`, `PartialEq`, `Eq`, `Hash`, `Ord` where applicable):

```rust
const fn implements_auto_traits<T: Sized + Send + Sync + Unpin>() {}
#[test]
const fn should_implement_auto_traits() {
    implements_auto_traits::<MyType>();
}

const fn implements_send<T: Send>() {}
const fn implements_sync<T: Sync>() {}

#[test]
const fn should_implement_send() {
    implements_send::<MyType>();
}

#[test]
const fn should_implement_sync() {
    implements_sync::<MyType>();
}

#[test]
const fn should_be_thread_safe() {
    implements_send::<MyType>();
    implements_sync::<MyType>();
}

const fn implements_sized<T: Sized>() {}
#[test]
const fn should_be_sized() {
    implements_sized::<MyType>();
}

const fn implements_hash<T: std::hash::Hash>() {}
#[test]
const fn should_implement_hash() {
    implements_hash::<MyType>();
}

const fn implements_partial_eq<T: PartialEq>() {}
#[test]
const fn should_implement_partial_eq() {
    implements_partial_eq::<MyType>();
}

const fn implements_eq<T: Eq>() {}
#[test]
const fn should_implement_eq() {
    implements_eq::<MyType>();
}

const fn implements_partial_ord<T: PartialOrd>() {}
#[test]
const fn should_implement_partial_ord() {
    implements_partial_ord::<MyType>();
}

const fn implements_ord<T: Ord>() {}
#[test]
const fn should_implement_ord() {
    implements_ord::<MyType>();
}

const fn implements_debug<T: std::fmt::Debug>() {}
#[test]
const fn should_implement_debug() {
    implements_debug::<MyType>();
}

const fn implements_clone<T: Clone>() {}
#[test]
const fn should_implement_clone() {
    implements_clone::<MyType>();
}

const fn implements_unpin<T: Unpin>() {}
#[test]
const fn should_implement_unpin() {
    implements_unpin::<MyType>();
}
```

**When to apply:** Every new struct in `shared/`, every state Input/Output/Context, every
domain concept. If you see a struct without these tests, add them proactively.

## Fakes

### What is a Fake

A fake is a minimal implementation of a trait that returns fixed, predictable responses.
Fakes decouple unit tests from real external systems (network, file I/O, timers) and
allow testing the trait's contract in isolation.

### Where to put them

Fakes live in the crate's test fixtures directory. For the `sec` crate: `sec/src/lib/tests/fixtures/sample_{concept_name}/`.

Example structure:

```text
sec/src/lib/tests/fixtures/
├── sample_http_client/
│   ├── sample_inner_client/
│   │   ├── always_succeeding.rs
│   │   └── always_failing.rs
│   └── sample_sec_client/
│       └── always_succeeding.rs
├── sample_request/
└── sample_response/
```

### How to define a Fake

A fake implements the trait with fixed behavior. No logic, no state mutation — just
return a predetermined value.

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
```

### Naming conventions

- **Low-level trait fakes:** `Always{Behavior}{ConceptName}` (e.g., `AlwaysSucceedingHttpClient`, `AlwaysFailingHttpClient`, `AlwaysReadyRateLimiter`)
- **Domain-level trait fakes:** `Fake{ConceptName}` with behavior in module names (e.g., `FakeSecClient` in `always_succeeding.rs`)

### When to create a Fake

- When a trait has async methods that would hit external systems
- When you need to test error handling paths (always-failing variant)
- When a state or component takes a trait as a dependency (via Context)

### When NOT to create a Fake

- For value types with no trait abstraction — test them directly
- For traits that are only compile-time bounds (marker traits)
- When an integration test with the real implementation is more appropriate

### How and where to use Fakes

Fakes are used in **unit tests for traits** — never in integration tests.
Integration tests use the real implementation against real systems.

Fakes live in `tests/fixtures/` and are imported where needed. A single fake can be
used across multiple trait tests.

```rust
// In a state's test module — use fake client to test compute logic
fn test_context() -> ExecuteSecRequestContext {
    ExecuteSecRequestContext::new(FakeSecClient::new())
}

#[tokio::test]
async fn should_compute_output_data_when_client_succeeds() {
    let mut state = ExecuteSecRequest::new(test_input(), test_context());

    let result = state.compute_output_data_async().await;

    assert!(result.is_ok());
}
```

## Unit Test Patterns

### Value type — happy path + error path

```rust
#[test]
fn should_create_valid_cik_struct_if_numeric_string_with_ten_digits_is_passed() {
    let cik_str = "1234567890";

    let expected_result = "1234567890";

    let result = Cik::new(cik_str)
        .expect("CIK creation should always succeed with hardcoded ten digit value");

    assert_eq!(result.value(), expected_result);
}

#[test]
fn should_fail_when_given_cik_str_that_contains_non_numeric_chars() {
    let cik_str = "12345abcde";

    let result = Cik::new(cik_str);

    assert!(
        result.is_err(),
        "CIK creation with non-numeric chars in hardcoded value should fail."
    );
}
```

### Error type — verify Display output

```rust
#[test]
fn should_format_display_as_expected_when_reason_is_too_long() {
    let invalid_cik = "123456789012345";
    let reason = InvalidCikReason::MaxLengthExceeded {
        cik_length: invalid_cik.len(),
    };
    let cik_error = CikError::new(reason.clone(), invalid_cik);

    let expected_result =
        format!("[CikError] Invalid CIK, Reason: '{reason}', Input: '{invalid_cik}'");

    let result = format!("{cik_error}");

    assert_eq!(result, expected_result);
}
```

### StateData — update tests

```rust
#[test]
fn should_update_state_data_when_update_contains_values() {
    let mut data = test_input();
    let update = InputUpdaterBuilder::default()
        .field_name("updated_value")
        .build();

    let expected = &TestInput::new("updated_value");

    StateData::update_state(&mut data, update)
        .expect("Update should succeed");
    let result = data.state();

    assert_eq!(result, expected);
}

#[test]
fn should_leave_state_data_unchanged_when_empty_update() {
    let mut data = test_input();
    let empty_update = InputUpdaterBuilder::default().build();

    let expected = &test_input();

    StateData::update_state(&mut data, empty_update)
        .expect("Update should succeed");
    let result = data.state();

    assert_eq!(result, expected);
}
```

### Trait-based (with fakes)

```rust
#[tokio::test]
async fn should_return_expected_success_response_for_fake_sec_client() {
    let client = FakeSecClient::new();
    let request = ();

    let expected_result = Ok(String::from("Simulated success response for sec request: ()"));

    let result = client.execute_sec_request(request).await;

    assert_eq!(result, expected_result);
}
```

## Integration Tests

### When to write integration tests

- A component wraps a real external system (HTTP client, file system)
- A pipeline end-to-end flow needs verification
- Rate limiting or timing behavior needs validation
- Cross-state transitions need real data verification

### Structure

Integration tests live in the crate's `tests/` directory, one file per component or concern:

```
sec/tests/
├── reqwest_client.rs       # HTTP client against httpbin.org
├── sec_client.rs           # SEC client against SEC EDGAR
├── sec_response.rs         # Response parsing with real data
└── pipeline_coverage/      # Multi-file pipeline test
    ├── main.rs
    ├── builder.rs
    └── constants.rs
```

### Long-running tests: use `#[ignore]`

Tests that hit live APIs or take significant time must be marked `#[ignore]` with a reason:

```rust
#[tokio::test(flavor = "multi_thread")]
#[ignore = "Hits the live SEC API for all S&P 500 CIKs (~3 minutes)"]
async fn should_meet_threshold_for_sp500_companies() {
    // ...
}
```

Add a module-level docstring explaining how to run ignored tests:

```rust
//! # Pipeline Coverage Integration Test
//!
//! ## Running
//! ```sh
//! cargo test --test pipeline_coverage -- --ignored --nocapture --test-threads=1
//! ```
```

### When NOT to use `#[ignore]`

- Tests that complete in under ~5 seconds (like `reqwest_client.rs` against httpbin.org)
- Tests that use fakes/fixtures instead of real endpoints
- Tests that validate logic without I/O

### Integration test example — rate limiter timing

```rust
#[tokio::test]
async fn should_pace_requests_to_configured_rate_when_acquiring_permits() {
    let limiter = Arc::new(GovernorRateLimiter::new(10));
    let num_requests = 5;

    let start = Instant::now();
    for _ in 0..num_requests {
        limiter.acquire().await;
    }
    let elapsed = start.elapsed();

    let expected_minimum_ms = 400;

    let result = elapsed.as_millis() >= expected_minimum_ms;

    assert!(
        result,
        "Expected at least {expected_minimum_ms}ms for {num_requests} requests at 10 req/s, got {}ms",
        elapsed.as_millis()
    );
}
```

### Integration test example — real HTTP client

```rust
#[tokio::test]
async fn should_return_ok_status_code_when_request_is_valid() {
    let client = test_client();
    let url = "https://httpbin.org/get";
    let request_url = reqwest::Url::parse(url)
        .expect(&format!("The hardcoded URL `{url}` should always be valid"));
    let request = Request::new(reqwest::Method::GET, request_url);

    let expected_result = reqwest::StatusCode::OK;

    let result = client
        .execute_request(request)
        .await
        .expect(&format!("A request to the URL `{url}` should always succeed"))
        .status();

    assert_eq!(result, expected_result);
}
```

### Integration test example — pipeline coverage with threshold

```rust
#[tokio::test(flavor = "multi_thread")]
#[ignore = "Hits the live SEC API for must-pass companies (~10 seconds)"]
async fn should_succeed_for_must_pass_companies() {
    let sec_client = SecClient::default();
    let ciks: Vec<&str> = MUST_PASS_CIKS.iter().map(|(cik, _)| *cik).collect();
    let stream_results: Vec<_> = futures_util::stream::iter(ciks.iter())
        .map(|cik| {
            let client = sec_client.clone();
            async move {
                (*cik, Pipeline::builder().cik(*cik).sec_client(client).build().run().await)
            }
        })
        .buffer_unordered(10)
        .collect()
        .await;

    let expected_result: Vec<String> = vec![];

    let result: Vec<String> = stream_results
        .iter()
        .filter_map(|(cik, r)| {
            r.as_ref().err().map(|e| {
                let name = lookup_must_pass_name(cik);
                format!("{name} (CIK {cik}): {e}")
            })
        })
        .collect();

    assert_eq!(result, expected_result);
}
```

## Proactive Behavior

When reviewing or writing tests:

1. **Apply boilerplate tests everywhere** — if a struct exists without auto-trait tests, add them without asking
2. **Suggest meaningful behavioral tests** — based on the component's public API:
   - Happy path for each public method
   - Error path for each fallible method
   - Edge cases (empty input, max values, unicode, whitespace)
   - Display formatting verification
3. **Suggest integration tests** when a component:
   - Wraps an external dependency
   - Is part of a pipeline
   - Has timing/ordering semantics
4. **Inquire with the user** about:
   - What inputs/outputs are most critical to verify
   - Which edge cases are most likely in production
   - What failure scenarios have been observed before
5. **Proactively propagate** — when fixing a test pattern in one file, search for all similar
   files with the same gap and fix them all in one pass

## Self-Improvement

After completing a testing session where the user corrected or refined a pattern:

1. Ask: "Should I update the testing skill with this pattern?"
2. If yes, update the relevant section (examples, conventions, checklist) in this SKILL.md.
3. Apply after user approval.

Examples of things worth capturing:
- New test patterns that emerged (e.g., a better way to test async streams)
- Corrections to the boilerplate (e.g., "we stopped testing PartialOrd for this type")
- Integration test patterns that proved valuable
- Few-shot examples that drifted from actual code — update them
- New tests that make good few-shot examples — add them
- Tests where the user had to correct the agent — add as examples to prevent repeating mistakes

Also periodically review: do the existing few-shot examples still match the codebase?
If not, update them. If a new test is a better or complementary example, add it.
The examples are authoritative guidance — they must reflect reality.

This keeps the skill growing from real usage rather than speculation.
