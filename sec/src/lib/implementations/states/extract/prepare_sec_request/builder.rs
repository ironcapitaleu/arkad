//! # Builder
//!
//! Provides the typestate [`PrepareSecRequestBuilder`], which builds a [`PrepareSecRequest`] for
//! one request kind.
//!
//! Selecting a kind fixes the setter for its input: [`cik`](PrepareSecRequestBuilder::cik) for
//! [`CompanyFacts`] and [`period`](PrepareSecRequestBuilder::period) for [`NotesFile`].
//! [`build`](PrepareSecRequestBuilder::build) exists only once the builder holds the input of its
//! kind. If the builder lacks it, the call to `build` does not compile.
//!
//! ## Usage
//!
//! ```
//! use sec::implementations::states::extract::prepare_sec_request::PrepareSecRequest;
//! use sec::shared::notes_period::NotesPeriod;
//! use state_maschine::prelude::State as SMState;
//!
//! let period = NotesPeriod::new("2024q1").expect("A hardcoded valid period should always parse");
//! let state = PrepareSecRequest::builder().notes_file().period(period).build();
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
//!
//! let state = PrepareSecRequest::builder().all_company_facts().build();
//! ```
//!
//! The state holds no storage repository. The super-state passes the repository to the store
//! state, so the builder has no step for it:
//!
//! ```compile_fail,E0599
//! use sec::implementations::states::extract::prepare_sec_request::PrepareSecRequest;
//! use sec::shared::cik::Cik;
//! use sec::shared::raw_document::RawDocument;
//! use sec::shared::raw_source::RawCompanyFacts;
//! use storage::implementations::repository::filesystem::FilesystemRepository;
//!
//! let repository = FilesystemRepository::builder()
//!     .writer()
//!     .document_type::<RawDocument<RawCompanyFacts>>()
//!     .root("data")
//!     .build();
//! let cik = Cik::new("1067983").expect("A hardcoded valid CIK should always parse");
//! let state = PrepareSecRequest::builder()
//!     .all_company_facts()
//!     .cik(cik)
//!     .repository(repository);
//! ```

use std::marker::PhantomData;

use crate::implementations::states::extract::prepare_sec_request::{
    PrepareSecRequest, PrepareSecRequestContext, PrepareSecRequestInput,
};
use crate::shared::cik::Cik;
use crate::shared::notes_period::NotesPeriod;
use crate::shared::request::implementations::sec_request::{CompanyFacts, NotesFile, RequestKind};

/// Marks a builder that has no request kind yet.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct NoKind;

/// Marks a builder that holds no request input yet.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct NoRequestInput;

/// Builds a [`PrepareSecRequest`] for one request kind.
///
/// Select the kind with [`all_company_facts`](Self::all_company_facts) or
/// [`notes_file`](Self::notes_file), and set its input. Then call [`build`](Self::build).
///
/// The type parameters track what the builder holds: `K` is the kind, and `I` the request input.
#[derive(Debug)]
pub struct PrepareSecRequestBuilder<K, I> {
    request_input: I,
    kind: PhantomData<K>,
}

impl PrepareSecRequestBuilder<NoKind, NoRequestInput> {
    /// Creates a new [`PrepareSecRequestBuilder`] that holds no field.
    pub(crate) const fn new() -> Self {
        Self {
            request_input: NoRequestInput,
            kind: PhantomData,
        }
    }

    /// Selects the [`CompanyFacts`] request kind.
    #[must_use]
    pub const fn all_company_facts(self) -> PrepareSecRequestBuilder<CompanyFacts, NoRequestInput> {
        PrepareSecRequestBuilder::with_kind()
    }

    /// Selects the [`NotesFile`] request kind.
    #[must_use]
    pub const fn notes_file(self) -> PrepareSecRequestBuilder<NotesFile, NoRequestInput> {
        PrepareSecRequestBuilder::with_kind()
    }
}

impl<K: RequestKind> PrepareSecRequestBuilder<K, NoRequestInput> {
    const fn with_kind() -> Self {
        Self {
            request_input: NoRequestInput,
            kind: PhantomData,
        }
    }
}

impl PrepareSecRequestBuilder<CompanyFacts, NoRequestInput> {
    /// Sets the validated CIK of the Company Facts request.
    #[must_use]
    pub const fn cik(self, cik: Cik) -> PrepareSecRequestBuilder<CompanyFacts, Cik> {
        PrepareSecRequestBuilder::with_request_input(cik)
    }
}

impl PrepareSecRequestBuilder<NotesFile, NoRequestInput> {
    /// Sets the period of the Notes file request.
    #[must_use]
    pub const fn period(
        self,
        period: NotesPeriod,
    ) -> PrepareSecRequestBuilder<NotesFile, NotesPeriod> {
        PrepareSecRequestBuilder::with_request_input(period)
    }
}

impl<K: RequestKind> PrepareSecRequestBuilder<K, K::Input> {
    const fn with_request_input(request_input: K::Input) -> Self {
        Self {
            request_input,
            kind: PhantomData,
        }
    }
}

impl<K: RequestKind> PrepareSecRequestBuilder<K, K::Input> {
    /// Builds a [`PrepareSecRequest`] with no output computed yet.
    ///
    /// # Examples
    ///
    /// ```
    /// use sec::implementations::states::extract::prepare_sec_request::PrepareSecRequest;
    /// use sec::shared::cik::Cik;
    /// use state_maschine::prelude::State as SMState;
    ///
    /// let cik = Cik::new("1067983").expect("A hardcoded valid CIK should always parse");
    /// let state = PrepareSecRequest::builder().all_company_facts().cik(cik.clone()).build();
    ///
    /// let expected_result = &cik;
    ///
    /// let result = state.context_data().request_input();
    ///
    /// assert_eq!(result, expected_result);
    /// ```
    #[must_use]
    pub fn build(self) -> PrepareSecRequest<K> {
        let input = PrepareSecRequestInput::new(self.request_input.clone());
        let context = PrepareSecRequestContext::new(self.request_input);
        PrepareSecRequest::new(input, context)
    }
}

// `builder` is on the Company Facts state only, so `PrepareSecRequest::builder()` compiles without
// naming a kind. The kind methods of the builder set the kind.
impl PrepareSecRequest<CompanyFacts> {
    /// Returns a [`PrepareSecRequestBuilder`] that holds no field yet.
    #[must_use]
    pub const fn builder() -> PrepareSecRequestBuilder<NoKind, NoRequestInput> {
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

    #[test]
    fn should_build_company_facts_state_for_cik() {
        let cik = Cik::new(BERKSHIRE_HATHAWAY_CIK_RAW)
            .expect("Given a hardcoded valid CIK, Cik creation should always succeed");

        let expected_result = PrepareSecRequest::new(
            PrepareSecRequestInput::new(cik.clone()),
            PrepareSecRequestContext::new(cik.clone()),
        );

        let result = PrepareSecRequest::builder()
            .all_company_facts()
            .cik(cik)
            .build();

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_build_notes_file_state_for_period() {
        let period = NotesPeriod::new("2024q1")
            .expect("Given a published period, NotesPeriod creation should always succeed");

        let expected_result = PrepareSecRequest::<NotesFile>::new(
            PrepareSecRequestInput::new(period),
            PrepareSecRequestContext::new(period),
        );

        let result = PrepareSecRequest::builder()
            .notes_file()
            .period(period)
            .build();

        assert_eq!(result, expected_result);
    }
}
