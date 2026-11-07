//! # Notes File List Errors
//!
//! Provides the [`NotesFileListError`] returned when the SEC page that links the Notes files gives
//! no valid list.

use thiserror::Error;

use crate::shared::http_client::implementations::sec_client::error::FailedSecRequest;
use crate::shared::notes_data_sets_page::NotesDataSetsPage;
use crate::shared::notes_period::{NotesPeriod, NotesPeriodError};

/// Error indicating that the SEC page that links the Notes files gives no valid list.
#[derive(Debug, Error, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum NotesFileListError {
    /// The request for the page failed.
    #[error("[FailedNotesPageRequest] Failed to fetch the page '{page}', Caused by: {source}")]
    FailedNotesPageRequest {
        /// The page that the request asked for.
        page: NotesDataSetsPage,
        /// The error of the request.
        #[source]
        source: FailedSecRequest,
    },
    /// The page links no Notes file.
    #[error("[MissingNotesFileLink] The page '{page}' links no Notes file")]
    MissingNotesFileLink {
        /// The page without a Notes file link.
        page: NotesDataSetsPage,
    },
    /// A link into the folder of the Notes files does not name a Notes file.
    #[error(
        "[UnexpectedNotesFileName] The page links '{file_name}', which is not a Notes file name"
    )]
    UnexpectedNotesFileName {
        /// The rest of the link after the folder.
        file_name: String,
    },
    /// The name of a linked file does not start with a valid Notes period.
    #[error(
        "[InvalidNotesFileLink] Failed to read the period of the linked file '{file_name}', Caused by: {source}"
    )]
    InvalidNotesFileLink {
        /// The name of the linked file.
        file_name: String,
        /// The error of the period validation.
        #[source]
        source: NotesPeriodError,
    },
    /// The page links two different files for one period.
    #[error(
        "[AmbiguousNotesPeriod] The page links two files for the period '{period}': '{first_file_name}' and '{second_file_name}'"
    )]
    AmbiguousNotesPeriod {
        /// The period with two files.
        period: NotesPeriod,
        /// The name of the first file, in the order of the file names.
        first_file_name: String,
        /// The name of the second file.
        second_file_name: String,
    },
}

#[cfg(test)]
mod tests {
    use pretty_assertions::assert_eq;

    use super::*;
    use crate::shared::notes_period::InvalidNotesPeriodReason;

    #[test]
    fn should_name_the_page_when_the_page_links_no_notes_file() {
        let error = NotesFileListError::MissingNotesFileLink {
            page: NotesDataSetsPage,
        };

        let expected_result = "[MissingNotesFileLink] The page 'https://www.sec.gov/data-research/sec-markets-data/financial-statement-notes-data-sets' links no Notes file";

        let result = error.to_string();

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_chain_the_period_error_when_a_link_has_an_invalid_period() {
        let error = NotesFileListError::InvalidNotesFileLink {
            file_name: "2025q5_notes.zip".to_string(),
            source: NotesPeriodError::new(
                InvalidNotesPeriodReason::InvalidNotesPeriodFormat,
                "2025q5",
            ),
        };

        let expected_result = "[InvalidNotesFileLink] Failed to read the period of the linked file '2025q5_notes.zip', Caused by: [NotesPeriodError] Invalid Notes period, Reason: 'Expected the form YYYYqN or YYYY_MM', Input: '2025q5'";

        let result = error.to_string();

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_name_both_files_when_a_period_has_two_file_names() {
        let error = NotesFileListError::AmbiguousNotesPeriod {
            period: NotesPeriod::new("2010q1")
                .expect("Given a hardcoded valid quarter, parsing should always succeed"),
            first_file_name: "2010q1_notes.zip".to_string(),
            second_file_name: "2010q1_notes_1.zip".to_string(),
        };

        let expected_result = "[AmbiguousNotesPeriod] The page links two files for the period '2010q1': '2010q1_notes.zip' and '2010q1_notes_1.zip'";

        let result = error.to_string();

        assert_eq!(result, expected_result);
    }

    const fn implements_auto_traits<T: Sized + Send + Sync + Unpin>() {}

    #[test]
    const fn should_implement_auto_traits_for_notes_file_list_error() {
        implements_auto_traits::<NotesFileListError>();
    }
}
