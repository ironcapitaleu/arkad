//! # Builder
//!
//! Provides [`DetectFilingsSuperStateBuilder`], which builds a [`DetectFilingsSuperState`] at its
//! entry state [`ValidateIndexDate`].
//!
//! Each setter changes the builder's type. [`build`](DetectFilingsSuperStateBuilder::build) exists
//! only once the builder holds a date, an SEC client, an `execution_id`, and a repository, so a
//! super-state without one of them does not compile.

use std::marker::PhantomData;

use uuid::Uuid;

use crate::implementations::states::extract::daily_index::{
    DetectFilingsSuperState, DetectFilingsSuperStateContext, DetectFilingsSuperStateData,
};
use crate::implementations::states::extract::validate_index_date::{
    ValidateIndexDate, ValidateIndexDateContext, ValidateIndexDateInput,
};
use crate::shared::http_client::SecClient as SecClientTrait;
use crate::shared::http_client::implementations::sec_client::error::FailedSecRequest;
use crate::shared::raw_repository::RawRepository;
use crate::shared::raw_source::RawDailyIndex;
use crate::shared::response::implementations::sec_response::{SecResponse, ZipBody};

/// Marks a builder that holds no date yet.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct NoDate;

/// Marks a builder that holds no SEC client yet.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct NoSecClient;

/// Marks a builder that holds no `execution_id` yet.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct NoExecutionId;

/// Marks a builder that holds no repository of type `R` yet.
///
/// The marker carries `R`. The compiler infers `R` from the value passed to
/// [`repository`](DetectFilingsSuperStateBuilder::repository).
// Deviation: the derives bound `R` on each trait, although the marker holds no `R`. Every
// `RawRepository<RawDailyIndex>` implements these traits, so the bounds cost nothing here.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct NoRepository<R>(PhantomData<fn() -> R>);

impl<R> NoRepository<R> {
    /// Creates a new [`NoRepository`] marker.
    #[must_use]
    pub const fn new() -> Self {
        Self(PhantomData)
    }
}

impl<R> Default for NoRepository<R> {
    fn default() -> Self {
        Self::new()
    }
}

/// Builds a [`DetectFilingsSuperState`] at its entry state [`ValidateIndexDate`].
///
/// Set the raw date with [`date`](Self::date), the client with [`sec_client`](Self::sec_client),
/// the run identifier with [`execution_id`](Self::execution_id), and the store with
/// [`repository`](Self::repository), in any order. Then call [`build`](Self::build).
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct DetectFilingsSuperStateBuilder<D, C, E, R> {
    date: D,
    sec_client: C,
    execution_id: E,
    repository: R,
}

impl<R> DetectFilingsSuperStateBuilder<NoDate, NoSecClient, NoExecutionId, NoRepository<R>> {
    /// Creates a new [`DetectFilingsSuperStateBuilder`] that holds no field.
    pub(crate) const fn new() -> Self {
        Self {
            date: NoDate,
            sec_client: NoSecClient,
            execution_id: NoExecutionId,
            repository: NoRepository::new(),
        }
    }
}

impl<C, E, R> DetectFilingsSuperStateBuilder<NoDate, C, E, R> {
    /// Sets the raw date, for example `2026-10-27`. The built super-state validates it in
    /// [`ValidateIndexDate`].
    #[must_use]
    pub fn date(self, date: impl Into<String>) -> DetectFilingsSuperStateBuilder<String, C, E, R> {
        DetectFilingsSuperStateBuilder {
            date: date.into(),
            sec_client: self.sec_client,
            execution_id: self.execution_id,
            repository: self.repository,
        }
    }
}

impl<D, E, R> DetectFilingsSuperStateBuilder<D, NoSecClient, E, R> {
    /// Sets the SEC client that downloads the daily index and the quarter listings.
    ///
    /// The client must return an [`SecResponse<ZipBody>`] and a [`FailedSecRequest`], as
    /// `DownloadDailyIndex` requires.
    #[must_use]
    pub fn sec_client<C>(self, sec_client: C) -> DetectFilingsSuperStateBuilder<D, C, E, R>
    where
        C: SecClientTrait<Response<ZipBody> = SecResponse<ZipBody>, Error = FailedSecRequest>,
    {
        DetectFilingsSuperStateBuilder {
            date: self.date,
            sec_client,
            execution_id: self.execution_id,
            repository: self.repository,
        }
    }
}

impl<D, C, R> DetectFilingsSuperStateBuilder<D, C, NoExecutionId, R> {
    /// Sets the `execution_id` that tags the stream and the stored document of the run.
    #[must_use]
    pub fn execution_id(self, execution_id: Uuid) -> DetectFilingsSuperStateBuilder<D, C, Uuid, R> {
        DetectFilingsSuperStateBuilder {
            date: self.date,
            sec_client: self.sec_client,
            execution_id,
            repository: self.repository,
        }
    }
}

impl<D, C, E, R> DetectFilingsSuperStateBuilder<D, C, E, NoRepository<R>> {
    /// Sets the repository that stores the daily index.
    #[must_use]
    pub fn repository(self, repository: R) -> DetectFilingsSuperStateBuilder<D, C, E, R> {
        DetectFilingsSuperStateBuilder {
            date: self.date,
            sec_client: self.sec_client,
            execution_id: self.execution_id,
            repository,
        }
    }
}

impl<C: SecClientTrait, R: RawRepository<RawDailyIndex>>
    DetectFilingsSuperStateBuilder<String, C, Uuid, R>
{
    /// Builds a [`DetectFilingsSuperState`] at [`ValidateIndexDate`] with the fields this builder
    /// holds.
    ///
    /// # Examples
    ///
    /// A builder without a repository has no `build` method, so this does not compile:
    ///
    /// ```compile_fail,E0599
    /// use sec::implementations::states::extract::daily_index::DetectFilingsSuperState;
    /// use sec::shared::http_client::implementations::sec_client::SecClient;
    /// use sec::shared::raw_document::RawDocument;
    /// use sec::shared::raw_source::RawDailyIndex;
    /// use storage::implementations::repository::filesystem::{FilesystemRepository, Write};
    /// use uuid::Uuid;
    ///
    /// let detect_state = DetectFilingsSuperState::<
    ///     _,
    ///     FilesystemRepository<Write, RawDocument<RawDailyIndex>>,
    /// >::builder()
    /// .date("2026-10-27")
    /// .sec_client(SecClient::default())
    /// .execution_id(Uuid::new_v4())
    /// .build();
    /// ```
    #[must_use]
    pub fn build(self) -> DetectFilingsSuperState<ValidateIndexDate, R, C> {
        let input = ValidateIndexDateInput::new(self.date);
        let state_context = ValidateIndexDateContext::new(self.execution_id);

        DetectFilingsSuperState {
            current_state: ValidateIndexDate::new(input, state_context),
            input: DetectFilingsSuperStateData,
            output: None,
            context: DetectFilingsSuperStateContext::new(
                self.sec_client,
                self.execution_id,
                self.repository,
            ),
        }
    }
}

#[cfg(test)]
mod tests {
    use std::fmt::Debug;
    use std::hash::Hash;

    use pretty_assertions::{assert_eq, assert_ne};

    use super::*;
    use crate::tests::fixtures::sample_http_client::sample_sec_client::routing::RoutingSecClient;
    use crate::tests::fixtures::sample_write_repository::FakeDailyIndexRepository;

    type EmptyBuilder = DetectFilingsSuperStateBuilder<
        NoDate,
        NoSecClient,
        NoExecutionId,
        NoRepository<FakeDailyIndexRepository>,
    >;

    fn expected_super_state(
        raw_date: &str,
        execution_id: Uuid,
    ) -> DetectFilingsSuperState<ValidateIndexDate, FakeDailyIndexRepository, RoutingSecClient>
    {
        DetectFilingsSuperState {
            current_state: ValidateIndexDate::new(
                ValidateIndexDateInput::new(raw_date),
                ValidateIndexDateContext::new(execution_id),
            ),
            input: DetectFilingsSuperStateData,
            output: None,
            context: DetectFilingsSuperStateContext::new(
                RoutingSecClient::new(),
                execution_id,
                FakeDailyIndexRepository::new(),
            ),
        }
    }

    const fn implements_auto_traits<T: Sized + Send + Sync + Unpin>() {}
    #[test]
    const fn should_implement_auto_traits_for_detect_filings_super_state_builder() {
        implements_auto_traits::<EmptyBuilder>();
    }

    const fn implements_value_traits<
        T: Debug + Clone + PartialEq + Eq + PartialOrd + Ord + Hash,
    >() {
    }
    #[test]
    const fn should_implement_value_traits_for_detect_filings_super_state_builder() {
        implements_value_traits::<EmptyBuilder>();
    }

    const fn holds_no_field(_: EmptyBuilder) {}
    #[test]
    const fn should_hold_no_field_when_starting_a_builder() {
        holds_no_field(DetectFilingsSuperState::builder());
    }

    #[test]
    fn should_build_a_super_state_at_validate_index_date_when_setting_every_field() {
        let builder = DetectFilingsSuperState::builder()
            .date("2026-10-27")
            .sec_client(RoutingSecClient::new())
            .execution_id(Uuid::from_u128(7));
        let repository = FakeDailyIndexRepository::new();

        let expected_result = expected_super_state("2026-10-27", Uuid::from_u128(7));

        let result = builder.repository(repository).build();

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_build_the_same_super_state_when_setting_the_fields_in_reverse_order() {
        let builder = DetectFilingsSuperState::builder()
            .repository(FakeDailyIndexRepository::new())
            .execution_id(Uuid::from_u128(7))
            .sec_client(RoutingSecClient::new());
        let date = "2026-10-27";

        let expected_result = expected_super_state("2026-10-27", Uuid::from_u128(7));

        let result = builder.date(date).build();

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_compare_unequal_when_two_builders_hold_different_dates() {
        let builder =
            DetectFilingsSuperState::<ValidateIndexDate, FakeDailyIndexRepository>::builder();
        let date = "2026-10-28";

        let expected_result =
            DetectFilingsSuperState::<ValidateIndexDate, FakeDailyIndexRepository>::builder()
                .date("2026-10-27");

        let result = builder.date(date);

        assert_ne!(result, expected_result);
    }
}
