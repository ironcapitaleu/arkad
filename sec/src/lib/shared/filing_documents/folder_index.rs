//! # Folder Index
//!
//! Provides [`FolderIndex`], the member names of the folder of one filing, parsed from its
//! `index.json`, and the rules that pick the XBRL instance and the label file from them.

use serde::{Deserialize, Serialize};

use crate::shared::filing_documents::{
    FilingDocumentRole, FilingDocuments, FilingDocumentsError, InvalidFolderIndex,
};
use crate::shared::financial::accession_number::AccessionNumber;
use crate::shared::response::SecResponse as SecResponseTrait;
use crate::shared::response::implementations::sec_response::SecResponse;

/// The suffix of an XBRL instance extracted from an inline XBRL document.
const INLINE_INSTANCE_SUFFIX: &str = "_htm.xml";

/// The suffix of the label linkbase.
const LABEL_FILE_SUFFIX: &str = "_lab.xml";

/// The suffixes of the four linkbases, which are never the instance.
const LINKBASE_SUFFIXES: [&str; 4] = ["_cal.xml", "_def.xml", LABEL_FILE_SUFFIX, "_pre.xml"];

/// The member that the SEC adds to each folder with XBRL. It is not the instance.
const FILING_SUMMARY: &str = "FilingSummary.xml";

/// The member names of the folder of one filing, in index order.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize)]
pub struct FolderIndex {
    member_names: Vec<String>,
}

impl FolderIndex {
    /// Creates a new [`FolderIndex`] from the member names of a folder.
    #[must_use]
    pub const fn new(member_names: Vec<String>) -> Self {
        Self { member_names }
    }

    /// Returns the member names, in index order.
    #[must_use]
    pub fn member_names(&self) -> &[String] {
        &self.member_names
    }

    /// Picks the XBRL instance and the label file of the folder.
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
    /// - [`FilingDocumentsError::MissingXbrlInstance`] if no member matches a rule for the
    ///   instance.
    /// - [`FilingDocumentsError::AmbiguousFilingDocument`] if more than one member matches the
    ///   rule that decides a document.
    ///
    /// # Examples
    ///
    /// ```
    /// use sec::shared::filing_documents::FolderIndex;
    /// use sec::shared::financial::accession_number::AccessionNumber;
    ///
    /// let folder_index = FolderIndex::new(vec![
    ///     "alco-20260908_htm.xml".to_string(),
    ///     "alco-20260908_lab.xml".to_string(),
    /// ]);
    /// let accession_number = AccessionNumber::new("0000003545-26-000047");
    /// let documents = folder_index
    ///     .select_documents(&accession_number)
    ///     .expect("Given a folder with one inline instance, the selection should always succeed");
    ///
    /// let expected_result = "alco-20260908_htm.xml";
    ///
    /// let result = documents.instance();
    ///
    /// assert_eq!(result, expected_result);
    /// ```
    pub fn select_documents(
        &self,
        accession_number: &AccessionNumber,
    ) -> Result<FilingDocuments, FilingDocumentsError> {
        let instance = match self.single_match(is_inline_instance) {
            Ok(None) => self.single_match(is_plain_instance),
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
        let label_file = self
            .single_match(is_label_file)
            .map_err(|candidate_count| {
                ambiguous(
                    accession_number,
                    FilingDocumentRole::LabelFile,
                    candidate_count,
                )
            })?;

        Ok(FilingDocuments::new(instance, label_file))
    }

    /// Returns the one name that matches `rule`, `None` if no name matches, or the number of
    /// matches if more than one name matches.
    fn single_match(&self, rule: impl Fn(&str) -> bool) -> Result<Option<String>, usize> {
        let candidates: Vec<&String> = self.member_names.iter().filter(|name| rule(name)).collect();
        match candidates.as_slice() {
            [] => Ok(None),
            [name] => Ok(Some((*name).clone())),
            _ => Err(candidates.len()),
        }
    }
}

impl TryFrom<&SecResponse> for FolderIndex {
    type Error = InvalidFolderIndex;

    /// Parses the body of the response to a
    /// [`FilingIndex`](crate::shared::request::implementations::sec_request::FilingIndex) request
    /// and keeps the `name` of each member of `directory.item`.
    ///
    /// # Errors
    ///
    /// Returns an [`InvalidFolderIndex`] that names the URL of the response if the body is not
    /// JSON with a `directory.item` list of members with a `name`.
    fn try_from(response: &SecResponse) -> Result<Self, Self::Error> {
        let raw: RawFolderIndex =
            serde_json::from_slice(response.raw_body()).map_err(|e| InvalidFolderIndex {
                url: response.url().to_string(),
                reason: e.to_string(),
            })?;
        Ok(Self::new(
            raw.directory
                .item
                .into_iter()
                .map(|member| member.name)
                .collect(),
        ))
    }
}

/// The part of the `index.json` that [`FolderIndex`] reads.
#[derive(Deserialize)]
struct RawFolderIndex {
    directory: RawDirectory,
}

/// The `directory` object of the `index.json`.
#[derive(Deserialize)]
struct RawDirectory {
    item: Vec<RawMember>,
}

/// One member of the folder. The index also holds its type, size, and date, which
/// [`FolderIndex`] does not read.
#[derive(Deserialize)]
struct RawMember {
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
    use std::collections::HashMap;

    use pretty_assertions::assert_eq;

    use super::*;
    use crate::shared::content_type::ContentType;
    use crate::shared::headers::Headers;
    use crate::shared::status_code::StatusCode;
    use crate::tests::fixtures::data::filing_index::{
        INLINE_XBRL_FOLDER_INDEX, PLAIN_XBRL_FOLDER_INDEX,
    };

    /// Returns a folder index whose members have the given names.
    fn folder_index(names: &[&str]) -> FolderIndex {
        FolderIndex::new(names.iter().map(|name| (*name).to_string()).collect())
    }

    const FOLDER_INDEX_URL: &str =
        "https://www.sec.gov/Archives/edgar/data/3545/000000354526000047/index.json";

    /// Returns a `200 OK` JSON response with `body`, as the SEC sends a folder index.
    fn response(body: &[u8]) -> SecResponse {
        SecResponse::from_parts(
            FOLDER_INDEX_URL
                .parse()
                .expect("Given a hardcoded valid URL, parsing should always succeed"),
            Headers::new(HashMap::new()),
            ContentType::Json,
            StatusCode::Ok,
            body.to_vec(),
        )
        .expect("Given a valid JSON body, building the response should always succeed")
    }

    fn parsed(json: &[u8]) -> FolderIndex {
        FolderIndex::try_from(&response(json))
            .expect("Given a folder index downloaded from the SEC, parsing should always succeed")
    }

    #[test]
    fn should_pick_the_htm_xml_instance_and_the_label_file_for_inline_xbrl_filing() {
        let accession_number = AccessionNumber::new("0000003545-26-000047");

        let expected_result = Ok(FilingDocuments::new(
            "alco-20260908_htm.xml",
            Some("alco-20260908_lab.xml".to_string()),
        ));

        let result = parsed(INLINE_XBRL_FOLDER_INDEX).select_documents(&accession_number);

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_pick_the_plain_xml_instance_when_the_folder_has_no_htm_xml_member() {
        let accession_number = AccessionNumber::new("0000320193-17-000070");

        let expected_result = Ok(FilingDocuments::new(
            "aapl-20170930.xml",
            Some("aapl-20170930_lab.xml".to_string()),
        ));

        let result = parsed(PLAIN_XBRL_FOLDER_INDEX).select_documents(&accession_number);

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_prefer_the_htm_xml_instance_when_the_folder_also_has_a_plain_xml_member() {
        let accession_number = AccessionNumber::new("0000003545-26-000047");
        let index = folder_index(&["alco-20260908.xml", "alco-20260908_htm.xml"]);

        let expected_result = Ok(FilingDocuments::new("alco-20260908_htm.xml", None));

        let result = index.select_documents(&accession_number);

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

        let result = index.select_documents(&accession_number);

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_return_no_label_file_when_the_folder_has_no_lab_xml_member() {
        let accession_number = AccessionNumber::new("0000003545-26-000047");
        let index = folder_index(&["alco-20260908_htm.xml", "alco-20260908_pre.xml"]);

        let expected_result = Ok(FilingDocuments::new("alco-20260908_htm.xml", None));

        let result = index.select_documents(&accession_number);

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

        let result = index.select_documents(&accession_number);

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

        let result = index.select_documents(&accession_number);

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

        let result = index.select_documents(&accession_number);

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

        let result = index.select_documents(&accession_number);

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_read_the_member_names_in_index_order_for_a_folder_index() {
        let filing_index_response = response(
            br#"{"directory": {"item": [{"name": "a_htm.xml", "size": "1"}, {"name": "b_lab.xml"}], "name": "/x"}}"#,
        );

        let expected_result = Ok(FolderIndex::new(vec![
            "a_htm.xml".to_string(),
            "b_lab.xml".to_string(),
        ]));

        let result = FolderIndex::try_from(&filing_index_response);

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_name_the_url_in_invalid_folder_index_when_the_json_has_no_directory() {
        let filing_index_response = response(br#"{"item": []}"#);

        let result = FolderIndex::try_from(&filing_index_response);

        // The reason text belongs to `serde_json`, so the test checks the variant and the URL.
        assert!(matches!(result, Err(InvalidFolderIndex { url, .. }) if url == FOLDER_INDEX_URL));
    }
}
