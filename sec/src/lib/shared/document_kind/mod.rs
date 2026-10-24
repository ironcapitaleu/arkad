//! # Document Kind
//!
//! Provides [`SecDocumentKind`], the kinds of document that the SEC state machines store.

use std::fmt::{self, Display, Formatter};

/// The kinds of document that the SEC state machines store.
///
/// Each variant's [`Display`] text is the `kind` the store writes into a document's metadata. The
/// text must stay unique across every crate that stores documents.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum SecDocumentKind {
    /// The Company Facts JSON exactly as the SEC returned it.
    RawCompanyFacts,
}

impl Display for SecDocumentKind {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        match self {
            Self::RawCompanyFacts => write!(f, "raw_company_facts"),
        }
    }
}

#[cfg(test)]
mod tests {
    use std::fmt::Debug;
    use std::hash::Hash;

    use pretty_assertions::assert_eq;

    use super::*;

    const fn implements_auto_traits<T: Sized + Send + Sync + Unpin>() {}
    #[test]
    const fn should_implement_auto_traits_for_sec_document_kind() {
        implements_auto_traits::<SecDocumentKind>();
    }

    const fn implements_value_traits<
        T: Debug + Clone + Copy + PartialEq + Eq + PartialOrd + Ord + Hash,
    >() {
    }
    #[test]
    const fn should_implement_value_traits_for_sec_document_kind() {
        implements_value_traits::<SecDocumentKind>();
    }

    #[test]
    fn should_display_raw_company_facts_for_the_raw_company_facts_kind() {
        let expected_result = "raw_company_facts";

        let result = SecDocumentKind::RawCompanyFacts.to_string();

        assert_eq!(result, expected_result);
    }
}
