//! # Prepare SEC Request Context
//!
//! Provides the [`PrepareSecRequestContext`] carried alongside the
//! [`PrepareSecRequest`](crate::implementations::states::extract::prepare_sec_request::PrepareSecRequest)
//! state, together with its updater and builder.
//!
//! The context holds the validated [`Cik`], the storage repository, and the retry budget.
//!
//! ## Usage
//!
//! ```rust
//! use sec::implementations::states::extract::prepare_sec_request::context::*;
//! use sec::shared::cik::Cik;
//! use state_maschine::prelude::*;
//!
//! let cik = Cik::new("1234567890").expect("A hardcoded valid CIK should always parse");
//! # use sec::shared::raw_document::RawDocument;
//! # use sec::shared::raw_source::RawCompanyFacts;
//! # use storage::implementations::repository::filesystem::FilesystemRepository;
//! # let repository = FilesystemRepository::builder()
//! #     .writer()
//! #     .document_type::<RawDocument<RawCompanyFacts>>()
//! #     .root("data")
//! #     .build();
//! let mut context = PrepareSecRequestContext::new(cik, repository);
//! let update = PrepareSecRequestContextUpdater::builder()
//!     .max_retries(5)
//!     .build();
//! context.update_context(update);
//!
//! let expected_result = 5;
//!
//! let result = context.max_retries;
//!
//! assert_eq!(result, expected_result);
//! ```
//!
//! ## See Also
//!
//! - [`crate::traits::state_machine::state::Context`]: Trait that defines context updates and the retry budget.
//! - [`crate::implementations::states::extract::prepare_sec_request`]: Parent module for the preparation state and its data types.

use std::fmt;

use serde::Serialize;
use state_maschine::prelude::Context as SMContext;

use crate::implementations::states::extract::store_company_facts::RawCompanyFactsRepository;
use crate::shared::cik::Cik;
use crate::traits::state_machine::state::Context;

#[derive(Debug, Clone, PartialEq, PartialOrd, Hash, Eq, Ord, Serialize)]
/// Context for the [`PrepareSecRequest`](super::PrepareSecRequest) state.
///
/// Holds the validated [`Cik`], the storage repository, and the retry budget.
pub struct PrepareSecRequestContext<R> {
    /// The validated CIK the prepared request targets.
    pub cik: Cik,
    /// The storage repository.
    pub repository: R,
    /// Maximum number of times the state may be retried on failure.
    pub max_retries: u32,
}

impl<R> PrepareSecRequestContext<R> {
    /// Creates a new [`PrepareSecRequestContext`] from a validated CIK and the storage repository,
    /// with the retry budget at zero.
    #[must_use]
    pub const fn new(cik: Cik, repository: R) -> Self {
        Self {
            cik,
            repository,
            max_retries: 0,
        }
    }

    /// Returns a reference to the storage repository.
    #[must_use]
    pub const fn repository(&self) -> &R {
        &self.repository
    }
}

impl<R: RawCompanyFactsRepository> Context for PrepareSecRequestContext<R> {
    /// Returns the maximum number of retries allowed for the SEC request.
    fn max_retries(&self) -> u32 {
        self.max_retries
    }
}

impl<R: RawCompanyFactsRepository> SMContext for PrepareSecRequestContext<R> {
    type UpdateType = PrepareSecRequestContextUpdater<R>;

    /// Returns a reference to the current context.
    fn context(&self) -> &Self {
        self
    }

    /// Updates the context fields using the provided updater.
    ///
    /// Only fields set in the updater will be changed.
    fn update_context(&mut self, updates: Self::UpdateType) {
        if let Some(repository) = updates.repository {
            self.repository = repository;
        }
        if let Some(max_retries) = updates.max_retries {
            self.max_retries = max_retries;
        }

        if let Some(cik) = updates.cik {
            self.cik = cik;
        }
    }
}

impl<R> fmt::Display for PrepareSecRequestContext<R> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "Context Data:\nCIK (validated): {}\nMax retries: {}",
            self.cik, self.max_retries
        )
    }
}

#[derive(Debug, Clone, PartialEq, PartialOrd, Hash, Eq, Ord)]
/// Updater for modifying [`PrepareSecRequestContext`].
///
/// Fields set to `None` are left unchanged when the updater is applied.
pub struct PrepareSecRequestContextUpdater<R> {
    /// Optional new value for the validated CIK.
    pub cik: Option<Cik>,
    /// Optional new value for the storage repository.
    pub repository: Option<R>,
    /// Optional new value for the retry budget.
    pub max_retries: Option<u32>,
}

impl<R> PrepareSecRequestContextUpdater<R> {
    /// Creates a new builder for constructing [`PrepareSecRequestContextUpdater`] instances.
    #[must_use]
    pub const fn builder() -> PrepareSecRequestContextUpdaterBuilder<R> {
        PrepareSecRequestContextUpdaterBuilder::new()
    }
}

/// Fluent builder for a [`PrepareSecRequestContextUpdater`].
pub struct PrepareSecRequestContextUpdaterBuilder<R> {
    cik: Option<Cik>,
    repository: Option<R>,
    max_retries: Option<u32>,
}
impl<R> PrepareSecRequestContextUpdaterBuilder<R> {
    /// Creates a new [`PrepareSecRequestContextUpdaterBuilder`] with all fields initialized to `None`.
    #[must_use]
    pub const fn new() -> Self {
        Self {
            cik: None,
            repository: None,
            max_retries: None,
        }
    }

    /// Sets the `max_retries` field.
    #[must_use]
    #[allow(clippy::missing_const_for_fn)]
    pub fn max_retries(mut self, max_retries: u32) -> Self {
        self.max_retries = Some(max_retries);
        self
    }

    /// Sets the CIK field.
    #[must_use]
    pub fn cik(mut self, cik: Cik) -> Self {
        self.cik = Some(cik);
        self
    }

    /// Sets the storage repository field.
    #[must_use]
    pub fn repository(mut self, repository: R) -> Self {
        self.repository = Some(repository);
        self
    }

    /// Builds the [`PrepareSecRequestContextUpdater`] with the specified fields.
    #[must_use]
    pub fn build(self) -> PrepareSecRequestContextUpdater<R> {
        PrepareSecRequestContextUpdater {
            cik: self.cik,
            repository: self.repository,
            max_retries: self.max_retries,
        }
    }
}

impl<R> Default for PrepareSecRequestContextUpdaterBuilder<R> {
    /// Returns a new [`PrepareSecRequestContextUpdaterBuilder`] with all fields initialized to `None`.
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use std::{fmt::Debug, hash::Hash};

    use pretty_assertions::{assert_eq, assert_ne};
    use state_maschine::prelude::*;
    use storage::WriteError;

    use super::{
        PrepareSecRequestContext, PrepareSecRequestContextUpdater,
        PrepareSecRequestContextUpdaterBuilder,
    };
    use crate::shared::cik::Cik;
    use crate::shared::cik::constants::BERKSHIRE_HATHAWAY_CIK_RAW;
    use crate::tests::fixtures::sample_write_repository::FakeWriteRepository;

    fn create_test_context() -> PrepareSecRequestContext<FakeWriteRepository> {
        let cik =
            Cik::new(BERKSHIRE_HATHAWAY_CIK_RAW).expect("Hardcoded CIK should always be valid");
        PrepareSecRequestContext::new(cik, FakeWriteRepository::new())
    }

    #[test]
    fn should_update_context_max_retries_to_specified_value_when_update_contains_specified_value() {
        let mut context = create_test_context();
        let update = PrepareSecRequestContextUpdater::builder()
            .max_retries(5)
            .build();
        let cik =
            Cik::new(BERKSHIRE_HATHAWAY_CIK_RAW).expect("Hardcoded CIK should always be valid");

        let expected_result = &PrepareSecRequestContext {
            cik,
            repository: FakeWriteRepository::new(),
            max_retries: 5,
        };

        context.update_context(update);
        let result = context.context();

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_update_max_retries_to_latest_specified_value_when_multiple_updates_in_builder() {
        let mut context = create_test_context();
        let update = PrepareSecRequestContextUpdater::builder()
            .max_retries(5)
            .max_retries(10)
            .build();
        let cik =
            Cik::new(BERKSHIRE_HATHAWAY_CIK_RAW).expect("Hardcoded CIK should always be valid");

        let expected_result = &PrepareSecRequestContext {
            cik,
            repository: FakeWriteRepository::new(),
            max_retries: 10,
        };

        context.update_context(update);
        let result = context.context();

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_not_leave_context_max_retries_the_default_when_update_contains_a_different_value() {
        let mut context = create_test_context();
        let update = PrepareSecRequestContextUpdater::builder()
            .max_retries(5)
            .build();

        let expected_result = 0;

        context.update_context(update);
        let result = context.context().max_retries;

        assert_ne!(result, expected_result);
    }

    #[test]
    fn should_leave_context_unchanged_when_empty_update() {
        let mut context = create_test_context();
        let empty_update = PrepareSecRequestContextUpdaterBuilder::default().build();

        let expected_result = &create_test_context();

        context.update_context(empty_update);
        let result = context.context();

        assert_eq!(result, expected_result);
    }

    const fn implements_auto_traits<T: Sized + Send + Sync + Unpin>() {}
    #[test]
    const fn should_implement_auto_traits_for_prepare_sec_request_context() {
        implements_auto_traits::<PrepareSecRequestContext<FakeWriteRepository>>();
    }

    const fn implements_send<T: Send>() {}
    const fn implements_sync<T: Sync>() {}

    #[test]
    const fn should_implement_send_for_prepare_sec_request_context() {
        implements_send::<PrepareSecRequestContext<FakeWriteRepository>>();
    }

    #[test]
    const fn should_implement_sync_for_prepare_sec_request_context() {
        implements_sync::<PrepareSecRequestContext<FakeWriteRepository>>();
    }

    #[test]
    const fn should_be_thread_safe_for_prepare_sec_request_context() {
        implements_send::<PrepareSecRequestContext<FakeWriteRepository>>();
        implements_sync::<PrepareSecRequestContext<FakeWriteRepository>>();
    }

    const fn implements_sized<T: Sized>() {}
    #[test]
    const fn should_be_sized_for_prepare_sec_request_context() {
        implements_sized::<PrepareSecRequestContext<FakeWriteRepository>>();
    }

    const fn implements_hash<T: Hash>() {}
    #[test]
    const fn should_implement_hash_for_prepare_sec_request_context() {
        implements_hash::<PrepareSecRequestContext<FakeWriteRepository>>();
    }

    const fn implements_partial_eq<T: PartialEq>() {}
    #[test]
    const fn should_implement_partial_eq_for_prepare_sec_request_context() {
        implements_partial_eq::<PrepareSecRequestContext<FakeWriteRepository>>();
    }

    const fn implements_eq<T: Eq>() {}
    #[test]
    const fn should_implement_eq_for_prepare_sec_request_context() {
        implements_eq::<PrepareSecRequestContext<FakeWriteRepository>>();
    }

    const fn implements_partial_ord<T: PartialOrd>() {}
    #[test]
    const fn should_implement_partial_ord_for_prepare_sec_request_context() {
        implements_partial_ord::<PrepareSecRequestContext<FakeWriteRepository>>();
    }

    const fn implements_ord<T: Ord>() {}
    #[test]
    const fn should_implement_ord_for_prepare_sec_request_context() {
        implements_ord::<PrepareSecRequestContext<FakeWriteRepository>>();
    }

    const fn implements_debug<T: Debug>() {}
    #[test]
    const fn should_implement_debug_for_prepare_sec_request_context() {
        implements_debug::<PrepareSecRequestContext<FakeWriteRepository>>();
    }

    const fn implements_clone<T: Clone>() {}
    #[test]
    const fn should_implement_clone_for_prepare_sec_request_context() {
        implements_clone::<PrepareSecRequestContext<FakeWriteRepository>>();
    }

    const fn implements_unpin<T: Unpin>() {}
    #[test]
    const fn should_implement_unpin_for_prepare_sec_request_context() {
        implements_unpin::<PrepareSecRequestContext<FakeWriteRepository>>();
    }

    #[test]
    fn should_return_repository_when_accessing_repository_of_prepare_sec_request_context() {
        let context = PrepareSecRequestContext::new(
            Cik::new(BERKSHIRE_HATHAWAY_CIK_RAW).expect("Hardcoded CIK should always be valid"),
            FakeWriteRepository::rejecting(WriteError::conflicting_write("Sample conflict")),
        );

        let expected_result =
            &FakeWriteRepository::rejecting(WriteError::conflicting_write("Sample conflict"));

        let result = context.repository();

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_update_repository_when_updater_contains_repository_for_prepare_sec_request_context() {
        let mut context = create_test_context();
        let update = PrepareSecRequestContextUpdater::builder()
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
        let context = create_test_context();

        let expected_result = serde_json::json!({
            "cik": { "value": "0001067983" },
            "repository": { "kind": "fake", "rejection": null },
            "max_retries": 0,
        });

        let result = serde_json::to_value(context)
            .expect("Given a context of domain types, serialization should always succeed");

        assert_eq!(result, expected_result);
    }
}
