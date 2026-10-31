//! # Filing Documents
//!
//! Provides [`FilingDocuments`], the XBRL instance and the optional label file of one filing, and
//! the rules that pick them from the folder index of the filing.
//!
//! ## Modules
//!
//! - [`filing_documents_error`]: The [`FilingDocumentsError`] returned when the folder index does
//!   not parse or no single document matches a rule, and the [`FilingDocumentRole`] it names.

use serde::{Deserialize, Serialize};

use crate::shared::financial::accession_number::AccessionNumber;

pub mod filing_documents_error;

pub use filing_documents_error::{FilingDocumentRole, FilingDocumentsError};

/// The suffix of an XBRL instance extracted from an inline XBRL document.
const INLINE_INSTANCE_SUFFIX: &str = "_htm.xml";

/// The suffix of the label linkbase.
const LABEL_FILE_SUFFIX: &str = "_lab.xml";

/// The suffixes of the four linkbases, which are never the instance.
const LINKBASE_SUFFIXES: [&str; 4] = ["_cal.xml", "_def.xml", LABEL_FILE_SUFFIX, "_pre.xml"];

/// The member that the SEC adds to each folder with XBRL. It is not the instance.
const FILING_SUMMARY: &str = "FilingSummary.xml";

/// The file names of the XBRL instance and the label file of one filing.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize)]
pub struct FilingDocuments {
    instance: String,
    label_file: Option<String>,
}

impl FilingDocuments {
    /// Creates a new [`FilingDocuments`] from the file names of the instance and the label file.
    #[must_use]
    pub fn new(instance: impl Into<String>, label_file: Option<String>) -> Self {
        Self {
            instance: instance.into(),
            label_file,
        }
    }

    /// Picks the instance and the label file from the folder index JSON of one filing.
    ///
    /// - The instance is the one member that ends in `_htm.xml`. If none exists, it is the one
    ///   `.xml` member that is not `FilingSummary.xml` and is not a linkbase.
    /// - The label file is the one member that ends in `_lab.xml`, or `None`.
    ///
    /// The rules compare ASCII letters without case.
    ///
    /// # Errors
    ///
    /// Returns a [`FilingDocumentsError`] that names `accession_number`:
    /// - [`FilingDocumentsError::InvalidFolderIndex`] if the JSON has no `directory.item` list.
    /// - [`FilingDocumentsError::MissingXbrlInstance`] if no member matches a rule for the
    ///   instance.
    /// - [`FilingDocumentsError::AmbiguousFilingDocument`] if more than one member matches the
    ///   rule that decides a document.
    ///
    /// # Examples
    ///
    /// ```
    /// use sec::shared::filing_documents::FilingDocuments;
    /// use sec::shared::financial::accession_number::AccessionNumber;
    ///
    /// let accession_number = AccessionNumber::new("0000003545-26-000047");
    /// let folder_index = br#"{"directory": {"item": [
    ///     {"name": "alco-20260908_htm.xml"}, {"name": "alco-20260908_lab.xml"}
    /// ]}}"#;
    /// let documents = FilingDocuments::select(&accession_number, folder_index)
    ///     .expect("Given a folder with one inline instance, the selection should always succeed");
    ///
    /// let expected_result = "alco-20260908_htm.xml";
    ///
    /// let result = documents.instance();
    ///
    /// assert_eq!(result, expected_result);
    /// ```
    pub fn select(
        accession_number: &AccessionNumber,
        folder_index: &[u8],
    ) -> Result<Self, FilingDocumentsError> {
        let folder_index: FolderIndex = serde_json::from_slice(folder_index).map_err(|e| {
            FilingDocumentsError::InvalidFolderIndex {
                accession_number: accession_number.clone(),
                reason: e.to_string(),
            }
        })?;
        let names: Vec<&str> = folder_index
            .directory
            .item
            .iter()
            .map(|member| member.name.as_str())
            .collect();

        let instance = match single_match(&names, is_inline_instance) {
            Ok(None) => single_match(&names, is_plain_instance),
            inline_instance => inline_instance,
        }
        .map_err(|candidate_count| {
            ambiguous(
                accession_number,
                FilingDocumentRole::XbrlInstance,
                candidate_count,
            )
        })?
        .ok_or_else(|| FilingDocumentsError::MissingXbrlInstance {
            accession_number: accession_number.clone(),
        })?;
        let label_file = single_match(&names, is_label_file).map_err(|candidate_count| {
            ambiguous(
                accession_number,
                FilingDocumentRole::LabelFile,
                candidate_count,
            )
        })?;

        Ok(Self::new(instance, label_file))
    }

    /// Returns the file name of the XBRL instance.
    #[must_use]
    pub fn instance(&self) -> &str {
        &self.instance
    }

    /// Returns the file name of the label file, or `None` if the folder has none.
    #[must_use]
    pub fn label_file(&self) -> Option<&str> {
        self.label_file.as_deref()
    }
}

/// The part of the folder index JSON that the selection reads.
#[derive(Deserialize)]
struct FolderIndex {
    directory: Directory,
}

/// The `directory` object of the folder index.
#[derive(Deserialize)]
struct Directory {
    item: Vec<Member>,
}

/// One member of the folder. The index also holds its type, size, and date, which the selection
/// does not read.
#[derive(Deserialize)]
struct Member {
    name: String,
}

fn is_inline_instance(name: &str) -> bool {
    ends_with_ignoring_case(name, INLINE_INSTANCE_SUFFIX)
}

fn is_plain_instance(name: &str) -> bool {
    ends_with_ignoring_case(name, ".xml")
        && !name.eq_ignore_ascii_case(FILING_SUMMARY)
        && !LINKBASE_SUFFIXES
            .iter()
            .any(|suffix| ends_with_ignoring_case(name, suffix))
}

fn is_label_file(name: &str) -> bool {
    ends_with_ignoring_case(name, LABEL_FILE_SUFFIX)
}

/// Returns `true` if `name` ends in `suffix`, with ASCII letters compared without case.
fn ends_with_ignoring_case(name: &str, suffix: &str) -> bool {
    name.len() >= suffix.len()
        && name.as_bytes()[name.len() - suffix.len()..].eq_ignore_ascii_case(suffix.as_bytes())
}

/// Returns the one name that matches `rule`, `None` if no name matches, or the number of matches
/// if more than one name matches.
fn single_match(names: &[&str], rule: impl Fn(&str) -> bool) -> Result<Option<String>, usize> {
    let candidates: Vec<&str> = names.iter().copied().filter(|name| rule(name)).collect();
    match candidates.as_slice() {
        [] => Ok(None),
        [name] => Ok(Some((*name).to_string())),
        _ => Err(candidates.len()),
    }
}

fn ambiguous(
    accession_number: &AccessionNumber,
    role: FilingDocumentRole,
    candidate_count: usize,
) -> FilingDocumentsError {
    FilingDocumentsError::AmbiguousFilingDocument {
        accession_number: accession_number.clone(),
        role,
        candidate_count,
    }
}

#[cfg(test)]
mod tests {
    use pretty_assertions::assert_eq;

    use super::*;
    use crate::tests::fixtures::data::filing_index::{
        INLINE_XBRL_FOLDER_INDEX, PLAIN_XBRL_FOLDER_INDEX,
    };

    /// Returns a folder index JSON whose members have the given names.
    fn folder_index(names: &[&str]) -> Vec<u8> {
        let members: Vec<String> = names
            .iter()
            .map(|name| format!(r#"{{"name": "{name}", "type": "text.gif"}}"#))
            .collect();
        format!(r#"{{"directory": {{"item": [{}]}}}}"#, members.join(", ")).into_bytes()
    }

    #[test]
    fn should_pick_the_htm_xml_instance_and_the_label_file_for_inline_xbrl_filing() {
        let accession_number = AccessionNumber::new("0000003545-26-000047");

        let expected_result = Ok(FilingDocuments::new(
            "alco-20260908_htm.xml",
            Some("alco-20260908_lab.xml".to_string()),
        ));

        let result = FilingDocuments::select(&accession_number, INLINE_XBRL_FOLDER_INDEX);

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_pick_the_plain_xml_instance_when_the_folder_has_no_htm_xml_member() {
        let accession_number = AccessionNumber::new("0000320193-17-000070");

        let expected_result = Ok(FilingDocuments::new(
            "aapl-20170930.xml",
            Some("aapl-20170930_lab.xml".to_string()),
        ));

        let result = FilingDocuments::select(&accession_number, PLAIN_XBRL_FOLDER_INDEX);

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_prefer_the_htm_xml_instance_when_the_folder_also_has_a_plain_xml_member() {
        let accession_number = AccessionNumber::new("0000003545-26-000047");
        let index = folder_index(&["alco-20260908.xml", "alco-20260908_htm.xml"]);

        let expected_result = Ok(FilingDocuments::new("alco-20260908_htm.xml", None));

        let result = FilingDocuments::select(&accession_number, &index);

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_pick_the_plain_xml_instance_when_the_member_names_are_upper_case() {
        let accession_number = AccessionNumber::new("0000320193-17-000070");
        let index = folder_index(&[
            "FILINGSUMMARY.XML",
            "AAPL-20170930.XML",
            "AAPL-20170930_LAB.XML",
        ]);

        let expected_result = Ok(FilingDocuments::new(
            "AAPL-20170930.XML",
            Some("AAPL-20170930_LAB.XML".to_string()),
        ));

        let result = FilingDocuments::select(&accession_number, &index);

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_return_no_label_file_when_the_folder_has_no_lab_xml_member() {
        let accession_number = AccessionNumber::new("0000003545-26-000047");
        let index = folder_index(&["alco-20260908_htm.xml", "alco-20260908_pre.xml"]);

        let expected_result = Ok(FilingDocuments::new("alco-20260908_htm.xml", None));

        let result = FilingDocuments::select(&accession_number, &index);

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_return_missing_xbrl_instance_when_the_folder_has_only_linkbases_and_the_summary() {
        let accession_number = AccessionNumber::new("0000003545-26-000047");
        let index = folder_index(&[
            "FilingSummary.xml",
            "alco-20260908_cal.xml",
            "alco-20260908_def.xml",
            "alco-20260908_lab.xml",
            "alco-20260908_pre.xml",
            "alco-20260908.htm",
        ]);

        let expected_result = Err(FilingDocumentsError::MissingXbrlInstance {
            accession_number: AccessionNumber::new("0000003545-26-000047"),
        });

        let result = FilingDocuments::select(&accession_number, &index);

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_return_ambiguous_filing_document_when_the_folder_has_two_plain_xml_members() {
        let accession_number = AccessionNumber::new("0000320193-17-000070");
        let index = folder_index(&["aapl-20170930.xml", "exhibit.xml"]);

        let expected_result = Err(FilingDocumentsError::AmbiguousFilingDocument {
            accession_number: AccessionNumber::new("0000320193-17-000070"),
            role: FilingDocumentRole::XbrlInstance,
            candidate_count: 2,
        });

        let result = FilingDocuments::select(&accession_number, &index);

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_return_ambiguous_filing_document_when_the_folder_has_two_htm_xml_members() {
        let accession_number = AccessionNumber::new("0000003545-26-000047");
        let index = folder_index(&["alco-20260908_htm.xml", "exhibit_htm.xml"]);

        let expected_result = Err(FilingDocumentsError::AmbiguousFilingDocument {
            accession_number: AccessionNumber::new("0000003545-26-000047"),
            role: FilingDocumentRole::XbrlInstance,
            candidate_count: 2,
        });

        let result = FilingDocuments::select(&accession_number, &index);

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_return_ambiguous_filing_document_when_the_folder_has_two_lab_xml_members() {
        let accession_number = AccessionNumber::new("0000003545-26-000047");
        let index = folder_index(&[
            "alco-20260908_htm.xml",
            "alco-20260908_lab.xml",
            "other_lab.xml",
        ]);

        let expected_result = Err(FilingDocumentsError::AmbiguousFilingDocument {
            accession_number: AccessionNumber::new("0000003545-26-000047"),
            role: FilingDocumentRole::LabelFile,
            candidate_count: 2,
        });

        let result = FilingDocuments::select(&accession_number, &index);

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_return_invalid_folder_index_when_the_json_has_no_directory() {
        let accession_number = AccessionNumber::new("0000003545-26-000047");
        let index = br#"{"item": []}"#;

        let result = FilingDocuments::select(&accession_number, index);

        // The reason text belongs to `serde_json`, so the test checks the variant and the accession
        // number.
        assert!(matches!(
            result,
            Err(FilingDocumentsError::InvalidFolderIndex { accession_number, .. })
                if accession_number.value() == "0000003545-26-000047"
        ));
    }
}
