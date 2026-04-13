# Development Guidelines for This Rust Project

---

## General Rules (For All Code)

### Code Quality

- Code must be **properly formatted** (`rustfmt`).
- Dependencies must be **free of known security vulnerabilities** (`cargo audit`).
- Documentation must **build without warnings** (`cargo doc --no-deps --document-private-items --workspace`).
- Code must **compile without errors** and pass:
  - Linting (`clippy`)
  - Doc linting (`rustdoc` broken/redundant link warnings are errors)
  - Unit tests
  - Integration tests
  - Doctests

### Documentation

- Public items in libraries **must include docstrings** (`///`).
- All implementation changes must be **reflected in documentation**, including:
  - Docstrings
  - Design documents (if applicable, e.g., mermaid diagrams)
- **Documentation must be version controlled**.

---

## Import Order Conventions

Group and order all Rust imports as follows:

1. **Standard Library Imports** — `std::*` first
2. **External Crate Imports** — third-party crates (e.g., `serde`, `tracing`)
3. **Internal Crate Imports** — `crate::*` and `super::*` last

Within each group, order alphabetically by path. Separate groups with a single blank line.

After the final line of imports, module declarations (`mod foo;`) can be made. Public re-exports (`pub use foo::Bar;`) should follow module declarations.

**Example:**

```rust
use std::collections::HashMap;
use std::fmt;

use pretty_assertions::assert_eq;
use serde::Deserialize;

use crate::traits::state_machine::State;
use super::context_data::ContextData;
```

---

## Error Naming Conventions

### Error Types

**Adjective-First Pattern** (for describing the *state* of something):

- Format: `Invalid[Noun]`, `Missing[Noun]`, `Unexpected[Noun]`
- Examples: `InvalidCikFormat`, `InvalidSecResponse`, `MissingOutput`

**Failed-First Pattern** (for describing *failed actions*):

- Format: `Failed[Action/Noun]`
- Examples: `FailedClientCreation`, `FailedRequestExecution`
- Avoid: `[Action]Failed`

**General Guidelines:**

- Keep error names concise but descriptive
- Be consistent within the same error domain or module
- Prefer specific names over generic ones

### Error Display Format

Error messages follow a consistent chaining convention:

- **Chaining errors** (wrapping an inner error): Use `Caused by:` to link to the next error in the chain. Do **not** wrap the inner error in quotes — it formats itself.
- **Leaf errors** (the root cause, no inner error): Use `Reason:` followed by a human-readable description. Primitive values may be quoted.

**Format pattern:**

- `[ErrorName] High-level description, Caused by: {inner_error}` — for errors that wrap another error
- `[ErrorName] High-level description, Reason: '{detail}'` — for leaf errors describing the root cause

No periods between segments — commas separate the description from `Caused by:` or `Reason:`. Every error segment must have a `[BracketedName]` prefix.

**Example chain:**

```text
[StateError] A state level error occurred, Caused by: [FailedRequestExecution] Failure in State: 'Execute SEC Request', Caused by: [FailedSecRequest] SEC request failed, Caused by: [InvalidResponse] Response validation failed, Caused by: [InvalidSecResponse] Invalid SEC Response, Reason: 'Expected a success status code (2xx), got '404' status code instead'
```

Use `thiserror` with `#[error("...")]` and `#[source]` for deriving `Display` and `Error` on error types.

---

## Library Code

### Testing

- Write a **comprehensive unit test suite** for all implemented code.
- Write **integration tests** where applicable.
- Include **doctests** where useful.
- Use **pretty assertions** (via `pretty_assertions` crate).
- Follow the **"Arrange, Define, Act, Assert"** pattern:
  - **Arrange**: Set up the test environment
  - **Define**: Define expected result (usually `expected_result`)
  - **Act**: Execute code under test, capture result (usually `result`)
    - Use `Result<T, E>` for error handling
  - **Assert**: Verify results
    - Use `assert_eq!`, `assert_ne!`, or `assert!(condition)`
    - **IMPORTANT**: Write **EXACTLY ONE** `assert!(...)` per test function
- Unit tests go in the same file as the code under a `#[cfg(test)]` module.
- Test function names use `should_..._when` in `snake_case`.
  - Names can be verbose — clarity over brevity.
- Integration tests go in the `tests/` directory, each in its own file.
- `.expect()` messages must explain **why the operation should not fail** in that context:
  - Bad: `.expect("should work")`
  - Good: `.expect("Given a valid hardcoded CIK, the creation of a CIK object should always succeed")`
  - Capitalize the first word. Do **NOT** end with a period.

---

## Application Code

### Logging

- Use **structured logging** in application code only (not in libraries).
- Logs must:
  - Be formatted as **JSON**
  - **Avoid sensitive data** (passwords, API keys, PII)
  - Use the correct log level:
    - `info`: Regular application state
    - `debug`: Development-level details
    - `warn`: Unexpected but non-breaking situations
    - `error`: Critical issues

### Structured Logging Format

All logs must be JSON with exactly **five required fields**:

- **`level`**: `info`, `debug`, `warn`, or `error`
- **`timestamp`**: ISO 8601 UTC (e.g., `2024-10-12T14:30:00Z`)
- **`event`**: `snake_case` singular noun identifier for the event type
- **`message`**: Human-readable summary of what happened
- **`context`**: Nested object with arbitrary key-value pairs for additional state

**Example:**

```json
{
  "level": "info",
  "timestamp": "2024-10-12T14:30:00Z",
  "event": "user_authentication_success",
  "message": "User successfully authenticated",
  "context": {
    "user_id": "12345",
    "session_id": "abc-xyz-789",
    "ip_address": "192.168.1.100"
  }
}
```

### Logging Infrastructure

- Use `log` crate as a **facade**
- Use `tracing` crate as the **implementation backend**

---

## Commit Guidelines

```sh
<type>[<scope>]: <short summary>

[<commit body>]

[<footer>]
```

### Commit Types

- **`feat`**: Adds new functionality
- **`fix`**: Fixes bugs (including linting errors)
- **`refactor`**: Improves existing code without adding functionality
- **`style`**: Formatting/indentation changes only (no logic changes)
- **`perf`**: Performance-focused refactoring
- **`test`**: Adds or modifies tests
- **`doc`**: Documentation-only changes
- **`ci`**: CI pipeline configuration changes
- **`cd`**: CD pipeline configuration changes
- **`build`**: Build system or dependency changes (e.g., `Cargo.toml`)
- **`revert`**: Reverts a previous commit
- **`chore`**: Routine maintenance not affecting app logic, CI/CD, or build output

### Scope (Optional)

- Package, service, or module name
- Examples: `feat(auth): add user login validation`

### Short Summary

- Imperative mood ("add", not "added")
- Under 72 characters

### Commit Body (Optional)

- Explain **why**, not what
- Separate from summary with a blank line

---

## PR Review Guidelines

### Code Quality Review

- Readability and maintainability
- Flag duplicated code
- Functions should be focused and not overly long
- Follow single responsibility principle
- Avoid side effects in functions
- Naming must follow **Ottinger's Naming Rules** (reveal intent, be pronounceable, avoid encodings)
- Code must follow the **Step-Down Rule** (higher-level concepts first, details later)

### Performance

- Flag inefficiencies; avoid premature optimization

### Correctness & Safety

- Spot potential bugs and edge cases
- Verify sufficient error handling

### Security

- Identify insecure practices (e.g., hardcoded secrets)
- Flag outdated or vulnerable libraries

### Style & Documentation

- Ensure style conventions are followed
- Check for meaningful comments and docstrings

### Documentation Consistency

- Cross-check code changes against `README.md`, API docs, and inline examples
- Flag when function names, parameters, or behaviors change but docs are not updated
- Ensure new features or breaking changes are properly documented

### Testing Review

- Confirm sufficient test coverage
- Suggest missing edge cases or error condition tests
- Verify tests follow **"Arrange, Define, Act, Assert"** pattern
- Verify each test has **EXACTLY ONE** `assert!(...)` statement
- Unit tests in same file; integration tests in `tests/` directory

### What NOT to Do

- Avoid nitpicks on trivial formatting
- Do not suggest unnecessary rewrites if code is clear and correct
- Do not enforce rules not listed in these guidelines

---

## Deviations

If you must deviate from any guideline, **include a code comment** explaining why. Consistency, safety, and clarity are the priorities in this project.
