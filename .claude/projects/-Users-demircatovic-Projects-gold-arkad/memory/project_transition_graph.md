---
name: Transition graph visualization
description: Future plan to add a TransitionGraph trait that returns the state machine's transition graph as data, useful for visualization and mermaid diagram generation
type: project
---

Plan to add a `TransitionGraph` trait that returns the state machine's transition graph as runtime data. Useful for visualization, documentation, and generating mermaid diagrams.

**Why:** The `NonTerminal` trait defines the graph edge-by-edge at the type level. A complementary `TransitionGraph` trait could expose the same graph as runtime data for tooling.

**How to apply:** When implementing visualization or documentation features, consider generating `TransitionGraph` from the existing `NonTerminal` impls to keep a single source of truth.
