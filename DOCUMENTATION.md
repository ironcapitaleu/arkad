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

### The W-Fragen Principle

Every doc comment answers a subset of these questions, in this priority order:

| Question | Answers | Required? |
|----------|---------|-----------|
| **Was?** (What?) | What does this item do or represent? | Always — this is the summary line |
| **Warum?** (Why?) | Why does it exist? What problem does it solve? | When the "what" alone doesn't justify the item's existence |
| **Wie?** (How?) | How does it achieve its purpose? What approach or delegation? | When the mechanism is non-obvious from the signature/type |
| **Wer?** (Who?) | Who uses this? Who implements this? | Primarily for traits and extension points |
| **Wann?** (When?) | Under what conditions is this triggered / produced? | Primarily for error variants and callbacks |
| **Wo?** (Where?) | Where does this fit in the system? Where does it originate? | When the item's placement is non-obvious |

Not every item needs all questions answered. Simple accessors need only *Was?*. Complex traits may need all six. The goal is that a reader never has to guess the answer to a question they'd naturally ask.

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

### Content Structure

Module docs follow a fixed three-part structure:

1. **Title** — `# Module Name` on the first line.
2. **What-sentence** — One sentence answering *"What does this module provide?"*. This is the summary that appears in parent module listings.
3. **Why/How paragraph** — After a blank line, two to three sentences answering *"Why does this module exist?"* and *"How does it achieve its purpose?"*. This grounds the reader in motivation and approach.

After the introductory prose, include structured sections:

4. **`## Modules`** — List of child modules with one-line descriptions. Use intra-doc links.
5. **`## See Also`** *(optional)* — Links to related modules, external crates, or design documents.

### Optional Sections (use when relevant)

- **`## Usage`** — Brief prose or code example showing how to use the module (answers *"How do I use this?"*).
- **`## Integration`** — How this module fits into the larger system (answers *"Where does this belong?"*).

### Template

```rust
//! # Crate / Module Name
//!
//! One sentence: what this module provides.
//!
//! Why it exists (the problem it solves or the responsibility it owns).
//! How it achieves that (the approach, key abstractions, or delegation strategy).
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

### Example

```rust
//! # Central Index Key (CIK) Utilities
//!
//! Provides the [`Cik`] type for parsing and validating SEC Central Index Keys.
//!
//! The SEC identifies every filer by a numeric CIK that must be exactly 10 digits,
//! zero-padded. This module encapsulates that invariant so downstream code can rely
//! on a [`Cik`] value being well-formed without re-validating.
//!
//! ## Modules
//!
//! - [`cik_error`]: Error types and reasons for invalid CIKs.
//! - [`constants`]: Formatting constants such as [`CIK_LENGTH`].
//!
//! ## See Also
//!
//! - [`crate::implementations::states::extract::validate_cik_format`]: State that uses [`Cik`] for input validation.
```

---

## Item-Level Documentation (`///`)

### Content Structure

Item docs follow the same W-Fragen principle as module docs, scaled to the item's complexity:

1. **What-sentence** (required) — One sentence answering *"What does this item do / represent?"*
   - Functions/methods: start with a verb in third-person present tense ("Creates", "Returns", "Validates").
   - Types (structs, enums, traits): describe what it *is* ("Input data for...", "Error variants for...").
   - Must not exceed ~80 characters.
   - Must be a complete sentence (capitalize, end with period).

2. **Why/How paragraph** (when the what-sentence is insufficient) — After a blank doc line, one to three sentences answering:
   - *"Why does this exist?"* — The motivation or responsibility.
   - *"How does it work?"* — The approach, delegation, or key constraint.

Skip the why/how paragraph for trivial items (simple accessors, single-field wrappers) where the what-sentence says everything.

### Examples

Minimal (accessor — what-sentence only):

```rust
/// Returns the observed value.
#[must_use]
pub const fn value(&self) -> i64 { ... }
```

Full (constructor — what + why/how):

```rust
/// Creates a new [`Cik`] from any value implementing [`ToString`].
///
/// Trims whitespace and zero-pads to 10 digits so that downstream code
/// can rely on a uniform format without re-validating.
///
/// # Errors
///
/// Returns a [`CikError`] if the input contains non-numeric characters
/// or exceeds the maximum allowed length.
pub fn new(cik: &(impl ToString + ?Sized)) -> Result<Self, CikError> { ... }
```

Type (struct — what + why/how):

```rust
/// Strongly-typed wrapper for a validated SEC Central Index Key (CIK).
///
/// Ensures that only valid, 10-digit, zero-padded numeric CIKs are constructed
/// and used throughout the library. Use [`Cik::new`] to construct and validate.
pub struct Cik { ... }
```

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
/// Creates a new [`Cik`] from any value implementing [`ToString`].
///
/// # Examples
///
/// ```
/// use sec::shared::cik::Cik;
///
/// let cik = Cik::new("123456789").expect("CIK creation with the hardcoded value should always succeed");
/// assert_eq!(cik.value(), "0123456789");
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
  /// SEC Format: "Company Name email@domain.com"
  /// ```
  ```

---

## Struct and Enum Documentation

### Structs

The struct-level doc follows the standard what/why/how structure. Fields answer *"What does this hold?"* and optionally *"Why is it here?"*.

```rust
/// Input data for the SEC request preparation state.
///
/// Bundles the validated CIK and shared HTTP client required by
/// [`PrepareSecRequest`](crate::implementations::states::extract::prepare_sec_request).
/// Passed in from the super-state context to avoid per-state client construction.
#[derive(Debug, Clone, PartialEq)]
pub struct PrepareSecRequestInput {
    /// The validated CIK that will be used for the SEC API request.
    pub validated_cik: Cik,
    /// The shared HTTP client passed down from the super-state context.
    pub sec_client: SecClient,
}
```

- Document **every** public field with `///`.
- The field doc answers *"What does this hold?"* and, if non-obvious, *"Why is it here?"* or *"Where does it come from?"*.

### Enums

Each variant answers *"When does this occur?"* (the triggering condition). Named fields within variants answer *"What does this carry?"*.

```rust
/// Time period for a financial observation.
///
/// Distinguishes between point-in-time snapshots and measurements over a date range.
/// SEC XBRL data uses instant periods for Balance Sheet items and duration periods
/// for Income Statement and Cash Flow items.
#[non_exhaustive]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize)]
pub enum Period {
    /// A point-in-time snapshot (e.g., Balance Sheet items).
    Instant {
        /// The date of the measurement.
        date: NaiveDate,
    },
    /// A measurement over a date range (e.g., Income Statement items).
    Duration {
        /// The start of the measurement period (inclusive).
        start: NaiveDate,
        /// The end of the measurement period (inclusive).
        end: NaiveDate,
    },
}
```

Error enum example:

```rust
/// Reason for an invalid CIK format.
#[non_exhaustive]
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum InvalidCikReason {
    /// The CIK exceeds the maximum allowed digit count.
    MaxLengthExceeded {
        /// The actual length of the provided CIK string.
        cik_length: usize,
    },
    /// The CIK contains non-numeric characters.
    ContainsNonNumericCharacters,
}
```

- Document **every** variant with `///`.
- Document **every** named field within variants.
- For unit variants, the variant doc is sufficient.

---

## Trait Documentation

Traits define contracts for multiple implementors, so their docs must answer additional questions:

- **What** — What responsibility does this trait represent?
- **Why** — Why is this a trait rather than a concrete type? (polymorphism, testability, decoupling)
- **Who** — Who implements it? (brief mention of key implementors or the intended audience)
- **How** — How should an implementor fulfil the contract? (key invariants, ordering constraints)

```rust
/// Defines the interface of an HTTP response from the SEC API.
///
/// Exists as a trait to decouple the library from any specific HTTP client
/// (e.g., `reqwest`), enabling unit-testable states that operate on response
/// data without requiring live network calls. Implementors must consume an
/// inner response via [`SecResponse::from_inner`] and expose its parts
/// through the accessor methods.
///
/// # Associated Types
///
/// - `Inner`: The raw HTTP response type consumed during construction. Must implement [`InnerResponse`].
/// - `Url`: The URL type of the response.
/// - `Headers`: The headers type of the response.
/// - `StatusCode`: The HTTP status code type.
/// - `ContentType`: The content type type.
/// - `Error`: Errors that can occur when processing the response.
#[async_trait]
pub trait SecResponse: Send + Sync + Debug + Sized {
    type Inner: InnerResponse;
    type Error;
    // ...

    /// Consumes the inner response and constructs a new [`SecResponse`] instance.
    ///
    /// Asynchronous because reading the response body may involve I/O.
    ///
    /// # Errors
    ///
    /// Returns `Self::Error` if the response body cannot be read or parsed.
    async fn from_inner(inner: Self::Inner) -> Result<Self, Self::Error>;

    /// Returns a reference to the response body as a valid JSON value.
    fn body(&self) -> &serde_json::Value;
}
```

- Document each associated type in the trait-level doc under `# Associated Types`.
- Document each method with its own `///` including `# Returns` and `# Errors` where applicable.
- If a trait has a default implementation, state whether implementors should override it and when.

---

## Error Documentation

Error types follow additional conventions specific to this project.

### Enum-Level

The error enum doc answers:
- **What** — What category of failures does this enum cover?
- **Where** — Where in the system do these errors originate?

### Variant-Level

Each error variant answers:
- **When** — Under what condition is this variant produced?
- **What** — What do the named fields carry? (for struct variants)

### `# Errors` Section on Functions

Functions returning `Result` must list possible error variants, answering *"When can this fail and with what?"*:

```rust
/// # Errors
///
/// Returns a [`CikError`] if the input contains non-numeric characters
/// ([`InvalidCikReason::ContainsNonNumericCharacters`]) or exceeds the
/// maximum allowed length ([`InvalidCikReason::MaxLengthExceeded`]).
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
- [ ] Field-level docs exist on all public struct fields and enum variant fields.
- [ ] Each doc comment answers the relevant W-Fragen (at minimum *Was?*).
