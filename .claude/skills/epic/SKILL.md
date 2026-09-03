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

As a …, I want …, so that … .

## Description

The goal of this epic, in a few sentences.

## Notes

Domain context and standing preferences that steer decisions.

## Definition of Done

- [ ] <top-level outcome — one per milestone>
- [ ] …

## Open Questions (Fog)

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

## Mode: Define

Use this mode to scaffold a new epic, or to flesh out a bare project.

1. **Gather the destination.** Use `AskUserQuestion` for the goal, the scope, and the out-of-scope
   boundary. For requirements, use the `state-design` skill. For the domain model, use the
   `domain-concept` skill.
2. **Write the Project.** Create or update the Linear Project with the description template above
   (`mcp__Linear__save_project`). State the destination in the User Story and the Description.
3. **Create milestones** from the Definition-of-Done top-level items
   (`mcp__Linear__save_milestone`), each with a target date.
4. **Seed the decisions ledger** with decisions already made. Each entry links its in-repo artifact
   and the ticket that resolved it.
5. **Seed the fog log** with the known unknowns. Apply the fog-versus-ticket test: create a SPIKE or
   DESIGN ticket for each sharp question now, and wire its blocking edge.
6. **Create the first design doc** in the repo, under the owning crate's `design/` directory, with a
   mermaid diagram. Link it under Related Artifacts.

## Mode: Refine

Use this mode to develop the epic as work proceeds.

1. **Load the Project** — its description, milestones, issues, fog log, and ledger.
2. **Take one frontier ticket** — the open, unblocked, unassigned issues are takeable. Claim it by
   assigning it to yourself, then work it.
3. **When a decision resolves:** record its artifact — a design decision as a mermaid design doc or
   an ADR committed to the repo, a research finding as a repo doc or a Linear document. Add a
   one-line entry to the decisions ledger linking the artifact and the ticket, remove the matching
   item from the fog log, and check its Definition-of-Done item or milestone.
4. **When fog sharpens:** create the SPIKE or DESIGN ticket, and wire its blocking edge.
5. **Keep the plan honest** — prune the fog log, keep the ledger current, and keep each milestone's
   progress true.

Resolve one decision per session, then re-examine the fog. A decision can reshape it. Research
tickets are the exception — they can run in parallel.

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
- **One decision per session.** Resolve a single decision, then stop and re-examine the fog.
  Research tickets can run in parallel.
- **Claim before work.** Assign a ticket to yourself before working it, so two sessions never take
  the same one.
- **Refer by name.** Name an issue or milestone by its title, not a bare id.

## Authoritative Sources

- `linear` skill — ticket templates, labels, and the Ticket Sizing rule.
- `AGENTS.md` and `DOCUMENTATION.md` — the conventions for in-repo design docs and diagrams.
- The `SEC ETL Pipeline - Extract SuperState` project — the reference epic to model.

## Self-Improvement

After an epic session where the user corrected the shape or the workflow:

1. Ask whether the fix belongs in this skill.
2. Add the confirmed pattern to the project template or the mode steps.

Common additions: a description section that proved useful, a fog-log rule, a clearer boundary
between the plan in Linear and the artifacts in the repo.
