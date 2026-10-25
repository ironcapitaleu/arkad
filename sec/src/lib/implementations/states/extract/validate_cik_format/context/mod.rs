//! # Validate CIK Format Context
//!
//! Provides the [`ValidateCikFormatContext`] carried alongside the
//! [`ValidateCikFormat`](crate::implementations::states::extract::validate_cik_format::ValidateCikFormat)
//! state, together with its updater and builder.
//!
//! The context holds the raw CIK under validation, the shared [`SecClient`], the storage repository
//! and the retry budget. These outlive any single input/output pair. The [`Context`] trait applies
//! updates from an updater that a builder constructs, so each update is explicit and partial.
//!
//! ## Usage
//!
//! ```rust
//! use sec::implementations::states::extract::validate_cik_format::context::*;
//! use sec::shared::http_client::implementations::sec_client::SecClient;
//! use state_maschine::prelude::*;
//!
//! # use sec::shared::raw_document::RawDocument;
//! # use sec::shared::raw_source::RawCompanyFacts;
//! # use storage::implementations::repository::filesystem::FilesystemRepository;
//! # let repository = FilesystemRepository::builder()
//! #     .writer()
//! #     .document_type::<RawDocument<RawCompanyFacts>>()
//! #     .root("data")
//! #     .build();
//! let mut context = ValidateCikFormatContext::new("1067983", SecClient::default(), repository);
//! let update = ValidateCikFormatContextUpdater::builder()
//!     .cik("0000000001")
//!     .build();
//! context.update_context(update);
//!
//! let expected_result = "0000000001";
//!
//! let result = context.cik().as_str();
//!
//! assert_eq!(result, expected_result);
//! ```
//!
//! ## See Also
//!
//! - [`crate::traits::state_machine::state::Context`]: Trait that defines context updates and the retry budget.
//! - [`crate::implementations::states::extract::validate_cik_format`]: Parent module for the validation state and its data types.

use std::fmt;

use serde::Serialize;
use state_maschine::prelude::Context as SMContext;

use crate::implementations::states::extract::store_company_facts::RawCompanyFactsRepository;
use crate::shared::http_client::implementations::sec_client::SecClient;
use crate::traits::state_machine::state::Context;

#[derive(Debug, Clone, PartialEq, PartialOrd, Hash, Eq, Ord, Serialize)]
/// Context for the [`ValidateCikFormat`](super::ValidateCikFormat) state.
///
/// Holds the raw CIK under validation, the shared [`SecClient`], and the storage repository of
/// the run. It also holds the retry budget.
pub struct ValidateCikFormatContext<R> {
    /// The unvalidated CIK string provided for validation.
    pub raw_cik: String,
    /// The shared HTTP client for SEC API requests.
    pub sec_client: SecClient,
    /// The storage repository.
    pub repository: R,
    /// Maximum number of times the state may be retried on failure.
    pub max_retries: u32,
}

impl<R> ValidateCikFormatContext<R> {
    /// Creates a new [`ValidateCikFormatContext`] from a raw CIK, a shared HTTP client, and the
    /// storage repository.
    pub fn new(cik: impl Into<String>, sec_client: SecClient, repository: R) -> Self {
        Self {
            raw_cik: cik.into(),
            sec_client,
            repository,
            max_retries: 0,
        }
    }

    /// Returns a reference to the current raw CIK string in the context.
    #[must_use]
    pub const fn cik(&self) -> &String {
        &self.raw_cik
    }

    /// Returns a reference to the HTTP client.
    #[must_use]
    pub const fn sec_client(&self) -> &SecClient {
        &self.sec_client
    }

    /// Returns a reference to the storage repository.
    #[must_use]
    pub const fn repository(&self) -> &R {
        &self.repository
    }
}

impl<R: RawCompanyFactsRepository> Context for ValidateCikFormatContext<R> {
    /// Returns the maximum number of retries allowed for CIK validation.
    fn max_retries(&self) -> u32 {
        self.max_retries
    }
}

impl<R: RawCompanyFactsRepository> SMContext for ValidateCikFormatContext<R> {
    type UpdateType = ValidateCikFormatContextUpdater<R>;

    /// Returns a reference to the current context.
    fn context(&self) -> &Self {
        self
    }

    /// Updates the context fields using the provided updater.
    ///
    /// Only fields set in the updater will be changed.
    fn update_context(&mut self, updates: Self::UpdateType) {
        if let Some(cik) = updates.raw_cik {
            self.raw_cik = cik;
        }
        if let Some(sec_client) = updates.sec_client {
            self.sec_client = sec_client;
        }
        if let Some(repository) = updates.repository {
            self.repository = repository;
        }
        if let Some(max_retries) = updates.max_retries {
            self.max_retries = max_retries;
        }
    }
}

impl<R> fmt::Display for ValidateCikFormatContext<R> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Unvalidated CIK: {}", self.raw_cik)
    }
}

#[derive(Debug, Clone, PartialEq, PartialOrd, Hash, Eq, Ord)]
/// Updater for modifying [`ValidateCikFormatContext`].
///
/// Fields set to `None` are left unchanged when the updater is applied.
pub struct ValidateCikFormatContextUpdater<R> {
    /// Optional new raw CIK string value.
    pub raw_cik: Option<String>,
    /// Optional new HTTP client value.
    pub sec_client: Option<SecClient>,
    /// Optional new value for the storage repository.
    pub repository: Option<R>,
    /// Optional new maximum retries value.
    pub max_retries: Option<u32>,
}

impl<R> ValidateCikFormatContextUpdater<R> {
    /// Creates a new builder for constructing [`ValidateCikFormatContextUpdater`] instances.
    #[must_use]
    pub const fn builder() -> ValidateCikFormatContextUpdaterBuilder<R> {
        ValidateCikFormatContextUpdaterBuilder::new()
    }
}

/// Fluent builder for a [`ValidateCikFormatContextUpdater`].
pub struct ValidateCikFormatContextUpdaterBuilder<R> {
    raw_cik: Option<String>,
    sec_client: Option<SecClient>,
    repository: Option<R>,
    max_retries: Option<u32>,
}
impl<R> ValidateCikFormatContextUpdaterBuilder<R> {
    /// Creates a new [`ValidateCikFormatContextUpdaterBuilder`] with all fields initialized to `None`.
    #[must_use]
    pub const fn new() -> Self {
        Self {
            raw_cik: None,
            sec_client: None,
            repository: None,
            max_retries: None,
        }
    }

    /// Sets the raw CIK field.
    #[must_use]
    #[allow(clippy::missing_const_for_fn)]
    pub fn cik(mut self, cik: impl Into<String>) -> Self {
        self.raw_cik = Some(cik.into());
        self
    }

    /// Sets the SEC client field.
    #[must_use]
    pub fn sec_client(mut self, sec_client: SecClient) -> Self {
        self.sec_client = Some(sec_client);
        self
    }

    /// Sets the `max_retries` field.
    #[must_use]
    pub const fn max_retries(mut self, max_retries: u32) -> Self {
        self.max_retries = Some(max_retries);
        self
    }

    /// Sets the storage repository field.
    #[must_use]
    pub fn repository(mut self, repository: R) -> Self {
        self.repository = Some(repository);
        self
    }

    /// Builds the [`ValidateCikFormatContextUpdater`].
    #[must_use]
    pub fn build(self) -> ValidateCikFormatContextUpdater<R> {
        ValidateCikFormatContextUpdater {
            raw_cik: self.raw_cik,
            sec_client: self.sec_client,
            repository: self.repository,
            max_retries: self.max_retries,
        }
    }
}

impl<R> Default for ValidateCikFormatContextUpdaterBuilder<R> {
    /// Returns a new [`ValidateCikFormatContextUpdaterBuilder`] with all fields initialized to `None`.
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
        ValidateCikFormatContext, ValidateCikFormatContextUpdater,
        ValidateCikFormatContextUpdaterBuilder,
    };
    use crate::shared::cik::constants::BERKSHIRE_HATHAWAY_CIK_RAW;
    use crate::shared::http_client::implementations::sec_client::SecClient;
    use crate::shared::user_agent::constants::DEFAULT_SEC_USER_AGENT;
    use crate::tests::fixtures::sample_write_repository::FakeWriteRepository;

    fn test_context() -> ValidateCikFormatContext<FakeWriteRepository> {
        let sec_client = SecClient::default();
        ValidateCikFormatContext::new(
            BERKSHIRE_HATHAWAY_CIK_RAW,
            sec_client,
            FakeWriteRepository::new(),
        )
    }

    #[test]
    fn should_return_reference_to_default_validation_context_when_initialized_with_default() {
        let validation_context = test_context();

        let expected_result = &test_context();

        let result = validation_context.context();

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_create_different_context_with_custom_data_when_using_new_as_constructor() {
        let sec_client = SecClient::default();
        let validation_context =
            &ValidateCikFormatContext::new("0000000000", sec_client, FakeWriteRepository::new());

        let expected_result = &test_context();

        let result = validation_context.context();

        assert_ne!(result, expected_result);
    }

    #[test]
    fn should_update_context_cik_data_to_specified_string_when_update_contains_specified_string() {
        let mut context = test_context();
        let update = ValidateCikFormatContextUpdater::builder()
            .cik("Updated CIK!")
            .build();

        let sec_client = SecClient::default();
        let expected_result =
            &ValidateCikFormatContext::new("Updated CIK!", sec_client, FakeWriteRepository::new());

        context.update_context(update);
        let result = context.context();

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_update_cik_to_latest_specified_string_when_multiple_updates_in_builder() {
        let mut context = test_context();
        let update = ValidateCikFormatContextUpdater::builder()
            .cik("First CIK Update!")
            .cik("Latest CIK Update!")
            .build();

        let sec_client = SecClient::default();
        let expected_result = &ValidateCikFormatContext::new(
            "Latest CIK Update!",
            sec_client,
            FakeWriteRepository::new(),
        );

        context.update_context(update);
        let result = context.context();

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_not_leave_context_cik_data_the_default_when_update_contains_a_different_string() {
        let mut context = test_context();
        let update = ValidateCikFormatContextUpdater::builder()
            .cik("Updated CIK!")
            .build();

        let expected_result = BERKSHIRE_HATHAWAY_CIK_RAW;

        context.update_context(update);
        let result = context.context().cik();

        assert_ne!(result, expected_result);
    }

    #[test]
    fn should_leave_context_unchanged_when_empty_update() {
        let mut context = test_context();
        let empty_update = ValidateCikFormatContextUpdaterBuilder::default().build();

        let expected_result = &test_context();

        context.update_context(empty_update);
        let result = context.context();

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_update_max_retries_when_updater_contains_max_retries() {
        let mut context = test_context();
        let update = ValidateCikFormatContextUpdater::builder()
            .max_retries(5)
            .build();

        let expected_result = 5;

        context.update_context(update);
        let result = context.max_retries;

        assert_eq!(result, expected_result);
    }

    const fn implements_auto_traits<T: Sized + Send + Sync + Unpin>() {}
    #[test]
    const fn should_implement_auto_traits_for_validate_cik_format_context() {
        implements_auto_traits::<ValidateCikFormatContext<FakeWriteRepository>>();
    }

    const fn implements_send<T: Send>() {}
    const fn implements_sync<T: Sync>() {}

    #[test]
    const fn should_implement_send_for_validate_cik_format_context() {
        implements_send::<ValidateCikFormatContext<FakeWriteRepository>>();
    }

    #[test]
    const fn should_implement_sync_for_validate_cik_format_context() {
        implements_sync::<ValidateCikFormatContext<FakeWriteRepository>>();
    }

    #[test]
    const fn should_be_thread_safe_for_validate_cik_format_context() {
        implements_send::<ValidateCikFormatContext<FakeWriteRepository>>();
        implements_sync::<ValidateCikFormatContext<FakeWriteRepository>>();
    }

    const fn implements_sized<T: Sized>() {}
    #[test]
    const fn should_be_sized_for_validate_cik_format_context() {
        implements_sized::<ValidateCikFormatContext<FakeWriteRepository>>();
    }

    const fn implements_hash<T: Hash>() {}
    #[test]
    const fn should_implement_hash_for_validate_cik_format_context() {
        implements_hash::<ValidateCikFormatContext<FakeWriteRepository>>();
    }

    const fn implements_partial_eq<T: PartialEq>() {}
    #[test]
    const fn should_implement_partial_eq_for_validate_cik_format_context() {
        implements_partial_eq::<ValidateCikFormatContext<FakeWriteRepository>>();
    }

    const fn implements_eq<T: Eq>() {}
    #[test]
    const fn should_implement_eq_for_validate_cik_format_context() {
        implements_eq::<ValidateCikFormatContext<FakeWriteRepository>>();
    }

    const fn implements_partial_ord<T: PartialOrd>() {}
    #[test]
    const fn should_implement_partial_ord_for_validate_cik_format_context() {
        implements_partial_ord::<ValidateCikFormatContext<FakeWriteRepository>>();
    }

    const fn implements_ord<T: Ord>() {}
    #[test]
    const fn should_implement_ord_for_validate_cik_format_context() {
        implements_ord::<ValidateCikFormatContext<FakeWriteRepository>>();
    }

    const fn implements_debug<T: Debug>() {}
    #[test]
    const fn should_implement_debug_for_validate_cik_format_context() {
        implements_debug::<ValidateCikFormatContext<FakeWriteRepository>>();
    }

    const fn implements_clone<T: Clone>() {}
    #[test]
    const fn should_implement_clone_for_validate_cik_format_context() {
        implements_clone::<ValidateCikFormatContext<FakeWriteRepository>>();
    }

    const fn implements_unpin<T: Unpin>() {}
    #[test]
    const fn should_implement_unpin_for_validate_cik_format_context() {
        implements_unpin::<ValidateCikFormatContext<FakeWriteRepository>>();
    }

    #[test]
    fn should_return_repository_when_accessing_repository_of_validate_cik_format_context() {
        let context = ValidateCikFormatContext::new(
            BERKSHIRE_HATHAWAY_CIK_RAW,
            SecClient::default(),
            FakeWriteRepository::rejecting(WriteError::conflicting_write("Sample conflict")),
        );

        let expected_result =
            &FakeWriteRepository::rejecting(WriteError::conflicting_write("Sample conflict"));

        let result = context.repository();

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_update_repository_when_updater_contains_repository_for_validate_cik_format_context() {
        let mut context = test_context();
        let update = ValidateCikFormatContextUpdater::builder()
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
        let context = test_context();

        let expected_result = serde_json::json!({
            "raw_cik": BERKSHIRE_HATHAWAY_CIK_RAW,
            "sec_client": { "user_agent": DEFAULT_SEC_USER_AGENT },
            "repository": { "kind": "fake", "rejection": null },
            "max_retries": 0,
        });

        let result = serde_json::to_value(context)
            .expect("Given a context of domain types, serialization should always succeed");

        assert_eq!(result, expected_result);
    }
}
