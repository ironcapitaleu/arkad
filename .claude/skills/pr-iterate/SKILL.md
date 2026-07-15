---
name: pr-iterate
description: >
  This skill should be used when the user asks to "iterate on the PR", "handle PR feedback",
  "address review comments", "drive the PR loop", "implement PR review feedback", or wants
  to enter a tight iterate-on-PR-feedback workflow. It drives the cycle of: read review
  comments → implement valid feedback → inform the human → request re-review → repeat.
version: 0.1.0
argument-hint: "[PR number or URL]"
allowed-tools: [Read, Write, Edit, Bash, AskUserQuestion, Agent]
---

# PR Iterate Skill

## Purpose

Drive a tight feedback loop on a PR: read review comments, implement clearly valid feedback,
inform the human about changes made and ambiguous feedback, request a re-review, and repeat
until the PR is clean.

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

### Step 7: Monitor and Repeat

After requesting re-review:
1. Wait for the review to complete (check workflow runs)
2. Fetch new comments
3. If new feedback exists, go back to Step 2
4. If no new feedback or only approvals, report to the human: "PR is clean, ready to merge"

### Exit Conditions

Stop iterating when:
- The review comes back clean (no new comments)
- All remaining comments are ambiguous/human-decision items (escalated)
- The human says to stop
- 3 iterations have passed without convergence (flag to human)

## Resolving Conversations

When you implement feedback from a specific review comment:
- Resolve that conversation thread on GitHub if the platform supports it
- Only resolve comments YOU addressed — never resolve human-escalated items

```bash
# Resolve a review comment thread (if API supports it)
gh api repos/{owner}/{repo}/pulls/{pr_number}/comments/{comment_id}/replies \
  --method POST -f body="Implemented — see commit {sha}"
```

## Guardrails

- **Never force-push** during iteration
- **Never resolve a comment without implementing it** (or explicitly explaining why it was skipped)
- **Max 3 autonomous iterations** before checking in with the human
- **Never change the PR scope** — if feedback suggests new features or large refactors, flag
  to the human as a potential follow-up ticket
- **Follow AGENTS.md** — all code changes must follow the project's development guidelines
- **Commit granularity:** One commit per iteration batch, not one commit per comment

## Integration with CI

The skill works alongside the `claude.yaml` workflow:
- The workflow's `claude-auto-review` job does the initial review on PR open
- This skill handles the iteration after that first review (or any subsequent `@claude review`)
- You can invoke this skill at any point to catch up on unaddressed feedback

## Example Invocation

User: "iterate on the PR"
→ Skill fetches PR #111's review comments
→ Implements 3 clearly valid fixes
→ Asks human about 1 architectural question
→ Commits, pushes, comments `@claude review the latest changes`
→ Waits for new review
→ Reports: "New review is clean. PR ready to merge."
