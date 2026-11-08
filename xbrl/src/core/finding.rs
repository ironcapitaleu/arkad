//! # Finding
//!
//! Provides [`Finding`], a fact about the data that a step reports without failing the run.

use super::concept::NamespaceFamily;
use super::filing::FilingId;
use super::invalid_value::InvalidValue;

/// A fact about the data that a step reports without failing the run.
///
/// A parser returns findings next to its facts. Later steps add their own variants.
#[non_exhaustive]
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Finding {
    /// The filing holds facts of a namespace family that no plug-in declares. The parser keeps the
    /// facts with the full Notes `version` or URI as family and version. Such facts do not join
    /// between sources or across releases until a plug-in declares the family.
    UndeclaredFamily {
        /// The filing.
        filing: FilingId,
        /// The family, for example `xyz/2027`.
        family: NamespaceFamily,
    },
    /// The parser could not map a row of the source into a fact, so it dropped the row.
    UnmappedRow {
        /// The filing.
        filing: FilingId,
        /// The table and the primary key of the row.
        row: String,
        /// The rule the row breaks.
        reason: InvalidValue,
    },
}

#[cfg(test)]
mod tests {
    use super::*;

    const fn implements_auto_traits<T: Sized + Send + Sync + Unpin>() {}
    #[test]
    const fn should_implement_auto_traits_for_finding() {
        implements_auto_traits::<Finding>();
    }
}
