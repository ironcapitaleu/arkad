//! # Prepare SEC Request Context
//!
//! Provides the [`PrepareSecRequestContext`] carried alongside the
//! [`PrepareSecRequest`](crate::implementations::states::extract::prepare_sec_request::PrepareSecRequest)
//! state, together with its updater and builder.
//!
//! The context holds three values: the request input (for example a [`Cik`]), the storage
//! repository, and the retry budget.
//!
//! ## Usage
//!
//! ```rust
//! use sec::implementations::states::extract::prepare_sec_request::context::*;
//! use sec::shared::cik::Cik;
//! use sec::shared::request::implementations::sec_request::CompanyFacts;
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
//! let mut context = PrepareSecRequestContext::<CompanyFacts, _>::new(cik, repository);
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

use serde::ser::SerializeStruct;
use serde::{Serialize, Serializer};
use state_maschine::prelude::Context as SMContext;

use crate::implementations::states::extract::state_repository::StateRepository;
#[cfg(doc)]
use crate::shared::cik::Cik;
use crate::shared::request::implementations::sec_request::RequestKind;
use crate::traits::state_machine::state::Context;

/// Context for the [`PrepareSecRequest`](super::PrepareSecRequest) state.
///
/// Holds three values: the request input of kind `K` (for example a [`Cik`]), the storage
/// repository, and the retry budget.
#[derive(Debug, Clone, PartialEq, PartialOrd, Hash, Eq, Ord)]
pub struct PrepareSecRequestContext<K: RequestKind, R> {
    /// The input of the request, for example the [`Cik`] of a Company Facts request.
    pub request_input: K::Input,
    /// The storage repository.
    pub repository: R,
    /// Maximum number of times the state may be retried on failure.
    pub max_retries: u32,
}

impl<K: RequestKind, R> PrepareSecRequestContext<K, R> {
    /// Creates a new [`PrepareSecRequestContext`] from the request input and the storage
    /// repository, with the retry budget at zero.
    #[must_use]
    pub const fn new(request_input: K::Input, repository: R) -> Self {
        Self {
            request_input,
            repository,
            max_retries: 0,
        }
    }

    /// Returns a reference to the request input.
    #[must_use]
    pub const fn request_input(&self) -> &K::Input {
        &self.request_input
    }

    /// Returns a reference to the storage repository.
    #[must_use]
    pub const fn repository(&self) -> &R {
        &self.repository
    }
}

/// Serializes the request input under the name its kind gives it, for example `cik`.
impl<K: RequestKind, R: Serialize> Serialize for PrepareSecRequestContext<K, R> {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let mut state = serializer.serialize_struct("PrepareSecRequestContext", 3)?;
        state.serialize_field(K::INPUT_NAME, &self.request_input)?;
        state.serialize_field("repository", &self.repository)?;
        state.serialize_field("max_retries", &self.max_retries)?;
        state.end()
    }
}

impl<K: RequestKind, R: StateRepository> Context for PrepareSecRequestContext<K, R> {
    /// Returns the maximum number of retries allowed for the SEC request.
    fn max_retries(&self) -> u32 {
        self.max_retries
    }
}

impl<K: RequestKind, R: StateRepository> SMContext for PrepareSecRequestContext<K, R> {
    type UpdateType = PrepareSecRequestContextUpdater<K, R>;

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
        if let Some(request_input) = updates.request_input {
            self.request_input = request_input;
        }
    }
}

impl<K: RequestKind, R> fmt::Display for PrepareSecRequestContext<K, R> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "Context Data:\n{}: {}\nMax retries: {}",
            K::INPUT_NAME,
            self.request_input,
            self.max_retries
        )
    }
}

/// Updater for modifying [`PrepareSecRequestContext`].
///
/// Fields set to `None` are left unchanged when the updater is applied.
#[derive(Debug, Clone, PartialEq, PartialOrd, Hash, Eq, Ord)]
pub struct PrepareSecRequestContextUpdater<K: RequestKind, R> {
    /// Optional new value for the request input.
    pub request_input: Option<K::Input>,
    /// Optional new value for the storage repository.
    pub repository: Option<R>,
    /// Optional new value for the retry budget.
    pub max_retries: Option<u32>,
}

impl<K: RequestKind, R> PrepareSecRequestContextUpdater<K, R> {
    /// Creates a new builder for constructing [`PrepareSecRequestContextUpdater`] instances.
    #[must_use]
    pub const fn builder() -> PrepareSecRequestContextUpdaterBuilder<K, R> {
        PrepareSecRequestContextUpdaterBuilder::new()
    }
}

/// Fluent builder for a [`PrepareSecRequestContextUpdater`].
pub struct PrepareSecRequestContextUpdaterBuilder<K: RequestKind, R> {
    request_input: Option<K::Input>,
    repository: Option<R>,
    max_retries: Option<u32>,
}
impl<K: RequestKind, R> PrepareSecRequestContextUpdaterBuilder<K, R> {
    /// Creates a new [`PrepareSecRequestContextUpdaterBuilder`] with all fields initialized to `None`.
    #[must_use]
    pub const fn new() -> Self {
        Self {
            request_input: None,
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

    /// Sets the request input field.
    #[must_use]
    pub fn request_input(mut self, request_input: K::Input) -> Self {
        self.request_input = Some(request_input);
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
    pub fn build(self) -> PrepareSecRequestContextUpdater<K, R> {
        PrepareSecRequestContextUpdater {
            request_input: self.request_input,
            repository: self.repository,
            max_retries: self.max_retries,
        }
    }
}

impl<K: RequestKind, R> Default for PrepareSecRequestContextUpdaterBuilder<K, R> {
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
    use crate::shared::notes_period::NotesPeriod;
    use crate::shared::request::implementations::sec_request::{CompanyFacts, NotesFile};
    use crate::tests::fixtures::sample_write_repository::FakeWriteRepository;

    fn create_test_context() -> PrepareSecRequestContext<CompanyFacts, FakeWriteRepository> {
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
            request_input: cik,
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
            request_input: cik,
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
        let empty_update =
            PrepareSecRequestContextUpdaterBuilder::<CompanyFacts, _>::default().build();

        let expected_result = &create_test_context();

        context.update_context(empty_update);
        let result = context.context();

        assert_eq!(result, expected_result);
    }

    const fn implements_auto_traits<T: Sized + Send + Sync + Unpin>() {}
    #[test]
    const fn should_implement_auto_traits_for_prepare_sec_request_context() {
        implements_auto_traits::<PrepareSecRequestContext<CompanyFacts, FakeWriteRepository>>();
    }

    const fn implements_send<T: Send>() {}
    const fn implements_sync<T: Sync>() {}

    #[test]
    const fn should_implement_send_for_prepare_sec_request_context() {
        implements_send::<PrepareSecRequestContext<CompanyFacts, FakeWriteRepository>>();
    }

    #[test]
    const fn should_implement_sync_for_prepare_sec_request_context() {
        implements_sync::<PrepareSecRequestContext<CompanyFacts, FakeWriteRepository>>();
    }

    #[test]
    const fn should_be_thread_safe_for_prepare_sec_request_context() {
        implements_send::<PrepareSecRequestContext<CompanyFacts, FakeWriteRepository>>();
        implements_sync::<PrepareSecRequestContext<CompanyFacts, FakeWriteRepository>>();
    }

    const fn implements_sized<T: Sized>() {}
    #[test]
    const fn should_be_sized_for_prepare_sec_request_context() {
        implements_sized::<PrepareSecRequestContext<CompanyFacts, FakeWriteRepository>>();
    }

    const fn implements_hash<T: Hash>() {}
    #[test]
    const fn should_implement_hash_for_prepare_sec_request_context() {
        implements_hash::<PrepareSecRequestContext<CompanyFacts, FakeWriteRepository>>();
    }

    const fn implements_partial_eq<T: PartialEq>() {}
    #[test]
    const fn should_implement_partial_eq_for_prepare_sec_request_context() {
        implements_partial_eq::<PrepareSecRequestContext<CompanyFacts, FakeWriteRepository>>();
    }

    const fn implements_eq<T: Eq>() {}
    #[test]
    const fn should_implement_eq_for_prepare_sec_request_context() {
        implements_eq::<PrepareSecRequestContext<CompanyFacts, FakeWriteRepository>>();
    }

    const fn implements_partial_ord<T: PartialOrd>() {}
    #[test]
    const fn should_implement_partial_ord_for_prepare_sec_request_context() {
        implements_partial_ord::<PrepareSecRequestContext<CompanyFacts, FakeWriteRepository>>();
    }

    const fn implements_ord<T: Ord>() {}
    #[test]
    const fn should_implement_ord_for_prepare_sec_request_context() {
        implements_ord::<PrepareSecRequestContext<CompanyFacts, FakeWriteRepository>>();
    }

    const fn implements_debug<T: Debug>() {}
    #[test]
    const fn should_implement_debug_for_prepare_sec_request_context() {
        implements_debug::<PrepareSecRequestContext<CompanyFacts, FakeWriteRepository>>();
    }

    const fn implements_clone<T: Clone>() {}
    #[test]
    const fn should_implement_clone_for_prepare_sec_request_context() {
        implements_clone::<PrepareSecRequestContext<CompanyFacts, FakeWriteRepository>>();
    }

    const fn implements_unpin<T: Unpin>() {}
    #[test]
    const fn should_implement_unpin_for_prepare_sec_request_context() {
        implements_unpin::<PrepareSecRequestContext<CompanyFacts, FakeWriteRepository>>();
    }

    #[test]
    fn should_return_repository_when_accessing_repository_of_prepare_sec_request_context() {
        let context = PrepareSecRequestContext::<CompanyFacts, _>::new(
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

    #[test]
    fn should_write_request_input_under_notes_period_when_serializing_notes_file_context() {
        let period = NotesPeriod::new("2024q1")
            .expect("Given a published period, NotesPeriod creation should always succeed");
        let context =
            PrepareSecRequestContext::<NotesFile, _>::new(period, FakeWriteRepository::new());

        let expected_result = serde_json::json!({
            "notes_period": serde_json::to_value(period)
                .expect("Given a domain type, serialization should always succeed"),
            "repository": { "kind": "fake", "rejection": null },
            "max_retries": 0,
        });

        let result = serde_json::to_value(context)
            .expect("Given a context of domain types, serialization should always succeed");

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_name_request_input_by_kind_when_displaying_company_facts_context() {
        let context = create_test_context();

        let expected_result = "Context Data:\ncik: 0001067983\nMax retries: 0";

        let result = context.to_string();

        assert_eq!(result, expected_result);
    }
}
