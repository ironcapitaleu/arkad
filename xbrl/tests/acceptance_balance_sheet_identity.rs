//! # Acceptance Test A3: Balance Sheet Identity
//!
//! Runs the checks of the roll-up tree on each compressed Notes filing of the fixture set alone,
//! after the rules of A1 (`acceptance_roll_ups.rs`), and lists the balance sheet identities that
//! fail: `Assets` differs from `LiabilitiesAndEquity` by more than 0.1%.
//!
//! No filing fails. The 10-K of General Mills for fiscal year 2013 reports no liabilities and
//! equity total, so the closed group `liabilities-and-equity` derives one. The derived total holds
//! the redeemable interest of the filer, which the part rule of `TemporaryEquity` resolves.
//!
//! ## External Dependencies
//!
//! None. The fixtures live in `src/tests/fixtures/data/`.

use std::collections::BTreeSet;

use jiff::civil::Date;
use pretty_assertions::assert_eq;
use xbrl::core::finding::{Finding, RollUpCheck};
use xbrl::core::roll_up_check::check_roll_ups;
use xbrl::tests::fixtures::data::GZIP_EXTENSION;
use xbrl::tests::fixtures::data::manifest::manifest;
use xbrl::tests::fixtures::data::resolved::{derived_values, resolve_fixture_filings};

/// The prefix of the path of a Notes fixture file in the manifest.
const NOTES_PREFIX: &str = "notes/";

/// Runs the rules of A1 and the checks of the roll-up tree on the fixture filing `accession`, and
/// returns the accession and the period end of each failed balance sheet identity.
fn balance_sheet_mismatches(accession: &str) -> Vec<(String, Date)> {
    let series = resolve_fixture_filings(&[accession]).series();
    let added = derived_values(&series);
    check_roll_ups(&series, &added)
        .into_iter()
        .filter_map(|finding| match finding {
            Finding::RollUpMismatch {
                period,
                reason: RollUpCheck::BalanceSheet,
                ..
            } => Some((accession.to_string(), period.end())),
            _ => None,
        })
        .collect()
}

#[test]
fn should_hold_the_balance_sheet_identity_for_each_fixture_filing() {
    let accessions: BTreeSet<String> = manifest()
        .into_iter()
        .filter(|entry| {
            entry.file.starts_with(NOTES_PREFIX) && entry.file.ends_with(GZIP_EXTENSION)
        })
        .map(|entry| entry.accession)
        .collect();

    let expected_result: Vec<(String, Date)> = Vec::new();

    let result: Vec<(String, Date)> = accessions
        .iter()
        .flat_map(|accession| balance_sheet_mismatches(accession))
        .collect();

    assert_eq!(result, expected_result);
}
