//! # Raw Filing Index
//!
//! Provides [`RawFilingIndex`], the [`RawSource`] marker for the folder `index.json` of one
//! filing.

use crate::shared::document_type::SecDocumentType;
use crate::shared::raw_source::traits::RawSource;

/// Marks the folder `index.json` of one filing, as the SEC returns it. The file lists the files of
/// the folder, so a stored copy shows whether the folder holds an XBRL instance.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct RawFilingIndex;

impl RawSource for RawFilingIndex {
    const DOCUMENT_TYPE: SecDocumentType = SecDocumentType::RawFilingIndex;
    const DOCUMENT_TYPE_VERSION: u32 = 1;
}

#[cfg(test)]
mod tests {
    use pretty_assertions::assert_eq;

    use super::*;

    const fn implements_raw_source<T: RawSource>() {}
    #[test]
    const fn should_implement_raw_source_for_raw_filing_index() {
        implements_raw_source::<RawFilingIndex>();
    }

    #[test]
    fn should_name_the_raw_filing_index_document_type_for_raw_filing_index() {
        let expected_result = SecDocumentType::RawFilingIndex;

        let result = RawFilingIndex::DOCUMENT_TYPE;

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_name_document_type_version_one_for_raw_filing_index() {
        let expected_result = 1;

        let result = RawFilingIndex::DOCUMENT_TYPE_VERSION;

        assert_eq!(result, expected_result);
    }
}
