# State Maschine

Welcome to the State Maschine Library for Rust! This library provides a modular and flexible framework for implementing state machines in Rust, suitable for various applications such as workflow management, game development, and more.

## Features

- **State Management**: Define and manage states and their data with ease.
- **Transitions**: Handle complex state transitions, including hierarchical states and super states.
- **Trait Implementations**: Supports essential Rust traits like `Debug`, `Clone`, `PartialEq`, `Eq`, `Hash`, `Ord`, and `Unpin`.

## Modules

- **state_machine**: Contains core traits and implementations for state machines, states, super states, and transitions. The `StateMachine` trait defines the behavior and structure required for state machine implementations.

## Getting Started

To get started with this library, add it to your `Cargo.toml`:

```toml
[dependencies]
state_maschine = "0.1.1" 
