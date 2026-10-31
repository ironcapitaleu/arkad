//! # Filing Document Set Reference
//!
//! Provides [`FilingDocumentSetRef`], which names the XBRL instance and the optional label file of
//! one filing.

use std::fmt::{self, Display, Formatter};

use serde::Serialize;

use crate::shared::filing_ref::FilingRef;

use super::FilingDocuments;

/// The XBRL instance and the optional label file of one filing.
///
/// The input of a [`FilingDocumentSet`](crate::shared::request::implementations::sec_request::FilingDocumentSet)
/// request.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize)]
pub struct FilingDocumentSetRef {
    filing: FilingRef,
    documents: FilingDocuments,
}

impl FilingDocumentSetRef {
    /// Creates a new [`FilingDocumentSetRef`] from the filing and the documents picked from its
    /// folder.
    #[must_use]
    pub const fn new(filing: FilingRef, documents: FilingDocuments) -> Self {
        Self { filing, documents }
    }

    /// Returns the filing that holds the documents.
    #[must_use]
    pub const fn filing(&self) -> &FilingRef {
        &self.filing
    }

    /// Returns the file names of the instance and the label file.
    #[must_use]
    pub const fn documents(&self) -> &FilingDocuments {
        &self.documents
    }
}

impl Display for FilingDocumentSetRef {
    /// Writes the accession number and both file names, for example
    /// `0000003545-26-000047 (Instance: alco-20260908_htm.xml, Label File: None)`.
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(
            f,
            "{} (Instance: {}, Label File: {})",
            self.filing,
            self.documents.instance(),
            self.documents.label_file().unwrap_or("None")
        )
    }
}

#[cfg(test)]
mod tests {
    use chrono::NaiveDate;
    use pretty_assertions::assert_eq;

    use crate::shared::cik::Cik;
    use crate::shared::filing_documents::FilingDocuments;
    use crate::shared::filing_ref::FilingRef;
    use crate::shared::financial::accession_number::AccessionNumber;
    use crate::shared::financial::form::{Form, ParsedForm};

    use super::FilingDocumentSetRef;

    fn filing_ref() -> FilingRef {
        FilingRef::new(
            Cik::new("3545").expect("Given a hardcoded valid CIK, parsing should always succeed"),
            AccessionNumber::new("0000003545-26-000047").expect(
                "Given a hardcoded accession number of the right shape, creation should always succeed",
            ),
            ParsedForm::new(Form::TenQ, false),
            NaiveDate::from_ymd_opt(2026, 9, 9)
                .expect("Given a hardcoded calendar date, the date should always build"),
        )
    }

    #[test]
    fn should_write_both_file_names_when_the_filing_has_a_label_file() {
        let documents = FilingDocuments::new(
            "alco-20260908_htm.xml",
            Some("alco-20260908_lab.xml".to_string()),
        );
        let document_set = FilingDocumentSetRef::new(filing_ref(), documents);

        let expected_result = "0000003545-26-000047 (Instance: alco-20260908_htm.xml, Label File: alco-20260908_lab.xml)";

        let result = document_set.to_string();

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_write_none_for_the_label_file_when_the_filing_has_none() {
        let documents = FilingDocuments::new("alco-20260908_htm.xml", None);
        let document_set = FilingDocumentSetRef::new(filing_ref(), documents);

        let expected_result =
            "0000003545-26-000047 (Instance: alco-20260908_htm.xml, Label File: None)";

        let result = document_set.to_string();

        assert_eq!(result, expected_result);
    }

    const fn implements_auto_traits<T: Send + Sync + Unpin>() {}

    #[test]
    const fn should_implement_auto_traits_for_filing_document_set_ref() {
        implements_auto_traits::<FilingDocumentSetRef>();
    }
}
