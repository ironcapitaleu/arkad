---
name: steward
description: >
  Use right after opening a PR, and on every review event on a PR you own.
version: 0.1.0
---

# Steward

## Purpose

Makes `pr-iterate` the default after a PR opens.

## Procedure

1. After you open a PR, run the `pr-iterate` skill on it at once. Do not wait for a request.
2. On a review event, continue the `pr-iterate` loop until one of its exit conditions holds.
