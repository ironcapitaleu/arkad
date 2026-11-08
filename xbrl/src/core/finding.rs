//! # Finding
//!
//! Provides [`Finding`], a fact about the data that a step reports without failing the run.

use super::concept::NamespaceFamily;
use super::elements::CanonicalElement;
use super::fact::FactKey;
use super::filing::{EntityId, FilingId};
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
    /// At least one copy rank of a value key disagrees with rank 0, so the copies are separate
    /// facts. `core/dedup/more-precise@1` keeps every rank and reports the conflict.
    Conflict {
        /// The entity the facts are about.
        entity: EntityId,
        /// The value key of rank 0. It names the filing, the as-is concept, and the period.
        key: FactKey,
        /// The ranks that disagree with rank 0, in ascending order.
        ranks: Vec<u16>,
        /// The standard concept of the as-is concept, if a plug-in maps it.
        standard_concept: Option<CanonicalElement>,
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
