//! # US GAAP 2026 Schema Integration Tests
//!
//! Reads the vendored element schema of the US GAAP 2026 release and counts its concepts.
//!
//! ## External Dependencies
//!
//! None. The schema lives in `taxonomy/us-gaap/2026/elts/`. The `generator` feature is on for every
//! test build.

use std::fs;
use std::path::PathBuf;

use pretty_assertions::assert_eq;
use xbrl::us_gaap::generator::schema::concepts;

const SCHEMA_FILE: &str = "taxonomy/us-gaap/2026/elts/us-gaap-2026.xsd";

#[test]
fn should_read_every_concept_for_us_gaap_2026_schema() {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(SCHEMA_FILE);
    let text = fs::read_to_string(&path)
        .expect("Given the vendored 2026 schema, the file should always be readable");

    let expected_result = 17_179;

    let result = concepts(&path, &text)
        .expect("Given the vendored FASB schema, the read should always succeed")
        .len();

    assert_eq!(result, expected_result);
}
