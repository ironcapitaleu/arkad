//! # Raw Company Facts
//!
//! Provides [`RawCompanyFacts`], the [`RawSource`] marker for the Company Facts JSON of one
//! company.

use crate::shared::document_kind::SecDocumentKind;
use crate::shared::raw_source::traits::RawSource;

/// Marks the Company Facts JSON of one company, as the SEC returns it.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct RawCompanyFacts;

impl RawSource for RawCompanyFacts {
    const KIND: SecDocumentKind = SecDocumentKind::RawCompanyFacts;
    const KIND_VERSION: u32 = 1;
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
    fn should_name_the_raw_company_facts_kind_for_raw_company_facts() {
        let expected_result = SecDocumentKind::RawCompanyFacts;

        let result = RawCompanyFacts::KIND;

        assert_eq!(result, expected_result);
    }
}
