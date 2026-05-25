//! # Fiscal Year
//!
//! Newtype wrapper for fiscal year values.

use std::fmt;

/// A fiscal year identifier.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, serde::Serialize)]
pub struct FiscalYear(u16);

impl FiscalYear {
    /// Creates a new [`FiscalYear`] from a raw year value.
    #[must_use]
    pub const fn new(year: u16) -> Self {
        Self(year)
    }

    /// Returns the underlying year value.
    #[must_use]
    pub const fn value(self) -> u16 {
        self.0
    }
}

impl From<u16> for FiscalYear {
    fn from(year: u16) -> Self {
        Self(year)
    }
}

impl fmt::Display for FiscalYear {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}
