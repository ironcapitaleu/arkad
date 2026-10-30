//! # Builder
//!
//! Provides the typestate [`PrepareSecRequestBuilder`], which builds a [`PrepareSecRequest`] for
//! one request kind.
//!
//! Selecting a kind fixes the setter for its input: [`cik`](PrepareSecRequestBuilder::cik) for
//! [`CompanyFacts`] and [`period`](PrepareSecRequestBuilder::period) for [`NotesFile`].
//! [`build`](PrepareSecRequestBuilder::build) exists only once the builder holds the input of its
//! kind and a repository, so a state without one of them does not compile.
//!
//! ## Usage
//!
//! ```
//! use sec::implementations::states::extract::prepare_sec_request::PrepareSecRequest;
//! use sec::shared::notes_period::NotesPeriod;
//! use state_maschine::prelude::State as SMState;
//! # use sec::shared::raw_document::RawDocument;
//! # use sec::shared::raw_source::RawCompanyFacts;
//! # use storage::implementations::repository::filesystem::FilesystemRepository;
//! # // The repository bound is the Company Facts one until STA-385.
//! # let repository = FilesystemRepository::builder()
//! #     .writer()
//! #     .document_type::<RawDocument<RawCompanyFacts>>()
//! #     .root("data")
//! #     .build();
//!
//! let period = NotesPeriod::new("2024q1").expect("A hardcoded valid period should always parse");
//! let state = PrepareSecRequest::builder()
//!     .notes_file()
//!     .period(period)
//!     .repository(repository)
//!     .build();
//!
//! let expected_result = &period;
//!
//! let result = state.input_data().request_input();
//!
//! assert_eq!(result, expected_result);
//! ```
//!
//! A builder without the input of its kind has no `build` method, so this does not compile:
//!
//! ```compile_fail,E0599
//! use sec::implementations::states::extract::prepare_sec_request::PrepareSecRequest;
//! # use sec::shared::raw_document::RawDocument;
//! # use sec::shared::raw_source::RawCompanyFacts;
//! # use storage::implementations::repository::filesystem::FilesystemRepository;
//! # let repository = FilesystemRepository::builder()
//! #     .writer()
//! #     .document_type::<RawDocument<RawCompanyFacts>>()
//! #     .root("data")
//! #     .build();
//!
//! let state = PrepareSecRequest::builder()
//!     .all_company_facts()
//!     .repository(repository)
//!     .build();
//! ```
//!
//! A builder without a repository has no `build` method either:
//!
//! ```compile_fail,E0599
//! use sec::implementations::states::extract::prepare_sec_request::PrepareSecRequest;
//! use sec::shared::cik::Cik;
//!
//! let cik = Cik::new("1067983").expect("A hardcoded valid CIK should always parse");
//! let state = PrepareSecRequest::builder().all_company_facts().cik(cik).build();
//! ```

use std::marker::PhantomData;

use crate::implementations::states::extract::prepare_sec_request::{
    PrepareSecRequest, PrepareSecRequestContext, PrepareSecRequestInput,
};
use crate::implementations::states::extract::store_company_facts::RawCompanyFactsRepository;
use crate::shared::cik::Cik;
use crate::shared::notes_period::NotesPeriod;
use crate::shared::request::implementations::sec_request::{CompanyFacts, NotesFile, RequestKind};

/// Marks a builder that has no request kind yet.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct NoKind;

/// Marks a builder that holds no request input yet.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct NoRequestInput;

/// Marks a builder that holds no repository of type `R` yet.
///
/// The marker carries `R`. The compiler infers `R` from the value passed to
/// [`repository`](PrepareSecRequestBuilder::repository).
#[derive(Debug)]
pub struct NoRepository<R>(PhantomData<fn() -> R>);

/// Builds a [`PrepareSecRequest`] for one request kind.
///
/// Select the kind with [`all_company_facts`](Self::all_company_facts) or
/// [`notes_file`](Self::notes_file), and set its input. Set the store with
/// [`repository`](Self::repository) at any point. Then call [`build`](Self::build).
///
/// The type parameters track what the builder holds: `K` is the kind, `I` the request input, and
/// `R` the repository.
#[derive(Debug)]
pub struct PrepareSecRequestBuilder<K, I, R> {
    request_input: I,
    repository: R,
    kind: PhantomData<K>,
}

impl<R> PrepareSecRequestBuilder<NoKind, NoRequestInput, NoRepository<R>> {
    /// Creates a new [`PrepareSecRequestBuilder`] that holds no field.
    pub(crate) const fn new() -> Self {
        Self {
            request_input: NoRequestInput,
            repository: NoRepository(PhantomData),
            kind: PhantomData,
        }
    }
}

impl<R> PrepareSecRequestBuilder<NoKind, NoRequestInput, R> {
    /// Selects the [`CompanyFacts`] request kind.
    #[must_use]
    pub fn all_company_facts(self) -> PrepareSecRequestBuilder<CompanyFacts, NoRequestInput, R> {
        self.with_kind()
    }

    /// Selects the [`NotesFile`] request kind.
    #[must_use]
    pub fn notes_file(self) -> PrepareSecRequestBuilder<NotesFile, NoRequestInput, R> {
        self.with_kind()
    }

    fn with_kind<K: RequestKind>(self) -> PrepareSecRequestBuilder<K, NoRequestInput, R> {
        PrepareSecRequestBuilder {
            request_input: NoRequestInput,
            repository: self.repository,
            kind: PhantomData,
        }
    }
}

impl<R> PrepareSecRequestBuilder<CompanyFacts, NoRequestInput, R> {
    /// Sets the validated CIK of the Company Facts request.
    #[must_use]
    pub fn cik(self, cik: Cik) -> PrepareSecRequestBuilder<CompanyFacts, Cik, R> {
        self.with_request_input(cik)
    }
}

impl<R> PrepareSecRequestBuilder<NotesFile, NoRequestInput, R> {
    /// Sets the period of the Notes file request.
    #[must_use]
    pub fn period(
        self,
        period: NotesPeriod,
    ) -> PrepareSecRequestBuilder<NotesFile, NotesPeriod, R> {
        self.with_request_input(period)
    }
}

impl<K: RequestKind, R> PrepareSecRequestBuilder<K, NoRequestInput, R> {
    fn with_request_input(
        self,
        request_input: K::Input,
    ) -> PrepareSecRequestBuilder<K, K::Input, R> {
        PrepareSecRequestBuilder {
            request_input,
            repository: self.repository,
            kind: PhantomData,
        }
    }
}

impl<K, I, R> PrepareSecRequestBuilder<K, I, NoRepository<R>> {
    /// Sets the storage repository.
    #[must_use]
    pub fn repository(self, repository: R) -> PrepareSecRequestBuilder<K, I, R> {
        PrepareSecRequestBuilder {
            request_input: self.request_input,
            repository,
            kind: PhantomData,
        }
    }
}

impl<K: RequestKind, R: RawCompanyFactsRepository> PrepareSecRequestBuilder<K, K::Input, R> {
    /// Builds a [`PrepareSecRequest`] with no output computed yet.
    ///
    /// # Examples
    ///
    /// ```
    /// use sec::implementations::states::extract::prepare_sec_request::PrepareSecRequest;
    /// use sec::shared::cik::Cik;
    /// use state_maschine::prelude::State as SMState;
    /// # use sec::shared::raw_document::RawDocument;
    /// # use sec::shared::raw_source::RawCompanyFacts;
    /// # use storage::implementations::repository::filesystem::FilesystemRepository;
    /// # let repository = FilesystemRepository::builder()
    /// #     .writer()
    /// #     .document_type::<RawDocument<RawCompanyFacts>>()
    /// #     .root("data")
    /// #     .build();
    ///
    /// let cik = Cik::new("1067983").expect("A hardcoded valid CIK should always parse");
    /// let state = PrepareSecRequest::builder()
    ///     .all_company_facts()
    ///     .cik(cik.clone())
    ///     .repository(repository)
    ///     .build();
    ///
    /// let expected_result = &cik;
    ///
    /// let result = state.context_data().request_input();
    ///
    /// assert_eq!(result, expected_result);
    /// ```
    #[must_use]
    pub fn build(self) -> PrepareSecRequest<K, R> {
        let input = PrepareSecRequestInput::new(self.request_input.clone());
        let context = PrepareSecRequestContext::new(self.request_input, self.repository);

        PrepareSecRequest::new(input, context)
    }
}

// `builder` is on the Company Facts state only, so `PrepareSecRequest::builder()` compiles without
// naming a kind. The kind methods of the builder set the kind.
impl<R> PrepareSecRequest<CompanyFacts, R> {
    /// Returns a [`PrepareSecRequestBuilder`] that holds no field yet.
    #[must_use]
    pub const fn builder() -> PrepareSecRequestBuilder<NoKind, NoRequestInput, NoRepository<R>> {
        PrepareSecRequestBuilder::new()
    }
}

#[cfg(test)]
mod tests {
    use pretty_assertions::assert_eq;

    use crate::implementations::states::extract::prepare_sec_request::{
        PrepareSecRequest, PrepareSecRequestContext, PrepareSecRequestInput,
    };
    use crate::shared::cik::Cik;
    use crate::shared::cik::constants::BERKSHIRE_HATHAWAY_CIK_RAW;
    use crate::shared::notes_period::NotesPeriod;
    use crate::shared::request::implementations::sec_request::NotesFile;
    use crate::tests::fixtures::sample_write_repository::FakeWriteRepository;

    #[test]
    fn should_build_company_facts_state_for_cik_and_repository() {
        let cik = Cik::new(BERKSHIRE_HATHAWAY_CIK_RAW)
            .expect("Given a hardcoded valid CIK, Cik creation should always succeed");

        let expected_result = PrepareSecRequest::new(
            PrepareSecRequestInput::new(cik.clone()),
            PrepareSecRequestContext::new(cik.clone(), FakeWriteRepository::new()),
        );

        let result = PrepareSecRequest::builder()
            .all_company_facts()
            .cik(cik)
            .repository(FakeWriteRepository::new())
            .build();

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_build_notes_file_state_for_period_and_repository() {
        let period = NotesPeriod::new("2024q1")
            .expect("Given a published period, NotesPeriod creation should always succeed");

        let expected_result = PrepareSecRequest::<NotesFile, _>::new(
            PrepareSecRequestInput::new(period),
            PrepareSecRequestContext::new(period, FakeWriteRepository::new()),
        );

        let result = PrepareSecRequest::builder()
            .notes_file()
            .period(period)
            .repository(FakeWriteRepository::new())
            .build();

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_build_same_state_when_repository_is_set_before_kind() {
        let cik = Cik::new(BERKSHIRE_HATHAWAY_CIK_RAW)
            .expect("Given a hardcoded valid CIK, Cik creation should always succeed");

        let expected_result = PrepareSecRequest::builder()
            .all_company_facts()
            .cik(cik.clone())
            .repository(FakeWriteRepository::new())
            .build();

        let result = PrepareSecRequest::builder()
            .repository(FakeWriteRepository::new())
            .all_company_facts()
            .cik(cik)
            .build();

        assert_eq!(result, expected_result);
    }
}
