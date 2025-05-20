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

---

## 📦 Library Code

### 🧪 Testing
- Write a **comprehensive unit test suite** for the implemented code.
- If applicable, write **integration tests**.
- Include **doctests** where useful.
- Use **pretty assertions** (e.g., via `pretty_assertions` crate) for improved readability.

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

## ⚠️ Deviations
If you must deviate from any guideline, **include a code comment** explaining why. Consistency, safety, and clarity are the priorities in this project.

---
