//! # Parser Comparison Integration Tests
//!
//! Runs [`xbrl::adapter::edgar::notes::parse`] and [`xbrl::adapter::edgar::instance::parse`] on one
//! real filing in both sources, the 10-Q `0001628280-26-068246`, and compares their as-is facts.
//!
//! The Notes data set loses some content on the way from the instance. The comparison allows these
//! losses, as `sec/design/pipeline/ingestion_paths.md`, "Measured Result", lists them:
//!
//! - Notes rounds a number to 4 decimals.
//! - Notes drops non-ASCII characters and collapses whitespace.
//! - Notes removes the markup of a text block, so a text with markup counts only as a text.
//! - Notes cuts a footnote at 512 characters, and misses a footnote that no statement page shows.
//!
//! Any other difference is a difference in parser logic, and fails the test.
//!
//! ## External Dependencies
//!
//! None. The fixtures live in `tests/fixtures/notes/` and `tests/fixtures/instance/`.

use std::collections::HashMap;

use bigdecimal::RoundingMode;
use pretty_assertions::assert_eq;
use xbrl::adapter::edgar::notes::parse;
use xbrl::core::fact::{AsIsFact, FactKey, FactValue};
use xbrl::core::taxonomy::registry::Registry;

use self::instance_fixture::parse_fixture;
use self::notes_fixture::notes_filing;
use self::taxonomy::us_gaap;

#[path = "support/instance_fixture.rs"]
mod instance_fixture;
#[path = "support/notes_fixture.rs"]
mod notes_fixture;
#[path = "support/taxonomy.rs"]
mod taxonomy;

/// The decimals that Notes keeps of a number.
const NOTES_SCALE: i64 = 4;

/// The characters at which Notes cuts a footnote.
const NOTES_FOOTNOTE_LENGTH: usize = 512;

/// The start of markup in a text.
const MARKUP_START: char = '<';

fn notes_facts() -> Vec<AsIsFact> {
    let rows = notes_filing();
    let plugin = us_gaap();
    let registry = Registry::new(vec![&plugin])
        .expect("Given one plug-in with four families, the creation should always succeed");
    parse(&rows, &registry)
        .expect("Given the valid sub row of a real filing, the parse should always succeed")
        .facts
}

fn sorted_keys(facts: &[AsIsFact]) -> Vec<FactKey> {
    let mut keys: Vec<FactKey> = facts.iter().map(AsIsFact::key).collect();
    keys.sort();
    keys
}

/// Returns the text as Notes writes it: ASCII only, with whitespace collapsed.
fn as_notes_text(text: &str) -> String {
    let ascii: String = text.chars().filter(char::is_ascii).collect();
    ascii.split_whitespace().collect::<Vec<_>>().join(" ")
}

/// Returns `true` if the instance value equals the Notes value after the Notes losses.
fn same_value(notes: &FactValue, instance: &FactValue) -> bool {
    match (notes, instance) {
        (FactValue::Numeric(notes), FactValue::Numeric(instance)) => {
            *notes == instance.with_scale_round(NOTES_SCALE, RoundingMode::HalfUp)
        }
        (FactValue::Text(_), FactValue::Text(instance)) if instance.contains(MARKUP_START) => true,
        (FactValue::Text(notes), FactValue::Text(instance)) => {
            as_notes_text(notes) == as_notes_text(instance)
        }
        (notes, instance) => notes == instance,
    }
}

/// Returns `true` if the instance footnotes equal the Notes footnotes after the Notes losses.
fn same_footnotes(notes: &[String], instance: &[String]) -> bool {
    if notes.is_empty() {
        return true;
    }
    let instance: Vec<String> = instance.iter().map(|text| as_notes_text(text)).collect();
    notes.len() == instance.len()
        && notes.iter().zip(&instance).all(|(notes, instance)| {
            if notes.len() >= NOTES_FOOTNOTE_LENGTH {
                instance.starts_with(notes.as_str())
            } else {
                as_notes_text(notes) == *instance
            }
        })
}

#[test]
fn should_give_the_same_value_keys_from_both_parsers_for_a_real_filing() {
    let notes = notes_facts();
    let instance = parse_fixture().facts;

    let expected_result = sorted_keys(&notes);

    let result = sorted_keys(&instance);

    assert_eq!(result, expected_result);
}

#[test]
fn should_give_the_same_content_per_value_key_from_both_parsers_for_a_real_filing() {
    let notes = notes_facts();
    let instance: HashMap<FactKey, AsIsFact> = parse_fixture()
        .facts
        .into_iter()
        .map(|fact| (fact.key(), fact))
        .collect();

    let expected_result: Vec<String> = vec![];

    let result: Vec<String> = notes
        .iter()
        .filter_map(|notes| instance.get(&notes.key()).map(|instance| (notes, instance)))
        .filter(|(notes, instance)| {
            !same_value(&notes.value, &instance.value)
                || notes.decimals != instance.decimals
                || notes.context.entity != instance.context.entity
                || !same_footnotes(&notes.footnotes, &instance.footnotes)
        })
        .map(|(notes, _)| {
            let key = notes.key();
            format!(
                "{} {} rank {}",
                key.concept, notes.context.period, key.copy_rank
            )
        })
        .collect();

    assert_eq!(result, expected_result);
}
