//! # Execute SEC Request Context
//!
//! Provides the [`ExecuteSecRequestContext`] carried alongside the
//! [`ExecuteSecRequest`](crate::implementations::states::extract::execute_sec_request::ExecuteSecRequest)
//! state, together with its updater and builder.
//!
//! The context tracks the [`Cik`] being processed, the `execution_id` and the `User-Agent` of the
//! run, and the retry budget. All of them persist across updates and transitions.
//!
//! ## Usage
//!
//! ```rust
//! use sec::implementations::states::extract::execute_sec_request::context::*;
//! use sec::shared::cik::Cik;
//! use state_maschine::prelude::*;
//!
//! let cik = Cik::new("1234567890").expect("A hardcoded valid CIK should always parse");
//! # use sec::shared::user_agent::UserAgent;
//! # use uuid::Uuid;
//! let user_agent = UserAgent::new("Sample Company contact@example.com")
//!     .expect("Given a hardcoded SEC-format user agent, validation should always succeed");
//! let mut context = ExecuteSecRequestContext::new(cik, Uuid::new_v4(), user_agent);
//! let update = ExecuteSecRequestContextUpdater::builder()
//!     .cik(Cik::new("0987654321").expect("A hardcoded valid CIK should always parse"))
//!     .build();
//! context.update_context(update);
//!
//! let expected_result = "0987654321";
//!
//! let result = context.cik().to_string();
//!
//! assert_eq!(result, expected_result);
//! ```
//!
//! ## See Also
//!
//! - [`crate::traits::state_machine::state::Context`]: Trait that defines context updates and the retry budget.
//! - [`crate::implementations::states::extract::execute_sec_request`]: Parent module for the execution state and its data types.

use std::fmt;

use serde::Serialize;
use state_maschine::prelude::Context as SMContext;
use uuid::Uuid;

use crate::shared::cik::Cik;
use crate::shared::user_agent::UserAgent;
use crate::traits::state_machine::state::Context;

/// Context for the [`ExecuteSecRequest`](super::ExecuteSecRequest) state.
///
/// Tracks the [`Cik`] being processed, the `execution_id` and the `User-Agent` of the run, and
/// the retry budget. All of them persist across updates and transitions.
#[derive(Debug, Clone, PartialEq, PartialOrd, Hash, Eq, Ord, Serialize)]
pub struct ExecuteSecRequestContext {
    /// The Central Index Key (CIK) being processed in this execution context.
    pub cik: Cik,
    /// The `execution_id` of the run.
    pub execution_id: Uuid,
    /// The `User-Agent` that the run sends to the SEC.
    pub user_agent: UserAgent,
    /// Maximum number of times the state may be retried on failure.
    pub max_retries: u32,
}

impl ExecuteSecRequestContext {
    /// Creates a new [`ExecuteSecRequestContext`] from a CIK and the `execution_id` and the
    /// `User-Agent` of the run, with the retry budget at zero.
    #[must_use]
    pub const fn new(cik: Cik, execution_id: Uuid, user_agent: UserAgent) -> Self {
        Self {
            cik,
            execution_id,
            user_agent,
            max_retries: 0,
        }
    }

    /// Returns a reference to the CIK being processed.
    #[must_use]
    pub const fn cik(&self) -> &Cik {
        &self.cik
    }

    /// Returns the `execution_id` of the run.
    #[must_use]
    pub const fn execution_id(&self) -> Uuid {
        self.execution_id
    }

    /// Returns the `User-Agent` that the run sends to the SEC.
    #[must_use]
    pub const fn user_agent(&self) -> &UserAgent {
        &self.user_agent
    }
}

impl Context for ExecuteSecRequestContext {
    fn max_retries(&self) -> u32 {
        self.max_retries
    }
}

impl SMContext for ExecuteSecRequestContext {
    type UpdateType = ExecuteSecRequestContextUpdater;

    fn context(&self) -> &Self {
        self
    }

    fn update_context(&mut self, updates: Self::UpdateType) {
        if let Some(cik) = updates.cik {
            self.cik = cik;
        }
        if let Some(execution_id) = updates.execution_id {
            self.execution_id = execution_id;
        }
        if let Some(user_agent) = updates.user_agent {
            self.user_agent = user_agent;
        }
        if let Some(max_retries) = updates.max_retries {
            self.max_retries = max_retries;
        }
    }
}

impl fmt::Display for ExecuteSecRequestContext {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Context Data: {}", self.cik)
    }
}

/// Updater for modifying [`ExecuteSecRequestContext`].
///
/// Fields set to `None` are left unchanged when the updater is applied.
#[derive(Debug, Clone, PartialEq, PartialOrd, Hash, Eq, Ord)]
pub struct ExecuteSecRequestContextUpdater {
    /// Optional new value for the CIK.
    pub cik: Option<Cik>,
    /// Optional new value for the `execution_id`.
    pub execution_id: Option<Uuid>,
    /// Optional new value for the `User-Agent`.
    pub user_agent: Option<UserAgent>,
    /// Optional new value for the retry budget.
    pub max_retries: Option<u32>,
}

impl ExecuteSecRequestContextUpdater {
    /// Creates a new builder for constructing [`ExecuteSecRequestContextUpdater`] instances.
    #[must_use]
    pub const fn builder() -> ExecuteSecRequestContextUpdaterBuilder {
        ExecuteSecRequestContextUpdaterBuilder::new()
    }
}

/// Fluent builder for an [`ExecuteSecRequestContextUpdater`].
pub struct ExecuteSecRequestContextUpdaterBuilder {
    cik: Option<Cik>,
    execution_id: Option<Uuid>,
    user_agent: Option<UserAgent>,
    max_retries: Option<u32>,
}

impl ExecuteSecRequestContextUpdaterBuilder {
    /// Creates a new [`ExecuteSecRequestContextUpdaterBuilder`] with all fields initialized to `None`.
    #[must_use]
    pub const fn new() -> Self {
        Self {
            cik: None,
            execution_id: None,
            user_agent: None,
            max_retries: None,
        }
    }

    /// Sets the CIK field.
    #[must_use]
    #[allow(clippy::missing_const_for_fn)]
    pub fn cik(mut self, cik: Cik) -> Self {
        self.cik = Some(cik);
        self
    }

    /// Sets the `max_retries` field.
    #[must_use]
    #[allow(clippy::missing_const_for_fn)]
    pub fn max_retries(mut self, max_retries: u32) -> Self {
        self.max_retries = Some(max_retries);
        self
    }

    /// Sets the `execution_id` field.
    #[must_use]
    pub const fn execution_id(mut self, execution_id: Uuid) -> Self {
        self.execution_id = Some(execution_id);
        self
    }

    /// Sets the `User-Agent` field.
    #[must_use]
    pub fn user_agent(mut self, user_agent: UserAgent) -> Self {
        self.user_agent = Some(user_agent);
        self
    }

    /// Builds the [`ExecuteSecRequestContextUpdater`].
    #[must_use]
    pub fn build(self) -> ExecuteSecRequestContextUpdater {
        ExecuteSecRequestContextUpdater {
            cik: self.cik,
            execution_id: self.execution_id,
            user_agent: self.user_agent,
            max_retries: self.max_retries,
        }
    }
}

impl Default for ExecuteSecRequestContextUpdaterBuilder {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use std::{fmt::Debug, hash::Hash};

    use pretty_assertions::assert_eq;
    use state_maschine::prelude::Context as SMContext;

    use super::*;
    use crate::shared::cik::Cik;
    use crate::tests::fixtures::sample_run::{SAMPLE_EXECUTION_ID, sample_user_agent};

    #[test]
    fn should_set_all_fields_when_created_with_new() {
        let cik = Cik::new("1234567890").expect("Hardcoded CIK should always be valid");

        let expected_result = ExecuteSecRequestContext {
            cik: cik.clone(),
            execution_id: SAMPLE_EXECUTION_ID,
            user_agent: sample_user_agent(),
            max_retries: 0,
        };

        let result = ExecuteSecRequestContext::new(cik, SAMPLE_EXECUTION_ID, sample_user_agent());

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_return_cik_reference_when_accessing_cik() {
        let cik = Cik::new("1234567890").expect("Hardcoded CIK should always be valid");
        let context =
            ExecuteSecRequestContext::new(cik.clone(), SAMPLE_EXECUTION_ID, sample_user_agent());

        let expected_result = &cik;
        let result = context.cik();

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_return_max_retries_when_accessing_max_retries() {
        let cik = Cik::new("1234567890").expect("Hardcoded CIK should always be valid");
        let context = ExecuteSecRequestContext::new(cik, SAMPLE_EXECUTION_ID, sample_user_agent());

        let expected_result = 0;
        let result = context.max_retries();

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_update_cik_when_updater_contains_cik() {
        let original_cik = Cik::new("1234567890").expect("Hardcoded CIK should always be valid");
        let new_cik = Cik::new("0987654321").expect("Hardcoded CIK should always be valid");
        let mut context =
            ExecuteSecRequestContext::new(original_cik, SAMPLE_EXECUTION_ID, sample_user_agent());

        let updater = ExecuteSecRequestContextUpdater::builder()
            .cik(new_cik.clone())
            .build();

        let expected_result = &new_cik;

        context.update_context(updater);
        let result = context.cik();

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_not_update_fields_when_updater_is_empty() {
        let cik = Cik::new("1234567890").expect("Hardcoded CIK should always be valid");
        let mut context =
            ExecuteSecRequestContext::new(cik.clone(), SAMPLE_EXECUTION_ID, sample_user_agent());
        let updater = ExecuteSecRequestContextUpdater::builder().build();

        let expected_result =
            ExecuteSecRequestContext::new(cik, SAMPLE_EXECUTION_ID, sample_user_agent());

        context.update_context(updater);
        let result = context;

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_update_max_retries_when_updater_contains_max_retries() {
        let cik = Cik::new("1234567890").expect("Hardcoded CIK should always be valid");
        let mut context =
            ExecuteSecRequestContext::new(cik, SAMPLE_EXECUTION_ID, sample_user_agent());

        let updater = ExecuteSecRequestContextUpdater::builder()
            .max_retries(5)
            .build();

        let expected_result = 5;

        context.update_context(updater);
        let result = context.max_retries;

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_return_context_reference_when_accessing_context() {
        let cik = Cik::new("1234567890").expect("Hardcoded CIK should always be valid");
        let context = ExecuteSecRequestContext::new(cik, SAMPLE_EXECUTION_ID, sample_user_agent());

        let expected_result = &context;
        let result = context.context();

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_display_context_information_when_formatted() {
        let cik = Cik::new("1234567890").expect("Hardcoded CIK should always be valid");
        let context = ExecuteSecRequestContext::new(cik, SAMPLE_EXECUTION_ID, sample_user_agent());

        let expected_result = "Context Data: 1234567890";
        let result = format!("{context}");

        assert_eq!(result, expected_result);
    }

    // Trait implementation tests
    const fn implements_auto_traits<T: Sized + Send + Sync + Unpin>() {}
    #[test]
    const fn should_implement_auto_traits_for_execute_sec_request_context() {
        implements_auto_traits::<ExecuteSecRequestContext>();
    }

    const fn implements_send<T: Send>() {}
    const fn implements_sync<T: Sync>() {}

    #[test]
    const fn should_implement_send_for_execute_sec_request_context() {
        implements_send::<ExecuteSecRequestContext>();
    }

    #[test]
    const fn should_implement_sync_for_execute_sec_request_context() {
        implements_sync::<ExecuteSecRequestContext>();
    }

    #[test]
    const fn should_be_thread_safe_for_execute_sec_request_context() {
        implements_send::<ExecuteSecRequestContext>();
        implements_sync::<ExecuteSecRequestContext>();
    }

    const fn implements_sized<T: Sized>() {}
    #[test]
    const fn should_be_sized_for_execute_sec_request_context() {
        implements_sized::<ExecuteSecRequestContext>();
    }

    const fn implements_hash<T: Hash>() {}
    #[test]
    const fn should_implement_hash_for_execute_sec_request_context() {
        implements_hash::<ExecuteSecRequestContext>();
    }

    const fn implements_partial_eq<T: PartialEq>() {}
    #[test]
    const fn should_implement_partial_eq_for_execute_sec_request_context() {
        implements_partial_eq::<ExecuteSecRequestContext>();
    }

    const fn implements_eq<T: Eq>() {}
    #[test]
    const fn should_implement_eq_for_execute_sec_request_context() {
        implements_eq::<ExecuteSecRequestContext>();
    }

    const fn implements_partial_ord<T: PartialOrd>() {}
    #[test]
    const fn should_implement_partial_ord_for_execute_sec_request_context() {
        implements_partial_ord::<ExecuteSecRequestContext>();
    }

    const fn implements_ord<T: Ord>() {}
    #[test]
    const fn should_implement_ord_for_execute_sec_request_context() {
        implements_ord::<ExecuteSecRequestContext>();
    }

    const fn implements_debug<T: Debug>() {}
    #[test]
    const fn should_implement_debug_for_execute_sec_request_context() {
        implements_debug::<ExecuteSecRequestContext>();
    }

    const fn implements_clone<T: Clone>() {}
    #[test]
    const fn should_implement_clone_for_execute_sec_request_context() {
        implements_clone::<ExecuteSecRequestContext>();
    }

    const fn implements_unpin<T: Unpin>() {}
    #[test]
    const fn should_implement_unpin_for_execute_sec_request_context() {
        implements_unpin::<ExecuteSecRequestContext>();
    }

    #[test]
    fn should_return_execution_id_when_accessing_execution_id_of_execute_sec_request_context() {
        let context = ExecuteSecRequestContext::new(
            Cik::new("1234567890").expect("Hardcoded CIK should always be valid"),
            SAMPLE_EXECUTION_ID,
            sample_user_agent(),
        );

        let expected_result = SAMPLE_EXECUTION_ID;

        let result = context.execution_id();

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_return_user_agent_when_accessing_user_agent_of_execute_sec_request_context() {
        let context = ExecuteSecRequestContext::new(
            Cik::new("1234567890").expect("Hardcoded CIK should always be valid"),
            SAMPLE_EXECUTION_ID,
            sample_user_agent(),
        );

        let expected_result = &sample_user_agent();

        let result = context.user_agent();

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_update_execution_id_when_updater_contains_execution_id_for_execute_sec_request_context()
     {
        let mut context = ExecuteSecRequestContext::new(
            Cik::new("1234567890").expect("Hardcoded CIK should always be valid"),
            SAMPLE_EXECUTION_ID,
            sample_user_agent(),
        );
        let update = ExecuteSecRequestContextUpdater::builder()
            .execution_id(Uuid::from_u128(42))
            .build();

        let expected_result = Uuid::from_u128(42);

        context.update_context(update);
        let result = context.execution_id();

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_update_user_agent_when_updater_contains_user_agent_for_execute_sec_request_context() {
        let mut context = ExecuteSecRequestContext::new(
            Cik::new("1234567890").expect("Hardcoded CIK should always be valid"),
            SAMPLE_EXECUTION_ID,
            sample_user_agent(),
        );
        let other_user_agent = UserAgent::new("Other Company other@example.com")
            .expect("Given a hardcoded SEC-format user agent, validation should always succeed");
        let update = ExecuteSecRequestContextUpdater::builder()
            .user_agent(other_user_agent.clone())
            .build();

        let expected_result = &other_user_agent;

        context.update_context(update);
        let result = context.user_agent();

        assert_eq!(result, expected_result);
    }
}
