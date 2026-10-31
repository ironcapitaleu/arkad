//! # Filing Document Reference
//!
//! Provides [`FilingDocumentRef`], which names one file in the folder of one filing.

use std::fmt::{self, Display, Formatter};

use serde::Serialize;

use crate::shared::filing_ref::FilingRef;

/// One file in the folder of one filing, for example its XBRL instance.
///
/// The input of a [`FilingDocument`](crate::shared::request::implementations::sec_request::FilingDocument)
/// request.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize)]
pub struct FilingDocumentRef {
    filing: FilingRef,
    file_name: String,
}

impl FilingDocumentRef {
    /// Creates a new [`FilingDocumentRef`] from the filing and the name of one file in its folder.
    #[must_use]
    pub fn new(filing: FilingRef, file_name: impl Into<String>) -> Self {
        Self {
            filing,
            file_name: file_name.into(),
        }
    }

    /// Returns the filing that holds the file.
    #[must_use]
    pub const fn filing(&self) -> &FilingRef {
        &self.filing
    }

    /// Returns the name of the file in the folder of the filing.
    #[must_use]
    pub fn file_name(&self) -> &str {
        &self.file_name
    }
}

impl Display for FilingDocumentRef {
    /// Writes the accession number and the file name, for example
    /// `0000003545-26-000047/alco-20260908_htm.xml`.
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(f, "{}/{}", self.filing, self.file_name)
    }
}

#[cfg(test)]
mod tests {
    use chrono::NaiveDate;
    use pretty_assertions::assert_eq;

    use crate::shared::cik::Cik;
    use crate::shared::filing_ref::FilingRef;
    use crate::shared::financial::accession_number::AccessionNumber;
    use crate::shared::financial::form::{Form, ParsedForm};

    use super::FilingDocumentRef;

    fn filing_ref() -> FilingRef {
        FilingRef::new(
            Cik::new("3545").expect("Given a hardcoded valid CIK, parsing should always succeed"),
            AccessionNumber::new("0000003545-26-000047").expect(
                "Given a hardcoded accession number of the right shape, creation should always succeed",
            ),
            ParsedForm::new(Form::TenK, false),
            NaiveDate::from_ymd_opt(2026, 9, 8)
                .expect("Given a hardcoded calendar date, the date should always build"),
        )
    }

    #[test]
    fn should_write_the_accession_number_and_the_file_name_when_displayed() {
        let document = FilingDocumentRef::new(filing_ref(), "alco-20260908_htm.xml");

        let expected_result = "0000003545-26-000047/alco-20260908_htm.xml";

        let result = document.to_string();

        assert_eq!(result, expected_result);
    }

    const fn implements_auto_traits<T: Send + Sync + Unpin>() {}

    #[test]
    const fn should_implement_auto_traits_for_filing_document_ref() {
        implements_auto_traits::<FilingDocumentRef>();
    }
}
