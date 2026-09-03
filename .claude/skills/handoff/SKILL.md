---
name: handoff
description: >
  Use when the user asks to "hand off", "create a handoff", "start a new agent session",
  "spin up a driver session", "compact this session for another agent", or invokes `/handoff`.
  Compacts the current conversation into a self-contained prompt that a fresh Claude Code session
  can act on without this session's context. Adapted to the Claude Code Remote `create_session`
  flow.
version: 0.1.0
argument-hint: "[what the next session will do]"
allowed-tools: [Read, Write, Bash, AskUserQuestion]
---

# Handoff Skill

## Purpose

Compact the current conversation into a **standalone session prompt**. A fresh Claude Code session
reads only that prompt, so it must carry every fact the successor needs. The successor has none of
this session's context.

Use this to spawn a worker session (a PR driver, a long build, a parallel task) through the Claude
Code Remote `create_session` tool, or to hand the user a prompt they paste into a new session.

## When to Use

- The user wants a fresh session to own a bounded task while this session continues or ends.
- A task is large enough to run on its own (drive a PR to green, run a migration, a long refactor).
- The user says "hand off", "start a new session for this", or invokes `/handoff`.

## When Not to Use

- To leave a resumable note on an **In Progress Linear ticket** — use the `linear` skill's
  state-of-play comment instead. That note persists on the ticket. A handoff prompt is ephemeral
  and feeds a new agent session.
- To write project documentation — use the `documentation` skill.

## The Handoff Document

The output is one Markdown document. It is a prompt for a fresh session, so write it in the second
person and make it self-contained. Use this template:

```markdown
# Mission

<One or two sentences. The exact outcome the new session must reach. From the user's argument.>

# Context

<What the repository is, the working branch, and where the work stands right now. Name the crate or
package in scope.>

# Current State

- Done: <what already landed — commits, merged PRs, files written>
- In flight: <open branches, open PRs with numbers, tickets In Progress>
- Blocked / undecided: <anything waiting on a decision, and what would unblock it>

# Key Decisions

<Each decision the successor must respect, one line each, with a link to the ADR, design doc, or
ticket that records it. Reference — never restate the full content.>

# Artifacts

<Every artifact the successor needs, by path or URL, never duplicated:
- Linear tickets (STA-xxx + URL)
- PRs (owner/repo#number)
- branches (name)
- design docs / ADRs (repo path)>

# Suggested Skills

<The skills the successor must load for this task, for example `state-implementation`,
`domain-concept`, `linear`, `plain-english`. Name each and say when to use it.>

# Next Steps

1. <Ordered, concrete first action.>
2. <...>

# Guardrails

<The constraints the successor must hold: the branch it must push to, the AGENTS.md rules that
apply, and any "never" from this session. State each as `must` or `never`.>

# Reporting

<When and how to report back: report on completion or on a blocker it cannot resolve.>
```

## Procedure

1. **Read the argument.** If the user gave no focus, ask once with `AskUserQuestion`: what will the
   next session do, and does it end at a branch, a merged PR, or a report?
2. **Compile the document** from the conversation, following the template. Reference every artifact
   by path or URL. Do not paste file contents the successor can read itself.
3. **Redact secrets.** Remove API keys, tokens, passwords, and personal data. Reference the
   secret's location instead, never its value.
4. **Write the document** to the scratchpad directory named in the system prompt, as
   `handoff-<short-slug>.md`. Do not commit it. It is ephemeral context, not repository content.
5. **Offer the two paths** with the finished document:
   - **Spawn a session** — call the Claude Code Remote `create_session` tool with the arkad repo as
     the source and the document as the `prompt`. Load the tool through `ToolSearch` first.
   - **Hand it over** — give the user the document to paste into a new session themselves.

## Critical Invariants

- **Self-contained.** The successor sees only this prompt. If a fact is not in it, the successor
  does not have it.
- **Reference, do not duplicate.** Link tickets, PRs, branches, and docs. Never copy their bodies.
- **Redact before writing.** No secret value reaches the document.
- **Ephemeral.** Write to the scratchpad, never to the repository. A handoff prompt is not a design
  doc.
- **Plain English.** The `plain-english` skill applies. State the mission, the state, and the next
  steps as facts and instructions.

## Self-Improvement

After a handoff where the successor missed context or the user corrected the shape:

1. Ask whether the fix belongs in this skill.
2. Add the confirmed pattern to the template or the procedure.

Common additions: a template section the successor needed, a guardrail that was easy to miss, a
skill that should be suggested for a class of task.
