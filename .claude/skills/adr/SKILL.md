---
name: adr
description: >
  Use when the user asks to "write an ADR", "record this decision", "amend the ADR", "supersede
  the ADR", or asks "does this need an ADR" or "why is the code like this". Also use when a task
  makes you choose between options and the test in AGENTS.md, "Architecture Decision Records",
  says to write an ADR. Also use when you remove history or a past decision from a document, to
  check whether the decision needs an ADR first.
version: 0.2.0
---

# ADR Skill

## Purpose

An Architecture Decision Record (ADR) keeps one decision, the options, and the facts behind it in
the repository. It records the best way forward known at its date. A later reader learns why one
option won without the ticket or the chat. The current documents say what the code does, and the
ADRs say why. This skill
holds the template and the procedure. `AGENTS.md`, "Architecture Decision Records", holds the test
for when a decision needs an ADR.

## Procedure

- **To answer "why is the code like this"**, search `docs/adr/` and the design docs that describe
  the code. Cite the ADR that holds the decision. If none holds it, say so, and name the commit or
  PR that `git log` shows for the change. Write no ADR unless the user asks for one.
- **Before you remove a past decision from a document**, run the test below on that decision. If
  it passes and no ADR holds it, write the ADR in the same PR. Set `Date` to the date of the
  decision, and name the source of that date.

To write an ADR:

1. **Run the test** of `AGENTS.md`, "Architecture Decision Records". If the test is unclear, name
   the decision and both options in the PR description, and ask the owner.
2. **Search `docs/adr/`** for an ADR on the same subject.
   - If an accepted ADR covers the subject and the decision stays, link to it and stop.
   - If the decision changes in part, add an amendment (see "Amend or Supersede").
   - If the decision changes as a whole, write a new ADR that supersedes the old one.
3. **Collect the facts before you write.** Measure each option that you can measure. Write down
   the command or the source, the date, and the result. A reason without a number, a source, or a
   mechanism is an opinion. Leave it out or mark it as open.
4. **Pick the number.** Find the highest number in `docs/adr/` on `main`. Check each open PR that
   adds a file to `docs/adr/` for a higher number, because `main` does not show it yet. Add 1,
   and use four digits. Name the file `NNNN-<short-description>.md`, with a few kebab-case
   words that name the subject.
5. **Write the ADR** from the template. Follow the `plain-english` skill.
6. **Link it.** Each design doc that describes the affected code links to the ADR. If the decision
   adds a rule for all code, add the rule to `AGENTS.md` with a link to the ADR.
7. **Ship it** in the PR that first acts on the decision. A decision from a ticket without a PR,
   such as a SPIKE, ships in the first PR that changes the code. A past decision that leaves a
   document ships in the PR that removes it. Name the ADR in the PR description, and say who
   confirms it.

## Template

```markdown
# ADR NNNN: <The Decision in a Few Words>

| Field | Value |
| ----- | ----- |
| Status | <Proposed. <Who> confirms \| Accepted by <who> on YYYY-MM-DD \| Superseded by ADR NNNN> |
| Date | YYYY-MM-DD |
| Deciders | <who decides> |
| Ticket | <link to the ticket that needs the decision> |

## Context

<The problem and the facts that force a choice. Say which code, tickets, or runs the choice binds.
Define each term that the document uses in a narrow sense.>

## Decision

1. **<The decision as one sentence.>** <What follows from it.>
2. **<Each further part of the decision.>**

## Options Considered

Measured on YYYY-MM-DD <where, and on which commit or data>.

| Option | <Fact 1> | <Fact 2> |
| ------ | -------- | -------- |
| **A (chosen)** | | |
| B | | |

<One short list: why the chosen option won, and what each other option lacks.>

## Consequences

<What changes because of the decision: the gains, the costs, and each case that the decision does
not cover. One bullet each.>

## Open Questions

<What the decision leaves open, and who or what answers it. Name each event that reopens the
decision.>

## Sources

<Each source of a fact above: a file, a URL, a ticket comment, or a command, with its date.>
```

Keep the order of the six sections. Leave out a section only if it has no content. A long ADR can
add sections, as `docs/adr/0001-mvp-database.md` does.

## Amend or Supersede

- **Amend** when a part of the decision changes and the rest stays. Add a section
  `## Amendment N: <Title>` after "Decision", with its own date, reason, decision, and
  consequences. Add the amendment and its status to the `Status` row. Leave the original text as
  it is.
- **Supersede** when the decision changes as a whole. Write a new ADR. Set the `Status` row of the
  old ADR to "Superseded by ADR NNNN", and change nothing else in it.

## Critical Invariants

- One ADR holds one decision. Two decisions that can change apart from each other get two ADRs.
- An agent sets the status "Proposed". Only the decider named in the ADR accepts it.
- Never rewrite the decision of an accepted ADR. Amend it or supersede it.
- Every count, size, and time has a date and a source.
- State what the decision does not cover. A reader must not take a gap for a covered case.
- The reasons and the options live in the ADR only. `AGENTS.md` can state the rule that the
  decision adds, with a link to the ADR. A design doc links to the ADR and does not repeat the
  decision.

## Authoritative Sources

- `docs/adr/`: the ADRs of this project. `0002-static-binaries.md` is a short example.
- `AGENTS.md`, "Architecture Decision Records": the test for when to write an ADR.

## Self-Improvement

After the owner corrects the shape or the content of an ADR:

1. Ask whether the correction belongs in this skill.
2. Add the confirmed pattern to the template, the procedure, or the invariants.
