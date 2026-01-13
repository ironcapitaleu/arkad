# 🛠️ Copilot Instructions – Development Guidelines for Rust Projects

> Use this file to guide GitHub Copilot and developers to follow consistent, high-quality practices when writing or updating code in this project.

---

## ✅ General Rules (For All Code)

### 🧹 Code Quality
- The code is **properly formatted** (`rustfmt`).
- Dependencies must be **free of known security vulnerabilities** (`cargo audit`).
- Code **compiles without errors** and passes:
  - Linting (`clippy`)
  - Unit tests
  - Integration tests
  - Doctests

### 📚 Documentation
- Public items in libraries **must include docstrings** (`///`).
- All implementation changes must be **reflected in documentation**, including:
  - Docstrings
  - Design documents (if applicable, e.g., mermaid diagrams)
- **Documentation must be version controlled**.

## 📥 Import Order Conventions

To ensure readability and consistency, all Rust imports in this project must be grouped and ordered as follows:

1. **Standard Library Imports**  
   All imports from the Rust standard library (e.g., `std::collections::HashMap`, `std::fmt`) should appear first.

2. **External Crate Imports**  
   Imports from third-party crates (e.g., `serde`, `tracing`, `pretty_assertions`) should follow, grouped together.

3. **Internal Crate Imports**  
   Imports from within this workspace or crate (e.g., `crate::module::Type`, `super::submodule`) should come last.

Within each group, order imports alphabetically by path. Separate each group with a single blank line for clarity.

After the final line of imports, new module declarations (e.g., `mod foo;`) can be made. Imports, and public re-exports (e.g., `pub use foo::Bar;`) should be placed after the module declarations in this case.

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

## Naming Conventions

### Error Types
Error types must follow consistent naming patterns based on the kind of error they represent:

**Adjective-First Pattern** (for describing the *state* of something):
- Use when the error represents an invalid or unexpected *quality* of data or state
- Format: `Invalid[Noun]`, `Missing[Noun]`, `Unexpected[Noun]`
- Examples: `InvalidCikFormat`, `InvalidSecResponse`, `InvalidInput`, `MissingOutput`

**Failed-First Pattern** (for describing *failed actions*):
- Use when the error represents a specific action or operation that failed
- Format: `Failed[Action/Noun]`
- Examples: `FailedClientCreation`, `FailedRequestExecution`, `FailedOutputComputation`
- Avoid: `[Action]Failed`

**General Guidelines:**
- Keep error names concise but descriptive
- The error type name should clearly indicate what went wrong
- Be consistent within the same error domain or module
- Prefer specific names over generic ones (e.g., `FailedClientCreation` over `CreationError`)

---

## 📦 Library Code

### 🧪 Testing
- Write a **comprehensive unit test suite** for the implemented code.
- If applicable, write **integration tests**.
- Include **doctests** where useful.
- Use **pretty assertions** (e.g., via `pretty_assertions` crate) for improved readability.
- Unit tests should follow a modified version of the  **"Arrange, Act, Assert"** pattern, that we call the **"Arrange, Define, Act, Assert"** pattern:
  - **Arrange**: Set up the test environment (create any necessary objects, mimic dependencies, etc.)
  - **Define**: Define the expected result (usually in a variable called `expected_result`)
  - **Act**: Execute the code under test and capture the result (usually in a variable called `result`)
    - **Note**: Use `Result<T, E>` for error handling
  - **Assert**: Verify the results (i.e., that the `result` matches the `expected_result`)
    - **Note**: Use `assert_eq!` for equality checks, `assert_ne!` for inequality checks, and `assert!(condition)` for boolean checks.
- Unit tests should be placed in the same file as the code they test, using a `#[cfg(test)]` module.
- Unit tests should follow the `should_..._when` naming convention for the test function names, where `...` is a description of the expected behavior.
  - **Note**: Test function names should be in `snake_case` and start with `should_`. Test names can be verbose, explicit but clear naming is favored over brevity.
- Integration tests should be placed in the `tests` directory, with each test in its own file.
- When using `.expect()` in tests, provide **clear, specific messages** that explain **why the operation should not fail** given the test context:
  - Bad example: `.expect("should work")`, `.expect("should create it")`
  - Good example: `.expect("Given a valid hardcoded CIK, the creation of a CIK object should always succeed")`
  - Good example: `.expect("Parsing a well-formed JSON response with status 200 should never fail")`
  - The message should reference the specific test setup and why failure is unexpected in that scenario. Use capitalized first words. Do **NOT** end with a period.
  
---

## 🖥️ Application Code

### 📈 Logging
- Use **structured logging** in application code only (not in libraries).
- Logs must:
  - Be formatted as **JSON**
  - Include **specific fields** (see format below)
  - **Avoid sensitive data**
  - Use the correct **log level**:
    - `info`: Regular application state (e.g., "Server started")
    - `debug`: Development-level details (e.g., variable values)
    - `warn`: Unexpected but non-breaking situations (e.g., deprecated usage)
    - `error`: Critical issues (e.g., failure to connect to a database)

## 📝 Structured Logging Format

All structured logs must be formatted as **JSON documents** with exactly **five required fields**:

- **`level`**: Log severity level
  - Valid values: `info`, `debug`, `warn`, `error`
- **`timestamp`**: When the event occurred
  - Format: **ISO 8601 UTC** (e.g., `2024-10-12T14:30:00Z`)
- **`event`**: The specific event that triggered log creation
  - Brief, descriptive identifier for the event type
  - Use singular nouns (e.g., "user_login" instead of "user_logins")
  - A set of predefined event names should be maintained and used consistently - likely maintained as a non-exhaustive enum with a catch_all variant for unknown events (that can be extended in the future)
- **`message`**: High-level information about the `event`
  - Human-readable summary of what happened
  - free text string explaining the `event`
- **`context`**: Detailed contextual information
  - Nested field that can contain arbitrary key-value pairs
  - Additional state information needed to understand the event
  - Include relevant variables, IDs, or environmental details

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

### ⚙️ Logging Infrastructure
- Use `log` crate as a **facade**
- Use `tracing` crate as the **implementation backend**

---

## 💡 Copilot Guidance

When Copilot generates code, it should:
- Follow existing conventions and module structure
- Include docstrings for public items
- Generate unit tests (and integration tests if relevant)
- Add structured logging in application code
- Avoid logging or exposing sensitive data
- Prefer `Result<T, E>` for error handling with meaningful error types

---

## PR Review Guidelines

### ✅ Code Quality
- Readability and maintainability  
- Flag duplicated code  
- Ensure functions are focused and not overly long
- Make sure structs and functions follow the single responsibility principle
- Avoid side effects in functions
- Naming of variables, structs, and functions must follow **Ottinger’s Naming Rules** ([what they are](https://objectmentor.com/resources/articles/naming.htm)) (names should reveal intent, be pronounceable, avoid encodings, not be too cute, etc.)  
- Code within files and modules must follow the **Step-Down Rule** ([see explanation](https://dzone.com/articles/the-stepdown-rule)) (functions stay on one layer of abstraction, inside a module order functions by higher-level concepts first, details later)


### ⚡ Performance
- Flag inefficiencies  
- Avoid premature optimization  

### 🛡️ Correctness & Safety
- Spot potential bugs and edge cases  
- Verify sufficient error handling  

### 🔒 Security
- Identify insecure practices (e.g., hardcoded secrets)  
- Flag outdated or vulnerable libraries  

### 📘 Style & Documentation
- Ensure style conventions are followed  
- Check for meaningful comments and docstrings  
- Suggest clearer names and documentation where needed  

### 🧾 Documentation Consistency
- Cross-check code changes against `README.md`, API docs, usage guides, and inline examples  
- Flag when function names, parameters, or behaviors change but docs are not updated  
- Highlight outdated instructions or examples caused by code changes  
- Ensure new features or breaking changes are properly documented  

### 🧪 Testing
- Confirm sufficient test coverage  
- Suggest missing edge cases or error condition tests  

### 🛑 What NOT to Do
- Avoid nitpicks on trivial formatting  
- Do not suggest unnecessary rewrites if code is clear and correct  
- Do not enforce rules not listed in these guidelines  

---

## 📝 Commit Guidelines

All commits must follow the following format:

```
<type>[<scope>]: <short summary>

[<commit body>]

[<footer>]
```

### 🏷️ Commit Types

- **`feat`**: Adds new functionality to code by adding functions or features
- **`fix`**: Restores intended functionality by fixing bugs (including linting errors) - does not intentionally add new functionality
- **`refactor`**: Improves existing code without adding functionality by, for example:
  - Simplifying code structure
  - Improving readability
  - Improving performance (time/space)
  - Reducing dependencies
  - etc.
- **`style`**: A specific type of refactoring. Formatting, indentation, or code style changes (no logic changes)
- **`perf`**: A specific type of refactoring. Performance-focused refactoring without changing external behavior
- **`test`**: Adds or modifies automated tests (specifications for expected behavior)
- **`docs`**: Changes only to software documentation. Usually in the form of docstrings or markdown files (design docs, README, etc.)
- **`ci`**: **Direct** changes to Continuous Integration pipeline configuration
- **`cd`**: **Direct** changes to Continuous Deployment pipeline configuration
- **`build`**: Changes affecting build system or dependencies. Changes to resulting build output, i.e., the binary. (e.g., updating/ adding a new library dependency, changing compiler flags). Usually in `Cargo.toml`, `Cargo.lock` or `.cargo/config.toml`.
- **`revert`**: Reverts a previous commit
- **`chore`**: Catchall commit type. For routine maintenance tasks not affecting app logic, CI/CD, or build output. (e.g., updating files such as `.gitignore`, LICENSE files, generic project management templates, or updating automation scripts like `Makefile` files or similar.)

### 🎯 Scope (Optional)

Add scope for area-specific changes when it helps understanding:
- Package names
- Service or module names
- Component areas

**Examples:**
- `feat(auth): add user login validation`
- `fix(database): handle connection timeout errors`

### ✍️ Short Summary

- Use **imperative mood** ("add", not "added" or "adds")
- Keep under **72 characters**
- Be descriptive but concise

### 📄 Commit Body (Optional)

- Explain **why** the change was made, not what it does
- Use when additional context is needed
- Separate from summary with blank line

### 📄 Footer (Optional)
- Reference issues or breaking changes
- Use when relevant for context

**Example Commit:**
```
fix: prevent racing of requests

Introduce a request id and a reference to latest request. Dismiss
incoming responses other than from latest request.

Remove timeouts which were used to mitigate the racing issue but are
obsolete now.

Reviewed-by: Z
Refs: #123
```

---

## ⚠️ Deviations
If you must deviate from any guideline, **include a code comment** explaining why. Consistency, safety, and clarity are the priorities in this project.

---
