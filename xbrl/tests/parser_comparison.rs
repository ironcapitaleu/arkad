//! # Parser Comparison Integration Tests
//!
//! Runs [`xbrl::adapter::edgar::notes::parse`] and [`xbrl::adapter::edgar::instance::parse`] on one
//! real filing in both sources, the 10-Q `0001628280-26-068246`, and compares their as-is facts.
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
//!
//! Any other difference is a difference in parser logic, and fails the test.
//!
//! ## External Dependencies
//!
//! None. The fixtures live in `tests/fixtures/notes/` and `tests/fixtures/instance/`.

use std::collections::HashMap;

use bigdecimal::RoundingMode;
use pretty_assertions::assert_eq;
use xbrl::core::fact::{AsIsFact, FactKey, FactValue};

use self::instance_fixture::parse_instance_fixture;
use self::notes_fixture::parse_notes_fixture;

#[path = "support/instance_fixture.rs"]
mod instance_fixture;
#[path = "support/notes_fixture.rs"]
mod notes_fixture;
#[path = "support/taxonomy.rs"]
mod taxonomy;

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
        .filter(|(notes, instance)| {
            !same_value(&notes.value, &instance.value)
                || notes.decimals != instance.decimals
                || notes.context.entity != instance.context.entity
                || !same_footnotes(&notes.footnotes, &instance.footnotes)
        })
        .map(|(notes, _)| format!("{:?}", notes.key()))
        .collect();

    assert_eq!(result, expected_result);
}
