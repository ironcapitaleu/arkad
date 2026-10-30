//! # Execute SEC Request Context
//!
//! Provides the [`ExecuteSecRequestContext`] carried alongside the
//! [`ExecuteSecRequest`](crate::implementations::states::extract::execute_sec_request::ExecuteSecRequest)
//! state, together with its updater and builder.
//!
//! The context tracks three values: the request input (for example a [`Cik`]), the storage
//! repository, and the retry budget.
//!
//! ## Usage
//!
//! ```rust
//! use sec::implementations::states::extract::execute_sec_request::context::*;
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
//! let mut context = ExecuteSecRequestContext::<CompanyFacts, _>::new(cik, repository);
//! let update = ExecuteSecRequestContextUpdater::builder()
//!     .request_input(Cik::new("0987654321").expect("A hardcoded valid CIK should always parse"))
//!     .build();
//! context.update_context(update);
//!
//! let expected_result = "0987654321";
//!
//! let result = context.request_input().to_string();
//!
//! assert_eq!(result, expected_result);
//! ```
//!
//! ## See Also
//!
//! - [`crate::traits::state_machine::state::Context`]: Trait that defines context updates and the retry budget.
//! - [`crate::implementations::states::extract::execute_sec_request`]: Parent module for the execution state and its data types.

use std::fmt;

use serde::ser::SerializeStruct;
use serde::{Serialize, Serializer};
use state_maschine::prelude::Context as SMContext;

use crate::implementations::states::extract::store_company_facts::RawCompanyFactsRepository;
#[cfg(doc)]
use crate::shared::cik::Cik;
use crate::shared::request::implementations::sec_request::RequestKind;
use crate::traits::state_machine::state::Context;

/// Context for the [`ExecuteSecRequest`](super::ExecuteSecRequest) state.
///
/// Tracks three values: the request input of endpoint `E` (for example a [`Cik`]), the storage
/// repository, and the retry budget.
#[derive(Debug, Clone, PartialEq, PartialOrd, Hash, Eq, Ord)]
pub struct ExecuteSecRequestContext<E: RequestKind, R> {
    /// The input of the request, for example the [`Cik`] of a Company Facts request.
    pub request_input: E::Input,
    /// The storage repository.
    pub repository: R,
    /// Maximum number of times the state may be retried on failure.
    pub max_retries: u32,
}

impl<E: RequestKind, R> ExecuteSecRequestContext<E, R> {
    /// Creates a new [`ExecuteSecRequestContext`] from the request input and the storage
    /// repository, with the retry budget at zero.
    #[must_use]
    pub const fn new(request_input: E::Input, repository: R) -> Self {
        Self {
            request_input,
            repository,
            max_retries: 0,
        }
    }

    /// Returns a reference to the request input.
    #[must_use]
    pub const fn request_input(&self) -> &E::Input {
        &self.request_input
    }

    /// Returns a reference to the storage repository.
    #[must_use]
    pub const fn repository(&self) -> &R {
        &self.repository
    }
}

/// Serializes the request input under the name its endpoint gives it, for example `cik`.
impl<E: RequestKind, R: Serialize> Serialize for ExecuteSecRequestContext<E, R> {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let mut state = serializer.serialize_struct("ExecuteSecRequestContext", 3)?;
        state.serialize_field(E::INPUT_NAME, &self.request_input)?;
        state.serialize_field("repository", &self.repository)?;
        state.serialize_field("max_retries", &self.max_retries)?;
        state.end()
    }
}

impl<E: RequestKind, R: RawCompanyFactsRepository> Context for ExecuteSecRequestContext<E, R> {
    fn max_retries(&self) -> u32 {
        self.max_retries
    }
}

impl<E: RequestKind, R: RawCompanyFactsRepository> SMContext for ExecuteSecRequestContext<E, R> {
    type UpdateType = ExecuteSecRequestContextUpdater<E, R>;

    fn context(&self) -> &Self {
        self
    }

    fn update_context(&mut self, updates: Self::UpdateType) {
        if let Some(request_input) = updates.request_input {
            self.request_input = request_input;
        }
        if let Some(repository) = updates.repository {
            self.repository = repository;
        }
        if let Some(max_retries) = updates.max_retries {
            self.max_retries = max_retries;
        }
    }
}

impl<E: RequestKind, R> fmt::Display for ExecuteSecRequestContext<E, R> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Context Data: {}", self.request_input)
    }
}

/// Updater for modifying [`ExecuteSecRequestContext`].
///
/// Fields set to `None` are left unchanged when the updater is applied.
#[derive(Debug, Clone, PartialEq, PartialOrd, Hash, Eq, Ord)]
pub struct ExecuteSecRequestContextUpdater<E: RequestKind, R> {
    /// Optional new value for the request input.
    pub request_input: Option<E::Input>,
    /// Optional new value for the storage repository.
    pub repository: Option<R>,
    /// Optional new value for the retry budget.
    pub max_retries: Option<u32>,
}

impl<E: RequestKind, R> ExecuteSecRequestContextUpdater<E, R> {
    /// Creates a new builder for constructing [`ExecuteSecRequestContextUpdater`] instances.
    #[must_use]
    pub const fn builder() -> ExecuteSecRequestContextUpdaterBuilder<E, R> {
        ExecuteSecRequestContextUpdaterBuilder::new()
    }
}

/// Fluent builder for an [`ExecuteSecRequestContextUpdater`].
pub struct ExecuteSecRequestContextUpdaterBuilder<E: RequestKind, R> {
    request_input: Option<E::Input>,
    repository: Option<R>,
    max_retries: Option<u32>,
}

impl<E: RequestKind, R> ExecuteSecRequestContextUpdaterBuilder<E, R> {
    /// Creates a new [`ExecuteSecRequestContextUpdaterBuilder`] with all fields initialized to `None`.
    #[must_use]
    pub const fn new() -> Self {
        Self {
            request_input: None,
            repository: None,
            max_retries: None,
        }
    }

    /// Sets the request input field.
    #[must_use]
    #[allow(clippy::missing_const_for_fn)]
    pub fn request_input(mut self, request_input: E::Input) -> Self {
        self.request_input = Some(request_input);
        self
    }

    /// Sets the `max_retries` field.
    #[must_use]
    #[allow(clippy::missing_const_for_fn)]
    pub fn max_retries(mut self, max_retries: u32) -> Self {
        self.max_retries = Some(max_retries);
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
    pub fn build(self) -> ExecuteSecRequestContextUpdater<E, R> {
        ExecuteSecRequestContextUpdater {
            request_input: self.request_input,
            repository: self.repository,
            max_retries: self.max_retries,
        }
    }
}

impl<E: RequestKind, R> Default for ExecuteSecRequestContextUpdaterBuilder<E, R> {
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
    use crate::shared::notes_period::NotesPeriod;
    use crate::shared::request::implementations::sec_request::{CompanyFacts, NotesFile};
    use crate::tests::fixtures::sample_write_repository::FakeWriteRepository;

    type TestContext = ExecuteSecRequestContext<CompanyFacts, FakeWriteRepository>;

    #[test]
    fn should_set_cik_and_zero_max_retries_when_created_with_new() {
        let cik = Cik::new("1234567890").expect("Hardcoded CIK should always be valid");

        let expected_result = TestContext {
            request_input: cik.clone(),
            repository: FakeWriteRepository::new(),
            max_retries: 0,
        };

        let result = TestContext::new(cik, FakeWriteRepository::new());

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_return_cik_reference_when_accessing_request_input() {
        let cik = Cik::new("1234567890").expect("Hardcoded CIK should always be valid");
        let context = TestContext::new(cik.clone(), FakeWriteRepository::new());

        let expected_result = &cik;
        let result = context.request_input();

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_return_max_retries_when_accessing_max_retries() {
        let cik = Cik::new("1234567890").expect("Hardcoded CIK should always be valid");
        let context = TestContext::new(cik, FakeWriteRepository::new());

        let expected_result = 0;
        let result = context.max_retries();

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_update_cik_when_updater_contains_request_input() {
        let original_cik = Cik::new("1234567890").expect("Hardcoded CIK should always be valid");
        let new_cik = Cik::new("0987654321").expect("Hardcoded CIK should always be valid");
        let mut context = TestContext::new(original_cik, FakeWriteRepository::new());

        let updater = ExecuteSecRequestContextUpdater::<CompanyFacts, _>::builder()
            .request_input(new_cik.clone())
            .build();

        let expected_result = &new_cik;

        context.update_context(updater);
        let result = context.request_input();

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_not_update_fields_when_updater_is_empty() {
        let cik = Cik::new("1234567890").expect("Hardcoded CIK should always be valid");
        let mut context = TestContext::new(cik.clone(), FakeWriteRepository::new());
        let updater = ExecuteSecRequestContextUpdater::<CompanyFacts, _>::builder().build();

        let expected_result = TestContext::new(cik, FakeWriteRepository::new());

        context.update_context(updater);
        let result = context;

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_update_max_retries_when_updater_contains_max_retries() {
        let cik = Cik::new("1234567890").expect("Hardcoded CIK should always be valid");
        let mut context = TestContext::new(cik, FakeWriteRepository::new());

        let updater = ExecuteSecRequestContextUpdater::<CompanyFacts, _>::builder()
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
        let context = TestContext::new(cik, FakeWriteRepository::new());

        let expected_result = &context;
        let result = context.context();

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_display_context_information_when_formatted() {
        let cik = Cik::new("1234567890").expect("Hardcoded CIK should always be valid");
        let context = TestContext::new(cik, FakeWriteRepository::new());

        let expected_result = "Context Data: 1234567890";
        let result = format!("{context}");

        assert_eq!(result, expected_result);
    }

    // Trait implementation tests
    const fn implements_auto_traits<T: Sized + Send + Sync + Unpin>() {}
    #[test]
    const fn should_implement_auto_traits_for_execute_sec_request_context() {
        implements_auto_traits::<TestContext>();
    }

    const fn implements_send<T: Send>() {}
    const fn implements_sync<T: Sync>() {}

    #[test]
    const fn should_implement_send_for_execute_sec_request_context() {
        implements_send::<TestContext>();
    }

    #[test]
    const fn should_implement_sync_for_execute_sec_request_context() {
        implements_sync::<TestContext>();
    }

    #[test]
    const fn should_be_thread_safe_for_execute_sec_request_context() {
        implements_send::<TestContext>();
        implements_sync::<TestContext>();
    }

    const fn implements_sized<T: Sized>() {}
    #[test]
    const fn should_be_sized_for_execute_sec_request_context() {
        implements_sized::<TestContext>();
    }

    const fn implements_hash<T: Hash>() {}
    #[test]
    const fn should_implement_hash_for_execute_sec_request_context() {
        implements_hash::<TestContext>();
    }

    const fn implements_partial_eq<T: PartialEq>() {}
    #[test]
    const fn should_implement_partial_eq_for_execute_sec_request_context() {
        implements_partial_eq::<TestContext>();
    }

    const fn implements_eq<T: Eq>() {}
    #[test]
    const fn should_implement_eq_for_execute_sec_request_context() {
        implements_eq::<TestContext>();
    }

    const fn implements_partial_ord<T: PartialOrd>() {}
    #[test]
    const fn should_implement_partial_ord_for_execute_sec_request_context() {
        implements_partial_ord::<TestContext>();
    }

    const fn implements_ord<T: Ord>() {}
    #[test]
    const fn should_implement_ord_for_execute_sec_request_context() {
        implements_ord::<TestContext>();
    }

    const fn implements_debug<T: Debug>() {}
    #[test]
    const fn should_implement_debug_for_execute_sec_request_context() {
        implements_debug::<TestContext>();
    }

    const fn implements_clone<T: Clone>() {}
    #[test]
    const fn should_implement_clone_for_execute_sec_request_context() {
        implements_clone::<TestContext>();
    }

    const fn implements_unpin<T: Unpin>() {}
    #[test]
    const fn should_implement_unpin_for_execute_sec_request_context() {
        implements_unpin::<TestContext>();
    }

    #[test]
    fn should_return_repository_when_accessing_repository_of_execute_sec_request_context() {
        let context = TestContext::new(
            Cik::new("1234567890").expect("Hardcoded CIK should always be valid"),
            FakeWriteRepository::rejecting(WriteError::conflicting_write("Sample conflict")),
        );

        let expected_result =
            &FakeWriteRepository::rejecting(WriteError::conflicting_write("Sample conflict"));

        let result = context.repository();

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_update_repository_when_updater_contains_repository_for_execute_sec_request_context() {
        let mut context = TestContext::new(
            Cik::new("1234567890").expect("Hardcoded CIK should always be valid"),
            FakeWriteRepository::new(),
        );
        let update = ExecuteSecRequestContextUpdater::<CompanyFacts, _>::builder()
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
    fn should_name_the_request_input_notes_period_when_serializing_a_notes_file_context() {
        let period = NotesPeriod::new("2024q1")
            .expect("Given a hardcoded published quarter, parsing should always succeed");
        let context =
            ExecuteSecRequestContext::<NotesFile, _>::new(period, FakeWriteRepository::new());

        let expected_result = serde_json::json!({
            "notes_period": "2024q1",
            "repository": { "kind": "fake", "rejection": null },
            "max_retries": 0,
        });

        let result = serde_json::to_value(context)
            .expect("Given a context of domain types, serialization should always succeed");

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_include_the_repository_when_serializing_the_context() {
        let context = TestContext::new(
            Cik::new("1234567890").expect("Hardcoded CIK should always be valid"),
            FakeWriteRepository::new(),
        );

        let expected_result = serde_json::json!({
            "cik": { "value": "1234567890" },
            "repository": { "kind": "fake", "rejection": null },
            "max_retries": 0,
        });

        let result = serde_json::to_value(context)
            .expect("Given a context of domain types, serialization should always succeed");

        assert_eq!(result, expected_result);
    }
}
