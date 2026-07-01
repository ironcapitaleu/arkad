# Documentation Guidelines

This document defines the standards for writing Rust documentation in this project. It tells you **what** to document, **how** to structure it, and **where** to place doc-tests.

The document is organized as follows:

1. **General Principles** — the W-Fragen framework and documentable items overview
2. **Per-Item Sections** — modules, functions, structs, enums, traits, constants
3. **Cross-Cutting Reference** — intra-doc links, doc-test mechanics, CI, checklist

---

## General Principles

- Every public item in a library crate **must** have a doc comment.
- Documentation describes the **contract** (what it does, guarantees, and constraints), not the implementation.
- Write for a reader who understands Rust but is unfamiliar with this codebase.
- Keep the first line short and self-contained as `rustdoc` uses it as a preview in module listings.
- Use present tense, third-person: "Returns the validated CIK", not "This will return..." or "Return the CIK".
- Choose `a`/`an` by the **spoken sound**, not the first letter. "SEC" is read "ess-ee-see", so write "an SEC response", not "a SEC response".
- Prefer clarity over terseness. A few extra words that remove ambiguity beat a tighter phrase that reads as jargon (e.g. "concepts that are missing inside a response" over "concepts missing from a response").
- When tightening documentation that is already good, **blend** rather than replace: keep the established, well-worded opening and enrich it, instead of rewriting it wholesale.

### Documentable Items

The following item kinds are used in this project and require documentation when public:

| Item kind | Example | Doc syntax |
|-----------|---------|------------|
| Module | `mod cik;` | `//!` (inside the module file) |
| Function / Method | `fn validate() {}` | `///` (above) |
| Struct | `struct Cik { value: String }` | `///` (above struct + each public field) |
| Enum | `enum StatusCode { Ok, NotFound }` | `///` (above enum + each variant + variant fields) |
| Trait | `trait SecResponse {}` | `///` (above trait + each method + associated types) |
| Implementation | `impl SecResponse for ... {}` | `///` only when behavior is surprising |
| Type alias | `type StateMachineStream = Pin<...>;` | `///` (above) |
| Constant | `const STATE_NAME: &str = "...";` | `///` (above) |
| Re-export | `pub use cik::Cik;` | Not required (target item's doc is used) |

### The W-Fragen Principle

Every doc comment answers a subset of these questions, in this priority order:

| Question | Answers | Applicable items | Required? |
|----------|---------|------------------|-----------|
| **Was?** (What?) | What does this item do or represent? | All | Always — this is the summary line |
| **Warum?** (Why?) | Why does it exist? What problem does it solve? | Modules, structs, traits, enums, type aliases | When the "what" alone doesn't justify the item's existence |
| **Wie?** (How?) | How does it achieve its purpose? What approach or delegation? | Modules, functions, traits | When the mechanism is non-obvious from the signature/type |
| **Wer?** (Who?) | Who uses this? Who implements this? | Traits, modules | Primarily for traits and extension points |
| **Wann?** (When?) | Under what conditions is this triggered / produced? | Enum variants, functions | Primarily for error variants and callbacks |
| **Wo?** (Where?) | Where does this fit in the system? Where does it originate? | Modules, structs, constants | When the item's placement is non-obvious |

Not every item needs all questions answered. Simple accessors need only *Was?*. Complex traits may need all six. The goal is that a reader never has to guess the answer to a question they'd naturally ask.

### Comment Syntax

| Scope | Syntax | Placement |
|-------|--------|-----------|
| Item-level (structs, enums, traits, functions, methods, fields) | `///` | Directly above the item |
| Module / crate-level | `//!` | At the top of `lib.rs` or `mod.rs` |

Never use `/** */` block comments for documentation. They are harder to maintain and conflict with `rustfmt`.

---

## Modules

Modules are both documentable items (with their own `rustdoc` page) and containers for other items. The `//!` syntax documents *the module itself*, while `///` documents the items inside it.

Every `lib.rs` and every `mod.rs` **must** have module-level documentation.

### Module Categories

This project uses two kinds of modules:

| Category | Purpose | Example |
|----------|---------|---------|
| **Grouping module** | Organizes multiple related child modules under a shared namespace | `shared/mod.rs` (groups `cik`, `http_client`, `response`, etc.) |
| **Single-type module** | Namespaces one primary struct or enum with its supporting code | `fiscal_year/mod.rs` (holds `FiscalYear` newtype + impls + tests) |

Both require module docs, but at different levels of detail:
- **Grouping modules** need the full structure: title, what, why/how, `## Modules`, `## See Also`.
- **Single-type modules** need only: title + what-sentence. The type inside carries the bulk of documentation.

### Content Structure

Module docs follow a fixed structure:

1. **Title** — `# Module Name` on the first line.
2. **What-sentence** — One sentence answering *"What does this module provide?"*. This is the summary that appears in parent module listings.
3. **Why/How paragraph** — After a blank line, two to three sentences answering *"Why does this module exist?"* and *"How does it achieve its purpose?"*. (Can be omitted for single-type modules.)

After the introductory prose, include structured sections:

4. **`## Modules`** — List of child modules with one-line descriptions. Use intra-doc links. (Only for grouping modules.)
5. **`## See Also`** *(optional)* — Links to related modules, external crates, or design documents.

Optional additional sections:

- **`## Usage`** — Brief prose or code example showing how the module's items compose together.
- **`## Integration`** — How this module fits into the larger system.

### Doc-Tests in Modules

Doc-tests are **not required** in module-level documentation. When they are present, they belong in a `## Usage` section.

Module-level doc-tests make sense when **multiple items must be imported and composed together** to demonstrate the module's purpose — something that no single item's doc-test could show on its own. For example:

- Showing how to construct a state with specific config, then run the state machine computation.
- Demonstrating how a prelude import + multiple types work together as a workflow.

If the usage can be shown on a single function or constructor, prefer placing the doc-test there instead.

A module-level example should **demonstrate construction and composition, not assert on results**.
Show how the pieces are built and wired together; do not tack an `assert_eq!` on the end (that
reads as a test, not documentation). Match the plain construction style of sibling module examples.
Result-checking assertions belong in a constructor's or method's own `# Examples` doc-test, where
they illustrate the return value.

### Template (Grouping Module)

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

### Template (Single-Type Module)

```rust
//! # Type Name
//!
//! One sentence: what the primary type in this module represents.
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

## Functions and Methods

### W-Fragen

- **Was?** — Always. Start with a verb in third-person present tense ("Creates", "Returns", "Validates").
- **Warum?/Wie?** — When the function's purpose or mechanism is non-obvious from the signature.
- **Wann?** — For callbacks or conditionally-invoked methods.

### Content Structure

1. **What-sentence** (required) — Must not exceed ~80 characters. Must be a complete sentence.
2. **Why/How paragraph** (when needed) — After a blank doc line.
3. **`# Sections`** (when applicable, in this order):

| Section | When to include |
|---------|-----------------|
| `# Arguments` | Parameters that stay ambiguous even after a clear signature — first try to make the signature self-documenting (see below) |
| `# Returns` | When the return value needs clarification beyond the type |
| `# Errors` | Any function returning `Result` — list each error variant and when it occurs |
| `# Panics` | Any code path that can panic — document the condition |
| `# Examples` | Constructors and primary entry points (see doc-tests below) |

Prefer making the **signature self-documenting** over adding `# Arguments` / `# Returns`. Rename a
parameter so its name and type carry the meaning (e.g. `domain_error: CikError` instead of a bare
`err`), and a one-line summary is then enough on its own. Reach for `# Arguments` / `# Returns` only
when a signature genuinely cannot be made clear — then do add them.

### Doc-Tests

- **Required** on public constructors (`fn new`) and primary API entry points.
- **Required** on any fallible function whose error conditions are non-obvious.
- **Optional** on trivial accessors — prefer no example over a meaningless one.

Place doc-tests on the function itself under `# Examples`, not on the containing struct or module.

### Constructors

- Keep the summary a **one-liner that names the constructed type** with an intra-doc link:
  `Creates a new [`InvalidCikFormat`] error.` — not a generic `Creates a new error from ...`.
- Include a descriptive qualifier when it adds meaning: `Creates a new state-level [`InvalidCikFormat`] error.`
- With a self-documenting signature (see above), the one-liner needs no `# Arguments` / `# Returns`.

### Conversions

Conversion methods get a **one-liner naming both source and target** with intra-doc links, even
though they are trait-impl methods. Do not leave them undocumented.

- `From::from` — `Converts an [`InvalidCikFormat`] into a [`StateError::InvalidCikFormat`] variant.`
  (or `Wraps the error in the [`StateError::IncompleteCompanyFacts`] variant.`)
- `TryFrom::try_from` — same shape, plus an `# Errors` line for the failure case.
- Domain conversion traits (e.g. `FromDomainError::from_domain_error`) — `Converts a domain-level [`CikError`] into a state-level [`InvalidCikFormat`] error.`

### Examples

Minimal (accessor — what-sentence only):

```rust
/// Returns the observed value.
#[must_use]
pub const fn value(&self) -> i64 { ... }
```

Full (constructor — what + why/how + # sections + doc-test):

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
///
/// # Examples
///
/// ```
/// use sec::shared::cik::Cik;
///
/// let cik = Cik::new("123456789").expect("CIK creation with the hardcoded value should always succeed");
/// assert_eq!(cik.value(), "0123456789");
/// ```
pub fn new(cik: &(impl ToString + ?Sized)) -> Result<Self, CikError> { ... }
```

---

## Structs

### W-Fragen

- **Was?** — What does this type represent?
- **Warum?** — Why does it exist as its own type? (especially for newtypes/wrappers)
- **Wie?** — How is it constructed? (point to the constructor)

### Content Structure

1. **What-sentence** on the struct.
2. **Why/How paragraph** when the what-sentence is insufficient.
3. **Field docs** — Every public field gets `///` answering *"What does this hold?"* and optionally *"Where does it come from?"*.

When a newtype exists specifically to impose a stable `Display` format, **document the concrete
output shape** rather than narrating why the newtype exists:

```rust
/// Formats as `["Revenue", "Total Assets"]`: brackets around the list, quotes around items, comma-separated.
```

Showing the reader the exact shape is more useful than explaining the design decision behind it.

### Applicable `# Sections`

| Section | When to include |
|---------|-----------------|
| `# Examples` | Only if there is no constructor method — otherwise place the doc-test on the constructor |

### Doc-Tests

Place doc-tests on the **constructor method** (`fn new`), not on the struct declaration itself. The struct's `///` describes what it *is*; the constructor's `///` shows how to *use* it.

### Example

```rust
/// Strongly-typed wrapper for a validated SEC Central Index Key (CIK).
///
/// Ensures that only valid, 10-digit, zero-padded numeric CIKs are constructed
/// and used throughout the library. Use [`Cik::new`] to construct and validate.
pub struct Cik { ... }
```

With fields:

```rust
/// Input data for the SEC request preparation state.
///
/// Bundles the validated CIK and shared HTTP client required by
/// [`PrepareSecRequest`](crate::implementations::states::extract::prepare_sec_request).
/// [`SecClient`] passed in from the super-state context to avoid per-state client construction.
#[derive(Debug, Clone, PartialEq)]
pub struct PrepareSecRequestInput {
    /// The validated CIK that will be used for the SEC API request.
    pub validated_cik: Cik,
    /// The shared HTTP client passed down from the super-state context.
    pub sec_client: SecClient,
}
```

---

## Enums

### W-Fragen

- **Was?** — What family of values does this enum represent?
- **Warum?** — Why are these grouped together? (especially: data enums vs error enums)
- **Wann?** — On each variant: under what condition is this variant produced?

### Content Structure

1. **What-sentence** on the enum.
2. **Why/How paragraph** when the enum's purpose needs justification.
3. **Variant docs** — Each variant gets `///` answering *"When does this occur?"*.
4. **Variant field docs** — Each named field gets `///` answering *"What does this carry?"*.

### Applicable `# Sections`

| Section | When to include |
|---------|-----------------|
| `# Examples` | Rare — only when showing variant construction aids understanding |

### Doc-Tests

Do **not** place doc-tests on the enum declaration. If a doc-test is needed, place it on an associated function or method of the enum (e.g., `StatusCode::from_u16`).

### Examples

Data enum:

```rust
/// HTTP status code classification for SEC API responses.
///
/// Models the specific HTTP codes relevant to SEC API interactions as explicit
/// variants, so match arms can handle rate-limiting (429) or not-found (404)
/// without raw integer comparisons. Unrecognized codes are captured by `Other`.
#[non_exhaustive]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize)]
pub enum StatusCode {
    /// 200 OK.
    Ok,
    /// 404 Not Found.
    NotFound,
    /// 429 Too Many Requests.
    TooManyRequests,
    /// Any other valid HTTP status code not explicitly modeled.
    Other(u16),
}
```

Error enum:

```rust
/// Reason for an invalid CIK format.
///
/// Distinguishes between structural violations (non-numeric characters) and
/// length violations so callers can provide targeted user-facing messages.
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

---

## Traits

Traits define contracts for multiple implementors, so their docs must answer additional questions beyond the standard W-Fragen.

### W-Fragen

- **Was?** — What responsibility does this trait represent?
- **Warum?** — Why is this a trait rather than a concrete type? (polymorphism, testability, decoupling)
- **Wer?** — Who implements it? (brief mention of key implementors or the intended audience)
- **Wie?** — How should an implementor fulfil the contract? (key invariants, ordering constraints)

### Content Structure

1. **What-sentence** on the trait.
2. **Why/Who/How paragraph** explaining motivation and contract.
3. **`# Associated Types`** — Document each associated type.
4. **Method docs** — Each method gets its own `///` with applicable `# Sections`.

### Applicable `# Sections`

On the trait itself:

| Section | When to include |
|---------|-----------------|
| `# Associated Types` | When the trait has associated types |
| `# Required Traits` | When supertraits need explanation |

On individual trait methods:

| Section | When to include |
|---------|-----------------|
| `# Errors` | Methods returning `Result` |
| `# Panics` | Methods that can panic |
| `# Examples` | Methods that a consumer calls to initiate work (e.g., `from_inner`, `compute_output_data_async`) |

### Doc-Tests

Place doc-tests on **individual trait methods**, not on the trait declaration. The trait-level doc describes the contract; method-level docs show usage.

### Default Implementations

If a trait provides a default implementation, state whether implementors should override it and under what conditions.

### Example

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
/// - `ContentType`: The content type returned from the HTTP response payload.
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

---

## Constants and Type Aliases

### W-Fragen

- **Was?** — Always. What does this value/type represent?
- **Warum?** — When the constant's existence is non-obvious (why not inline the value?).
- **Wo?** — When the constant's placement in this module needs explanation.

### Content Structure

A what-sentence is sufficient for most constants. Add a why/how paragraph only when the value's purpose is non-obvious.

### Doc-Tests

Not required. Constants and type aliases do not need `# Examples`.

### Examples

```rust
/// Human-readable name of the "Validate CIK Format" state, used in error messages and logging.
pub const STATE_NAME: &str = "Validate CIK Format";
```

```rust
/// A boxed, `Send`-able stream of state machine execution results.
///
/// Each item is `Ok(StreamItem)` for successful events or `Err(StreamError)` for failures.
pub type StateMachineStream = Pin<Box<dyn Stream<Item = Result<StreamItem, StreamError>> + Send>>;
```

---

## Builders

- The **Builder** doc must link to the type it constructs.
- Each builder method documents the field it sets (what-sentence is sufficient).
- Doc-tests: not required on individual setter methods. Place one on `build()` if the construction is complex.

---

## Error Documentation

Error types follow additional conventions specific to this project.

### Wording

- Open an error struct's doc with **"Error representing …"** or **"Error indicating …"**:
  `Error representing a CIK validation failure, tagged with the state it occurred in.`
- Use **"error"** for the type itself — it is a meaningful term in Rust. Reserve "failure" for the
  *event or action* the error represents ("Error representing a CIK validation failure"), not the type.
- For a state-level wrapper error, the established two-sentence shape works well: (1) `Error representing <failure>, tagged with the state it occurred in.` then (2) `Wraps a domain-level [`CikError`] together with ... the state in which the error occurred, making it suitable for use in state machine error handling.`

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

## Re-exports and Preludes

- Prelude modules (`prelude.rs`) must have module-level documentation explaining what is re-exported and why.
- Individual `pub use` re-exports do **not** need their own doc comment.

---

## What Not to Document

- **Private items** — Document only if the logic is subtle enough to warrant it.
- **Obvious accessors** — A one-line summary suffices; skip `# Arguments` / `# Returns` / `# Examples`.
- **Trait implementations** — Only document if the implementation has surprising behavior or deviates from the trait's contract. Standard `From`, `Display`, `Default` implementations typically need no doc.
- **Test modules** — `#[cfg(test)] mod tests` does not need documentation.

---

## Cross-Cutting Reference

### Intra-Doc Links

Use Rustdoc's intra-doc link syntax to create hyperlinks within the generated documentation. These links are **checked at compile time** when `rustdoc` lints are enabled.

#### Syntax

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

#### Link Direction Principle

Intra-doc links follow the same direction as `use` statements — they point toward **dependencies**, never toward **dependents**:

| Direction | Allowed? | Example |
|-----------|----------|---------|
| **Downward** (toward your dependencies) | Yes | `ValidateCikFormat` linking to [`ValidateCikFormatInput`] |
| **Sideways** (toward siblings in the same module) | Yes | `PrepareSecRequestInput` linking to [`SecClient`] |
| **Upward** (toward things that depend on you) | No | `Cik` linking to a state that *uses* `Cik` |

This keeps documentation decoupled the same way the code is. If you add a new implementor of a trait, you do **not** go back to the trait's docs to add a link to it. The implementor links to the trait it implements, not the other way around.

Exception: `## See Also` in module-level docs may link upward for discoverability, since module docs serve as navigation aids.

#### When to Link

Link only when the reader would **need to navigate** to the target to understand the current item:

- **Link** our own types: states, errors, shared domain types, traits — the reader likely needs to see their definition.
- **Don't link** standard library types (`String`, `Vec`, `Result`, `Option`) or obvious external crate types (`serde::Serialize`) — the reader already knows what these are.
- **Don't link** the same target repeatedly within one doc comment — one link is enough.

Rule of thumb: if removing the link would leave the reader guessing what something is or where to find it, include it. If the type is self-explanatory, don't.

#### Syntax Rules

- Prefer short-form links (`` [`Type`] ``) when the target is unambiguous in scope.
- Use fully-qualified paths (`` [`crate::module::Type`] ``) when linking across module boundaries to avoid ambiguity.
- Use renamed links when the fully-qualified path would hurt readability in prose.
- All intra-doc links **must resolve**. Broken links are treated as errors in CI.

### Doc-Test Mechanics

Doc-tests answer the *Wie?* (How?) question concretely — they show a consumer how to use the item. These are the formatting rules for writing them.

#### Rules

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

### Section Formatting Rules

- Use `# Heading` (single `#`) inside doc comments. `rustdoc` renders these as `<h2>` within the item.
- List items use `-` consistently in this project.
- Wrap parameter names in backticks: `` `validated_cik` ``.

### CI Enforcement

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
- [ ] Doc-tests exist on constructors and fallible entry-point methods.
- [ ] Module-level doc-tests exist where multiple items must compose together to demonstrate usage.
- [ ] Field-level docs exist on all public struct fields and enum variant fields.
- [ ] Each doc comment answers the relevant W-Fragen (at minimum *Was?*).
