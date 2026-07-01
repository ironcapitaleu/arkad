---
name: state-create
description: >
  This skill should be used when the user asks to "create a full state", "design and implement a state",
  "end-to-end state creation", or wants to go through the complete state creation workflow
  (design → scaffold → implement) in a single session.
version: 0.1.0
argument-hint: "[state-name]"
allowed-tools: [Read, Write, Edit, Bash, AskUserQuestion]
---

# State Create — Full Workflow

## Purpose

Orchestrate the complete state creation workflow in a single session. Runs through design,
scaffold, and implementation sequentially, asking the user to approve each phase before
moving to the next.

## Workflow

### Phase 1: Design

Follow the full process from `.claude/skills/state-design/SKILL.md`:

1. Run the questionnaire (Steps 1–7)
2. Produce the design document
3. **Ask the user to approve the design before proceeding**

### Phase 2: Scaffold

Follow the full process from `.claude/skills/state-scaffold/SKILL.md`:

1. Read the design document produced in Phase 1
2. Generate all boilerplate (directory structure, structs, traits, tests)
3. Verify it compiles (`cargo check`)
4. **Ask the user to approve the scaffold before proceeding**

### Phase 3: Implementation

Follow the full process from `.claude/skills/state-implementation/SKILL.md`:

1. Read the design document from Phase 1
2. Implement domain concepts, compute logic, error handling
3. Run tests (`cargo test`)
4. **Present the result to the user**

### Phase 4: Transition (optional)

If the user wants to continue:

1. Follow `.claude/skills/transition-design/SKILL.md` for the transition from the previous state
2. Follow `.claude/skills/transition-implementation/SKILL.md` to implement it

## Key Rules

- **Never skip a phase** — always design before scaffolding, scaffold before implementing
- **Always get user approval** between phases — the user may want to adjust the design
- **Each phase reads from the previous** — the design doc is the single source of truth
- **If anything is unclear, ask** — do not assume requirements
