---
name: state-design
description: >
  This skill should be used when the user asks to "design a new state", "define state requirements",
  "plan a state", "create a state design", or needs to gather requirements and produce a design
  document for a new Sub-State before implementation begins.
version: 0.1.0
argument-hint: "[state-name]"
allowed-tools: [Read, Write, Edit, Bash, AskUserQuestion]
---

# State Design Skill

## Purpose

Gather requirements and produce a design document for a new Sub-State. This is the first phase
before scaffolding and implementation. The output is a clear specification that feeds into
`/state-scaffold` and `/state-implementation`.

## Design Process

### Step 1: Context Gathering

Gather from the user (incrementally, do not overwhelm):

1. **State name** — what is this state called? (e.g., `PrepareSecRequest`)
2. **SuperState** — which state machine does it belong to? (e.g., `Extract`)
3. **Position in pipeline** — which state comes before it? Which comes after?
4. **Purpose** — what does this state do at a high level? (one sentence)

### Step 2: Input Requirements

Walk through the input design:

1. **What data does this state receive?** — field names, types
2. **Where does it come from?** — output of the previous state? Context? External?
3. **Validation requirements** — any constraints on the input data?

### Step 3: Output Requirements

Walk through the output design:

1. **What does this state produce?** — field names, types
2. **Who consumes it?** — the next state? Multiple consumers?
3. **What shape does the next state need it in?** — this informs output struct design

### Step 4: Context Requirements

Walk through shared dependencies:

1. **What external dependencies does this state need?** — HTTP clients, config, credentials
2. **Are these shared with other states?** — if yes, they go in Context
3. **Max retries / configuration?** — operational parameters

### Step 5: Domain Concepts

Identify new domain types this state introduces:

1. **New types** — does this state introduce any domain concepts? (e.g., `SecRequest`, `Url`)
2. **Validation rules** — does the domain type have invariants? (e.g., URL must be valid)
3. **Existing types to reuse** — check `sec/src/lib/shared/` for existing domain concepts

### Step 6: Error Scenarios

Walk through what can go wrong:

1. **Domain errors** — what invalid states can the domain concepts be in?
2. **Computation errors** — what can fail during compute_output_data_async?
3. **Error chain** — how do errors fit into the existing hierarchy? (DomainError → StateError → SuperStateError)
4. **Reference** — check `sec/design/uml_class_diagram/sec_error_handling.md`

### Step 7: Async / Dependencies

1. **Is this state async?** — does it call external systems? (almost always yes for SEC)
2. **Dependency inversion** — should the external call be behind a trait for testability? (usually yes, for testability)
3. **Streamable** — does this state need to integrate into the async stream? (usually yes, for integration in async pipeline)

## Design Document Output

After gathering all requirements, produce a design document (write it to the appropriate location, e.g., alongside the state machine's design docs):

```markdown
# State Design: {StateName}

## Overview
- **SuperState:** {name}
- **Position:** after {PreviousState}, before {NextState}
- **Purpose:** {one sentence}

## Input
| Field | Type | Source |
| --- | --- | --- |
| ... | ... | from {PreviousState} output |

## Output
| Field | Type | Consumer |
| --- | --- | --- |
| ... | ... | {NextState} input |

## Context
| Field | Type | Purpose |
| --- | --- | --- |
| ... | ... | ... |

## Domain Concepts
- {TypeName}: {description, invariants}

## Error Scenarios
| Error | Type | When |
| --- | --- | --- |
| ... | Domain / State | ... |

## Async
- [x] Async compute
- [] Dependency inversion needed
- [x] Streamable integration

## Related Design Documents
- Error handling: {path}
- Class diagram: {path}
- State diagram: {path}
```

## After Design

Once the design is complete, the next steps are:

1. `/state-scaffold {StateName}` — generate all boilerplate from this design
2. `/state-implementation {StateName}` — implement the domain logic
3. `/transition-design` — design the transition from the previous state
4. `/transition-implementation` — implement the transition
