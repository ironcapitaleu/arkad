---
name: documentation
description: >
  Use when the user asks to "document", "add documentation", "check docs", "review docs",
  "refactor docs", or wants to improve documentation for any part of the codebase. Supports
  documenting a specific package, module, recently written code, or running a compliance check
  against DOCUMENTATION.md guidelines.
version: 0.4.0
argument-hint: "[target-path or 'check' or 'recent']"
allowed-tools: [Read, Write, Edit, Bash, AskUserQuestion, Agent]
---

# Documentation Skill

## Purpose

Guide the user through documenting, checking, or refactoring Rust documentation in this project.
All documentation must comply with the guidelines in `DOCUMENTATION.md` at the project root.

This skill operates in three modes:
1. **Write** — Add or rewrite documentation for a target
2. **Check** — Audit existing docs for compliance violations
3. **Improve guidelines** — When patterns emerge from user feedback, update `DOCUMENTATION.md`

## Entry Point

**Adaptive questionnaire** — only ask what you can't infer from the user's message.

When invoked, first parse the user's invocation message and conversation context for:
- **Mode** — are they asking to write, check, refactor, or improve guidelines?
- **Scope** — did they name a crate, module, file, or say "recent"?
- **Detail level** — did they say "fully", "just module docs", "doctests only"?

Then:
1. **State your understanding** back to the user in one sentence (e.g. "I'll fully document all
   public items in the recently added modules based on git diff.").
2. **Ask only for what's genuinely unclear** via `AskUserQuestion`. If everything is inferable,
   ask a single confirmation: "Does this look right, or should I adjust?"
3. On confirmation, proceed. On correction, adjust and re-confirm.

### Available modes:

- **Document new/undocumented code** — Write docs from scratch
- **Check existing docs for compliance** — Audit against DOCUMENTATION.md
- **Refactor existing docs** — Improve docs that exist but don't follow conventions
- **Improve the guidelines themselves** — Add clarifications to DOCUMENTATION.md

### Available scopes:

- **A specific crate** — e.g. `sec`, `state_machine`, `xbrl`
- **A specific module or directory** — e.g. `sec/src/lib/shared/cik/`
- **A specific file** — e.g. `sec/src/lib/shared/cik/mod.rs`
- **Recently written code** — based on `git diff` or conversation context
- **Everything** — full workspace scan (warn: this is large)

### Available detail levels (for Write/Refactor):

- **Module docs only** (`//!`) — title, what-sentence, why/how, modules list
- **All public items** — full pass including structs, enums, traits, methods, fields
- **Doc-tests only** — ensure ADAA pattern compliance on existing doctests

## Execution

### For "Write" or "Refactor" mode:

1. Read `DOCUMENTATION.md` to load the current conventions.
2. Read the target files to understand the code.
3. For each file in scope:
   - Check what documentation exists vs what's required.
   - Write/rewrite docs following the conventions.
   - For doctests, apply the ADAA pattern (Arrange, Define expected, Act, Assert).
4. Run `cargo doc --no-deps -p <crate>` to verify no broken links.
5. Run `cargo test --doc -p <crate>` to verify doctests pass.

### For "Check" mode:

1. Read `DOCUMENTATION.md` to load conventions.
2. Scan the target for violations. Check for:
   - Missing module docs (`//!`)
   - Missing struct/enum/trait/method docs
   - Constructor docs not naming their type with intra-doc link
   - Error docs not starting with "Error representing/indicating..."
   - Error reason enums not starting with "Enum representing the reason why..."
   - Coupling references (ordinals, naming consumers, "transport" jargon, sibling contrasts)
   - Internal-dependent references (naming a caller/state/pipeline or the concrete type that holds this one) — distinct from allowed references to the external system the crate serves; and domain vocabulary ("request", "SEC") leaking into a generic trait contract
   - Temporal / roadmap references (naming *when* something exists or will: "later", "deferred", "first/minimal slice", "the read arm arrives later", "for now", "exists now", "built X-side first", ticket/PR IDs) — docs describe the current contract, not a timeline
   - Positional / consumption references (describing *where an item sits* in a hierarchy or *how it is consumed* instead of what it is: "innermost leaf", "top-level", "outermost layer", "the union", "shared across every X", "embedded in Y rather than surfaced on its own", "the class returned by every write method", "for the shared consumers") — say what the item represents; for errors, mirror `sec/src/lib/error/` ("Error occurring …")
   - Restating what the code already says ("the fallible downcast" on a `TryFrom`, narrating `#[non_exhaustive]`, jargon like "sentinel")
   - Module docs carrying team policy, process, or rationale (where fixtures live, when to extract a shared crate, why a past decision was made) instead of what the module provides — that belongs in design docs or a skill
   - Doctests not following ADAA pattern
   - Redundant/tautological assertions
   - "a SEC" instead of "an SEC" (vowel-sound rule)
   - `# Arguments` / `# Returns` on self-documenting signatures
   - Builder `new()` not naming the type
   - `build()` not naming the return type
3. Report findings as a list: file, line, violation, suggested fix.
4. Optionally apply fixes if user agrees.

### For "Improve guidelines" mode:

When the user has iterated with you on a documentation question and reached a conclusion:

1. Identify the new convention or clarification.
2. Find the right section in `DOCUMENTATION.md` to add it.
3. Write it concisely with a concrete example (matching the existing style).
4. Show the user the proposed change before applying.

## Key Conventions (Quick Reference)

These are loaded from `DOCUMENTATION.md` but summarized here for speed:

- **Module docs**: Title + what-sentence + why/how + `## Modules` list
- **Constructors**: `Creates a new [`TypeName`] ...` with intra-doc link, qualifier when ambiguous
- **Errors**: Start with "Error representing..." or "Error indicating..."
- **Error reason enums**: Start with "Enum representing the reason why..."
- **Builders**: `new()` names the type + "with all fields initialized to `None`"; `build()` names the return type
- **Updaters**: "Updater for modifying [`TypeName`]." + "Fields set to `None` are left unchanged..."
- **Conversions**: One-liner naming source and target
- **Doc-tests**: ADAA pattern; omit assertion when construction is the point (use `let _x = ...`)
- **No coupling**: No ordinals from children, no naming consumers, no sibling contrasts
- **No temporal coupling**: Describe what an item is now, never *when* — no "later", "deferred", "first/minimal slice", "arrives with", "for now", "exists now", or ticket/PR IDs. State deliberate boundaries as timeless properties ("names no concrete backend"); let `#[non_exhaustive]` carry extensibility
- **No positional / consumption coupling**: Say what an item *is / does*, never where it sits in a hierarchy ("innermost leaf", "top-level", "outermost layer") or how it's consumed ("shared across …", "embedded in …", "for the shared consumers"). For errors, mirror `sec/src/lib/error/`: open with what failed ("Error occurring …"), not the error's position
- **Don't restate the code**: A `TryFrom` is obviously fallible (no "the fallible downcast"); an attribute is visible in the source (don't narrate `#[non_exhaustive]`); avoid empty jargon ("sentinel")
- **Module docs = what it provides**: a module doc states what the module provides plus its `## Modules` list — not team policy, process, or rationale (where fixtures live, when to extract a shared crate, why a past decision was made). That material belongs in design docs or a skill
- **External system vs. internal dependent**: Naming the external system the crate serves ("under the SEC's 10 req/s ceiling") is domain justification and allowed; naming an internal dependent (a caller, state, pipeline, or the concrete type that holds this one) is coupling and must be avoided. Generic traits stay domain-agnostic (no "SEC"/"request"); the concept's module doc, constants, and concrete impl may name the external system
- **Grammar**: "an SEC", "an [`SecRequest`]" (vowel sound)
- **Setters**: "Sets the X field." (one-liner)
- **Associated types on impls**: Link to concrete type (`/// The [`reqwest::Request`] type.`) for scannability
- **When unsure**: Look at sibling modules that already follow conventions and match them

## Proactive Propagation

When the user requests a documentation fix, correction, or improvement on a specific item:

1. **Apply the requested change** to the target item.
2. **Immediately reason**: "Can this same pattern/violation exist elsewhere in the codebase?"
3. **Search proactively** (via `grep`, `find`, or Agent) for all similar occurrences.
4. **Apply the fix everywhere** it applies — do not wait for the user to ask "are there more?"
5. **Report** what was found and fixed: "Found N more instances of the same pattern, fixed all."

This means: a single user correction propagates project-wide in one pass. The user should never
need to say "fix it everywhere" — that is the default behavior.

If uncertain whether a match is truly the same violation (ambiguous context), list it and ask
rather than silently skipping.

## Self-Improvement

After completing a documentation session where the user corrected or refined something:

1. Ask: "Should I add this as a guideline to DOCUMENTATION.md?"
2. If yes, propose the addition with a concrete example.
3. Apply after user approval.

This keeps the guidelines growing from real usage rather than speculation.
