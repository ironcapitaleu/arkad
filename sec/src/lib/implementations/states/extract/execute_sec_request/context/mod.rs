//! # Execute SEC Request Context
//!
//! Provides the [`ExecuteSecRequestContext`] carried alongside the
//! [`ExecuteSecRequest`](crate::implementations::states::extract::execute_sec_request::ExecuteSecRequest)
//! state, together with its updater and builder.
//!
//! The context tracks the [`Cik`] being processed, the `User-Agent` and the storage repository of
//! the run, and the retry budget. All of them persist across updates and transitions.
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
//! let user_agent = UserAgent::new("Sample Company contact@example.com")
//!     .expect("Given a hardcoded SEC-format user agent, validation should always succeed");
//! # use sec::shared::raw_document::RawDocument;
//! # use sec::shared::raw_source::RawCompanyFacts;
//! # use storage::implementations::repository::filesystem::FilesystemRepository;
//! # let repository = FilesystemRepository::builder()
//! #     .writer()
//! #     .document_type::<RawDocument<RawCompanyFacts>>()
//! #     .root("data")
//! #     .build();
//! let mut context = ExecuteSecRequestContext::new(cik, user_agent, repository);
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

use crate::implementations::states::extract::store_company_facts::RawCompanyFactsRepository;
use crate::shared::cik::Cik;
use crate::shared::user_agent::UserAgent;
use crate::traits::state_machine::state::Context;

/// Context for the [`ExecuteSecRequest`](super::ExecuteSecRequest) state.
///
/// Tracks the [`Cik`] being processed, the `User-Agent` and the storage repository of the run,
/// and the retry budget. All of them persist across updates and transitions.
#[derive(Debug, Clone, PartialEq, PartialOrd, Hash, Eq, Ord, Serialize)]
pub struct ExecuteSecRequestContext<R> {
    /// The Central Index Key (CIK) being processed in this execution context.
    pub cik: Cik,
    /// The `User-Agent` of the run. It must equal the header that the SEC client sends.
    pub user_agent: UserAgent,
    /// The storage repository of the run.
    pub repository: R,
    /// Maximum number of times the state may be retried on failure.
    pub max_retries: u32,
}

impl<R> ExecuteSecRequestContext<R> {
    /// Creates a new [`ExecuteSecRequestContext`] from a CIK and the `User-Agent` and the storage
    /// repository of the run, with the retry budget at zero.
    #[must_use]
    pub const fn new(cik: Cik, user_agent: UserAgent, repository: R) -> Self {
        Self {
            cik,
            user_agent,
            repository,
            max_retries: 0,
        }
    }

    /// Returns a reference to the CIK being processed.
    #[must_use]
    pub const fn cik(&self) -> &Cik {
        &self.cik
    }

    /// Returns the `User-Agent` of the run.
    #[must_use]
    pub const fn user_agent(&self) -> &UserAgent {
        &self.user_agent
    }

    /// Returns a reference to the storage repository of the run.
    #[must_use]
    pub const fn repository(&self) -> &R {
        &self.repository
    }
}

impl<R: RawCompanyFactsRepository> Context for ExecuteSecRequestContext<R> {
    fn max_retries(&self) -> u32 {
        self.max_retries
    }
}

impl<R: RawCompanyFactsRepository> SMContext for ExecuteSecRequestContext<R> {
    type UpdateType = ExecuteSecRequestContextUpdater<R>;

    fn context(&self) -> &Self {
        self
    }

    fn update_context(&mut self, updates: Self::UpdateType) {
        if let Some(cik) = updates.cik {
            self.cik = cik;
        }
        if let Some(user_agent) = updates.user_agent {
            self.user_agent = user_agent;
        }
        if let Some(repository) = updates.repository {
            self.repository = repository;
        }
        if let Some(max_retries) = updates.max_retries {
            self.max_retries = max_retries;
        }
    }
}

impl<R> fmt::Display for ExecuteSecRequestContext<R> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Context Data: {}", self.cik)
    }
}

/// Updater for modifying [`ExecuteSecRequestContext`].
///
/// Fields set to `None` are left unchanged when the updater is applied.
#[derive(Debug, Clone, PartialEq, PartialOrd, Hash, Eq, Ord)]
pub struct ExecuteSecRequestContextUpdater<R> {
    /// Optional new value for the CIK.
    pub cik: Option<Cik>,
    /// Optional new value for the `User-Agent`.
    pub user_agent: Option<UserAgent>,
    /// Optional new value for the storage repository.
    pub repository: Option<R>,
    /// Optional new value for the retry budget.
    pub max_retries: Option<u32>,
}

impl<R> ExecuteSecRequestContextUpdater<R> {
    /// Creates a new builder for constructing [`ExecuteSecRequestContextUpdater`] instances.
    #[must_use]
    pub const fn builder() -> ExecuteSecRequestContextUpdaterBuilder<R> {
        ExecuteSecRequestContextUpdaterBuilder::new()
    }
}

/// Fluent builder for an [`ExecuteSecRequestContextUpdater`].
pub struct ExecuteSecRequestContextUpdaterBuilder<R> {
    cik: Option<Cik>,
    user_agent: Option<UserAgent>,
    repository: Option<R>,
    max_retries: Option<u32>,
}

impl<R> ExecuteSecRequestContextUpdaterBuilder<R> {
    /// Creates a new [`ExecuteSecRequestContextUpdaterBuilder`] with all fields initialized to `None`.
    #[must_use]
    pub const fn new() -> Self {
        Self {
            cik: None,
            user_agent: None,
            repository: None,
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

    /// Sets the `User-Agent` field.
    #[must_use]
    pub fn user_agent(mut self, user_agent: UserAgent) -> Self {
        self.user_agent = Some(user_agent);
        self
    }

    /// Sets the storage repository field.
    #[must_use]
    pub fn repository(mut self, repository: R) -> Self {
        self.repository = Some(repository);
        self
    }

    /// Builds the [`ExecuteSecRequestContextUpdater`].
    #[must_use]
    pub fn build(self) -> ExecuteSecRequestContextUpdater<R> {
        ExecuteSecRequestContextUpdater {
            cik: self.cik,
            user_agent: self.user_agent,
            repository: self.repository,
            max_retries: self.max_retries,
        }
    }
}

impl<R> Default for ExecuteSecRequestContextUpdaterBuilder<R> {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use std::{fmt::Debug, hash::Hash};

    use pretty_assertions::assert_eq;
    use state_maschine::prelude::Context as SMContext;
    use storage::WriteError;

    use super::*;
    use crate::shared::cik::Cik;
    use crate::tests::fixtures::sample_run::sample_user_agent;
    use crate::tests::fixtures::sample_write_repository::FakeWriteRepository;

    #[test]
    fn should_set_all_fields_when_created_with_new() {
        let cik = Cik::new("1234567890").expect("Hardcoded CIK should always be valid");

        let expected_result = ExecuteSecRequestContext {
            cik: cik.clone(),
            user_agent: sample_user_agent(),
            repository: FakeWriteRepository::new(),
            max_retries: 0,
        };

        let result =
            ExecuteSecRequestContext::new(cik, sample_user_agent(), FakeWriteRepository::new());

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_return_cik_reference_when_accessing_cik() {
        let cik = Cik::new("1234567890").expect("Hardcoded CIK should always be valid");
        let context = ExecuteSecRequestContext::new(
            cik.clone(),
            sample_user_agent(),
            FakeWriteRepository::new(),
        );

        let expected_result = &cik;
        let result = context.cik();

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_return_max_retries_when_accessing_max_retries() {
        let cik = Cik::new("1234567890").expect("Hardcoded CIK should always be valid");
        let context =
            ExecuteSecRequestContext::new(cik, sample_user_agent(), FakeWriteRepository::new());

        let expected_result = 0;
        let result = context.max_retries();

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_update_cik_when_updater_contains_cik() {
        let original_cik = Cik::new("1234567890").expect("Hardcoded CIK should always be valid");
        let new_cik = Cik::new("0987654321").expect("Hardcoded CIK should always be valid");
        let mut context = ExecuteSecRequestContext::new(
            original_cik,
            sample_user_agent(),
            FakeWriteRepository::new(),
        );

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
        let mut context = ExecuteSecRequestContext::new(
            cik.clone(),
            sample_user_agent(),
            FakeWriteRepository::new(),
        );
        let updater = ExecuteSecRequestContextUpdater::builder().build();

        let expected_result =
            ExecuteSecRequestContext::new(cik, sample_user_agent(), FakeWriteRepository::new());

        context.update_context(updater);
        let result = context;

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_update_max_retries_when_updater_contains_max_retries() {
        let cik = Cik::new("1234567890").expect("Hardcoded CIK should always be valid");
        let mut context =
            ExecuteSecRequestContext::new(cik, sample_user_agent(), FakeWriteRepository::new());

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
        let context =
            ExecuteSecRequestContext::new(cik, sample_user_agent(), FakeWriteRepository::new());

        let expected_result = &context;
        let result = context.context();

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_display_context_information_when_formatted() {
        let cik = Cik::new("1234567890").expect("Hardcoded CIK should always be valid");
        let context =
            ExecuteSecRequestContext::new(cik, sample_user_agent(), FakeWriteRepository::new());

        let expected_result = "Context Data: 1234567890";
        let result = format!("{context}");

        assert_eq!(result, expected_result);
    }

    // Trait implementation tests
    const fn implements_auto_traits<T: Sized + Send + Sync + Unpin>() {}
    #[test]
    const fn should_implement_auto_traits_for_execute_sec_request_context() {
        implements_auto_traits::<ExecuteSecRequestContext<FakeWriteRepository>>();
    }

    const fn implements_send<T: Send>() {}
    const fn implements_sync<T: Sync>() {}

    #[test]
    const fn should_implement_send_for_execute_sec_request_context() {
        implements_send::<ExecuteSecRequestContext<FakeWriteRepository>>();
    }

    #[test]
    const fn should_implement_sync_for_execute_sec_request_context() {
        implements_sync::<ExecuteSecRequestContext<FakeWriteRepository>>();
    }

    #[test]
    const fn should_be_thread_safe_for_execute_sec_request_context() {
        implements_send::<ExecuteSecRequestContext<FakeWriteRepository>>();
        implements_sync::<ExecuteSecRequestContext<FakeWriteRepository>>();
    }

    const fn implements_sized<T: Sized>() {}
    #[test]
    const fn should_be_sized_for_execute_sec_request_context() {
        implements_sized::<ExecuteSecRequestContext<FakeWriteRepository>>();
    }

    const fn implements_hash<T: Hash>() {}
    #[test]
    const fn should_implement_hash_for_execute_sec_request_context() {
        implements_hash::<ExecuteSecRequestContext<FakeWriteRepository>>();
    }

    const fn implements_partial_eq<T: PartialEq>() {}
    #[test]
    const fn should_implement_partial_eq_for_execute_sec_request_context() {
        implements_partial_eq::<ExecuteSecRequestContext<FakeWriteRepository>>();
    }

    const fn implements_eq<T: Eq>() {}
    #[test]
    const fn should_implement_eq_for_execute_sec_request_context() {
        implements_eq::<ExecuteSecRequestContext<FakeWriteRepository>>();
    }

    const fn implements_partial_ord<T: PartialOrd>() {}
    #[test]
    const fn should_implement_partial_ord_for_execute_sec_request_context() {
        implements_partial_ord::<ExecuteSecRequestContext<FakeWriteRepository>>();
    }

    const fn implements_ord<T: Ord>() {}
    #[test]
    const fn should_implement_ord_for_execute_sec_request_context() {
        implements_ord::<ExecuteSecRequestContext<FakeWriteRepository>>();
    }

    const fn implements_debug<T: Debug>() {}
    #[test]
    const fn should_implement_debug_for_execute_sec_request_context() {
        implements_debug::<ExecuteSecRequestContext<FakeWriteRepository>>();
    }

    const fn implements_clone<T: Clone>() {}
    #[test]
    const fn should_implement_clone_for_execute_sec_request_context() {
        implements_clone::<ExecuteSecRequestContext<FakeWriteRepository>>();
    }

    const fn implements_unpin<T: Unpin>() {}
    #[test]
    const fn should_implement_unpin_for_execute_sec_request_context() {
        implements_unpin::<ExecuteSecRequestContext<FakeWriteRepository>>();
    }

    #[test]
    fn should_return_user_agent_when_accessing_user_agent_of_execute_sec_request_context() {
        let context = ExecuteSecRequestContext::new(
            Cik::new("1234567890").expect("Hardcoded CIK should always be valid"),
            sample_user_agent(),
            FakeWriteRepository::new(),
        );

        let expected_result = &sample_user_agent();

        let result = context.user_agent();

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_update_user_agent_when_updater_contains_user_agent_for_execute_sec_request_context() {
        let mut context = ExecuteSecRequestContext::new(
            Cik::new("1234567890").expect("Hardcoded CIK should always be valid"),
            sample_user_agent(),
            FakeWriteRepository::new(),
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

    #[test]
    fn should_return_repository_when_accessing_repository_of_execute_sec_request_context() {
        let context = ExecuteSecRequestContext::new(
            Cik::new("1234567890").expect("Hardcoded CIK should always be valid"),
            sample_user_agent(),
            FakeWriteRepository::rejecting(WriteError::conflicting_write("Sample conflict")),
        );

        let expected_result =
            &FakeWriteRepository::rejecting(WriteError::conflicting_write("Sample conflict"));

        let result = context.repository();

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_update_repository_when_updater_contains_repository_for_execute_sec_request_context() {
        let mut context = ExecuteSecRequestContext::new(
            Cik::new("1234567890").expect("Hardcoded CIK should always be valid"),
            sample_user_agent(),
            FakeWriteRepository::new(),
        );
        let update = ExecuteSecRequestContextUpdater::builder()
            .repository(FakeWriteRepository::rejecting(
                WriteError::conflicting_write("Sample conflict"),
            ))
            .build();

        let expected_result =
            &FakeWriteRepository::rejecting(WriteError::conflicting_write("Sample conflict"));

        context.update_context(update);
        let result = context.repository();

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_include_the_repository_when_serializing_the_context() {
        let context = ExecuteSecRequestContext::new(
            Cik::new("1234567890").expect("Hardcoded CIK should always be valid"),
            sample_user_agent(),
            FakeWriteRepository::new(),
        );

        let expected_result = serde_json::json!({
            "cik": { "value": "1234567890" },
            "user_agent": "Sample Company contact@example.com",
            "repository": { "kind": "fake", "rejection": null },
            "max_retries": 0,
        });

        let result = serde_json::to_value(context)
            .expect("Given a context of domain types, serialization should always succeed");

        assert_eq!(result, expected_result);
    }
}
