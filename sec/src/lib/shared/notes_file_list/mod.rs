//! # Notes File List
//!
//! Provides [`NotesFileList`], the Notes files that the SEC page "Financial Statement and Notes
//! Data Sets" links, and [`ListedNotesFile`], one of them.
//!
//! About a year after a quarter ends, the SEC merges the three monthly files of the quarter into
//! one quarterly file and removes the monthly files. The page lists the files that exist now.
//! Each listed period must pass [`NotesPeriod::new`].
//!
//! ## Modules
//!
//! - [`notes_file_list_error`]: The [`NotesFileListError`] returned when the page gives no valid
//!   list.

use std::sync::LazyLock;

use regex::Regex;

use crate::shared::http_client::SecClient as SecClientTrait;
use crate::shared::http_client::implementations::sec_client::error::FailedSecRequest;
use crate::shared::notes_data_sets_page::NotesDataSetsPage;
use crate::shared::notes_period::NotesPeriod;
use crate::shared::request::implementations::sec_request::SecRequest;
use crate::shared::response::SecResponse as SecResponseTrait;
use crate::shared::response::implementations::sec_response::{HtmlBody, SecResponse};

pub mod notes_file_list_error;

pub use notes_file_list_error::NotesFileListError;

/// Matches any link into the folder of the Notes files, and captures the rest of the link.
///
/// The match is wide on purpose. A link of an unknown form fails [`NotesFileList::parse`], so it
/// cannot drop out of the list.
static NOTES_FOLDER_LINK: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r#"href=["']?(?:https?://(?:www\.)?sec\.gov)?/files/dera/data/financial-statement-notes-data-sets/([^"'\s>]+)"#)
        .expect("A hardcoded regular expression should always compile")
});

/// Matches the name of a Notes file, for example `2010q1_notes_1.zip`, and captures the period.
static NOTES_FILE_NAME: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"^([0-9a-z_]+?)_notes(?:_[0-9]+)?\.zip$")
        .expect("A hardcoded regular expression should always compile")
});

/// One Notes file that the page links: its period and the file name of the link.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ListedNotesFile {
    period: NotesPeriod,
    file_name: String,
}

impl ListedNotesFile {
    /// Returns the period of the file.
    #[must_use]
    pub const fn period(&self) -> NotesPeriod {
        self.period
    }

    /// Returns the file name of the link, for example `2010q1_notes_1.zip`.
    ///
    /// The name can differ from [`NotesPeriod::file_name`]. The page is the source of this name.
    #[must_use]
    pub fn file_name(&self) -> &str {
        &self.file_name
    }
}

/// The Notes files that the page links, in increasing order of period, at least one.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NotesFileList(Vec<ListedNotesFile>);

impl NotesFileList {
    /// Fetches the page through `sec_client`, and parses it with [`parse`](Self::parse).
    ///
    /// # Errors
    ///
    /// Returns [`NotesFileListError::FailedNotesPageRequest`] if the request fails, and the errors
    /// of [`parse`](Self::parse).
    pub async fn fetch<C>(sec_client: &C) -> Result<Self, NotesFileListError>
    where
        C: SecClientTrait<Response<HtmlBody> = SecResponse<HtmlBody>, Error = FailedSecRequest>,
    {
        let request = SecRequest::builder().notes_file_listing().build();
        let response = sec_client
            .execute_sec_request(request)
            .await
            .map_err(|source| NotesFileListError::FailedNotesPageRequest {
                page: NotesDataSetsPage,
                source,
            })?;
        Self::parse(&String::from_utf8_lossy(response.body()))
    }

    /// Parses the HTML of the page into the list of linked Notes files.
    ///
    /// Every link into the folder of the Notes files must name a Notes file of the form
    /// `<period>_notes.zip` or `<period>_notes_<number>.zip`.
    ///
    /// # Errors
    ///
    /// - Returns [`NotesFileListError::MissingNotesFileLink`] if the page links no Notes file.
    /// - Returns [`NotesFileListError::UnexpectedNotesFileName`] if a link into the folder does
    ///   not name a Notes file.
    /// - Returns [`NotesFileListError::InvalidNotesFileLink`] if the name of a linked file does
    ///   not start with a valid Notes period.
    /// - Returns [`NotesFileListError::AmbiguousNotesPeriod`] if the page links two different
    ///   files for one period.
    ///
    /// # Examples
    ///
    /// ```
    /// use sec::shared::notes_file_list::NotesFileList;
    ///
    /// let page = r#"<a href="/files/dera/data/financial-statement-notes-data-sets/2010q1_notes_1.zip">2010 Q1</a>"#;
    ///
    /// let expected_result = vec!["2010q1_notes_1.zip"];
    ///
    /// let list = NotesFileList::parse(page).expect("A page with one valid link should always parse");
    /// let result: Vec<&str> = list.files().iter().map(|file| file.file_name()).collect();
    ///
    /// assert_eq!(result, expected_result);
    /// ```
    pub fn parse(page: &str) -> Result<Self, NotesFileListError> {
        let mut files = NOTES_FOLDER_LINK
            .captures_iter(page)
            .map(|link| listed_file(&link[1]))
            .collect::<Result<Vec<_>, _>>()?;
        files.sort();
        files.dedup();
        if let Some(pair) = files
            .windows(2)
            .find(|pair| pair[0].period == pair[1].period)
        {
            return Err(NotesFileListError::AmbiguousNotesPeriod {
                period: pair[0].period,
                first_file_name: pair[0].file_name.clone(),
                second_file_name: pair[1].file_name.clone(),
            });
        }
        if files.is_empty() {
            return Err(NotesFileListError::MissingNotesFileLink {
                page: NotesDataSetsPage,
            });
        }
        Ok(Self(files))
    }

    /// Returns the files in increasing order of period.
    #[must_use]
    pub fn files(&self) -> &[ListedNotesFile] {
        &self.0
    }
}

/// Builds the [`ListedNotesFile`] of one linked file name.
fn listed_file(file_name: &str) -> Result<ListedNotesFile, NotesFileListError> {
    let period = NOTES_FILE_NAME
        .captures(file_name)
        .and_then(|name| name.get(1))
        .ok_or_else(|| NotesFileListError::UnexpectedNotesFileName {
            file_name: file_name.to_string(),
        })?;
    NotesPeriod::new(period.as_str())
        .map(|period| ListedNotesFile {
            period,
            file_name: file_name.to_string(),
        })
        .map_err(|source| NotesFileListError::InvalidNotesFileLink {
            file_name: file_name.to_string(),
            source,
        })
}

#[cfg(test)]
mod tests {
    use pretty_assertions::assert_eq;

    use super::*;
    use crate::shared::notes_period::{InvalidNotesPeriodReason, NotesPeriodError};
    use crate::tests::fixtures::data::notes_data_sets_page::NOTES_DATA_SETS_PAGE;
    use crate::tests::fixtures::sample_http_client::sample_sec_client::routing::RoutingSecClient;

    const PAGE_URL: &str =
        "https://www.sec.gov/data-research/sec-markets-data/financial-statement-notes-data-sets";

    fn link(file_name: &str) -> String {
        format!(
            r#"<a href="/files/dera/data/financial-statement-notes-data-sets/{file_name}">{file_name}</a>"#
        )
    }

    fn file_names(list: &NotesFileList) -> Vec<&str> {
        list.files()
            .iter()
            .map(ListedNotesFile::file_name)
            .collect()
    }

    fn parsed_stored_page() -> NotesFileList {
        NotesFileList::parse(NOTES_DATA_SETS_PAGE)
            .expect("Given the stored page of 2026-11-07, parsing should always succeed")
    }

    #[test]
    fn should_list_80_files_for_the_stored_page_of_2026_11_07() {
        let list = parsed_stored_page();

        let expected_result = 80;

        let result = list.files().len();

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_start_at_2009q1_and_end_at_2026_10_for_the_stored_page_of_2026_11_07() {
        let list = parsed_stored_page();

        let expected_result = (Some("2009q1".to_string()), Some("2026_10".to_string()));

        let result = (
            list.files().first().map(|file| file.period().to_string()),
            list.files().last().map(|file| file.period().to_string()),
        );

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_map_2010q1_to_the_linked_file_name_for_the_stored_page_of_2026_11_07() {
        let list = parsed_stored_page();
        let period = NotesPeriod::new("2010q1")
            .expect("Given a hardcoded published quarter, parsing should always succeed");

        let expected_result = Some("2010q1_notes_1.zip");

        let result = list
            .files()
            .iter()
            .find(|file| file.period() == period)
            .map(ListedNotesFile::file_name);

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_order_the_files_by_period_when_the_page_lists_the_newest_first() {
        let page = [
            link("2025_10_notes.zip"),
            link("2025q3_notes.zip"),
            link("2009q1_notes.zip"),
        ]
        .concat();

        let expected_result = vec!["2009q1_notes.zip", "2025q3_notes.zip", "2025_10_notes.zip"];

        let list = NotesFileList::parse(&page)
            .expect("Given three valid links, parsing should always succeed");
        let result = file_names(&list);

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_list_a_file_once_when_the_page_links_it_twice() {
        let page = [link("2009q1_notes.zip"), link("2009q1_notes.zip")].concat();

        let expected_result = vec!["2009q1_notes.zip"];

        let list = NotesFileList::parse(&page)
            .expect("Given the same valid link twice, parsing should always succeed");
        let result = file_names(&list);

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_read_the_file_name_when_the_link_is_an_absolute_url() {
        let page = r#"<a href="https://www.sec.gov/files/dera/data/financial-statement-notes-data-sets/2009q1_notes.zip">"#;

        let expected_result = vec!["2009q1_notes.zip"];

        let list = NotesFileList::parse(page)
            .expect("Given one valid absolute link, parsing should always succeed");
        let result = file_names(&list);

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_ignore_a_link_when_it_points_to_another_folder() {
        let page = [
            r#"<a href="/files/dera/data/financial-statement-data-sets/2009q1.zip">"#.to_string(),
            link("2009q2_notes.zip"),
        ]
        .concat();

        let expected_result = vec!["2009q2_notes.zip"];

        let list = NotesFileList::parse(&page)
            .expect("Given one valid Notes link, parsing should always succeed");
        let result = file_names(&list);

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_return_unexpected_notes_file_name_when_a_link_has_a_query() {
        let page = [link("2009q1_notes.zip"), link("2024q1_notes.zip?v=2")].concat();

        let expected_result = Err(NotesFileListError::UnexpectedNotesFileName {
            file_name: "2024q1_notes.zip?v=2".to_string(),
        });

        let result = NotesFileList::parse(&page);

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_read_the_file_name_when_the_link_has_single_quotes() {
        let page =
            "<a href='/files/dera/data/financial-statement-notes-data-sets/2009q1_notes.zip'>";

        let expected_result = vec!["2009q1_notes.zip"];

        let list = NotesFileList::parse(page)
            .expect("Given one valid link in single quotes, parsing should always succeed");
        let result = file_names(&list);

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_return_missing_notes_file_link_when_the_page_has_no_notes_link() {
        let page = "<html><body>Page moved</body></html>";

        let expected_result = Err(NotesFileListError::MissingNotesFileLink {
            page: NotesDataSetsPage,
        });

        let result = NotesFileList::parse(page);

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_return_invalid_notes_file_link_when_a_link_has_an_invalid_period() {
        let page = [link("2009q1_notes.zip"), link("2025q5_notes.zip")].concat();

        let expected_result = Err(NotesFileListError::InvalidNotesFileLink {
            file_name: "2025q5_notes.zip".to_string(),
            source: NotesPeriodError::new(
                InvalidNotesPeriodReason::InvalidNotesPeriodFormat,
                "2025q5",
            ),
        });

        let result = NotesFileList::parse(&page);

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_return_ambiguous_notes_period_when_a_period_has_two_file_names() {
        let page = [link("2010q1_notes_1.zip"), link("2010q1_notes.zip")].concat();
        let period = NotesPeriod::new("2010q1")
            .expect("Given a hardcoded published quarter, parsing should always succeed");

        let expected_result = Err(NotesFileListError::AmbiguousNotesPeriod {
            period,
            first_file_name: "2010q1_notes.zip".to_string(),
            second_file_name: "2010q1_notes_1.zip".to_string(),
        });

        let result = NotesFileList::parse(&page);

        assert_eq!(result, expected_result);
    }

    #[tokio::test]
    async fn should_parse_the_fetched_page_when_the_sec_answers_with_html() {
        let sec_client = RoutingSecClient::new().route(
            PAGE_URL,
            200,
            "text/html; charset=utf-8",
            link("2009q1_notes.zip").as_bytes(),
        );

        let expected_result = vec!["2009q1_notes.zip"];

        let list = NotesFileList::fetch(&sec_client)
            .await
            .expect("Given a page with one valid link, fetching should always succeed");
        let result = file_names(&list);

        assert_eq!(result, expected_result);
    }

    #[tokio::test]
    async fn should_return_failed_notes_page_request_when_the_sec_answers_404() {
        let sec_client = RoutingSecClient::new();

        let expected_result = Some("[FailedNotesPageRequest] Failed to fetch the page 'https://www.sec.gov/data-research/sec-markets-data/financial-statement-notes-data-sets', Caused by: [InvalidSecResponse] Invalid SEC Response, Reason: 'Status code '404' is not accepted'".to_string());

        let result = NotesFileList::fetch(&sec_client)
            .await
            .err()
            .map(|error| error.to_string());

        assert_eq!(result, expected_result);
    }

    const fn implements_auto_traits<T: Sized + Send + Sync + Unpin>() {}

    #[test]
    const fn should_implement_auto_traits_for_notes_file_list() {
        implements_auto_traits::<NotesFileList>();
    }
}
