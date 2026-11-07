//! # Notes Data-Sets Page
//!
//! Provides [`NotesDataSetsPage`], the input of the request for the SEC page that links every
//! current Financial Statement and Notes file.

use std::fmt::{self, Display, Formatter};

use serde::{Serialize, Serializer};

use crate::shared::request::implementations::sec_request::constants::SEC_NOTES_DATA_SETS_PAGE_URL;

/// The SEC page "Financial Statement and Notes Data Sets", which links every current Notes file.
///
/// The SEC has one such page, so the type has no fields. It displays and serializes as the URL of
/// the page.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct NotesDataSetsPage;

impl Display for NotesDataSetsPage {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(f, "{SEC_NOTES_DATA_SETS_PAGE_URL}")
    }
}

impl Serialize for NotesDataSetsPage {
    /// Serializes the page as its URL.
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.collect_str(self)
    }
}

#[cfg(test)]
mod tests {
    use pretty_assertions::assert_eq;

    use super::*;

    #[test]
    fn should_display_the_page_url_for_notes_data_sets_page() {
        let page = NotesDataSetsPage;

        let expected_result = "https://www.sec.gov/data-research/sec-markets-data/financial-statement-notes-data-sets";

        let result = page.to_string();

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_serialize_the_page_url_for_notes_data_sets_page() {
        let page = NotesDataSetsPage;

        let expected_result = "\"https://www.sec.gov/data-research/sec-markets-data/financial-statement-notes-data-sets\"";

        let result = serde_json::to_string(&page)
            .expect("Serializing a page as a JSON string should always succeed");

        assert_eq!(result, expected_result);
    }

    const fn implements_auto_traits<T: Sized + Send + Sync + Unpin>() {}

    #[test]
    const fn should_implement_auto_traits_for_notes_data_sets_page() {
        implements_auto_traits::<NotesDataSetsPage>();
    }
}
