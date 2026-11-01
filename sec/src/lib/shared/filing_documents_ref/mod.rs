//! # Filing Documents Reference
//!
//! Provides the [`FilingDocumentsRef`] type, which names the stored XBRL instance and label file
//! of one filing without holding their bytes. [`instance_key`] and [`labels_key`] give the keys of
//! the two files of a filing, and [`filing_index_key`] gives the key of its stored folder
//! `index.json`.

use std::fmt::{self, Display, Formatter};

use serde::Serialize;
use storage::implementations::repository::filesystem::DocumentKey;

use crate::shared::filing_ref::FilingRef;

/// The start of the key of every stored file of a filing, before the CIK.
const FILING_KEY_PREFIX: &str = "sec/filings/CIK";

/// The file name of the stored XBRL instance of a filing.
const INSTANCE_FILE_NAME: &str = "instance.xml";

/// The file name of the stored label file of a filing.
const LABELS_FILE_NAME: &str = "lab.xml";

/// The file name of the stored folder `index.json` of a filing.
const FILING_INDEX_FILE_NAME: &str = "index.json";

/// The filing, the key of its stored XBRL instance, and the key of its stored label file.
///
/// `labels_key` is `None` if the filing has no label file.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize)]
pub struct FilingDocumentsRef {
    filing: FilingRef,
    instance_key: DocumentKey,
    labels_key: Option<DocumentKey>,
}

impl FilingDocumentsRef {
    /// Creates a new [`FilingDocumentsRef`] for the stored files of a filing.
    ///
    /// The keys come from [`instance_key`] and [`labels_key`], so they always belong to `filing`.
    /// `has_label_file` sets whether `labels_key` is `Some`.
    #[must_use]
    pub fn new(filing: FilingRef, has_label_file: bool) -> Self {
        let instance_key = instance_key(&filing);
        let labels_key = has_label_file.then(|| labels_key(&filing));
        Self {
            filing,
            instance_key,
            labels_key,
        }
    }

    /// Returns the filing.
    #[must_use]
    pub const fn filing(&self) -> &FilingRef {
        &self.filing
    }

    /// Returns the key of the stored XBRL instance.
    #[must_use]
    pub const fn instance_key(&self) -> &DocumentKey {
        &self.instance_key
    }

    /// Returns the key of the stored label file, or `None` if the filing has no label file.
    #[must_use]
    pub const fn labels_key(&self) -> Option<&DocumentKey> {
        self.labels_key.as_ref()
    }
}

/// Formats as `Filing: <accession number>, Instance: <key>, Labels: <key or None>`.
impl Display for FilingDocumentsRef {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(
            f,
            "Filing: {}, Instance: {}, Labels: ",
            self.filing,
            self.instance_key.as_path().display()
        )?;
        match &self.labels_key {
            Some(labels_key) => write!(f, "{}", labels_key.as_path().display()),
            None => write!(f, "None"),
        }
    }
}

/// Returns the key `sec/filings/CIK##########/<accession number>/instance.xml` of the XBRL
/// instance of a filing.
///
/// # Panics
///
/// Never panics: a CIK of 10 digits and an accession number of digits and dashes hold no path
/// separator, so the key path is always valid.
///
/// # Examples
///
/// ```
/// use chrono::NaiveDate;
/// use sec::shared::cik::Cik;
/// use sec::shared::filing_documents_ref::instance_key;
/// use sec::shared::filing_ref::FilingRef;
/// use sec::shared::financial::accession_number::AccessionNumber;
/// use sec::shared::financial::form::{Form, ParsedForm};
///
/// let filing = FilingRef::new(
///     Cik::new("3545").expect("A hardcoded valid CIK should always parse"),
///     AccessionNumber::new("0000003545-26-000047")
///         .expect("A hardcoded accession number of the right shape should always parse"),
///     ParsedForm::new(Form::TenK, false),
///     NaiveDate::from_ymd_opt(2026, 9, 8).expect("A hardcoded calendar date should always build"),
/// );
///
/// let expected_result = "sec/filings/CIK0000003545/0000003545-26-000047/instance.xml";
///
/// let result = instance_key(&filing).as_path().display().to_string();
///
/// assert_eq!(result, expected_result);
/// ```
#[must_use]
pub fn instance_key(filing: &FilingRef) -> DocumentKey {
    filing_file_key(filing, INSTANCE_FILE_NAME)
}

/// Returns the key `sec/filings/CIK##########/<accession number>/lab.xml` of the label file of a
/// filing.
///
/// # Panics
///
/// Never panics: a CIK of 10 digits and an accession number of digits and dashes hold no path
/// separator, so the key path is always valid.
#[must_use]
pub fn labels_key(filing: &FilingRef) -> DocumentKey {
    filing_file_key(filing, LABELS_FILE_NAME)
}

/// Returns the key `sec/filings/CIK##########/<accession number>/index.json` of the folder
/// `index.json` of a filing.
///
/// # Panics
///
/// Never panics: a CIK of 10 digits and an accession number of digits and dashes hold no path
/// separator, so the key path is always valid.
#[must_use]
pub fn filing_index_key(filing: &FilingRef) -> DocumentKey {
    filing_file_key(filing, FILING_INDEX_FILE_NAME)
}

/// Returns the key `sec/filings/CIK##########/<accession number>/<file name>`.
fn filing_file_key(filing: &FilingRef, file_name: &str) -> DocumentKey {
    DocumentKey::new(format!(
        "{FILING_KEY_PREFIX}{}/{}/{file_name}",
        filing.cik(),
        filing.accession_number()
    ))
    .expect("Given a CIK of 10 digits and an accession number of digits and dashes, the relative key path should always be valid")
}

#[cfg(test)]
mod tests {
    use std::fmt::Debug;
    use std::hash::Hash;

    use chrono::NaiveDate;
    use pretty_assertions::assert_eq;

    use super::*;
    use crate::shared::cik::Cik;
    use crate::shared::financial::accession_number::AccessionNumber;
    use crate::shared::financial::form::{Form, ParsedForm};

    fn sample_filing() -> FilingRef {
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

    fn key(path: &str) -> DocumentKey {
        DocumentKey::new(path)
            .expect("Given a hardcoded valid relative path, the key should always build")
    }

    #[test]
    fn should_build_the_instance_key_with_a_padded_cik_and_a_dashed_accession_number_for_a_filing()
    {
        let filing = sample_filing();

        let expected_result = key("sec/filings/CIK0000003545/0000003545-26-000047/instance.xml");

        let result = instance_key(&filing);

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_build_the_filing_index_key_with_a_padded_cik_and_a_dashed_accession_number_for_a_filing()
     {
        let filing = sample_filing();

        let expected_result = key("sec/filings/CIK0000003545/0000003545-26-000047/index.json");

        let result = filing_index_key(&filing);

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_build_the_labels_key_with_a_padded_cik_and_a_dashed_accession_number_for_a_filing() {
        let filing = sample_filing();

        let expected_result = key("sec/filings/CIK0000003545/0000003545-26-000047/lab.xml");

        let result = labels_key(&filing);

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_display_the_filing_and_both_keys_when_the_filing_has_a_label_file() {
        let documents = FilingDocumentsRef::new(sample_filing(), true);

        let expected_result = "Filing: 0000003545-26-000047, Instance: sec/filings/CIK0000003545/0000003545-26-000047/instance.xml, Labels: sec/filings/CIK0000003545/0000003545-26-000047/lab.xml";

        let result = documents.to_string();

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_display_none_for_the_labels_when_the_filing_has_no_label_file() {
        let documents = FilingDocumentsRef::new(sample_filing(), false);

        let expected_result = "Filing: 0000003545-26-000047, Instance: sec/filings/CIK0000003545/0000003545-26-000047/instance.xml, Labels: None";

        let result = documents.to_string();

        assert_eq!(result, expected_result);
    }

    const fn implements_value_traits<
        T: Sized + Send + Sync + Unpin + Clone + Debug + Hash + Eq + Ord,
    >() {
    }
    #[test]
    const fn should_implement_value_traits_for_filing_documents_ref() {
        implements_value_traits::<FilingDocumentsRef>();
    }
}
