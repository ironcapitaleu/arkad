//! # Parser Comparison Integration Tests
//!
//! Runs [`xbrl::adapter::edgar::notes::parse`] and [`xbrl::adapter::edgar::instance::parse`] on the
//! real filings in both sources, and compares their as-is facts. The 10-Q `0001628280-26-068246`
//! compares every fact. The five filings of the parser agreement set (AMD, Procter & Gamble,
//! Walmart, Disney, and Apple) compare their numeric facts. Their text differs in two ways the
//! losses below do not cover:
//!
//! - Notes writes some accented letters as the letter without the accent, as in `Comision` for
//!   `Comisión` in a Walmart text block. The comparison drops non-ASCII characters on the instance
//!   side, so it gives `Comisin`, and the text differs.
//! - The Procter & Gamble 10-K reports cybersecurity facts of the `cyd` family, which the fake
//!   registry does not declare. Each parser keeps another family and version for them, so their
//!   value keys differ.
//!
//! The Notes data set loses some content on the way from the instance. The comparison allows the
//! losses below. "readme" is `readme.htm` in each Notes ZIP. "Measured Result" is the section of
//! `sec/design/pipeline/ingestion_paths.md`.
//!
//! - Notes rounds a number to 4 decimals (readme and "Measured Result").
//! - Notes drops non-ASCII characters ("Measured Result").
//! - Notes collapses whitespace and removes the markup of a text block (readme).
//! - Notes cuts a footnote at 512 characters (readme), and drops a footnote that no statement page
//!   shows ("Measured Result").
//! - Notes writes a duration whose start equals its end with `qtrs` 0 and `durp` 0, as an instant.
//!
//! Any other difference is a difference in parser logic, and fails the test. The Notes parser
//! computes the exact start of every other duration from `durp`, so the comparison checks it.
//!
//! ## External Dependencies
//!
//! None. The fixtures live in `src/tests/fixtures/data/`.

use std::collections::HashMap;

use bigdecimal::RoundingMode;
use pretty_assertions::assert_eq;
use xbrl::core::fact::{AsIsFact, FactKey, FactValue};
use xbrl::core::period::Period;
use xbrl::tests::fixtures::data::instance::{
    parse_instance_fixture, parse_instance_fixture_filing,
};
use xbrl::tests::fixtures::data::notes::{parse_notes_fixture, parse_notes_fixture_filing};

/// The decimals that Notes keeps of a number.
const NOTES_SCALE: i64 = 4;

/// The characters at which Notes cuts a footnote.
const NOTES_FOOTNOTE_CHARACTERS: usize = 512;

/// The start of a tag in a text block.
const TAG_START: char = '<';

/// The end of a tag in a text block.
const TAG_END: char = '>';

/// The start of a character reference in a text block, as in `&#8217;`.
const REFERENCE_START: char = '&';

/// The end of a character reference in a text block.
const REFERENCE_END: char = ';';

/// The character before the number of a numeric character reference, as in `&#8217;`.
const REFERENCE_NUMBER_SIGN: char = '#';

/// The most characters between `&` and `;` of a character reference that the comparison skips.
const REFERENCE_MAX_CHARACTERS: usize = 10;

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

/// Returns the ASCII letters and digits of the text, without its tags and character references.
///
/// Notes removes the markup of a text block, and the spaces between tags vary. The letters and
/// digits stay, so they show whether both sides hold the same content. A `<` with no `>` after it
/// starts no tag, so the letters after it stay.
fn letters_and_digits(text: &str) -> String {
    let mut result = String::with_capacity(text.len());
    let mut rest = text;
    while let Some(character) = rest.chars().next() {
        rest = &rest[character.len_utf8()..];
        match character {
            TAG_START => rest = rest.split_once(TAG_END).map_or(rest, |(_, after)| after),
            REFERENCE_START => rest = after_reference(rest),
            _ if character.is_ascii_alphanumeric() => result.push(character),
            _ => {}
        }
    }
    result
}

/// Returns the text after a character reference, or the whole text if it starts with none. A bare
/// `&`, as in `R&D`, starts no reference, so the letters after it stay.
fn after_reference(text: &str) -> &str {
    match text.split_once(REFERENCE_END) {
        Some((name, after))
            if (1..=REFERENCE_MAX_CHARACTERS).contains(&name.chars().count())
                && name.chars().all(|character| {
                    character == REFERENCE_NUMBER_SIGN || character.is_ascii_alphanumeric()
                }) =>
        {
            after
        }
        _ => text,
    }
}

/// Returns `true` if the instance value equals the Notes value after the Notes losses.
fn same_value(notes: &FactValue, instance: &FactValue) -> bool {
    match (notes, instance) {
        (FactValue::Numeric(notes), FactValue::Numeric(instance)) => {
            *notes == instance.with_scale_round(NOTES_SCALE, RoundingMode::HalfUp)
        }
        (FactValue::Text(notes), FactValue::Text(instance)) if instance.contains(TAG_START) => {
            letters_and_digits(notes) == letters_and_digits(instance)
        }
        (FactValue::Text(notes), FactValue::Text(instance)) => {
            as_notes_text(notes) == as_notes_text(instance)
        }
        (notes, instance) => notes == instance,
    }
}

/// Returns `true` if the instance period equals the Notes period after the Notes losses.
fn same_period(notes: &Period, instance: &Period) -> bool {
    match (notes, instance) {
        (Period::Instant { end }, Period::Duration(duration)) => {
            duration.start() == *end && duration.end() == *end
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
            if notes.chars().count() >= NOTES_FOOTNOTE_CHARACTERS {
                instance.starts_with(notes.as_str())
            } else {
                as_notes_text(notes) == *instance
            }
        })
}

/// Returns `true` if the instance fact differs from the Notes fact after the Notes losses.
fn differs(notes: &AsIsFact, instance: &AsIsFact) -> bool {
    !same_value(&notes.value, &instance.value)
        || notes.decimals != instance.decimals
        || !same_period(&notes.context.period, &instance.context.period)
        || notes.context.entity != instance.context.entity
        || !same_footnotes(&notes.footnotes, &instance.footnotes)
}

#[test]
fn should_give_the_same_value_keys_from_both_parsers_for_a_real_filing() {
    let notes = parse_notes_fixture().facts;
    let instance = parse_instance_fixture().facts;

    // The Notes parser is the oracle here: 613 keys are too many to write out.
    let expected_result = sorted_keys(&notes);

    let result = sorted_keys(&instance);

    assert_eq!(result, expected_result);
}

#[test]
fn should_give_the_same_content_per_value_key_from_both_parsers_for_a_real_filing() {
    let notes = parse_notes_fixture().facts;
    let instance: HashMap<FactKey, AsIsFact> = parse_instance_fixture()
        .facts
        .into_iter()
        .map(|fact| (fact.key(), fact))
        .collect();

    let expected_result: Vec<String> = vec![];

    let result: Vec<String> = notes
        .iter()
        .filter_map(|notes| instance.get(&notes.key()).map(|instance| (notes, instance)))
        .filter(|(notes, instance)| differs(notes, instance))
        .map(|(notes, _)| format!("{:?}", notes.key()))
        .collect();

    assert_eq!(result, expected_result);
}

#[test]
fn should_give_every_notes_duration_the_exact_start_of_the_instance_for_a_real_filing() {
    let notes = parse_notes_fixture().facts;
    let instance: HashMap<FactKey, AsIsFact> = parse_instance_fixture()
        .facts
        .into_iter()
        .map(|fact| (fact.key(), fact))
        .collect();

    // 403 Notes durations, each with the start of the instance fact of its value key.
    let expected_result = (403, Vec::<String>::new());

    let durations: Vec<&AsIsFact> = notes
        .iter()
        .filter(|fact| matches!(fact.context.period, Period::Duration(_)))
        .collect();
    let differences: Vec<String> = durations
        .iter()
        .filter(|notes| {
            instance
                .get(&notes.key())
                .map(|fact| fact.context.period.start())
                != Some(notes.context.period.start())
        })
        .map(|notes| format!("{:?}", notes.key()))
        .collect();
    let result = (durations.len(), differences);

    assert_eq!(result, expected_result);
}

/// Returns the numeric as-is facts of the fixture filing `accession` from both parsers: Notes, then
/// instance. The instance parser takes the filed date of the `sub` row.
fn numeric_facts_of_both_sources(accession: &str) -> (Vec<AsIsFact>, Vec<AsIsFact>) {
    let is_numeric = |fact: &AsIsFact| matches!(fact.value, FactValue::Numeric(_));
    let notes = parse_notes_fixture_filing(accession);
    let instance = parse_instance_fixture_filing(accession, notes.filing.filed);
    (
        notes.facts.into_iter().filter(is_numeric).collect(),
        instance.facts.into_iter().filter(is_numeric).collect(),
    )
}

/// Returns the count of numeric Notes facts, the value keys that only one parser gives, and the
/// value keys whose content differs after the Notes losses.
fn numeric_agreement(
    notes: &[AsIsFact],
    instance: Vec<AsIsFact>,
) -> (usize, Vec<String>, Vec<String>) {
    let notes_keys = sorted_keys(notes);
    let instance_keys = sorted_keys(&instance);
    let only_one_parser = notes_keys
        .iter()
        .filter(|key| instance_keys.binary_search(key).is_err())
        .chain(
            instance_keys
                .iter()
                .filter(|key| notes_keys.binary_search(key).is_err()),
        )
        .map(|key| format!("{key:?}"))
        .collect();
    let instance: HashMap<FactKey, AsIsFact> = instance
        .into_iter()
        .map(|fact| (fact.key(), fact))
        .collect();
    let differences = notes
        .iter()
        .filter_map(|notes| instance.get(&notes.key()).map(|instance| (notes, instance)))
        .filter(|(notes, instance)| differs(notes, instance))
        .map(|(notes, _)| format!("{:?}", notes.key()))
        .collect();
    (notes.len(), only_one_parser, differences)
}

/// Writes one test per filing of the parser agreement set. Each checks that both parsers give
/// the same numeric value keys with the same content, and counts the numeric Notes facts.
macro_rules! parser_agreement_tests {
    ($($name:ident: $accession:literal => $count:literal,)+) => {
        $(
            #[test]
            fn $name() {
                let (notes, instance) = numeric_facts_of_both_sources($accession);

                let expected_result = ($count, Vec::<String>::new(), Vec::<String>::new());

                let result = numeric_agreement(&notes, instance);

                assert_eq!(result, expected_result);
            }
        )+
    };
}

parser_agreement_tests! {
    should_give_the_same_numeric_facts_from_both_parsers_for_amd_10_q_2026_q2: "0000002488-26-000123" => 784,
    should_give_the_same_numeric_facts_from_both_parsers_for_procter_and_gamble_10_k_2026_fy: "0000080424-26-000103" => 1614,
    should_give_the_same_numeric_facts_from_both_parsers_for_walmart_10_q_2027_q2: "0000104169-26-000154" => 705,
    should_give_the_same_numeric_facts_from_both_parsers_for_disney_10_q_2026_q3: "0001744489-26-000057" => 1433,
    should_give_the_same_numeric_facts_from_both_parsers_for_apple_10_q_2026_q3: "0000320193-26-000020" => 684,
}
