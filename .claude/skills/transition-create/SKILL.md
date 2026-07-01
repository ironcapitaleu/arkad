---
name: transition-create
description: >
  This skill should be used when the user asks to "create a full transition", "design and implement
  a transition", "end-to-end transition creation", or wants to go through the complete transition
  workflow (design → implement) in a single session.
version: 0.1.0
argument-hint: "[source-state] [target-state]"
allowed-tools: [Read, Write, Edit, Bash, AskUserQuestion]
---

# Transition Create — Full Workflow

## Purpose

Orchestrate the complete transition creation workflow in a single session. Runs through design
and implementation sequentially, asking the user to approve the design before implementing.

## Workflow

### Phase 1: Design

Follow the full process from `.claude/skills/transition-design/SKILL.md`:

1. Identify the states (source, target, SuperState)
2. Map data flow using `into_parts()`
3. Define context propagation
4. Identify error scenarios
5. Define SuperState integration
6. Produce the transition design document
7. **Ask the user to approve the design before proceeding**

### Phase 2: Implementation

Follow the full process from `.claude/skills/transition-implementation/SKILL.md`:

1. Implement the Transition trait using `into_parts()`
2. Write tests (happy path, missing output, data integrity, context propagation)
3. Wire into SuperState run logic and async stream
4. Integrate into affected binaries
5. Update design documents (state diagram, class diagram)
6. Run `cargo test` to verify
7. **Present the result to the user**

## Key Rules

- **Never skip the design phase** — always design before implementing
- **Always get user approval** between phases — the user may want to adjust the mapping
- **Both states must exist and compile** before starting — if not, direct user to create them first
- **If anything is unclear, ask** — do not assume data mappings
