//! # Fixture Data
//!
//! Real SEC files of the 10-Q `0001628280-26-068246` of FEDEX FREIGHT HOLDING COMPANY, INC., in
//! both sources, compiled in with `include_str!`, and helpers that parse them.
//!
//! ## Modules
//!
//! - [`notes`]: The rows of the filing in the Notes file `2026_10_notes.zip`.
//! - [`instance`]: The instance document of the filing, as EDGAR serves it.

use super::sample_taxonomy::{FakeTaxonomy, declaration, plugin};

pub mod instance;
pub mod notes;

/// Returns a plug-in that declares `us-gaap`, `dei`, `srt`, and `ecd`, the families of the fixture
/// filing.
#[must_use]
pub fn fixture_taxonomy() -> FakeTaxonomy {
    plugin(
        "us-gaap/2026",
        vec![
            declaration("us-gaap", "http://fasb.org/us-gaap/{release}", true),
            declaration("dei", "http://xbrl.sec.gov/dei/{release}", true),
            declaration("srt", "http://fasb.org/srt/{release}", true),
            declaration("ecd", "http://xbrl.sec.gov/ecd/{release}", true),
        ],
    )
}
