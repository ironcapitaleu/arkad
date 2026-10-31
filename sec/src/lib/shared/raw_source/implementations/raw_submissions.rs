//! # Raw Submissions
//!
//! Provides [`RawSubmissions`], the [`RawSource`] marker for the `submissions` JSON of one company.

use crate::shared::document_type::SecDocumentType;
use crate::shared::raw_source::traits::RawSource;

/// Marks the `submissions` JSON of one company, as the SEC returns it.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct RawSubmissions;

impl RawSource for RawSubmissions {
    const DOCUMENT_TYPE: SecDocumentType = SecDocumentType::RawSubmissions;
    const DOCUMENT_TYPE_VERSION: u32 = 1;
}

#[cfg(test)]
mod tests {
    use pretty_assertions::assert_eq;

    use super::*;

    const fn implements_raw_source<T: RawSource>() {}
    #[test]
    const fn should_implement_raw_source_for_raw_submissions() {
        implements_raw_source::<RawSubmissions>();
    }

    #[test]
    fn should_name_the_raw_submissions_document_type_for_raw_submissions() {
        let expected_result = SecDocumentType::RawSubmissions;

        let result = RawSubmissions::DOCUMENT_TYPE;

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_name_document_type_version_one_for_raw_submissions() {
        let expected_result = 1;

        let result = RawSubmissions::DOCUMENT_TYPE_VERSION;

        assert_eq!(result, expected_result);
    }
}
