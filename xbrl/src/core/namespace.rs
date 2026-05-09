//! # Namespace
//!
//! XBRL taxonomy namespaces as they appear in the SEC JSON API.

use std::fmt;

/// The XBRL taxonomy namespace a concept belongs to.
#[non_exhaustive]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, serde::Serialize)]
pub enum Namespace {
    /// US Generally Accepted Accounting Principles.
    UsGaap,
    /// Document and Entity Information.
    Dei,
    /// SEC Reporting Taxonomy.
    Srt,
    /// Investment company taxonomy.
    Invest,
    /// Filing fee data.
    Ffd,
    /// Executive compensation data.
    Ecd,
}

impl Namespace {
    /// Parses a [`Namespace`] from the JSON key name used by the SEC API.
    ///
    /// Returns `None` if the input does not match any known namespace.
    #[must_use]
    pub fn from_sec_str(s: &str) -> Option<Self> {
        match s {
            "us-gaap" => Some(Self::UsGaap),
            "dei" => Some(Self::Dei),
            "srt" => Some(Self::Srt),
            "invest" => Some(Self::Invest),
            "ffd" => Some(Self::Ffd),
            "ecd" => Some(Self::Ecd),
            _ => None,
        }
    }
}

impl fmt::Display for Namespace {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::UsGaap => write!(f, "us-gaap"),
            Self::Dei => write!(f, "dei"),
            Self::Srt => write!(f, "srt"),
            Self::Invest => write!(f, "invest"),
            Self::Ffd => write!(f, "ffd"),
            Self::Ecd => write!(f, "ecd"),
        }
    }
}
