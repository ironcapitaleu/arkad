//! # Raw Company Facts
//!
//! Provides [`RawCompanyFacts`], the [`RawSource`] marker for the Company Facts JSON of one
//! company.

use crate::shared::document_type::SecDocumentType;
use crate::shared::raw_source::traits::RawSource;

/// Marks the Company Facts JSON of one company, as the SEC returns it.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct RawCompanyFacts;

impl RawSource for RawCompanyFacts {
    const DOCUMENT_TYPE: SecDocumentType = SecDocumentType::RawCompanyFacts;
    const DOCUMENT_TYPE_VERSION: u32 = 1;
}

#[cfg(test)]
mod tests {
    use pretty_assertions::assert_eq;

    use super::*;

    const fn implements_raw_source<T: RawSource>() {}
    #[test]
    const fn should_implement_raw_source_for_raw_company_facts() {
        implements_raw_source::<RawCompanyFacts>();
    }

    #[test]
    fn should_name_the_raw_company_facts_document_type_for_raw_company_facts() {
        let expected_result = SecDocumentType::RawCompanyFacts;

        let result = RawCompanyFacts::DOCUMENT_TYPE;

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_name_document_type_version_one_for_raw_company_facts() {
        let expected_result = 1;

        let result = RawCompanyFacts::DOCUMENT_TYPE_VERSION;

        assert_eq!(result, expected_result);
    }
}
