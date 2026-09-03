---
name: pr-iterate
description: >
  This skill should be used when the user asks to "iterate on the PR", "handle PR feedback",
  "address review comments", "drive the PR loop", "implement PR review feedback", or wants
  to enter a tight iterate-on-PR-feedback workflow. It drives the cycle of: request a review on
  the current head → implement valid feedback → request a fresh review → repeat until nothing new
  → then notify the human.
version: 0.2.0
argument-hint: "[PR number or URL]"
allowed-tools: [Read, Write, Edit, Bash, AskUserQuestion, Agent]
---

# PR Iterate Skill

## Purpose

Drive a tight feedback loop on a PR until a fresh review returns nothing new, then notify the human.

**What "iterate" means.** To iterate on a PR is to request a review on the current head, address
every new finding, and request another review. Repeat until a review returns nothing new. Only then
notify the human. Escalate an ambiguous finding to the human as it arises. Do not wait on the
human to trigger each re-review — a fresh review after every push is part of the loop, not a
checkpoint.

The auto-review runs only on PR open, so a new push is not reviewed on its own. Each round must
request a fresh review with an `@claude review` comment.

**Two environments.** In a local interactive session (terminal / IDE), use the `gh` CLI, and
`gh run watch` to wait for the review. In a remote session (for example, Claude Code Remote) subscribed to the
PR, use the GitHub MCP tools (`mcp__github__*`) instead of `gh`. Do not poll with `sleep`. Request
the review, end the turn, and let the re-review arrive as a PR-activity wake event.

## Philosophy

- **Autonomous on the obvious:** Implement feedback that is clearly correct (bugs, security
  issues, naming violations, missing error handling, style convention violations per AGENTS.md).
- **Escalate the ambiguous:** Anything subjective, architectural, or strategic goes to the human.
  Taste-based choices, scope decisions, and tradeoffs that could affect project direction.
- **Keep the human in the loop:** Always summarize what feedback was received, what was
  implemented (and why), and what needs human input — even for changes you made autonomously.
- **Proactive re-review:** After implementing a batch of changes, comment `@claude review the
  latest changes` on the PR to trigger a new review cycle.

## Workflow

### Step 1: Identify the PR

If no PR number is provided, determine it from:
1. The current branch: `gh pr view --json number,url`
2. Ask the user if ambiguous

### Step 2: Fetch Review Comments

```bash
gh api repos/{owner}/{repo}/pulls/{pr_number}/comments --paginate
gh pr view {pr_number} --json reviews,comments
```

In a remote session, use `mcp__github__pull_request_read` with methods `get_reviews`,
`get_comments`, and `get_check_runs` instead of `gh`. Post the `@claude review` comment with
`mcp__github__add_issue_comment`.

Categorize each comment:

| Category | Action | Examples |
| --- | --- | --- |
| **Clearly valid** | Implement immediately | Bug fix, security issue, naming rule violation, missing test, style per AGENTS.md |
| **Ambiguous / subjective** | Escalate to human | Architecture choice, scope question, "maybe we should...", taste-based |
| **Already resolved** | Skip | Marked as resolved, outdated by subsequent commits |
| **Nitpick / trivial** | Implement if cheap, otherwise batch | Typo, minor formatting |

### Step 3: Implement Valid Feedback

For each clearly valid comment:
1. Make the code change
2. Track what was changed and why (mapping comment → change)

Group related changes into a single commit where possible.

### Step 4: Report to Human

Present a summary to the user:

```
## PR Feedback Summary

### Implemented (N changes):
- [file:line] Fixed X because reviewer flagged Y
- [file:line] Added error handling for Z

### Needs Your Input (N items):
- [file:line] Reviewer suggests refactoring to X pattern — this is an architectural choice.
  Options: (a) do it, (b) push back with reason, (c) defer to separate ticket
- [file:line] "Should we add caching here?" — scope decision

### Skipped (N items):
- [file:line] Already resolved in previous commit
```

Use `AskUserQuestion` for ambiguous items if there are 1-3 of them. If more, present
the list and ask the human to respond.

### Step 5: Commit and Push

After implementing changes (both autonomous and human-directed):

```bash
git add <specific files>
git commit -m "refactor: address PR review feedback

- Fix X (reviewer comment on file:line)
- Add Y (reviewer comment on file:line)
- ..."
git push
```

### Step 6: Request Re-Review

Comment on the PR to trigger a new Claude review:

```bash
gh pr comment {pr_number} --body "@claude review the latest changes — I addressed the previous feedback"
```

### Step 7: Wait and Continue

After requesting the re-review, wait for it by the means the environment allows:

- **Local session:** use `gh run watch` in the background until the review workflow completes, then
  fetch the new findings.
- **Remote session subscribed to the PR:** end the turn. The re-review arrives as a PR-activity
  wake event. Do not poll with `sleep`. On the event, fetch the new findings.

Then:

1. If new findings exist, return to Step 2.
2. If a review returns nothing new, or only approvals, notify the human: "The review is clean, the
   PR is ready to merge."

If the session is interrupted, re-invoke `/pr-iterate` to resume. The skill fetches the latest
findings and continues.

### Exit Conditions

Stop iterating when:
- The review returns clean (no new comments)
- All remaining comments are ambiguous/human-decision items (escalated)
- The human says to stop
- 3 iterations have passed without convergence (flag to human)

## Replying to Addressed Comments

When you implement feedback from a specific review comment, reply to confirm:

```bash
gh api repos/{owner}/{repo}/pulls/{pr_number}/comments/{comment_id}/replies \
  --method POST -f body="Implemented — see commit {sha}"
```

Note: GitHub's REST API does not support resolving threads — only the GraphQL
`resolvePullRequestReviewThread` mutation does. Posting a reply confirms the fix
but does not auto-resolve. Only reply to comments YOU addressed — never reply to
human-escalated items.

## Guardrails

- **Never force-push** during iteration
- **Never resolve a comment without implementing it** (or explicitly explaining why it was skipped)
- **Max 3 autonomous iterations** before consulting the human
- **Never change the PR scope** — if feedback suggests new features or large refactors, flag
  to the human as a potential follow-up ticket
- **Follow AGENTS.md** — all code changes must follow the project's development guidelines
- **Commit granularity:** One commit per iteration batch, not one commit per comment

## Integration with CI

The skill works alongside the `claude.yaml` workflow:
- The workflow's `claude-auto-review` job does the initial review on PR open (triggered by `pull_request: opened`)
- The `claude` job handles `@claude` comments (triggered by `issue_comment: created`)
- When this skill posts `@claude review the latest changes`, it triggers the `claude` job (not `claude-auto-review`)
- You can invoke this skill at any point to address unaddressed feedback

## Example Invocation

User: "iterate on the PR"
→ Skill fetches PR #111's review comments
→ Implements 3 clearly valid fixes
→ Asks human about 1 architectural question
→ Commits, pushes, comments `@claude review the latest changes`
→ Waits for new review
→ Reports: "New review is clean. PR ready to merge."
