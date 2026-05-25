//! # Form
//!
//! SEC filing form types.

use std::fmt;

/// The type of SEC filing a data point originates from.
#[non_exhaustive]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, serde::Serialize)]
pub enum Form {
    /// Annual report.
    TenK,
    /// Quarterly report.
    TenQ,
    /// Amended annual report.
    TenKA,
    /// Amended quarterly report.
    TenQA,
}

impl Form {
    /// Parses a [`Form`] from an SEC form string.
    ///
    /// Returns `None` if the input does not match any known form type.
    #[must_use]
    pub fn from_sec_str(s: &str) -> Option<Self> {
        match s {
            "10-K" => Some(Self::TenK),
            "10-Q" => Some(Self::TenQ),
            "10-K/A" => Some(Self::TenKA),
            "10-Q/A" => Some(Self::TenQA),
            _ => None,
        }
    }

    /// Returns `true` if this form is an amendment.
    #[must_use]
    pub const fn is_amendment(self) -> bool {
        matches!(self, Self::TenKA | Self::TenQA)
    }
}

impl fmt::Display for Form {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::TenK => write!(f, "10-K"),
            Self::TenQ => write!(f, "10-Q"),
            Self::TenKA => write!(f, "10-K/A"),
            Self::TenQA => write!(f, "10-Q/A"),
        }
    }
}
