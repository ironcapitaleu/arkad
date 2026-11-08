//! # Generated Tables Integration Tests
//!
//! Runs the generator on the vendored release and compares its output with the checked-in
//! `xbrl/src/us_gaap/taxonomy.rs`. A change to the vendored files or to the generator without a
//! new run of the generator fails the test.
//!
//! ## External Dependencies
//!
//! None. The files live in `xbrl/taxonomy/us-gaap/2026/`.

use std::fs;
use std::path::PathBuf;

use pretty_assertions::assert_eq;
use xbrl::us_gaap::generator::{OUTPUT_FILE, generate};

#[test]
fn should_match_checked_in_tables_for_vendored_release() {
    let xbrl_crate = PathBuf::from(env!("CARGO_MANIFEST_DIR"));

    let expected_result = fs::read_to_string(xbrl_crate.join(OUTPUT_FILE))
        .expect("Given the checked-in tables, the generated file should be readable");

    let result = generate(&xbrl_crate)
        .expect("Given the vendored release, the generator should always succeed");

    assert_eq!(result, expected_result);
}
