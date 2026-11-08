//! # Render
//!
//! Writes the Rust source of `us_gaap/taxonomy.rs` from the read tables.

use std::fmt::Write;

use crate::linkbase::CalculationArc;
use crate::schema::{Balance, PeriodType};

/// The text that the generated file writes for a concept without a balance.
const NO_BALANCE: &str = "None";

/// The text that the generated file writes for a debit balance.
const DEBIT_BALANCE: &str = "Some(Balance::Debit)";

/// The text that the generated file writes for a credit balance.
const CREDIT_BALANCE: &str = "Some(Balance::Credit)";

/// The text that the generated file writes for an instant period type.
const INSTANT_PERIOD_TYPE: &str = "PeriodType::Instant";

/// The text that the generated file writes for a duration period type.
const DURATION_PERIOD_TYPE: &str = "PeriodType::Duration";

/// One concept with its label, ready to render.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ElementLine {
    /// The local name.
    pub name: String,
    /// The standard label.
    pub label: String,
    /// The data type as the schema writes it.
    pub data_type: String,
    /// The balance, if the concept has one.
    pub balance: Option<Balance>,
    /// The period type.
    pub period_type: PeriodType,
}

/// The tables of one release, ready to render.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Tables {
    /// The release ID, for example `us-gaap/2026`.
    pub release_id: String,
    /// The folder of the vendored files, relative to the `xbrl` crate.
    pub release_folder: String,
    /// The SHA-256 of the `MANIFEST`, as 64 lowercase hex digits.
    pub checksum: String,
    /// The calculation arcs, sorted, without repeats.
    pub arcs: Vec<CalculationArc>,
    /// The elements, sorted by name.
    pub elements: Vec<ElementLine>,
}

/// Returns the Rust source of `us_gaap/taxonomy.rs` for `tables`.
#[must_use]
pub fn render(tables: &Tables) -> String {
    let mut source = String::new();
    write_header(&mut source, tables);
    write_arcs(&mut source, &tables.arcs);
    write_elements(&mut source, &tables.elements);
    source
}

/// Writes the module docs, the imports, and the release constants.
fn write_header(source: &mut String, tables: &Tables) {
    let Tables {
        release_id,
        release_folder,
        checksum,
        ..
    } = tables;
    writeln!(
        source,
        "//! # US GAAP Release Tables
//!
//! Provides the calculation arcs and the element reference data of the release `{release_id}`.
//!
//! `us_gaap_generator` writes this file from `{release_folder}/`. Do not edit it. After a change
//! to the vendored files, run `cargo run -p us_gaap_generator`.

use super::table::{{CalculationRow, ElementRow}};
use crate::core::taxonomy::reference::{{Balance, PeriodType}};

/// The ID of the release that the tables come from.
pub const RELEASE_ID: &str = \"{release_id}\";

/// The SHA-256 of the `MANIFEST` of the vendored files, as 64 lowercase hex digits.
pub const RELEASE_CHECKSUM: &str = \"{checksum}\";
"
    )
    .expect("Given a String, a write should never fail");
}

/// Writes the table of calculation arcs.
///
/// The `Debug` text of a `str` is a Rust string literal, with quotes and escapes, so the tables use
/// it for every text.
fn write_arcs(source: &mut String, arcs: &[CalculationArc]) {
    source.push_str(
        "/// The summation arcs of every calculation linkbase of the release, sorted, without repeats.
pub static CALCULATION_ARCS: &[CalculationRow] = &[
",
    );
    for arc in arcs {
        writeln!(
            source,
            "    CalculationRow::new({:?}, {:?}, {:?}, {:?}, {:?}),",
            arc.parent.family, arc.parent.name, arc.child.family, arc.child.name, arc.weight
        )
        .expect("Given a String, a write should never fail");
    }
    source.push_str("];\n\n");
}

/// Writes the table of elements.
fn write_elements(source: &mut String, elements: &[ElementLine]) {
    source.push_str(
        "/// The concepts of the element schema of the release, sorted by name.
pub static ELEMENTS: &[ElementRow] = &[
",
    );
    for element in elements {
        let balance = match element.balance {
            None => NO_BALANCE,
            Some(Balance::Debit) => DEBIT_BALANCE,
            Some(Balance::Credit) => CREDIT_BALANCE,
        };
        let period_type = match element.period_type {
            PeriodType::Instant => INSTANT_PERIOD_TYPE,
            PeriodType::Duration => DURATION_PERIOD_TYPE,
        };
        writeln!(
            source,
            "    ElementRow::new({:?}, {:?}, {:?}, {balance}, {period_type}),",
            element.name, element.label, element.data_type
        )
        .expect("Given a String, a write should never fail");
    }
    source.push_str("];\n");
}

#[cfg(test)]
mod tests {
    use pretty_assertions::assert_eq;

    use super::*;
    use crate::linkbase::ConceptRef;

    #[test]
    fn should_write_rust_literals_for_one_arc_and_one_element() {
        let tables = Tables {
            release_id: "us-gaap/2026".to_string(),
            release_folder: "taxonomy/us-gaap/2026".to_string(),
            checksum: "ab".to_string(),
            arcs: vec![CalculationArc {
                parent: ConceptRef {
                    family: "us-gaap".to_string(),
                    name: "Assets".to_string(),
                },
                child: ConceptRef {
                    family: "us-gaap".to_string(),
                    name: "AssetsCurrent".to_string(),
                },
                weight: "1.0".to_string(),
            }],
            elements: vec![ElementLine {
                name: "Assets".to_string(),
                label: "Assets \"total\"".to_string(),
                data_type: "xbrli:monetaryItemType".to_string(),
                balance: Some(Balance::Debit),
                period_type: PeriodType::Instant,
            }],
        };

        let expected_result = r#"//! # US GAAP Release Tables
//!
//! Provides the calculation arcs and the element reference data of the release `us-gaap/2026`.
//!
//! `us_gaap_generator` writes this file from `taxonomy/us-gaap/2026/`. Do not edit it. After a change
//! to the vendored files, run `cargo run -p us_gaap_generator`.

use super::table::{CalculationRow, ElementRow};
use crate::core::taxonomy::reference::{Balance, PeriodType};

/// The ID of the release that the tables come from.
pub const RELEASE_ID: &str = "us-gaap/2026";

/// The SHA-256 of the `MANIFEST` of the vendored files, as 64 lowercase hex digits.
pub const RELEASE_CHECKSUM: &str = "ab";

/// The summation arcs of every calculation linkbase of the release, sorted, without repeats.
pub static CALCULATION_ARCS: &[CalculationRow] = &[
    CalculationRow::new("us-gaap", "Assets", "us-gaap", "AssetsCurrent", "1.0"),
];

/// The concepts of the element schema of the release, sorted by name.
pub static ELEMENTS: &[ElementRow] = &[
    ElementRow::new("Assets", "Assets \"total\"", "xbrli:monetaryItemType", Some(Balance::Debit), PeriodType::Instant),
];
"#;

        let result = render(&tables);

        assert_eq!(result, expected_result);
    }
}
