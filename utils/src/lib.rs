/// Utilities for common adjacent tasks, interacting with backing services, or reading the config.
///
/// This crate provides reusable functions and types for connecting to and interacting with
/// external systems such as message queues (e.g., `RabbitMQ`).
/// These utilities are designed to support the "backing services"
/// principle of [The Twelve-Factor App](https://12factor.net/backing-services), enabling
/// applications to treat such resources as attached services.
///
/// Features
/// - Queue connectivity and messaging primitives
///
/// Features are extensible for additional third-party services
///

pub mod queue;

pub mod config;

#[cfg(test)]
pub mod tests;
