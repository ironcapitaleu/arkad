---
name: handoff
description: >
  Use when the user asks to "hand off", "create a handoff", "start a new agent session",
  "spin up a driver session", "compact this session for another agent", "leave a state-of-play
  note", or invokes `/handoff`. Compacts the current conversation into a self-contained handoff:
  a prompt for a fresh agent session, or a state-of-play note on the In Progress Linear ticket for
  a human takeover. Also updates the stale points of that ticket.
version: 0.5.0
argument-hint: "[what the next session will do]"
allowed-tools: [Read, Write, Bash, AskUserQuestion]
---

# Handoff Skill

## Purpose

Compact the current conversation into a **standalone session prompt**. A fresh agent session reads
only that prompt, so it must carry every fact the successor needs. The successor has none of this
session's context.

The same compacted content serves two destinations: a fresh agent session, or a state-of-play
comment on the Linear ticket for a human takeover. This skill writes the content and stops. It does
not spawn a session or take over a ticket itself.

Before it writes the content, the skill brings the ticket up to date (see "Update the Ticket"). A
successor reads the ticket first. A stale ticket sends it the wrong way, even if the handoff is
right.

## When to Use

- The user wants a fresh session to own a bounded task while this session continues or ends.
- A task is large enough to run on its own (drive a PR to green, run a migration, a long refactor).
- Work on an In Progress ticket pauses and a person or agent must resume it later.
- The user says "hand off", "start a new session for this", or invokes `/handoff`.

## Destinations

One compacted handoff, rendered for the reader that receives it:

- **A fresh agent session** — print the content as a second-person prompt in one fenced code block
  in the chat reply. In a web, desktop, or mobile chat client, the block has a copy button. The user
  copies it with one tap, on a desktop or on a phone. Ephemeral.
- **A Linear ticket** — post the content as a state-of-play comment on the In Progress ticket. Use
  third-person status voice so a person can take over. Persistent. Delegate the comment shape to
  the `linear` skill. Do not copy its template here.

Produce either, or both, from the same content. If the destination is unclear, ask the user.

## When Not to Use

- To write project documentation — use the `documentation` skill.

## The Handoff Document

The output is one self-contained Markdown document. Write it in the second person for an agent
session, or in third-person status voice for a Linear comment. Use this template:

```markdown
# Mission

<One or two sentences. The exact outcome the new session must reach. From the user's argument.
Use the end from Procedure step 1: an open PR, a branch, a document, or a report.>

# Context

<What the repository is, the working branch, and where the work stands right now. Name the crate or
package in scope.>

# Current State

- Done: <what already landed — commits, merged PRs, files written>
- In flight: <open branches, open PRs with numbers, tickets In Progress>
- Blocked / undecided: <anything waiting on a decision, and what would unblock it>

# Roadmap

<Only when the ticket belongs to an ordered set (see Procedure step 4). One row per ticket, in
working order. Mark the successor's ticket with ▶.>

Project <name>, milestone <name>. All tickets link to <design ticket>. Statuses as of <YYYY-MM-DD>.

| # | Ticket | Scope | Blocked by | Status |
| - | ------ | ----- | ---------- | ------ |
| 1 | STA-xxx | <three to six words> | none | Done |
| 2 | STA-xxx | <three to six words> | none | In Review (merged, #N) |
| 3 | ▶ STA-xxx | <three to six words> | STA-xxx | Backlog |

A ticket is finished when it is Done or its PR has merged. Every other ticket is open.

Critical path: <the longest chain of open tickets linked by blockers, as STA-xxx → STA-xxx → …>
Unblocked now: <every ticket in Backlog or Todo whose blockers are all finished>

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

- <If the work changes the repository:> You must push to `<branch name>`. If the session assigns
  another branch, such as `claude/*`, never push to it. If the push to `<branch name>` fails, stop
  and report the blocker.
- <If the end is a PR:> If "In flight" names an open PR for the branch, push to it and open no new
  PR. Otherwise, you must open the PR against `main` without asking. Then run `pr-iterate`. Never
  merge it.
  <If the end is a document, a branch, or a report:> Open no PR.

# Reporting

<When and how to report back: report on completion or on a blocker it cannot resolve.>
```

### Roadmap Example

The backfill tickets of milestone "Data in Postgres", from the handoff of STA-334 on 2026-10-29.
The statuses are a snapshot of that day. Read the live statuses from Linear for every new handoff.

```markdown
# Roadmap

Project MVP Data, milestone "Data in Postgres". All tickets link to STA-328. Statuses as of
2026-10-29.

| # | Ticket | Scope | Blocked by | Status |
| - | ------ | ----- | ---------- | ------ |
| 1 | STA-333 | rename to `ExtractCompanyFactsSuperState` | none | In Review (merged, #270) |
| 2 | ▶ STA-334 | `ingestion_mode` run field | none | Backlog |
| 3 | STA-335 | `NotesPeriod` | none | Backlog |
| 4 | STA-337 | `FetchNotesFile` and builder | STA-335 | Backlog |
| 5 | STA-336 | `SecFile` and `download_sec_file` | none | Backlog |
| 6 | STA-338 | `RawNotesArchive` | STA-334, STA-336 | Backlog |
| 7 | STA-339 | `ValidateNotesPeriod` and `PrepareNotesRequest` | STA-335, STA-337 | Backlog |
| 8 | STA-340 | `DownloadNotesArchive` | STA-336, STA-339 | Backlog |
| 9 | STA-341 | `StoreNotesArchive` | STA-338, STA-340 | Backlog |
| 10 | STA-342 | `ExtractNotesSuperState` | STA-341 | Backlog |
| 11 | STA-343 | `Extraction` facade | STA-342 | Backlog |
| 12 | STA-344 | `notes_backfill` binary | STA-343 | Backlog |

A ticket is finished when it is Done or its PR has merged. Every other ticket is open.

Critical path: STA-335 → STA-337 → STA-339 → STA-340 → STA-341 → STA-342 → STA-343 → STA-344
Unblocked now: STA-334, STA-335, STA-336

STA-334 is off the critical path. Only STA-338 needs it, and STA-341 waits on both STA-338 and
STA-340. STA-340 ends a chain of four open tickets, and STA-334 → STA-338 is two, so STA-334 has
two steps of slack.
```

The last line is optional. Add it when the successor's ticket sits off the critical path, so the
successor knows how much slack it has.

## Update the Ticket

A handoff for a ticket updates that ticket in Linear, for every destination. Do not only tell the
user that a point is stale. Fix it, or post it.

Read the ticket, its comments, its relations, and its linked PRs through the `linear` skill.
Compare each point with the facts of this session. A point is stale if a commit, a PR, a Linear
status, or a decision of the user contradicts it.

**Fix in place** each stale point that a fact proves. Change nothing else.

| Stale point | Fix | Proof that the fix needs |
| ----------- | --- | ------------------------ |
| A Definition of Done item is done, and its box is open | Check the box | A merged PR or a commit on the default branch that does this item. Name it in "Fixed on this ticket" |
| The status does not match the work | Set the status | Work started, a PR is open, or a PR merged. Never set Done unless the user asks |
| The ticket has no assignee, and work started | Assign the user | The user owns this session |
| A blocker is missing | Add the relation | The ticket text, a design doc, or the user names the dependency |
| A PR or a branch is not linked | Add the link | The PR or the branch exists |

A merged PR proves only the items that it does. Check no box for work that no PR or commit shows.
To check a box, change only `- [ ]` to `- [x]` on that line. Read the description again right
before the write, so the write drops no edit of the author. Never remove a blocker relation. Linear
marks a finished blocker as resolved, and the relation records the dependency.

**Post in a comment** each stale point that a fix in place cannot hold. Examples: a changed scope,
a decision that replaces a sentence of the description, a Definition of Done item that no longer
fits, an old measurement, a split of the ticket, or a file path or symbol that moved. Never
rewrite the user story, the description, or the wording of a Definition of Done item. The author of
the ticket owns them. Quote the stale sentence, state the fact that contradicts it, and name the
source.

At the delivery step, leave **one** current comment on the ticket through the `linear` skill, in
status voice. If an earlier handoff left a state-of-play comment and the session can edit a
comment, edit that comment. Otherwise post a new one, and say in its first line which comment it
replaces.

- For the Linear destination, the state-of-play comment holds two more lists: "Fixed on this
  ticket" and "Stale, for the author".
- For the agent-session destination, post a short update comment with the same two lists, the
  branch, and the PR. The ticket then matches the handoff that the successor receives.

If the destination is an agent session and no point was fixed or is stale, post no comment.

Every write goes through the Linear tools of the session, by the rules of the `linear` skill. If
the session has no tool for a write, such as a comment, a relation, or a link, do not use another
way. Name the write that is missing in the chat reply.

In the handoff document, list each point of "Stale, for the author" under "Blocked / undecided",
so the successor does not act on the stale text. In the chat reply, tell the user what you fixed
and what you posted.

If a ticket of the Roadmap other than the handoff ticket is stale, fix only its status and a
missing blocker, with the same proof. Do this in Procedure step 4, when you read those tickets.
Report every other stale point of such a ticket to the user.

If Linear is unreachable, change nothing, and name each stale point under "Blocked / undecided".
If Linear fails after a fix, list the fixes already made in the chat reply.

## Procedure

1. **Read the argument.** If the user gave no focus, ask the user once. Ask what the next session
   will do. For work that changes the repository, the default end is an open PR that a review finds
   nothing new in. A person then reviews and merges it. For work whose output lives outside the
   repository, such as a SPIKE that ends in a Linear document, the end is that document and a
   report, with no PR. Use this end unless the user names a different one.
2. **Update the ticket** if the handoff is for a ticket (see "Update the Ticket"). Make the fixes
   in place before you compile, so the document names the ticket as it now stands. Post the
   comment in step 7.
3. **Compile the document** from the conversation, following the template. Reference every artifact
   by path or URL. Do not paste file contents the successor can read itself. Pick the branch name
   with the `AGENTS.md` "Branch Naming" convention.
4. **Add the Roadmap** if the ticket belongs to an ordered set: an epic (a Linear project), a
   project milestone, or the "Implementation Tickets" list of a design doc. See "Roadmap Example".
   Read every ticket's status, blockers, and linked PR through the `linear` skill at the time you
   write the handoff. Never copy a roadmap from an earlier handoff without this check. Fix a stale
   status or a missing blocker of a Roadmap ticket by the rule in "Update the Ticket". Then
   recompute the critical path and the unblocked tickets from the fresh data. If Linear is unreachable,
   leave the Roadmap out and say so under "Blocked / undecided". The Linear note carries the
   Roadmap too, with its date, so a reader can tell how old the statuses are.
5. **Redact secrets.** Remove API keys, tokens, passwords, and personal data. Reference the
   secret's location instead, never its value.
6. **Choose the destination** — a fresh agent session, a Linear note, or both (see Destinations).
7. **Deliver:**
   - **Agent session** — print the whole document in the chat reply, inside one fenced code block
     tagged `markdown`. Put nothing else inside the block. Count the longest run of backticks that
     opens a fence in the document. Open and close the outer block with one more backtick than that
     run. Use at least three. Do not split the document across several blocks.
     Write a file only in these cases: the user asks for one, a session-spawning tool needs one, or
     the session runs in a terminal. Write it to a temporary directory outside the repository, as
     `handoff-<short-slug>.md`. Never commit it.
   - **Linear note** — post the content as a state-of-play comment on the In Progress ticket through
     the `linear` skill, in status voice.
   - **Every destination** — the comment carries the two lists of "Update the Ticket". For an
     agent session alone, post the short update comment only if a point was fixed or is stale.

   This skill writes the handoff content. It does not spawn a session or take over the ticket
   itself.

## Critical Invariants

- **The ticket matches the handoff.** After a handoff, no point of the ticket contradicts the
  handoff without a comment that says so. Fix a stale point only with proof. Never rewrite the
  words of the author.
- **Self-contained.** The successor sees only this prompt. If a fact is not in it, the successor
  does not have it.
- **Reference, do not duplicate.** Link tickets, PRs, branches, and docs. Never copy their bodies.
  The Roadmap table is an index, not a copy.
- **Redact before writing.** No secret value reaches the document. This matters most for the Linear
  note, which persists and other people read.
- **Never in the repository.** The agent-session prompt goes to the chat, plus a temporary file if
  one is needed. The Linear note goes to the ticket. A handoff is not a committed design doc.
- **One copyable block.** The agent-session prompt is one fenced code block, so one tap copies all
  of it.
- **Branch convention.** A new branch follows the `AGENTS.md` "Branch Naming" convention,
  `<type>/<short-description>`. Never name a session-assigned branch, such as `claude/*`, for new
  work. If the work continues an open PR, name that PR's head branch as it is.
- **Plain English.** The `plain-english` skill applies. State the mission, the state, and the next
  steps as facts and instructions.

## Authoritative Sources

No external sources. This skill encodes internal procedure only.

## Self-Improvement

After a handoff where the successor missed context or the user corrected the shape:

1. Ask whether the fix belongs in this skill.
2. Add the confirmed pattern to the template or the procedure.

Common additions: a template section the successor needed, a guardrail that was easy to miss, a
skill the successor must load for a class of task, a kind of stale point that the table of "Update
the Ticket" lacks.
