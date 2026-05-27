# Documentation Guidelines

This document defines the standards for writing Rust documentation in this project.
It covers module-level docs, item-level docs, sections, linking, doc-tests, and error documentation.

---

## General Principles

- Every public item in a library crate **must** have a doc comment.
- Documentation describes the **contract** (what it does, guarantees, and constraints), not the implementation.
- Write for a reader who understands Rust but is unfamiliar with this codebase.
- Keep the first line short and self-contained as `rustdoc` uses it as a preview in module listings.
- Use present tense, third-person: "Returns the validated CIK", not "This will return..." or "Return the CIK".

---

## Comment Syntax

| Scope | Syntax | Placement |
|-------|--------|-----------|
| Item-level (structs, enums, traits, functions, methods, fields) | `///` | Directly above the item |
| Module / crate-level | `//!` | At the top of `lib.rs` or `mod.rs` |

Never use `/** */` block comments for documentation. They are harder to maintain and conflict with `rustfmt`.

---

## Module-Level Documentation (`//!`)

Every `lib.rs` and every `mod.rs` that introduces a logical grouping **must** have module-level documentation.

### Required Sections

1. **Title** — `# Module Name` on the first line.
2. **Summary** — One to three sentences describing the module's purpose.
3. **`## Modules`** — List of child modules with one-line descriptions. Use intra-doc links.
4. **`## See Also`** *(optional)* — Links to related modules, external crates, or design documents.

### Optional Sections (use when relevant)

- **`## Features`** — Key capabilities the module provides.
- **`## Usage`** — Brief prose or code example showing how to use the module.
- **`## Integration`** — How this module fits into the larger system.
- **`## Examples`** — Pointer to tests or inline examples.

### Template

```rust
//! # Crate / Module Name
//!
//! Brief description of what this module provides and why it exists.
//!
//! ## Modules
//!
//! - [`child_a`]: One-line description.
//! - [`child_b`]: One-line description.
//!
//! ## See Also
//!
//! - [`crate::related_module`]: Why this is related.
```

---

## Item-Level Documentation (`///`)

### Summary Line

The first paragraph (up to the first blank `///` line) is the **summary**. It must:

- Start with a verb in third-person present tense (e.g., "Creates", "Returns", "Validates").
- Not exceed ~80 characters.
- Be a complete sentence (capitalize, end with period).

```rust
/// Returns a reference to the validated CIK.
```

For types (structs, enums, traits), the summary describes **what it is**, not a verb:

```rust
/// Input data for preparing SEC API requests.
pub struct PrepareSecRequestInput { ... }
```

### Extended Description

After a blank doc line, provide additional context if the summary alone is insufficient.
Keep it concise. Prefer bullet points over paragraphs for multiple pieces of information.

---

## Standard Sections

Use the following `# Heading` sections **in this order** when applicable.
Only include sections that provide value. Omit empty ones

| Section | When to include |
|---------|-----------------|
| `# Type Parameters` | Generic types that need explanation beyond their trait bounds |
| `# Arguments` | Functions/methods with non-obvious parameters |
| `# Returns` | When the return value needs clarification beyond the type signature |
| `# Errors` | Any function returning `Result`. List each error variant and when it occurs |
| `# Panics` | Any code path that can panic. Document the condition |
| `# Safety` | `unsafe` functions. Document the invariants the caller must uphold |
| `# Examples` | Public API entry points, constructors, and non-trivial methods |

### Section Formatting Rules

- Use `# Heading` (single `#`) inside doc comments. `rustdoc` renders these as `<h2>` within the item.
- List items use `* item` or `- item` consistently (this project uses `-`).
- Wrap parameter names in backticks: `` `validated_cik` ``.

---

## Intra-Doc Links

Use Rustdoc's intra-doc link syntax to create hyperlinks within the generated documentation.
These links are **checked at compile time** when `rustdoc` lints are enabled.

### Syntax

| Target | Syntax |
|--------|--------|
| Type in same crate | `` [`TypeName`] `` |
| Module (relative) | `` [`module_name`] `` |
| Module (absolute) | `` [`crate::path::to::module`] `` |
| Method | `` [`TypeName::method_name`] `` |
| Trait method | `` [`Trait::method`] `` |
| Enum variant | `` [`EnumName::Variant`] `` |
| Field | `` [`StructName::field`] `` |
| Renamed link | `` [`display text`](crate::path::Type) `` |
| External crate type | `` [`external_crate::Type`] `` |

### Rules

- Prefer short-form links (`` [`Type`] ``) when the target is unambiguous in scope.
- Use fully-qualified paths (`` [`crate::module::Type`] ``) when linking across module boundaries to avoid ambiguity.
- Use renamed links when the fully-qualified path would hurt readability in prose.
- All intra-doc links **must resolve**. Broken links are treated as errors in CI.

## Doc-Tests

Doc-tests serve dual purposes: they demonstrate usage and they verify the example compiles and runs.

### When to Write Doc-Tests

- **Required** on public constructors and primary API entry points.
- **Recommended** on any method whose usage is non-obvious from the signature alone.
- **Optional** on trivial accessors (prefer no example over a meaningless one).

### Formatting

```rust
/// Creates a new instance.
///
/// # Examples
///
/// ```
/// use sec::shared::cik::Cik;
///
/// let cik = Cik::new("1067983").expect("Hardcoded CIK should be valid");
/// assert_eq!(cik.as_str(), "CIK0001067983");
/// ```
```

### Rules

- Import paths must be fully qualified from the crate root (as an external consumer would write them).
- Use `#` to hide boilerplate lines that distract from the example's point:
  ```rust
  /// ```
  /// # use sec::shared::cik::Cik;
  /// let cik = Cik::new("1067983").expect("Hardcoded CIK should be valid");
  /// ```
  ```
- `.expect()` messages in doc-tests must explain **why** the operation should not fail (same rule as test code).
- Mark examples that should not run (e.g., they require network) with `no_run`:
  ```rust
  /// ```no_run
  /// // Example that requires a live SEC connection
  /// ```
  ```
- Mark examples that should not even compile (illustrating incorrect usage) with `compile_fail`:
  ```rust
  /// ```compile_fail
  /// // This demonstrates what NOT to do
  /// ```
  ```
- Mark examples that are not Rust code with the appropriate language tag or `text`:
  ```rust
  /// ```text
  /// ErrorKind
  ///   ::Xbrl(XbrlErrorKind)
  /// ```
  ```

---

## Struct and Enum Documentation

### Structs

```rust
/// Input data for preparing SEC API requests.
///
/// Holds the validated CIK and shared HTTP client required by the
/// [`PrepareSecRequest`](crate::implementations::states::extract::prepare_sec_request) state.
#[derive(Debug, Clone, PartialEq)]
pub struct PrepareSecRequestInput {
    /// The validated CIK that will be used for the SEC API request.
    pub validated_cik: Cik,
    /// The shared HTTP client passed down from the super-state context.
    pub sec_client: SecClient,
}
```

- Document **every** public field with `///`.
- The field doc describes what it holds and, if non-obvious, why.

### Enums

```rust
/// Specific parsing failure variants.
#[derive(Debug, PartialEq, Error)]
pub enum ParseErrorKind {
    /// The input cannot be parsed as valid JSON (malformed bytes).
    #[error("[InvalidJson] Failed to parse JSON body, Reason: '{reason}'")]
    InvalidJson {
        /// Description of the deserialization failure.
        reason: String,
    },
}
```

- Document **every** variant with `///`.
- Document **every** named field within variants.
- For unit variants, the variant doc is sufficient.

---

## Trait Documentation

Traits require extra care because they define contracts for multiple implementors.

```rust
/// Defines the behavior of a state within the state machine.
///
/// # Associated Types
///
/// - `InputData`: The data this state processes. Must implement [`StateData`].
/// - `OutputData`: The data this state produces. Must implement [`StateData`].
/// - `Context`: The environment data available during execution. Must implement [`Context`].
///
/// # Required Traits
///
/// Implementors must also satisfy: `Debug + Send + Sync + Clone + Eq + Hash + Ord`.
pub trait State: Debug + Send + Sync + Clone + Eq + Hash + Ord {
    type InputData: StateData;
    // ...
}
```

- Document each associated type in the trait-level doc under `# Associated Types`.
- Document each method with its own `///` including `# Returns` and `# Errors` where applicable.
- If a trait has a default implementation, state whether implementors should override it and when.

---

## Error Documentation

Error types follow additional conventions specific to this project.

### Variant-Level

Each error variant must document:
1. **When** this error occurs (the triggering condition).
2. **What** the named fields mean.

### `# Errors` Section on Functions

Functions returning `Result` must list possible error variants:

```rust
/// # Errors
///
/// Returns [`ErrorKind::InvalidCikFormat`] if the input string is empty or
/// contains non-numeric characters after stripping the optional "CIK" prefix.
```

---

## Builder and Updater Patterns

For types following the Data / Updater / UpdaterBuilder pattern:

- The **module doc** must list all three types and their roles.
- The **Updater** doc must state that `None` fields are not applied.
- The **Builder** doc must link back to the Updater it constructs.
- Each builder method must document the field it sets.

---

## Re-exports and Preludes

- Prelude modules (`prelude.rs`) must have module-level documentation explaining what is re-exported and why.
- Individual `pub use` re-exports do **not** need their own doc comment

## What Not to Document

- **Private items** — Document only if the logic is subtle enough to warrant it.
- **Obvious accessors** — A one-line summary suffices; skip `# Arguments` / `# Returns` / `# Examples`.
- **Trait implementations** — Only document if the implementation has surprising behavior or deviates from the trait's contract. Standard `From`, `Display`, `Default` implementations typically need no doc.
- **Test modules** — `#[cfg(test)] mod tests` does not need documentation.

---

## CI Enforcement

The following must pass in CI without warnings:

```sh
cargo doc --no-deps --document-private-items --workspace
```

Relevant `rustdoc` lints treated as errors:
- `broken_intra_doc_links`
- `private_intra_doc_links`
- `redundant_explicit_links`

---

## Checklist for New Code

Before submitting a PR, verify:

- [ ] Every public item has a `///` doc comment with a summary line.
- [ ] Every `mod.rs` / `lib.rs` has `//!` module-level documentation.
- [ ] All intra-doc links resolve (`cargo doc` passes).
- [ ] `# Errors` section exists for all `Result`-returning public functions.
- [ ] `# Panics` section exists for any function that can panic.
- [ ] Doc-tests exist for constructors and primary API methods.
- [ ] Error modules include a hierarchy diagram.
- [ ] Field-level docs exist on all public struct fields and enum variant fields.
