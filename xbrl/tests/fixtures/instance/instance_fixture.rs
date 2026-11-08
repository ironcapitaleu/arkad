//! # Instance Fixture
//!
//! Reads the instance document `fdxf-20260930_htm.xml` of the 10-Q `0001628280-26-068246` from
//! `tests/fixtures/instance/`, as EDGAR serves it, and parses it.

use std::fs;
use std::path::PathBuf;

use chrono::NaiveDate;
use xbrl::adapter::edgar::instance::{InstanceFacts, InstanceFiling, parse};
use xbrl::core::filing::FilingId;
use xbrl::core::taxonomy::registry::Registry;

use super::fake_taxonomy::us_gaap;

/// Returns the ID of the fixture filing.
pub fn filing_id() -> FilingId {
    FilingId::new("sec", "0001628280-26-068246")
        .expect("Given a hardcoded source and ID, the creation should always succeed")
}

/// Returns the filed date of the daily index.
pub const fn filed() -> NaiveDate {
    NaiveDate::from_ymd_opt(2026, 10, 29)
        .expect("Given a hardcoded valid date, the creation should always succeed")
}

/// Parses the fixture document.
pub fn parse_instance_fixture() -> InstanceFacts {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures/instance/0001628280-26-068246/fdxf-20260930_htm.xml");
    let document = fs::read_to_string(path)
        .expect("Given a fixture file in the repository, the read should always succeed");
    let plugin = us_gaap();
    let registry = Registry::new(vec![&plugin])
        .expect("Given one plug-in with four families, the creation should always succeed");
    let input = InstanceFiling {
        document: &document,
        id: filing_id(),
        filed: filed(),
    };
    parse(&input, &registry)
        .expect("Given the instance document of a real filing, the parse should always succeed")
}
