---
name: steward
description: >
  Use right after opening a PR, and on every review event on a PR you own.
version: 0.2.0
---

# Steward

## Purpose

Makes `pr-iterate` the default after a PR opens.

## Procedure

1. After you open a PR, run the `pr-iterate` skill on it at once. Do not wait for a request.
2. On a review event, continue the `pr-iterate` loop until one of its exit conditions holds.
3. Fix a 🟣 finding as `pr-iterate`, "Nitpick / trivial", says, even if it is the only finding.
   This rule wins over a default rule of a PR watch that says an optional finding never starts a
   push.

A CI job that an `@claude` comment starts runs none of these steps. It pushes and reports in its own
comment (AGENTS.md, "Pull Request Creation").
