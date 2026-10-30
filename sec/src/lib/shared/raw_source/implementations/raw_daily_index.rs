//! # Raw Daily Index
//!
//! Provides [`RawDailyIndex`], the [`RawSource`] marker for the EDGAR daily index of one day.

use crate::shared::document_type::SecDocumentType;
use crate::shared::raw_source::traits::RawSource;

/// Marks the EDGAR daily index `form.YYYYMMDD.idx` of one day, as the SEC returns it.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct RawDailyIndex;

impl RawSource for RawDailyIndex {
    const DOCUMENT_TYPE: SecDocumentType = SecDocumentType::RawDailyIndex;
    const DOCUMENT_TYPE_VERSION: u32 = 1;
}

#[cfg(test)]
mod tests {
    use pretty_assertions::assert_eq;

    use super::*;

    const fn implements_raw_source<T: RawSource>() {}
    #[test]
    const fn should_implement_raw_source_for_raw_daily_index() {
        implements_raw_source::<RawDailyIndex>();
    }

    #[test]
    fn should_name_the_raw_daily_index_document_type_for_raw_daily_index() {
        let expected_result = SecDocumentType::RawDailyIndex;

        let result = RawDailyIndex::DOCUMENT_TYPE;

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_name_document_type_version_one_for_raw_daily_index() {
        let expected_result = 1;

        let result = RawDailyIndex::DOCUMENT_TYPE_VERSION;

        assert_eq!(result, expected_result);
    }
}
