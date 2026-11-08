//! # Instance Fixture
//!
//! The instance document `fdxf-20260930_htm.xml` of the 10-Q `0001628280-26-068246`, as EDGAR
//! serves it, and a helper that parses it.

use chrono::NaiveDate;

use super::fixture_taxonomy;
use crate::adapter::edgar::instance::{InstanceFacts, InstanceFiling, parse};
use crate::core::filing::FilingId;
use crate::core::taxonomy::registry::Registry;

/// The instance document of the fixture filing.
pub const INSTANCE_DOCUMENT: &str = include_str!("0001628280-26-068246/fdxf-20260930_htm.xml");

/// Returns the ID of the fixture filing.
///
/// # Panics
///
/// Never panics: the source and the ID are valid.
#[must_use]
pub fn filing_id() -> FilingId {
    FilingId::new("sec", "0001628280-26-068246")
        .expect("Given a hardcoded source and ID, the creation should always succeed")
}

/// Returns the filed date of the fixture filing in the daily index.
///
/// # Panics
///
/// Never panics: the date is valid.
#[must_use]
pub const fn filed() -> NaiveDate {
    NaiveDate::from_ymd_opt(2026, 10, 29)
        .expect("Given a hardcoded valid date, the creation should always succeed")
}

/// Parses [`INSTANCE_DOCUMENT`] with a registry of [`fixture_taxonomy`].
///
/// # Panics
///
/// Never panics: the document is a valid instance with a document type.
#[must_use]
pub fn parse_instance_fixture() -> InstanceFacts {
    let plugin = fixture_taxonomy();
    let registry = Registry::new(vec![&plugin])
        .expect("Given one plug-in with four families, the creation should always succeed");
    let input = InstanceFiling {
        document: INSTANCE_DOCUMENT,
        id: filing_id(),
        filed: filed(),
    };
    parse(&input, &registry)
        .expect("Given the instance document of a real filing, the parse should always succeed")
}
