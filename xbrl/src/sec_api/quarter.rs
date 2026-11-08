//! # Quarter
//!
//! Calendar quarter representation (Q1–Q4).

use std::fmt;

/// A calendar quarter.
#[non_exhaustive]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, serde::Serialize)]
pub enum Quarter {
    /// First quarter (January–March).
    Q1,
    /// Second quarter (April–June).
    Q2,
    /// Third quarter (July–September).
    Q3,
    /// Fourth quarter (October–December).
    Q4,
}

impl fmt::Display for Quarter {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Q1 => write!(f, "Q1"),
            Self::Q2 => write!(f, "Q2"),
            Self::Q3 => write!(f, "Q3"),
            Self::Q4 => write!(f, "Q4"),
        }
    }
}
