/// # State Machine Library
///
/// This crate provides a modular and flexible framework for implementing state machines in Rust.
/// It allows users to define states, transitions, and state machines with ease, making it suitable
/// for a wide range of applications, including workflow management, game development, and more.
///
/// ## Modules
///
/// - [`state_machine`]: Contains core traits and implementations for state machines, states, super
///   states, and transitions. This module provides the building blocks for creating and managing
///   state machines. The `StateMachine` trait is the centerpiece of this module, defining the
///   behavior and structure that all state machine implementations must follow.
///
/// ## Features
///
/// - **State Management**: Easily define and manage states and their data, ensuring a clear and
///   organized structure for state transitions.
/// - **Transitions**: Implement complex state transitions, including hierarchical states and super
///   states, to model real-world scenarios.
/// - **Trait Implementations**: Supports common Rust traits like `Debug`, `Clone`, `PartialEq`, `Eq`,
///   `Hash`, `Ord`, and `Unpin` to provide a rich and type-safe interface.
///
/// ## Example Usage
///
/// To use this library, you would typically define your state types and implement the required traits
/// from the `state_machine` module. Then, use these types to create a state machine that can run,
/// transition between states, and perform actions based on state changes.
///
/// Example usage might involve creating a custom state and state machine, but specific implementations
/// will depend on the application's requirements. Ensure your state types implement the necessary traits,
/// and use the state machine to manage transitions and behaviors.
///
/// ## Testing
///
/// The crate includes a comprehensive suite of tests to ensure the correctness of state machine behavior.
/// The tests cover various scenarios, including state transitions, output data computation, and trait
/// implementations. You can run these tests using the standard Rust testing framework by executing
/// `cargo test`.
///
/// ## Contribution
///
/// Contributions to this library are welcome. Please follow the standard guidelines for Rust projects
/// and ensure that your code is well-documented and tested before submitting a pull request.
///
/// ## License
///
/// This library is licensed under the MIT License. See the LICENSE file for details.
pub mod state_machine;

#[cfg(test)]
mod tests;
