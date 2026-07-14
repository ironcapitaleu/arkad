---
name: linear
description: >
  This skill should be used when the user asks to "create a linear issue", "add a ticket",
  "make a linear ticket", "create a SPIKE", "create a FEATURE ticket", "list my issues",
  "search linear", "update a ticket", or needs to interact with Linear in any way
  (creating, reading, updating, or searching issues).
version: 0.1.0
argument-hint: "[create|list|search|get|update] [details]"
allowed-tools: [Read, Write, Edit, Bash, AskUserQuestion, mcp__linear__create_issue, mcp__linear__get_issue, mcp__linear__list_issues, mcp__linear__list_teams, mcp__linear__list_projects, mcp__linear__search_issues, mcp__linear__update_issue]
---

# Linear Issue Management Skill

## Purpose

Create, read, update, and search Linear issues for the State-machine team. All issues follow
a consistent template structure with typed labels, user stories, and definition-of-done checklists.

## Team & Workspace

- **Team:** State-machine
- **Team ID:** `2f641ff7-6c29-4aa7-9bc5-659ff838e4ad`
- **Team Key:** STA

## Issue Type Philosophy

Each issue type exists for a specific reason in the development workflow:

| Type | Purpose | Preconditions | Outputs |
| --- | --- | --- | --- |
| **SPIKE** | Reduce ambiguity. Research a topic so that a later implementation ticket can be written with clear, concrete requirements. | Unclear requirements, multiple possible approaches, or unfamiliar technology. | A findings document linked in the ticket. A recommendation for how to proceed. |
| **FEATURE** | Implement new functionality. Requirements should already be reasonably clear (often from a preceding SPIKE). | Clear acceptance criteria. Design docs or SPIKE findings available. | Working code, tests, PR merged. |
| **REFACTOR** | Restructure existing code without changing external behavior. | Code works but is hard to maintain, extend, or understand. | Same behavior, better structure. |
| **IMPLEMENTATION** | General implementation that doesn't fit FEATURE (e.g., wiring, integration, infra). | Requirements known. | Working code, tests, PR merged. |
| **ONBOARDING** | Structured learning path for team members to get productive. | New team member or new technology area. | Completed reading/exercises. |
| **DOCS** | Improve documentation quality and consistency. | Docs are missing, outdated, or inconsistent. | Updated documentation. |
| **TEST** | Add or review test coverage. | Code exists but tests are missing or inadequate. | Test suite improvements. |
| **FRONTEND** | UI/frontend implementation or changes. | Design or requirements available. | Working UI, tests, PR merged. |
| **DESIGN** | Architectural/design work producing diagrams or specs. | System needs to be designed before implementation. | Design documents, diagrams. |

### The SPIKE → FEATURE Pipeline

SPIKEs exist to **de-risk** implementation. The pattern is:

1. **SPIKE** — research, compare approaches, write findings, make a recommendation
2. **FEATURE** — implement the recommendation with clear acceptance criteria

SPIKEs always link to their related implementation ticket. FEATURE tickets that were preceded
by a SPIKE link back to it for context.

## Issue Types (Labels)

| Label | Title Prefix | Priority (typical) |
| --- | --- | --- |
| FEATURE | `[FEATURE]` | High (2) |
| SPIKE | `[SPIKE]` | High (2) |
| REFACTOR | `[REFACTOR]` | Medium (3) |
| IMPLEMENTATION | `[IMPLEMENTATION]` | High (2) |
| ONBOARDING | `[ONBOARDING]` | High (2) |
| DOCS | `[DOCS]` | High (2) or Medium (3) |
| TEST | `[TEST]` | Medium (3) |
| FRONTEND | `[FRONTEND]` | Medium (3) |
| DESIGN | `[DESIGN]` | High (2) |
| (none) | (no prefix) | Varies |

### Priority Scale

| Value | Label | Meaning |
| --- | --- | --- |
| 1 | Urgent | Blocking other work, needs immediate attention |
| 2 | High | Important, on the critical path, default for most tickets |
| 3 | Medium | Should be done but not blocking, backlog-worthy |
| 4 | Low | Nice to have, do when time permits |

## Title Format

Titles follow: `[PREFIX] Descriptive Title`

Use backtick-quoted code names where appropriate:

    [FEATURE] `ExecuteSecRequest` as Third Sub-`State` of `Extract`
    [SPIKE] Design a Rate Limiter for `Extract` `SuperState`
    [DOCS] Documentation Consistency
    [ONBOARDING] Rust Programming Concepts - Async Rust
    [FEATURE] `Transition` from `Extract<ValidateCikFormat>` to `Extract<PrepareSecRequest>`

**Exception:** Basic user tickets have NO prefix — just the descriptive title.

## Issue Templates

### FEATURE Template

```markdown
## **User Story:**

As a …,

I want to …,

so that … .

## **Description:**

/

## Definition of Done:

- [ ] …
- [ ] …
- [ ] The Software Development Guidelines outlined [here](<https://www.notion.so/Arkad-Software-Development-Guidelines-214cfe3cc9fb809082a0d15d3e6036cc>) are followed

## **Related Design Documents:**

/

## **Additional Notes:**

/

---
```

### REFACTOR Template

```markdown
## **User Story:**

As a …,

I want to …,

so that … .

## **Description:**

/

## Definition of Done:

- [ ] …
- [ ] …
- [ ] Make sure the Software Development Guidelines outlined [here](<https://www.notion.so/Arkad-Software-Development-Guidelines-214cfe3cc9fb809082a0d15d3e6036cc>) are followed

## **Related Design Documents:**

/

## **Additional Notes:**

/

---
```

### IMPLEMENTATION Template (General)

```markdown
## **User Story:**

As a …,

I want to …,

so that … .

## **Description:**

/

## Definition of Done:

- [ ] …
- [ ] …
- [ ] Follow the Software Development Guidelines outlined [here](<https://www.notion.so/Arkad-Software-Development-Guidelines-214cfe3cc9fb809082a0d15d3e6036cc>)

## **Related Design Documents:**

/

## **Additional Notes:**

/

---
```

### SPIKE Template

```markdown
## **User Story:**

As a developer,

I want to …

so that I can implement … .

## **Description:**

This SPIKE ticket is related to the implementation ticket [STA-XXX](url).

## Definition of Done:

- [ ] Review …
  - [ ] …
  - [ ] …
- [ ] Write a Google Doc containing the findings of this research.
  - [ ] Summarize these review findings and discuss potential alternatives (if they exist) on a very high level. Discuss pros and cons for each approach.
  - [ ] Based on the summary, make a recommendation as to how … should be ideally implemented.
  - [ ] Link the created written document in the "Link to Findings" section in this ticket

## **Additional Notes:**

/

## **Link to Findings:**

/
```

### ONBOARDING Template

```markdown
## **User Story:**

As a developer,

I want to …

so that I can be efficiently onboarded and start working productively.

## **Description:**

This ticket is the direct continuation of [STA-XXX](url).

## Definition of Done:

- [ ] Read certain chapters in Rust by Example
  - [ ] …
- [ ] Read chapter X
  - [ ] …
- [ ] Finish the Rustlings Exercises
  - [ ] XX …
- [ ] Read Chapter XX of Rust Notes
  - [ ] …

## **Additional Notes:**

/

---
```

### DOCS / TEST / FRONTEND / DESIGN Templates

These types do not have a dedicated description template. Use the **BASIC USER TICKET**
description structure below and adapt it to the specific needs of the issue.
The title prefix still applies (e.g., `[DOCS] ...`, `[TEST] ...`).

### BASIC USER TICKET Template (no prefix)

```markdown
## **User Story:**

As a …,

I want to …,

so that … .

## **Description:**

/

## Definition of Done:

- [ ] …
- [ ] …

## **Additional Notes:**

/

---
```

## Few-Shot Examples

### Example 1: SPIKE (STA-124)

**Title:** ``[SPIKE] Design a Rate Limiter for `Extract` `SuperState` ``
**Priority:** High (2)
**Label:** SPIKE

```markdown
## **User Story:**

As a developer,

I want to make sure that I have configurable global rate limiting across `StateMachine` runs

so that I can run ingestion pipelines at maximum throughput.

## **Description:**

This SPIKE ticket is related to the implementation ticket [STA-125](https://linear.app/state-machine/issue/STA-125/feature-implement-controllable-rate-limiting-for-extract-superstate).

## Definition of Done:

- [ ] Review how to best implement a Rate Limiter for our State Machine Executions
  - [ ] Review if and what libraries are needed for this
    - [ ] Is there a built-in `tokio` rate limiter, is it usable for us?
    - [ ] governor - not sure if this will do it instead?
  - [ ] how would they be best integrated into our existing async stream pipeline?
    - [ ] minimal disruptions to the async stream would be ideal
  - [ ] how and where would the rate limiter be initialized?
  - [ ] How about a Queue-based rate limiter, like the token bucket vs leaky bucket?
    - [ ] Compare the two and when each is preferred, and what you would suggest for our use case
- [ ] Review also how to best test the rate limiter
  - [ ] e.g., via timed integration tests or whatnot
- [ ] Also note or review limitations of this rate limiting strategy and how it could be implemented in a distributed fashion
- [ ] Does it make sense to remove `buffer_unordered(3)` once we have the rate limiter?
- [ ] Write a Google Doc containing the findings of this research.
  - [ ] Summarize these review findings and discuss potential alternatives (if they exist) on a very high level. Discuss pros and cons for each approach.
  - [ ] Based on the summary, make a recommendation as to how rate limiting should ideally be implemented with minimal disruptions to our async stream implementations for the ETL pipeline
  - [ ] Link the created written document in the "Link to Findings" section in this ticket

## **Additional Notes:**

/

## **Link to Findings:**

/
```

### Example 2: FEATURE (STA-74)

**Title:** ``[FEATURE] `ExecuteSecRequest` as Third Sub-`State` of `Extract` ``
**Priority:** High (2)
**Label:** FEATURE

```markdown
## **User Story:**

As a developer,

I want to implement the `ExecuteSecRequest` `State` of the `Extract` `SuperState`

so that I can continue working on the other Sub-`States` of the `Extract` `SuperState` in the ETL pipeline.

## **Description:**

This is the implementation of the Sub-`State` in the `Extract` `SuperState` that sends the actual request with the `Cik` to the SEC server and gets the company information returned.

Validation of only that the request has been successful will be made at this point. No validation about the structure of the data (?).

## Definition of Done:

- [ ] `ExecuteSecRequest` Sub-`State` implemented
  - [ ] Implement the third Sub-`State` of the `Extract` `SuperState`
  - [ ] Design an `ExecuteSecRequest` as a `State` for the `Extract` `StateMachine`
  - [ ] Implement the `ExecuteSecRequest` `State` according to the Retrieval State design doc
  - [ ] Implement the necessary domain concepts (if needed) inside the `arkad/sec/src/lib/shared/` directory
  - [ ] Implement Error Handling for the `ExecuteSecRequest` `State` according to the existing hybrid error handling model
  - [ ] (If applicable) Adapt the `Extract` `SuperState` and the `main()` function
- [ ] Follow the Software Development Guidelines outlined [here](<https://www.notion.so/Arkad-Software-Development-Guidelines-214cfe3cc9fb809082a0d15d3e6036cc>)

## **Related Design Documents:**

Error Handling: `arkad/sec/design/uml_class_diagram/sec_error_handling.md`

Class diagram (update `StateError` here): `arkad/sec/design/uml_class_diagram/sec_sample_state.md`

State diagram: `arkad/sec/design/state_diagram/sec_state_machine.md`

## **Additional Notes:**

Once this is done, a `Transition` from `Extract<PrepareSecRequest>` to `Extract<ProcessSecRequest>` can be implemented.

---
```

### Example 3: FEATURE — Transition (STA-73)

**Title:** ``[FEATURE] `Transition` from `Extract<ValidateCikFormat>` to `Extract<PrepareSecRequest>` ``
**Priority:** High (2)
**Label:** FEATURE

```markdown
## **User Story:**

As a developer

I want to implement the `Transition` from `ValidateCikFormat` to the `PrepareSecRequest` `State` of the `Extract` `SuperState`

so that I can continue working on the other Sub-`States` of the `Extract` `SuperState` in the ETL pipeline and have a first `Transition` implementation as a baseline for future implementations.

## **Description:**

Implement the `Transition` from `ValidateCikFormat` to `PrepareSecRequest`.

Implement any fixtures if needed.

Implement a first `TransitionError`.

This ticket is a direct continuation of [STA-72](https://linear.app/state-machine/issue/STA-72/feature-preparesecrequestas-second-sub-state-of-extract).

## Definition of Done:

- [ ] `Transition` from the `ValidateCikFormat` to the `PrepareSecRequest` is implemented
  - [ ] `Transition` trait with its methods has been implemented from `ValidateCikFormat` to `PrepareSecRequest`
  - [ ] Implement the necessary domain concepts (if needed) inside the `arkad/sec/src/lib/shared/` directory
  - [ ] Implement Error Handling for the `Transition` according to the existing hybrid error handling model (add/use a `Transition` error)
  - [ ] the main method inside the binary now initializes a `StateMachine` in the `Extract<ValidateCikFormat>` `State` and transfers to `Extract<PrepareSecRequest>` `State` - the logging statements have been adapted
- [ ] Follow the Software Development Guidelines outlined [here](<https://www.notion.so/Arkad-Software-Development-Guidelines-214cfe3cc9fb809082a0d15d3e6036cc>)

## **Related Design Documents:**

Error Handling: `arkad/sec/design/uml_class_diagram/sec_error_handling.md`

State diagram: `arkad/sec/design/state_diagram/sec_state_machine.md`

## **Additional Notes:**

Once this is done, a `Transition` from `Extract<PrepareSecRequest>` to `Extract<ExecuteSecRequest>` can be implemented.

---
```

### Example 4: REFACTOR (STA-90)

**Title:** ``[REFACTOR] `Extract` with Sub-`States` ``
**Priority:** Medium (3)
**Label:** REFACTOR

```markdown
## **User Story:**

As a developer,

I want to be able to ergonomically use `Extract` with all its Sub-`State`s

so that I can use and modify the `Extract` `State` easily.

## **Description:**

This is a refactoring ticket to finalize the implementation work of working on the `Extract` `SuperState` with all its Sub-States.

## Definition of Done:

- [ ] Review the code with respect to (but not limited to) the following aspects. Refactor them if any of these things apply.
  - [ ] Folder Structure and Naming
  - [ ] Idiomatic use of the `builder` pattern as a constructor
  - [ ] Renaming of some redundantly named concepts
  - [ ] Error Types and Error Structure
  - [ ] Check (and refactor) all `.expect(…)` messages
  - [ ] Use dependency injection for new added dependencies
  - [ ] Re-implement and simplify main function for `Extraction` with new implementation
- [ ] Make sure the Software Development Guidelines outlined [here](<https://www.notion.so/Arkad-Software-Development-Guidelines-214cfe3cc9fb809082a0d15d3e6036cc>) are followed

## **Related Design Documents:**

/

## **Additional Notes:**

/

---
```

### Example 5: ONBOARDING (STA-116)

**Title:** `[ONBOARDING] Rust Programming Concepts - Async Rust`
**Priority:** High (2)
**Label:** ONBOARDING

```markdown
## **User Story:**

As a developer,

I want to learn about parallel programming in Rust

so that I can be efficiently onboarded and start working productively.

## **Description:**

This ticket is the direct continuation of [STA-69](https://linear.app/state-machine/issue/STA-69/onboarding-rust-programming-concepts-threading).

## Definition of Done:

- [ ] Read chapter 17
  - [ ] Futures and the Async Syntax
  - [ ] Applying Concurrency with Async
  - [ ] Working with Any Number of Futures
  - [ ] Streams: Futures in Sequence
  - [ ] A Closer Look at the Traits for Async
  - [ ] Futures, Tasks, and Threads
- [ ] Read Chapter 17 of Rust Notes
  - [ ] Rust Async Overview Cheatsheet
  - [ ] Asynchronous Rust Programming
  - [ ] Working With Multiple Futures (Join, Race)
  - [ ] Message Passing
  - [ ] Streams: Futures in Sequence

## **Additional Notes:**

/

---
```

## Workflow: Creating an Issue

### Step 1: Gather Information

Use `AskUserQuestion` to collect:

1. **Issue type** — options: FEATURE, SPIKE, REFACTOR, IMPLEMENTATION, ONBOARDING, DOCS, TEST, FRONTEND, DESIGN, Basic (no prefix)
2. **Title** — what is this issue about? (will be formatted as `[TYPE] Title`, or just `Title` for basic)
3. **User story** — what do you want and why?
4. **Priority** — options: Urgent (1), High (2), Medium (3), Low (4) — default High (2)

If the user provided enough context when invoking the skill, skip questions you can already
answer. Infer what you can from conversation context.

### Step 2: Build the Description

Based on the issue type, fill in the appropriate template. Ask follow-up questions for:

- **FEATURE:** What are the acceptance criteria? Related design docs? Is there a preceding SPIKE?
- **SPIKE:** What is the related implementation ticket? What specific research questions need answering? What alternatives should be compared?
- **ONBOARDING:** What resources should be read? What's the learning path? What's the continuation of?
- **REFACTOR:** What is being refactored and why? What aspects should be reviewed?

### Step 3: Create the Issue

Use `mcp__linear__create_issue` with:
- `teamId`: `2f641ff7-6c29-4aa7-9bc5-659ff838e4ad`
- `title`: formatted as `[LABEL] Title` (or just `Title` for basic)
- `description`: filled template
- `priority`: user-selected (default 2)
- `labelIds`: match the label to existing IDs (see Label IDs section). Omit if the label ID is not yet captured.

### Step 4: Confirm

Show the user the created issue URL and summary.

## Workflow: Reading / Searching Issues

- **List issues:** Use `mcp__linear__list_issues` with team filter
- **Search:** Use `mcp__linear__search_issues` with keywords
- **Get details:** Use `mcp__linear__get_issue` with the issue ID
- **List teams:** Use `mcp__linear__list_teams` to discover team IDs when needed
- **List projects:** Use `mcp__linear__list_projects` to discover project IDs for filtering

When presenting issues to the user, show: identifier, title, status, assignee, and URL.

## Workflow: Updating an Issue

Use `mcp__linear__update_issue` to change status, assignee, priority, or description.
Always confirm with the user before making changes.

## Label IDs

| Label | ID |
| --- | --- |
| FEATURE | `c7fc7896-32b6-4c22-bcce-9f6cda3c935a` |
| SPIKE | `12ddfaf0-6644-459a-bd71-1765b2f7b795` |
| REFACTOR | `f4ab8267-331c-400a-8b54-4e6424b7f3e7` |
| DOCS | `4ae80929-fd9a-4d24-9e05-1c896d374700` |
| ONBOARDING | `339b3f41-9f48-4574-a958-ff76cf82795e` |

**Note:** Label IDs for IMPLEMENTATION, TEST, FRONTEND, and DESIGN are not yet captured.
When creating an issue with one of these types for the first time, omit `labelIds` — the
label can be added manually in Linear, and the ID should be captured here afterwards.

## Conventions

- **SPIKE before FEATURE (when needed):** When requirements are unclear or multiple approaches exist, create a SPIKE before the FEATURE. Not every FEATURE needs a SPIKE — only those with ambiguity that needs resolving. When a SPIKE exists, link it bidirectionally with its FEATURE.
- **Implementation requires clarity:** FEATURE and IMPLEMENTATION tickets should have concise requirements or designs in place before work begins. If requirements are unclear, create a SPIKE first.
- **Comments as progress logs:** Use comments for meeting notes (DONE / NEXT STEPS format) and progress updates.
- **Branch names:** Linear auto-generates branch names from titles — no manual naming needed.
- **Due dates:** Set when known, especially for SPIKEs.
- **Related tickets:** Always cross-reference related issues with `[STA-XXX](url)` format.
- **Continuation chains:** ONBOARDING and sequential FEATURE tickets reference their predecessor as "direct continuation of [STA-XXX]".
- **Empty fields:** Use `/` for sections with no content (Description, Related Design Documents, Additional Notes, Link to Findings).
- **Software Development Guidelines:** FEATURE, REFACTOR, and IMPLEMENTATION tickets always include the guidelines link in Definition of Done. Each uses slightly different phrasing — follow the template exactly:
  - FEATURE: "The Software Development Guidelines outlined [here](...) are followed"
  - REFACTOR: "Make sure the Software Development Guidelines outlined [here](...) are followed"
  - IMPLEMENTATION: "Follow the Software Development Guidelines outlined [here](...)"
- **Definition of Done structure:** Use nested checkboxes for sub-tasks. Top-level items are major deliverables; nested items are specific tasks within that deliverable.
- **Next steps in Additional Notes:** When a ticket enables follow-on work, note it (e.g., "Once this is done, a `Transition` from X to Y can be implemented.").

## Self-Improvement

After creating an issue where the user corrected or refined the template, naming, or workflow:

1. Ask: "Should I update the linear skill with this change?"
2. If yes, update the relevant section in this SKILL.md.
3. Apply after user approval.

Examples of things worth capturing:
- New issue types or labels that emerge (add to the tables and create a template if warranted)
- Template adjustments (new sections, reworded headings, different Definition of Done phrasing)
- New label IDs discovered when creating issues of previously-uncaptured types
- Priority conventions that differ from the documented defaults
- Workflow changes (e.g., new linking conventions, new fields to fill)
- Few-shot examples that better represent current practice — replace outdated ones
- Conventions the user establishes through repeated corrections

Nothing in this skill is set in stone — templates, headings, types, and conventions should
evolve as the team's process evolves. The skill is authoritative guidance, not a frozen spec.
