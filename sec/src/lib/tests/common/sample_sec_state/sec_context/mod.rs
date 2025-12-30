//! # Sample SEC State Context
//!
//! This module defines the context structures and updaters for the [`SampleSecState`](../mod.rs) fixture.
//!
//! The context provides information to demonstrate how context is handled
//! within a state. It is designed for use with the [`Context`] trait for testing and demonstration.
//!
//! ## Components
//! - [`SampleSecStateContext`]: Holds the current context for the sample state.
//! - [`SampleSecStateContextUpdater`]: Updater type for modifying context fields in a controlled way.
//! - [`SampleSecStateContextUpdaterBuilder`]: Builder for constructing context updaters.
//!
//! ## Usage
//! The context is used by the [`SampleSecState`](../mod.rs) fixture to show how a state can access
//! environmental or shared data. It supports updates via the builder pattern.
//!
//! ## See Also
//! - [`crate::traits::state_machine::state::Context`]: Trait for context management in states.
//! - [`crate::tests::common::sample_sec_state`]: Parent module for the sample state fixture.
use std::fmt;

use state_maschine::prelude::Context as SMContext;

use crate::traits::state_machine::state::Context;

#[derive(Debug, Clone, PartialEq, PartialOrd, Hash, Eq, Ord)]
/// State context for the `SampleSecState` fixture.
pub struct SampleSecStateContext {
    pub data: String,
    pub max_retries: u32,
}

impl SampleSecStateContext {
    /// Creates a new instance of the sample state context.
    pub fn new(data: &(impl ToString + ?Sized)) -> Self {
        Self {
            data: data.to_string(),
            max_retries: 0,
        }
    }

    /// Returns a reference to the context's inner data string.
    #[must_use]
    pub const fn data(&self) -> &String {
        &self.data
    }
}

impl Context for SampleSecStateContext {
    fn max_retries(&self) -> u32 {
        self.max_retries
    }
}

impl SMContext for SampleSecStateContext {
    type UpdateType = SampleSecStateContextUpdater;

    fn context(&self) -> &Self {
        self
    }

    fn update_context(&mut self, updates: Self::UpdateType) {
        if let Some(data) = updates.data {
            self.data = data;
        }
    }
}

impl Default for SampleSecStateContext {
    fn default() -> Self {
        Self::new("Default Data")
    }
}

impl fmt::Display for SampleSecStateContext {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Context Data: {}", self.data)
    }
}

#[derive(Debug, Clone, PartialEq, PartialOrd, Hash, Eq, Ord)]
/// Updater for the state context.
pub struct SampleSecStateContextUpdater {
    pub data: Option<String>,
}

/// Builder for `SampleSecStateContextUpdater`.
pub struct SampleSecStateContextUpdaterBuilder {
    data: Option<String>,
}
impl SampleSecStateContextUpdaterBuilder {
    #[must_use]
    pub const fn new() -> Self {
        Self { data: None }
    }

    #[must_use]
    #[allow(clippy::missing_const_for_fn)]
    pub fn data(mut self, data: &(impl ToString + ?Sized)) -> Self {
        self.data = Some(data.to_string());
        self
    }

    #[must_use]
    pub fn build(self) -> SampleSecStateContextUpdater {
        SampleSecStateContextUpdater { data: self.data }
    }
}

impl Default for SampleSecStateContextUpdaterBuilder {
    fn default() -> Self {
        Self::new()
    }
}
