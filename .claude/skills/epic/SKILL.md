---
name: epic
description: >
  Use when the user asks to "define an epic", "plan an epic", "define a Linear project",
  "refine a Linear project", "start a Linear project", "map out <project>", "refine the epic",
  "chart the fog for <project>", "update the epic map", or invokes `/epic`. An epic is a Linear
  Project. This skill works at the project level (the `linear` skill works at the ticket level).
  It defines and refines the project — destination, milestones, a living fog log, and a decisions
  ledger that links to in-repo artifacts.
version: 0.1.0
argument-hint: "[project name] [define|refine]"
allowed-tools: [Read, Write, Bash, AskUserQuestion, mcp__Linear__get_project, mcp__Linear__save_project, mcp__Linear__list_milestones, mcp__Linear__save_milestone, mcp__Linear__list_issues, mcp__Linear__save_issue, mcp__Linear__list_documents, mcp__Linear__save_document]
---

# Epic Skill

## Purpose

An epic is a build too large for one ticket and too foggy to plan in full at the start. This skill
defines a new epic and refines it across its life.

The epic is a **Linear Project**. This skill works at the project level, not the ticket level. It
keeps the planning state in Linear and the deliverable artifacts in the repository.

Every project description, ticket, ledger entry, and design doc this skill writes is English text.
The `plain-english` skill applies to all of this text.

## The Split: Linear Plans, the Repo Holds Artifacts

- **Linear (the Project)** holds the plan: the description, the milestones, the fog log, and the
  decisions ledger. The ledger holds one-line entries that **link** to artifacts.
- **The repository** holds the living design of the code: design docs and mermaid diagrams under
  `<crate>/design/`, and ADRs under `docs/adr/`. These are version-controlled and live close to the
  code. Never use an external drawing tool (no Excalidraw, no Google Docs) for a living design.
- **Linear can hold research.** A SPIKE finding or an exploratory research note can live as a Linear
  document attached to the project, the way the reference epic does.

The ledger links an artifact wherever its home is. A design decision is done when its artifact is
committed to the repo and the ledger links it. A research finding is done when its document exists,
in the repo or in Linear, and the ledger links it.

## The Project Shape

Model the project on `SEC ETL Pipeline - Extract SuperState`, the reference epic. The Project
description has these sections:

```markdown
## User Story

As a …,
I want …,
so that … .

## Description

The goal of this epic, in a few sentences.

## Notes

Domain context and standing preferences that steer decisions.

## Definition of Done

- [ ] <top-level outcome — one per milestone>
- [ ] …

## Open Questions (Fog)

Group the fog by artifact category: design documents, abstractions, implementations, tests,
dependencies.

- <an unknown not yet sharp enough to ticket>
- …

## Decisions Ledger

- <one-line decision> — <in-repo artifact path or URL> · <resolving ticket STA-xxx>
- …

## Related Artifacts

- <in-repo design docs, diagrams, ADRs, by path>

## Out of Scope

- <work ruled beyond the destination>
```

- **User Story**: put each clause on its own line — `As a …,` / `I want …,` / `so that … .`
- **Milestones**: one per Definition-of-Done top-level item, each with a target date.
- **Issues**: the tickets of the epic, created through the `linear` skill, each carrying its label
  and its `blockedBy` edges.

## The Fog Log

The fog log is the "Open Questions (Fog)" section. It lists unknowns that block the destination but
are too dim to phrase as a ticket.

**Fog-versus-ticket test:** if the question is sharp enough to state now, create a SPIKE or DESIGN
ticket for it and wire its blocking edge. If it is too dim to phrase precisely, keep it in the fog
log until it sharpens.

The **frontier** is the set of open, unblocked, unassigned issues. It shows what a session can take
next. Chart the fog breadth-first: surface the whole frontier before you study one thread in depth.

## Charting the Fog by Artifact

Chart the fog by the artifacts the epic must produce. After the destination, ask one question:
what artifacts do we build toward? Propose a first list. Refine it with the user.

Chart the fog in these categories:

- **Design documents** — the design docs, diagrams, and ADRs that describe the code.
- **Abstractions** — the traits, types, and interfaces the code needs.
- **Implementations** — the concrete implementations, adapters, and states that satisfy the
  abstractions.
- **Tests** — the unit tests, integration tests, and fixtures.
- **Dependencies** — the crates and external services the epic adds.

Structure-specific artifacts refine these categories. For a SuperState, the first question is which
sub-states it needs. Each sub-state is its own artifact, and each becomes its own ticket.

Propose a first list per category. The user confirms or corrects it. An undecided artifact stays as
fog. A decided artifact becomes a ticket and a Definition-of-Done item.

## Milestones Follow the Artifacts

Milestones sequence the artifacts toward the destination. Once the artifact list is shared, propose
milestones as an ordered grouping of those artifacts. Anchor the final set to the destination.

For a SuperState, the milestones follow the sub-states in order:

- Define the first sub-state.
- Define the second sub-state and its transition from the first.
- Define the third sub-state and its transition from the second.
- Finalize the design document.

**Sequence an abstraction before its implementation.** When an implementation depends on an
abstraction, the abstraction settles in an earlier milestone. A trait settles before the concrete
type that implements it. The ports settle before their backend adapter. This does not override the
sub-state order above. A sub-state combines its abstraction and its implementation into one unit.

Each milestone is a coherent increment the epic can reach and show. Propose the milestones. The user
confirms or corrects them.

## Tickets Follow the Milestones and the Fog

Tickets have two sources:

- A **milestone** breaks into the build tickets that reach it — FEATURE and IMPLEMENTATION tickets.
- The **fog** produces decision tickets — SPIKE and DESIGN — as an unknown sharpens.

When a milestone is the next to reach, break it into tickets. Do not break every milestone at the
start. A full upfront breakdown is a premature backlog. Cut the current milestone's tickets. Defer
the later milestones.

### The Development Cycle

Every unit of work runs the same fixed cycle. Cut its tickets in this order.

1. **Clear the fog.** If the unit is uncertain, cut a SPIKE. After the SPIKE, if the design needs
   its own step, cut a DESIGN ticket. These produce the findings and the high-level design.
2. **Settle the abstraction.** If the unit has an abstraction, it is a trait. Cut a ticket to name
   and implement the trait. Cut a ticket to test it through a fake, injected as a dependency.
   Refactor the trait until it is stable, because changing it later forces every dependent to change
   with it.
3. **Build the implementation.** Cut a ticket for the concrete type behind the trait.

A pure domain concept has no abstraction, so it skips step 2. `Cik` is a value with validation, so
it is implementation only. `SecClient` is a port, so it runs the full cycle: trait, fake, then the
real client.

Each step maps to a skill. The `domain-concept` skill builds a trait or a domain type with its
validation, errors, fake, and tests. It covers step 2 and the pure domain concept. A sub-state runs
through the `state-design`, `state-scaffold`, and `state-implementation` skills. A transition runs
through the `transition-design` and `transition-implementation` skills. The abstraction and its fake
settle before the implementation, the same order the milestones follow.

## Mode: Define

Use this mode to establish a new epic, or to expand a bare project. It sets the destination, the
artifacts, the milestones, and the first frontier of tickets.

1. **Gather the destination.** Use `AskUserQuestion` for the goal, the scope, and the out-of-scope
   boundary.
2. **Chart the artifact fog.** Propose the artifacts the epic must produce, by category (design
   documents, abstractions, implementations, tests, dependencies). Grill the user until the list is
   shared. For a SuperState, ask which sub-states it needs first. For requirements, use the
   `state-design` skill. For the domain model, use the `domain-concept` skill.
3. **Write the Project.** Create or update the Linear Project with the description template above
   (`mcp__Linear__save_project`). State the destination in the User Story and the Description.
4. **Create milestones** from the artifacts, sequenced toward the destination (see "Milestones
   Follow the Artifacts"). Each milestone is one Definition-of-Done top-level item. Give each a
   target date (`mcp__Linear__save_milestone`).
5. **Seed the decisions ledger** with decisions already made. Each entry links its in-repo artifact
   and the ticket that resolved it.
6. **Seed the fog log** from the artifact categories. Apply the fog-versus-ticket test: create a
   SPIKE or DESIGN ticket for each sharp question now. Wire its blocking edge.
7. **Create the first design doc** in the repo, under the owning crate's `design/` directory, with a
   mermaid diagram. Link it under Related Artifacts.

## Mode: Refine

Use this mode to advance an existing epic as work proceeds.

1. **Load the Project** — its description, milestones, issues, fog log, and ledger.
2. **Take one frontier ticket.** The open, unblocked, and unassigned issues are takeable. Assign it
   to yourself. Then work it.
3. **When a decision resolves:** record its artifact. A design decision becomes a mermaid design
   doc or an ADR committed to the repo. A research finding becomes a repo doc or a Linear document.
   Then add a one-line entry to the decisions ledger, linking the artifact and the ticket. Remove
   the matching item from the fog log. Check its Definition-of-Done item or milestone.
4. **When fog sharpens:** create the SPIKE or DESIGN ticket. Wire its blocking edge.
5. **When a milestone is next:** break it into its build tickets (FEATURE or IMPLEMENTATION). Wire
   their blocking edges.
6. **Update the plan.** Remove resolved items from the fog log. Add new decisions to the ledger.
   Set each milestone's progress to match the work done.

Resolve one decision per session. Re-examine the fog afterward. A decision can reshape it. Research
tickets are the exception. They can run in parallel.

## Critical Invariants

- **The living design lives in the repo.** Every design doc, diagram, and ADR that describes the
  code is version-controlled in the repo, never in an external drawing tool. Research and SPIKE
  findings can live as Linear documents. The ledger links each artifact by its home.
- **The Project is the single map.** Do not create a separate map issue next to it.
- **One decision → one ticket → one artifact → one ledger entry.** A decision lives in exactly one
  place.
- **Plan versus build.** A decision produces a SPIKE or DESIGN ticket. The build produces a FEATURE
  or IMPLEMENTATION ticket.
- **Ticket Sizing applies.** Every ticket the epic spawns follows the `linear` skill's Ticket
  Sizing rule — one ticket, one reviewable PR.
- **One decision per session.** Resolve a single decision. Re-examine the fog before the next one.
  Research tickets can run in parallel.
- **Claim before work.** Assign a ticket to yourself before working it, so two sessions never take
  the same one.
- **Refer by name.** Name an issue or milestone by its title, not a bare id.

## Authoritative Sources

- `plain-english` skill — the wording of every description, ticket, ledger entry, and design doc this skill writes.
- `linear` skill — ticket templates, labels, and the Ticket Sizing rule.
- `AGENTS.md` and `DOCUMENTATION.md` — the conventions for in-repo design docs and diagrams.
- The `SEC ETL Pipeline - Extract SuperState` project — the reference epic to model.

## Self-Improvement

After an epic session where the user corrected the shape or the workflow:

1. Ask whether the fix belongs in this skill.
2. Add the confirmed pattern to the project template or the mode steps.

Common additions: a description section that proved useful, a fog-log rule, a clearer boundary
between the plan in Linear and the artifacts in the repo.
