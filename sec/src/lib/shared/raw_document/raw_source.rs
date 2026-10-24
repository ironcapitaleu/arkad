//! # Raw Source
//!
//! Provides [`RawSource`], the trait that names the SEC file a
//! [`RawDocument`](super::RawDocument) holds, and one marker type per file.

use std::fmt::Debug;
use std::hash::Hash;

use crate::shared::document_kind::SecDocumentKind;

/// Names the SEC file that a [`RawDocument`](super::RawDocument) holds.
///
/// Each SEC file is a marker type that implements this trait. Its [`RawSource::KIND`] becomes the
/// document's kind, so a repository of one file never reads or writes another.
pub trait RawSource: Debug + Clone + Copy + PartialEq + Eq + PartialOrd + Ord + Hash {
    /// The kind of document this file is.
    const KIND: SecDocumentKind;

    /// The version of the raw document's context for this file.
    const KIND_VERSION: u32;
}

/// Marks the Company Facts JSON of one company.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct CompanyFacts;

impl RawSource for CompanyFacts {
    const KIND: SecDocumentKind = SecDocumentKind::RawCompanyFacts;
    const KIND_VERSION: u32 = 1;
}

#[cfg(test)]
mod tests {
    use pretty_assertions::assert_eq;

    use super::*;

    const fn implements_raw_source<T: RawSource>() {}
    #[test]
    const fn should_implement_raw_source_for_company_facts() {
        implements_raw_source::<CompanyFacts>();
    }

    #[test]
    fn should_name_the_raw_company_facts_kind_for_company_facts() {
        let expected_result = SecDocumentKind::RawCompanyFacts;

        let result = CompanyFacts::KIND;

        assert_eq!(result, expected_result);
    }
}
