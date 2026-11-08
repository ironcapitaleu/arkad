//! # US GAAP 2026 Label Integration Tests
//!
//! Reads the vendored label linkbase of the US GAAP 2026 release and counts its standard labels.
//!
//! ## External Dependencies
//!
//! None. The label linkbase lives in `xbrl/taxonomy/us-gaap/2026/elts/`.

use std::fs;
use std::path::PathBuf;

use pretty_assertions::assert_eq;
use us_gaap_generator::linkbase::standard_labels;

const LABEL_FILE: &str = "../taxonomy/us-gaap/2026/elts/us-gaap-lab-2026.xml";

#[test]
fn should_read_one_standard_label_per_concept_for_us_gaap_2026_labels() {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(LABEL_FILE);
    let text = fs::read_to_string(&path)
        .expect("Given the vendored 2026 label linkbase, the file should always be readable");

    let expected_result = 17_179;

    let result = standard_labels(&path, &text)
        .expect("Given the vendored FASB label linkbase, the read should always succeed")
        .len();

    assert_eq!(result, expected_result);
}
