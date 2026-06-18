---
name: transition-design
description: >
  This skill should be used when the user asks to "design a transition", "plan a state transition",
  "define transition requirements", or needs to specify how one state transitions into another
  before implementing the Transition trait.
version: 0.1.0
argument-hint: "[source-state] [target-state]"
allowed-tools: [Read, Write, Edit, Bash, AskUserQuestion]
---

# Transition Design Skill

## Purpose

Design the transition between two states in a SuperState. Defines how the output of one state
maps to the input of the next, what context is carried forward, and what can go wrong during
the transition.

## Design Process

### Step 1: Identify the States

1. **Source state** — which state is transitioning out? (e.g., `ValidateCikFormat`)
2. **Target state** — which state is transitioning into? (e.g., `PrepareSecRequest`)
3. **SuperState** — which state machine manages this transition? (e.g., `Extract`)

### Step 2: Data Mapping

Use `source_state.into_parts()` to decompose the source state into its
`(input, Option<output>, context)` tuple. From these parts, assemble the target state's
input and context.

Walk through how the parts become the new state:

1. **Source parts** — what does `into_parts()` return? (input struct, Option<output struct>, context struct)
2. **Target input fields** — read the target state's input struct
3. **Mapping** — which fields from source output (and/or source input, source context) map to target input?
4. **Transformations** — any conversions needed? (type changes, restructuring)
5. **New data** — does the target need data not available from source parts? (computed? external?)

### Step 3: Context Propagation

1. **Shared context** — which context fields carry over unchanged?
2. **Context changes** — does anything in context need to update during transition?
3. **New context fields** — does the target state need context the source didn't have?

### Step 4: Error Scenarios

1. **Missing output** — what if the source state's output is None (never computed)?
2. **Conversion failures** — can the output → input mapping fail? (type mismatches, invalid states)
3. **Precondition violations** — any invariants that must hold for the transition to succeed?

### Step 5: SuperState Integration

1. **State machine type** — how does the SuperState's generic parameter change? (`Extract<Source>` → `Extract<Target>`)
2. **Stream integration** — does the transition need to be part of the async stream?
3. **Binary wiring** — which binaries need this transition added?

## Design Document Output

```markdown
# Transition Design: {SourceState} → {TargetState}

## Overview
- **SuperState:** {name}
- **Direction:** {SourceState} → {TargetState}

## Data Mapping
| Source Output Field | Target Input Field | Transformation |
| --- | --- | --- |
| ... | ... | direct / conversion / computed |

## Context Propagation
| Field | Action |
| --- | --- |
| client | carry forward |
| ... | ... |

## Error Scenarios
| Error | When | Recovery |
| --- | --- | --- |
| MissingOutput | source never computed | fail transition |
| ... | ... | ... |

## Integration
- SuperState type change: `Extract<{Source}>` → `Extract<{Target}>`
- Binaries affected: {list}
- Stream position: after {Source} compute, before {Target} compute
```

## After Design

Once the transition design is complete:

1. `/transition-implementation {SourceState} {TargetState}` — implement the Transition trait
