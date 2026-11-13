//! # Stored Cut-Over Date
//!
//! Provides [`stored_cut_over_date`], which reads the cut-over date from the keys of the stored
//! Notes files, and its error [`MissingStoredCutOverDate`].
//!
//! The function sends no SEC request. Rule 1 of "Cut-Over Between the Paths" in
//! `sec/design/pipeline/ingestion_paths.md` defines the date, and "2. Cut-Over Date" in
//! `storage/design/data_model/repository_listing.md` defines the gap check.

use std::collections::BTreeSet;

use chrono::{Datelike, NaiveDate};
use futures_util::StreamExt;
use storage::ReadError;
use storage::implementations::repository::filesystem::{DocumentKey, KeyPrefix};
use storage::traits::repository::ListRepository;
use thiserror::Error;

use crate::implementations::states::extract::store_notes_archive::NOTES_KEY_PREFIX;
use crate::shared::notes_period::constants::{FILE_NAME_MARKER, FIRST_YEAR};
use crate::shared::notes_period::{InvalidNotesPeriodReason, NotesPeriod, NotesPeriodError};

/// Error indicating that the stored Notes files give no cut-over date.
#[derive(Debug, Error, Clone, PartialEq, Eq)]
pub enum MissingStoredCutOverDate {
    /// The store could not list a Notes file.
    #[error("[MissingStoredCutOverDate] Failed to list the stored Notes files, Caused by: {0}")]
    FailedNotesListing(#[source] ReadError),
    /// The file name of a key under `sec/notes` names no Notes period.
    #[error(
        "[MissingStoredCutOverDate] Failed to read the Notes period of the key '{key}', Caused by: {source}"
    )]
    InvalidNotesKey {
        /// The key, as a path relative to the root of the store.
        key: String,
        /// The error of the period text: the file name before `_notes`.
        #[source]
        source: NotesPeriodError,
    },
    /// The store holds no Notes file.
    #[error("[MissingStoredCutOverDate] The store holds no Notes file")]
    MissingStoredPeriod,
    /// No stored file covers a quarter before the newest stored period.
    #[error("[MissingStoredCutOverDate] No stored Notes file covers the quarter '{quarter}'")]
    UncoveredQuarter {
        /// The first quarter that no stored file covers.
        quarter: NotesPeriod,
    },
}

/// Returns the cut-over date from the Notes files under `sec/notes` of `notes_store`: the last day
/// of the newest stored period.
///
/// The newest period is the one with the latest [`NotesPeriod::last_day`]. Every quarter from the
/// first quarter of [`FIRST_YEAR`] through the quarter of the newest period must be covered: by its
/// quarterly file, or by its three monthly files. The quarter of the newest period needs only its
/// months up to the newest one. The store can hold a quarter and its months at once. Both count.
///
/// The check reads the keys alone. It sends no SEC request and reads no file.
///
/// # Errors
///
/// Returns a [`MissingStoredCutOverDate`] if:
/// - the listing yields an error item ([`FailedNotesListing`]).
/// - the file name of a key has no `_notes`, or its text before `_notes` is no Notes period
///   ([`InvalidNotesKey`]).
/// - the store holds no Notes file ([`MissingStoredPeriod`]).
/// - a quarter is not covered. The error names the first one ([`UncoveredQuarter`]).
///
/// [`FailedNotesListing`]: MissingStoredCutOverDate::FailedNotesListing
/// [`InvalidNotesKey`]: MissingStoredCutOverDate::InvalidNotesKey
/// [`MissingStoredPeriod`]: MissingStoredCutOverDate::MissingStoredPeriod
/// [`UncoveredQuarter`]: MissingStoredCutOverDate::UncoveredQuarter
///
/// # Panics
///
/// Never panics: [`NOTES_KEY_PREFIX`] is a valid relative key, and every quarter and month the
/// check builds comes from [`FIRST_YEAR`] on.
pub async fn stored_cut_over_date<R>(notes_store: &R) -> Result<NaiveDate, MissingStoredCutOverDate>
where
    R: ListRepository<Key = DocumentKey, Filter = KeyPrefix> + ?Sized,
{
    let stored = stored_periods(notes_store).await?;
    let newest = stored
        .iter()
        .map(NotesPeriod::last_day)
        .max()
        .ok_or(MissingStoredCutOverDate::MissingStoredPeriod)?;
    first_uncovered_quarter(&stored, newest).map_or(Ok(newest), |quarter| {
        Err(MissingStoredCutOverDate::UncoveredQuarter { quarter })
    })
}

/// Lists the keys under `sec/notes` and parses the period of each.
///
/// # Errors
///
/// Returns [`MissingStoredCutOverDate::FailedNotesListing`] or
/// [`MissingStoredCutOverDate::InvalidNotesKey`].
async fn stored_periods<R>(
    notes_store: &R,
) -> Result<BTreeSet<NotesPeriod>, MissingStoredCutOverDate>
where
    R: ListRepository<Key = DocumentKey, Filter = KeyPrefix> + ?Sized,
{
    let notes_directory = DocumentKey::new(NOTES_KEY_PREFIX)
        .expect("Given the constant relative path of the Notes files, the key should always build");
    let mut keys = notes_store.list_keys(KeyPrefix::from(notes_directory));
    let mut periods = BTreeSet::new();
    while let Some(key) = keys.next().await {
        let key = key.map_err(MissingStoredCutOverDate::FailedNotesListing)?;
        periods.insert(period_of(&key)?);
    }
    Ok(periods)
}

/// Parses the period of a Notes key from the text of its file name before `_notes`.
///
/// # Errors
///
/// Returns [`MissingStoredCutOverDate::InvalidNotesKey`] if the file name has no `_notes`, or its
/// text before `_notes` is no Notes period.
fn period_of(key: &DocumentKey) -> Result<NotesPeriod, MissingStoredCutOverDate> {
    let file_name = key
        .as_path()
        .file_name()
        .map(|name| name.to_string_lossy())
        .unwrap_or_default();
    file_name
        .split_once(FILE_NAME_MARKER)
        .map_or_else(
            || {
                Err(NotesPeriodError::new(
                    InvalidNotesPeriodReason::InvalidNotesPeriodFormat,
                    file_name.as_ref(),
                ))
            },
            |(period, _)| NotesPeriod::new(period),
        )
        .map_err(|source| MissingStoredCutOverDate::InvalidNotesKey {
            key: key.as_path().display().to_string(),
            source,
        })
}

/// Returns the first quarter from [`FIRST_YEAR`] through the month of `newest` that `stored` does
/// not cover, or `None` if it covers all of them.
fn first_uncovered_quarter(
    stored: &BTreeSet<NotesPeriod>,
    newest: NaiveDate,
) -> Option<NotesPeriod> {
    let newest_month = (newest.year(), newest.month());
    for year in i32::from(FIRST_YEAR.number())..=newest.year() {
        for quarter in 1..=4 {
            let months: Vec<u32> = (quarter * 3 - 2..=quarter * 3)
                .filter(|month| (year, *month) <= newest_month)
                .collect();
            if months.is_empty() {
                return None;
            }
            let quarter = period(&format!("{year}q{quarter}"));
            let is_covered = stored.contains(&quarter)
                || months
                    .iter()
                    .all(|month| stored.contains(&period(&format!("{year}_{month:02}"))));
            if !is_covered {
                return Some(quarter);
            }
        }
    }
    None
}

/// Parses a period that the check builds itself.
fn period(period: &str) -> NotesPeriod {
    NotesPeriod::new(period).expect(
        "Given a quarter or a zero-padded month from the first Notes year on, parsing should always succeed",
    )
}

#[cfg(test)]
mod tests {
    use futures_core::stream::BoxStream;
    use pretty_assertions::assert_eq;
    use storage::error::BackendError;

    use super::*;

    /// A store that lists its seeded keys under the directory of the filter, sorted by bytes.
    struct StoredKeys(Vec<Result<DocumentKey, ReadError>>);

    impl StoredKeys {
        fn of(keys: &[&str]) -> Self {
            Self(
                keys.iter()
                    .map(|key| {
                        Ok(DocumentKey::new(*key)
                            .expect("Given a hardcoded relative path, the key should always build"))
                    })
                    .collect(),
            )
        }
    }

    impl ListRepository for StoredKeys {
        type Key = DocumentKey;
        type Entry = ();
        type Filter = KeyPrefix;

        fn list_keys(&self, filter: KeyPrefix) -> BoxStream<'_, Result<DocumentKey, ReadError>> {
            let mut items: Vec<_> = self
                .0
                .iter()
                .filter(|item| {
                    item.as_ref()
                        .map_or(true, |key| key.as_path().starts_with(filter.as_path()))
                })
                .cloned()
                .collect();
            items.sort_by_key(|item| {
                item.as_ref()
                    .map(|key| key.as_path().display().to_string())
                    .unwrap_or_default()
            });
            Box::pin(futures_util::stream::iter(items))
        }

        fn list(&self, _filter: KeyPrefix) -> BoxStream<'_, Result<(), ReadError>> {
            Box::pin(futures_util::stream::empty())
        }
    }

    fn date(year: i32, month: u32, day: u32) -> NaiveDate {
        NaiveDate::from_ymd_opt(year, month, day)
            .expect("Given a hardcoded calendar date, the date should always be valid")
    }

    #[tokio::test]
    async fn should_give_the_same_date_when_a_merged_quarter_is_stored_beside_its_three_months() {
        let merged_only = StoredKeys::of(&[
            "sec/notes/2009q1_notes.zip",
            "sec/notes/2009q2_notes.zip",
            "sec/notes/2009q3_notes.zip",
        ]);
        let merged_and_months = StoredKeys::of(&[
            "sec/notes/2009q1_notes.zip",
            "sec/notes/2009q2_notes.zip",
            "sec/notes/2009q3_notes.zip",
            "sec/notes/2009_07_notes.zip",
            "sec/notes/2009_08_notes.zip",
            "sec/notes/2009_09_notes.zip",
        ]);

        let expected_result = (Ok(date(2009, 9, 30)), Ok(date(2009, 9, 30)));

        let result = (
            stored_cut_over_date(&merged_only).await,
            stored_cut_over_date(&merged_and_months).await,
        );

        assert_eq!(result, expected_result);
    }

    #[tokio::test]
    async fn should_count_a_quarter_as_covered_when_its_three_months_are_stored() {
        let notes_store = StoredKeys::of(&[
            "sec/notes/2009q1_notes.zip",
            "sec/notes/2009_04_notes.zip",
            "sec/notes/2009_05_notes.zip",
            "sec/notes/2009_06_notes.zip",
            "sec/notes/2009q3_notes.zip",
        ]);

        let expected_result = Ok(date(2009, 9, 30));

        let result = stored_cut_over_date(&notes_store).await;

        assert_eq!(result, expected_result);
    }

    #[tokio::test]
    async fn should_name_the_quarter_when_one_of_its_three_months_is_missing() {
        let notes_store = StoredKeys::of(&[
            "sec/notes/2009q1_notes.zip",
            "sec/notes/2009_04_notes.zip",
            "sec/notes/2009_06_notes.zip",
            "sec/notes/2009q3_notes.zip",
        ]);

        let expected_result = Err(MissingStoredCutOverDate::UncoveredQuarter {
            quarter: period("2009q2"),
        });

        let result = stored_cut_over_date(&notes_store).await;

        assert_eq!(result, expected_result);
    }

    #[tokio::test]
    async fn should_name_the_first_quarter_when_the_first_period_is_missing() {
        let notes_store = StoredKeys::of(&[
            "sec/notes/2009q2_notes.zip",
            "sec/notes/2009q3_notes.zip",
            "sec/notes/2009q4_notes.zip",
        ]);

        let expected_result = Err(MissingStoredCutOverDate::UncoveredQuarter {
            quarter: period("2009q1"),
        });

        let result = stored_cut_over_date(&notes_store).await;

        assert_eq!(result, expected_result);
    }

    #[tokio::test]
    async fn should_name_the_quarter_when_a_period_in_the_middle_is_missing() {
        let notes_store = StoredKeys::of(&[
            "sec/notes/2009q1_notes.zip",
            "sec/notes/2009q2_notes.zip",
            "sec/notes/2009q4_notes.zip",
            "sec/notes/2010q1_notes_1.zip",
        ]);

        let expected_result = Err(MissingStoredCutOverDate::UncoveredQuarter {
            quarter: period("2009q3"),
        });

        let result = stored_cut_over_date(&notes_store).await;

        assert_eq!(result, expected_result);
    }

    #[tokio::test]
    async fn should_fail_when_the_store_holds_no_notes_file() {
        let notes_store = StoredKeys::of(&[]);

        let expected_result = Err(MissingStoredCutOverDate::MissingStoredPeriod);

        let result = stored_cut_over_date(&notes_store).await;

        assert_eq!(result, expected_result);
    }

    #[tokio::test]
    async fn should_need_only_the_months_up_to_the_newest_when_the_newest_quarter_has_not_ended() {
        let notes_store = StoredKeys::of(&[
            "sec/notes/2009q1_notes.zip",
            "sec/notes/2009q2_notes.zip",
            "sec/notes/2009q3_notes.zip",
            "sec/notes/2009_10_notes.zip",
            "sec/notes/2009_11_notes.zip",
        ]);

        let expected_result = Ok(date(2009, 11, 30));

        let result = stored_cut_over_date(&notes_store).await;

        assert_eq!(result, expected_result);
    }

    #[tokio::test]
    async fn should_parse_2010q1_when_the_key_has_the_override_file_name_of_2010q1() {
        let notes_store = StoredKeys::of(&[
            "sec/notes/2009q1_notes.zip",
            "sec/notes/2009q2_notes.zip",
            "sec/notes/2009q3_notes.zip",
            "sec/notes/2009q4_notes.zip",
            "sec/notes/2010q1_notes_1.zip",
        ]);

        let expected_result = Ok(date(2010, 3, 31));

        let result = stored_cut_over_date(&notes_store).await;

        assert_eq!(result, expected_result);
    }

    #[tokio::test]
    async fn should_parse_2010q2_when_the_key_has_the_override_file_name_of_2010q2() {
        let notes_store = StoredKeys::of(&[
            "sec/notes/2009q1_notes.zip",
            "sec/notes/2009q2_notes.zip",
            "sec/notes/2009q3_notes.zip",
            "sec/notes/2009q4_notes.zip",
            "sec/notes/2010q1_notes_1.zip",
            "sec/notes/2010q2_notes_0.zip",
        ]);

        let expected_result = Ok(date(2010, 6, 30));

        let result = stored_cut_over_date(&notes_store).await;

        assert_eq!(result, expected_result);
    }

    #[tokio::test]
    async fn should_parse_2010q3_when_the_key_has_the_override_file_name_of_2010q3() {
        let notes_store = StoredKeys::of(&[
            "sec/notes/2009q1_notes.zip",
            "sec/notes/2009q2_notes.zip",
            "sec/notes/2009q3_notes.zip",
            "sec/notes/2009q4_notes.zip",
            "sec/notes/2010q1_notes_1.zip",
            "sec/notes/2010q2_notes_0.zip",
            "sec/notes/2010q3_notes_0.zip",
        ]);

        let expected_result = Ok(date(2010, 9, 30));

        let result = stored_cut_over_date(&notes_store).await;

        assert_eq!(result, expected_result);
    }

    #[tokio::test]
    async fn should_list_only_the_keys_under_sec_notes_when_the_store_holds_other_documents() {
        let notes_store = StoredKeys::of(&[
            "sec/notes/2009q1_notes.zip",
            "sec/filings/0000320193/000032019325000079/instance.xml",
        ]);

        let expected_result = Ok(date(2009, 3, 31));

        let result = stored_cut_over_date(&notes_store).await;

        assert_eq!(result, expected_result);
    }

    #[tokio::test]
    async fn should_fail_with_the_key_when_a_file_name_has_no_notes_marker() {
        let notes_store = StoredKeys::of(&["sec/notes/2009q1.zip"]);

        let expected_result = Err(MissingStoredCutOverDate::InvalidNotesKey {
            key: "sec/notes/2009q1.zip".to_string(),
            source: NotesPeriodError::new(
                InvalidNotesPeriodReason::InvalidNotesPeriodFormat,
                "2009q1.zip",
            ),
        });

        let result = stored_cut_over_date(&notes_store).await;

        assert_eq!(result, expected_result);
    }

    #[tokio::test]
    async fn should_fail_with_the_key_when_the_text_before_the_notes_marker_is_no_period() {
        let notes_store = StoredKeys::of(&["sec/notes/2008q4_notes.zip"]);

        let expected_result = Err(MissingStoredCutOverDate::InvalidNotesKey {
            key: "sec/notes/2008q4_notes.zip".to_string(),
            source: NotesPeriod::new("2008q4")
                .expect_err("Given a year before the first Notes year, parsing should always fail"),
        });

        let result = stored_cut_over_date(&notes_store).await;

        assert_eq!(result, expected_result);
    }

    #[tokio::test]
    async fn should_fail_when_the_listing_yields_an_error_item() {
        let listing_error = ReadError::Backend(BackendError::failed_operation(
            "Failed to list the directory",
        ));
        let mut notes_store = StoredKeys::of(&["sec/notes/2009q1_notes.zip"]);
        notes_store.0.push(Err(listing_error.clone()));

        let expected_result = Err(MissingStoredCutOverDate::FailedNotesListing(listing_error));

        let result = stored_cut_over_date(&notes_store).await;

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_display_the_quarter_when_a_quarter_is_not_covered() {
        let error = MissingStoredCutOverDate::UncoveredQuarter {
            quarter: period("2009q3"),
        };

        let expected_result =
            "[MissingStoredCutOverDate] No stored Notes file covers the quarter '2009q3'";

        let result = error.to_string();

        assert_eq!(result, expected_result);
    }
}
