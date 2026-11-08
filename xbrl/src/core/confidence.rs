//! # Confidence
//!
//! Resolution confidence levels indicating how a value was obtained.

use std::fmt;

/// Indicates which resolution tier produced a value.
///
/// Higher tiers carry less certainty about the correctness of the resolved value.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, serde::Serialize)]
pub enum Confidence {
    /// Tier 1: Resolved from the first concept of the alias list of a plug-in.
    Exact,
    /// Tier 2: Resolved from a later concept of the alias list, or, in a planned `ifrs` plug-in,
    /// from an ESEF anchor.
    Synonym,
    /// Tier 3: Derived from other standardized values by an SFAC 6 identity or a period identity.
    Derived,
    /// Tier 4: Computed as a sum along the calculation tree of a plug-in.
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
