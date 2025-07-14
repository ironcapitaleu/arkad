//! # Prepare SEC Request Context Module
//!
//! This module defines the context data structures and updaters for the [`PrepareSecRequest`](../mod.rs) state in the SEC filings extraction workflow.
//!
//! The context provides stateful information required during the preparation of an SEC request, such as retry configurations. It is designed to be used with the [`ContextData`] trait, enabling ergonomic context management and updates within state machines.
//!
//! ## Components
//! - [`PrepareSecRequestContext`]: Holds the current context for preparing an SEC request.
//! - [`PrepareSecRequestContextUpdater`]: Updater type for modifying context fields in a controlled way.
//! - [`PrepareSecRequestContextUpdaterBuilder`]: Builder for constructing context updaters with a fluent API.
//!
//! ## Usage
//! The context is used by the [`PrepareSecRequest`](../mod.rs) state to manage retry logic. It supports updates via the builder pattern, making it easy to compose context changes in state machine workflows.
//!
//! ## Example
//! ```rust
//! use sec::implementations::states::extract::prepare_sec_request::psr_context::*;
//! use state_maschine::prelude::*;
//!
//! let mut context = PrepareSecRequestContext::default();
//! let update = PrepareSecRequestContextUpdaterBuilder::new()
//!     .max_retries(5)
//!     .build();
//! context.update_context(update);
//! assert_eq!(context.max_retries, 5);
//! ```
//!
//! ## See Also
//! - [`crate::traits::state_machine::state::ContextData`]: Trait for context data management in states.
//! - [`crate::implementations::states::extract::prepare_sec_request`]: Parent module for the SEC request preparation state and data types.

use std::fmt;

use state_maschine::prelude::ContextData as SMContextData;

use crate::traits::state_machine::state::ContextData;

#[derive(Debug, Default, Clone, PartialEq, PartialOrd, Hash, Eq, Ord)]
/// State context for the SEC request preparation state.
pub struct PrepareSecRequestContext {
    pub max_retries: u32,
}

impl PrepareSecRequestContext {
    #[must_use]
    /// Creates a new instance of the state context for SEC request preparation.
    pub const fn new() -> Self {
        Self { max_retries: 0 }
    }
}

impl ContextData for PrepareSecRequestContext {
    /// Returns the maximum number of retries allowed for the SEC request.
    fn get_max_retries(&self) -> u32 {
        self.max_retries
    }
}

impl SMContextData for PrepareSecRequestContext {
    type UpdateType = PrepareSecRequestContextUpdater;

    /// Returns a reference to the current context.
    fn get_context(&self) -> &Self {
        self
    }

    /// Updates the context fields using the provided updater.
    ///
    /// Only fields set in the updater will be changed.
    fn update_context(&mut self, updates: Self::UpdateType) {
        if let Some(max_retries) = updates.max_retries {
            self.max_retries = max_retries;
        }
    }
}

impl fmt::Display for PrepareSecRequestContext {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Context Data:\nMax retries: {}", self.max_retries)
    }
}

#[derive(Debug, Clone, PartialEq, PartialOrd, Hash, Eq, Ord)]
/// Updater for the state context.
///
/// Using this struct allows you to update fields of `PrepareSecRequestContext` in a controlled way.
pub struct PrepareSecRequestContextUpdater {
    pub max_retries: Option<u32>,
}

/// Builder for `PrepareSecRequestContextUpdater`.
///
/// Use this builder to fluently construct an updater for the context.
pub struct PrepareSecRequestContextUpdaterBuilder {
    max_retries: Option<u32>,
}
impl PrepareSecRequestContextUpdaterBuilder {
    /// Creates a new updater builder with no fields set.
    #[must_use]
    pub const fn new() -> Self {
        Self { max_retries: None }
    }

    /// Sets the max_retries value inside the context to the provided update value.
    ///
    /// # Arguments
    /// * `max_retries` - The new value for max_retries.
    #[must_use]
    #[allow(clippy::missing_const_for_fn)]
    pub fn max_retries(mut self, max_retries: u32) -> Self {
        self.max_retries = Some(max_retries);
        self
    }

    /// Builds the updater with the specified fields.
    #[must_use]
    pub const fn build(self) -> PrepareSecRequestContextUpdater {
        PrepareSecRequestContextUpdater {
            max_retries: self.max_retries,
        }
    }
}

impl Default for PrepareSecRequestContextUpdaterBuilder {
    /// Returns a new context update builder with no fields set.
    fn default() -> Self {
        Self::new()
    }
}
