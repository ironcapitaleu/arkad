//! # Raw Full Index
//!
//! Provides [`RawFullIndex`], the [`RawSource`] marker for the EDGAR full index of one quarter.

use crate::shared::document_type::SecDocumentType;
use crate::shared::raw_source::traits::{RawFileSource, RawSource};

/// Marks the EDGAR full index `form.idx` of one quarter, as the SEC returns it.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct RawFullIndex;

impl RawSource for RawFullIndex {
    const DOCUMENT_TYPE: SecDocumentType = SecDocumentType::RawFullIndex;
    const DOCUMENT_TYPE_VERSION: u32 = 1;
}

impl RawFileSource for RawFullIndex {}

#[cfg(test)]
mod tests {
    use pretty_assertions::assert_eq;

    use super::*;

    const fn implements_raw_file_source<T: RawFileSource>() {}
    #[test]
    const fn should_implement_raw_file_source_for_raw_full_index() {
        implements_raw_file_source::<RawFullIndex>();
    }

    const fn implements_raw_source<T: RawSource>() {}
    #[test]
    const fn should_implement_raw_source_for_raw_full_index() {
        implements_raw_source::<RawFullIndex>();
    }

    #[test]
    fn should_name_the_raw_full_index_document_type_for_raw_full_index() {
        let expected_result = SecDocumentType::RawFullIndex;

        let result = RawFullIndex::DOCUMENT_TYPE;

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_name_document_type_version_one_for_raw_full_index() {
        let expected_result = 1;

        let result = RawFullIndex::DOCUMENT_TYPE_VERSION;

        assert_eq!(result, expected_result);
    }
}
