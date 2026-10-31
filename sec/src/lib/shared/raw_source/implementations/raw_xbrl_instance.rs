//! # Raw XBRL Instance
//!
//! Provides [`RawXbrlInstance`], the [`RawSource`] marker for the XBRL instance of one filing.

use crate::shared::document_type::SecDocumentType;
use crate::shared::raw_source::traits::{RawFileSource, RawSource};

/// Marks the XBRL instance of one filing, as the SEC returns it. The instance holds the values of
/// the filing.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct RawXbrlInstance;

impl RawSource for RawXbrlInstance {
    const DOCUMENT_TYPE: SecDocumentType = SecDocumentType::RawXbrlInstance;
    const DOCUMENT_TYPE_VERSION: u32 = 1;
}

impl RawFileSource for RawXbrlInstance {}

#[cfg(test)]
mod tests {
    use pretty_assertions::assert_eq;

    use super::*;

    const fn implements_raw_file_source<T: RawFileSource>() {}
    #[test]
    const fn should_implement_raw_file_source_for_raw_xbrl_instance() {
        implements_raw_file_source::<RawXbrlInstance>();
    }

    const fn implements_raw_source<T: RawSource>() {}
    #[test]
    const fn should_implement_raw_source_for_raw_xbrl_instance() {
        implements_raw_source::<RawXbrlInstance>();
    }

    #[test]
    fn should_name_the_raw_xbrl_instance_document_type_for_raw_xbrl_instance() {
        let expected_result = SecDocumentType::RawXbrlInstance;

        let result = RawXbrlInstance::DOCUMENT_TYPE;

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_name_document_type_version_one_for_raw_xbrl_instance() {
        let expected_result = 1;

        let result = RawXbrlInstance::DOCUMENT_TYPE_VERSION;

        assert_eq!(result, expected_result);
    }
}
