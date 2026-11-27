//! # Instance Fixture
//!
//! The instance documents of the fixture filings, as EDGAR serves them, and a helper that parses
//! the compiled-in one, `fdxf-20260930_htm.xml` of the 10-Q `0001628280-26-068246`. The
//! directories of the five filings of the parser agreement set (AMD, Procter & Gamble, Walmart,
//! Disney, and Apple) hold `.xml.gz` files, which [`parse_instance_fixture_filing`] parses.

use chrono::NaiveDate;

use super::manifest::manifest;
use super::{fixture_content, fixture_taxonomy};
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

/// The prefix of the path of an instance fixture file in the manifest.
const INSTANCE_PREFIX: &str = "instance/";

/// Parses the compressed instance document of the fixture filing `accession`, filed on `filed`,
/// with a registry of [`fixture_taxonomy`].
///
/// # Panics
///
/// Panics if the manifest names no instance document for `accession`, if the document is not UTF-8,
/// or if it is not a valid instance with a document type.
#[must_use]
pub fn parse_instance_fixture_filing(accession: &str, filed: NaiveDate) -> InstanceFacts {
    let file = manifest()
        .into_iter()
        .find(|entry| entry.accession == accession && entry.file.starts_with(INSTANCE_PREFIX))
        .unwrap_or_else(|| {
            panic!("Given the manifest, it should name an instance for '{accession}'")
        })
        .file;
    let document = String::from_utf8(fixture_content(&file)).expect(
        "Given an instance document EDGAR serves in UTF-8, the decoding should always succeed",
    );
    let plugin = fixture_taxonomy();
    let registry = Registry::new(vec![&plugin])
        .expect("Given one plug-in with four families, the creation should always succeed");
    let input = InstanceFiling {
        document: &document,
        id: FilingId::new("sec", accession).expect(
            "Given a source and an accession of the manifest, the creation should always succeed",
        ),
        filed,
    };
    parse(&input, &registry)
        .expect("Given the instance document of a real filing, the parse should always succeed")
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
