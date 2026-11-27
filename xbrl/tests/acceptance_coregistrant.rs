//! # Acceptance Test A12: Coregistrant Values
//!
//! Runs the rules of the standardized layer on the Dow 10-K for fiscal year 2025: the rules of one
//! filing with the US GAAP plug-in, `core/dedup/newest-filing@1`, `core/period/q4-from-fy@1`, and the
//! derivation from a closed group. The Notes rows of the filing hold 331 `coreg` values of The Dow
//! Chemical Company and Dow Inc. The parser puts each on the `LegalEntity` axis. With the values
//! that the filer tags on that axis itself, the filing holds 373 numeric values on it. Tiers 1 and 2
//! give none of them a candidate, so no standardized fact takes one as input.
//!
//! ## External Dependencies
//!
//! None. The fixtures live in `src/tests/fixtures/data/`.

use pretty_assertions::assert_eq;
use xbrl::core::fact::{AsIsFact, FactValue};
use xbrl::tests::fixtures::data::notes::parse_notes_fixture_filing;
use xbrl::tests::fixtures::data::resolved::{derived_values, resolve_fixture_filings};

/// The accession of the Dow 10-K for fiscal year 2025.
const DOW_10_K_2025: &str = "0001751788-26-000018";

/// The axis that holds the coregistrant of a value.
const LEGAL_ENTITY_AXIS: &str = "LegalEntity";

#[test]
fn should_take_no_coregistrant_value_into_a_standardized_fact_for_dow_10_k_2025_fy() {
    let legal_entity_values = parse_notes_fixture_filing(DOW_10_K_2025)
        .facts
        .iter()
        .filter(|fact| {
            matches!(fact.value, FactValue::Numeric(_))
                && fact
                    .context
                    .dimensions
                    .iter()
                    .any(|(axis, _)| axis == LEGAL_ENTITY_AXIS)
        })
        .map(AsIsFact::key)
        .collect::<Vec<_>>();
    let series = resolve_fixture_filings(&[DOW_10_K_2025]).series();
    let added = derived_values(&series);

    let expected_result = (373, 0);

    let result = (
        legal_entity_values.len(),
        series
            .facts
            .iter()
            .chain(&added)
            .flat_map(|fact| &fact.inputs)
            .filter(|input| legal_entity_values.contains(input))
            .count(),
    );

    assert_eq!(result, expected_result);
}
