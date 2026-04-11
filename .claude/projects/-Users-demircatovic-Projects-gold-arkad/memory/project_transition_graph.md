---
name: StateMachineGraph trait — removed, to be restored later
description: Removed StateMachineGraph trait with const EDGES/TERMINAL_STATES, adjacency list, predecessors, initial states, and mermaid generation. Code preserved here for reuse when building the Step Functions dashboard.
type: project
---

Removed `StateMachineGraph` trait to keep the codebase lean until the dashboard is needed.

**Why:** Not needed yet — `NonTerminal` + `IntoStateMachineStream` handle streaming without it. Will be needed for the Step Functions-style dashboard and visualization features.

**How to apply:** Restore the trait in `sec/src/lib/traits/state_machine/graph.rs`, add `pub mod graph` to `mod.rs`, re-export from `prelude.rs`, and implement for `ExtractSuperState`.

## Trait definition (`graph.rs`)

```rust
use std::collections::{HashMap, HashSet};

pub trait StateMachineGraph {
    const EDGES: &'static [(&'static str, &'static str)];
    const TERMINAL_STATES: &'static [&'static str];

    #[must_use]
    fn collect_states() -> HashSet<&'static str> {
        let mut states = HashSet::new();
        for (from, to) in Self::EDGES {
            states.insert(*from);
            states.insert(*to);
        }
        for terminal in Self::TERMINAL_STATES {
            states.insert(*terminal);
        }
        states
    }

    #[must_use]
    fn build_adjacency_list() -> HashMap<&'static str, Vec<&'static str>> {
        let mut adj: HashMap<&'static str, Vec<&'static str>> = HashMap::new();
        for state in Self::collect_states() {
            adj.entry(state).or_default();
        }
        for (from, to) in Self::EDGES {
            adj.entry(*from).or_default().push(*to);
        }
        adj
    }

    #[must_use]
    fn build_predecessor_list() -> HashMap<&'static str, Vec<&'static str>> {
        let mut preds: HashMap<&'static str, Vec<&'static str>> = HashMap::new();
        for state in Self::collect_states() {
            preds.entry(state).or_default();
        }
        for (from, to) in Self::EDGES {
            preds.entry(*to).or_default().push(*from);
        }
        preds
    }

    #[must_use]
    fn find_initial_states() -> Vec<&'static str> {
        let preds = Self::build_predecessor_list();
        preds
            .into_iter()
            .filter(|(_, p)| p.is_empty())
            .map(|(state, _)| state)
            .collect()
    }

    #[must_use]
    fn generate_mermaid() -> String {
        use std::fmt::Write;
        let mut diagram = String::from("graph LR\n");
        for (from, to) in Self::EDGES {
            let from_id = from.replace(' ', "_");
            let to_id = to.replace(' ', "_");
            let _ = writeln!(diagram, "    {from_id}[\"{from}\"] --> {to_id}[\"{to}\"]");
        }
        for terminal in Self::TERMINAL_STATES {
            let id = terminal.replace(' ', "_");
            let _ = writeln!(diagram, "    {id}:::terminal");
        }
        diagram.push_str("    classDef terminal fill:#f96,stroke:#333\n");
        diagram
    }
}
```

## Implementation for ExtractSuperState (`extract/mod.rs`)

```rust
use execute_sec_request::constants::STATE_NAME as EXECUTE_SEC_REQUEST;
use prepare_sec_request::constants::STATE_NAME as PREPARE_SEC_REQUEST;
use validate_cik_format::constants::STATE_NAME as VALIDATE_CIK_FORMAT;

impl<S: State> StateMachineGraph for ExtractSuperState<S> {
    const EDGES: &'static [(&'static str, &'static str)] = &[
        (VALIDATE_CIK_FORMAT, PREPARE_SEC_REQUEST),
        (PREPARE_SEC_REQUEST, EXECUTE_SEC_REQUEST),
    ];
    const TERMINAL_STATES: &'static [&'static str] = &[EXECUTE_SEC_REQUEST];
}
```

State name constants come from each state module's `constants.rs` (e.g., `validate_cik_format::constants::STATE_NAME`) to stay in sync with actual state definitions.
