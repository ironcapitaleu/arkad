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
  
---

## 🖥️ Application Code

### 📈 Logging
- Use **structured logging** in application code only (not in libraries).
- Logs must:
  - Be formatted as **JSON**
  - Include consistent fields like:
    - `"severity"`
    - `"message"`
    - `"timestamp"` (ISO 8601 format)
  - **Avoid sensitive data**
  - Use the correct **log level**:
    - `INFO`: Regular application state (e.g., "Server started")
    - `DEBUG`: Development-level details (e.g., variable values)
    - `WARN`: Unexpected but non-breaking situations (e.g., deprecated usage)
    - `ERROR`: Critical issues (e.g., failure to connect to a database)
    - `TRACE`: Extremely detailed trace logs (e.g., function calls, loop iterations)

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

## ⚠️ Deviations
If you must deviate from any guideline, **include a code comment** explaining why. Consistency, safety, and clarity are the priorities in this project.

---
