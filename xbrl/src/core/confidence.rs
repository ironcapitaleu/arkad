//! # Confidence
//!
//! Resolution confidence levels indicating how a value was obtained.

use std::fmt;

/// Indicates which resolution tier produced a value.
///
/// Higher tiers carry less certainty about the correctness of the resolved value.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, serde::Serialize)]
pub enum Confidence {
    /// Tier 1: Direct match on an exact XBRL concept name.
    Exact,
    /// Tier 2: Matched via a known synonym or alias.
    Synonym,
    /// Tier 3: Derived from other resolved values using an SFAC 6 identity.
    Derived,
    /// Tier 4: Computed by walking the FASB calculation linkbase tree.
    Computed,
}

impl fmt::Display for Confidence {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Exact => write!(f, "Exact"),
            Self::Synonym => write!(f, "Synonym"),
            Self::Derived => write!(f, "Derived"),
            Self::Computed => write!(f, "Computed"),
        }
    }
}
