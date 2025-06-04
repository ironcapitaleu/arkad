//! # SEC Context Data Trait
//!
//! This module defines the [`ContextData`] trait for SEC-specific state machines, extending the generic
//! [`state_maschine::state_machine::state::ContextData`] trait with domain-specific retry logic.
//!
//! Context data represents external or environmental information that may influence internal state computations
//! in the SEC state machine framework, but is usally not directly tied to or mutated by state transitions themselves.
//! Typical examples include retry policies, configuration parameters, or metadata required for workflows (e.g., time, ...).
//!
//! ## Usage
//! Implement [`ContextData`] for your SEC context data types to enable retry logic and context management
//! during state transitions. The trait enforces a consistent interface for querying retry capabilities and limits.
//!
//! See also:
//! - [`crate::traits::state_machine::state::StateData`]: For state data management.
//! - [`crate::implementations`]: For concrete context data implementations used in SEC ETL pipelines.
//! - [`crate::error`]: For error types used in context-aware operations.

use state_maschine::prelude::ContextData as SMContextData;

/// Trait for SEC-specific context data, extending the generic state machine context data trait with retry logic.
///
/// Implement this trait for SEC context data types to provide custom retry policies and metadata.
pub trait ContextData: SMContextData {
    /// Returns `true` if the state can be retried, based on the maximum allowed retries.
    fn can_retry(&self) -> bool {
        self.get_max_retries() > 0
    }

    /// Returns the maximum number of retries allowed for the state.
    fn get_max_retries(&self) -> u32;
}
