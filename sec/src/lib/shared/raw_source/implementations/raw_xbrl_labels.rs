//! # Raw XBRL Labels
//!
//! Provides [`RawXbrlLabels`], the [`RawSource`] marker for the `_lab.xml` label file of one filing.

use crate::shared::document_type::SecDocumentType;
use crate::shared::raw_source::traits::{RawFileSource, RawSource};

/// Marks the `_lab.xml` label file of one filing, as the SEC returns it. The label file holds the
/// company labels of the concepts.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct RawXbrlLabels;

impl RawSource for RawXbrlLabels {
    const DOCUMENT_TYPE: SecDocumentType = SecDocumentType::RawXbrlLabels;
    const DOCUMENT_TYPE_VERSION: u32 = 1;
}

impl RawFileSource for RawXbrlLabels {}

#[cfg(test)]
mod tests {
    use pretty_assertions::assert_eq;

    use super::*;

    const fn implements_raw_file_source<T: RawFileSource>() {}
    #[test]
    const fn should_implement_raw_file_source_for_raw_xbrl_labels() {
        implements_raw_file_source::<RawXbrlLabels>();
    }

    const fn implements_raw_source<T: RawSource>() {}
    #[test]
    const fn should_implement_raw_source_for_raw_xbrl_labels() {
        implements_raw_source::<RawXbrlLabels>();
    }

    #[test]
    fn should_name_the_raw_xbrl_labels_document_type_for_raw_xbrl_labels() {
        let expected_result = SecDocumentType::RawXbrlLabels;

        let result = RawXbrlLabels::DOCUMENT_TYPE;

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_name_document_type_version_one_for_raw_xbrl_labels() {
        let expected_result = 1;

        let result = RawXbrlLabels::DOCUMENT_TYPE_VERSION;

        assert_eq!(result, expected_result);
    }
}
