//! # Render
//!
//! Writes the Rust source of `us_gaap/taxonomy.rs` from the read tables.

use std::fmt::Write;

use super::linkbase::CalculationArc;
use crate::core::taxonomy::reference::{Balance, PeriodType};

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
///
/// The order of the fields sorts the lines by family, then by name.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ElementLine {
    /// The namespace family, for example `srt`.
    pub family: String,
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
    /// The vendored folders, relative to the `xbrl` crate, in the order of the checksum.
    pub folders: Vec<String>,
    /// The SHA-256 of the `MANIFEST` files of `folders`, joined in that order, as 64 lowercase hex
    /// digits.
    pub checksum: String,
    /// The calculation arcs, sorted by role, parent, and child, without repeats.
    pub arcs: Vec<CalculationArc>,
    /// The elements, sorted by family and name.
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
        folders,
        checksum,
        ..
    } = tables;
    writeln!(
        source,
        "//! # US GAAP Release Tables
//!
//! Provides the calculation arcs and the element reference data of the release `{release_id}`.
//!
//! The generator writes this file from the vendored folders below. Do not edit it. After a change
//! to the vendored files, run `cargo run -p xbrl --features generator --bin generate_us_gaap`.
//!"
    )
    .expect("Given a String, a write should never fail");
    for folder in folders {
        writeln!(source, "//! - `{folder}/`").expect("Given a String, a write should never fail");
    }
    writeln!(
        source,
        "
use super::table::{{CalculationRow, ElementRow}};
use crate::core::taxonomy::reference::{{Balance, PeriodType}};

/// The ID of the release that the tables come from.
pub const RELEASE_ID: &str = \"{release_id}\";

/// The SHA-256 of the `MANIFEST` files of the vendored folders, joined in the order of the list
/// above, as 64 lowercase hex digits.
pub const RELEASE_CHECKSUM: &str = \"{checksum}\";
"
    )
    .expect("Given a String, a write should never fail");
}

/// Writes the table of calculation arcs.
///
/// The tables write every text with the `Debug` format of `str`, which gives a Rust string literal.
/// A new toolchain can escape a non-ASCII character in another way. Then the generated file
/// changes, and the generator test fails until the generator runs again.
fn write_arcs(source: &mut String, arcs: &[CalculationArc]) {
    source.push_str(
        "/// The summation arcs of the statement calculation linkbases (`stm/`) of the release, sorted by
/// role, parent, and child. Within one role, an arc appears once.
pub static CALCULATION_ARCS: &[CalculationRow] = &[
",
    );
    for arc in arcs {
        writeln!(
            source,
            "    CalculationRow::new({:?}, {:?}, {:?}, {:?}, {:?}, {:?}),",
            arc.role,
            arc.parent.family,
            arc.parent.name,
            arc.child.family,
            arc.child.name,
            arc.weight
        )
        .expect("Given a String, a write should never fail");
    }
    source.push_str("];\n\n");
}

/// Writes the table of elements.
fn write_elements(source: &mut String, elements: &[ElementLine]) {
    source.push_str(
        "/// The concepts of the element schemas of the vendored folders, sorted by family and name.
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
            "    ElementRow::new({:?}, {:?}, {:?}, {:?}, {balance}, {period_type}),",
            element.family, element.name, element.label, element.data_type
        )
        .expect("Given a String, a write should never fail");
    }
    source.push_str("];\n");
}

#[cfg(test)]
mod tests {
    use pretty_assertions::assert_eq;

    use super::*;
    use crate::us_gaap::generator::linkbase::ConceptRef;

    #[test]
    fn should_write_rust_literals_for_two_folders_one_arc_and_one_element() {
        let tables = Tables {
            release_id: "us-gaap/2026".to_string(),
            folders: vec![
                "taxonomy/us-gaap/2026".to_string(),
                "taxonomy/srt/2026".to_string(),
            ],
            checksum: "ab".to_string(),
            arcs: vec![CalculationArc {
                role: "http://fasb.org/us-gaap/role/statement/A".to_string(),
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
                family: "us-gaap".to_string(),
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
//! The generator writes this file from the vendored folders below. Do not edit it. After a change
//! to the vendored files, run `cargo run -p xbrl --features generator --bin generate_us_gaap`.
//!
//! - `taxonomy/us-gaap/2026/`
//! - `taxonomy/srt/2026/`

use super::table::{CalculationRow, ElementRow};
use crate::core::taxonomy::reference::{Balance, PeriodType};

/// The ID of the release that the tables come from.
pub const RELEASE_ID: &str = "us-gaap/2026";

/// The SHA-256 of the `MANIFEST` files of the vendored folders, joined in the order of the list
/// above, as 64 lowercase hex digits.
pub const RELEASE_CHECKSUM: &str = "ab";

/// The summation arcs of the statement calculation linkbases (`stm/`) of the release, sorted by
/// role, parent, and child. Within one role, an arc appears once.
pub static CALCULATION_ARCS: &[CalculationRow] = &[
    CalculationRow::new("http://fasb.org/us-gaap/role/statement/A", "us-gaap", "Assets", "us-gaap", "AssetsCurrent", "1.0"),
];

/// The concepts of the element schemas of the vendored folders, sorted by family and name.
pub static ELEMENTS: &[ElementRow] = &[
    ElementRow::new("us-gaap", "Assets", "Assets \"total\"", "xbrli:monetaryItemType", Some(Balance::Debit), PeriodType::Instant),
];
"#;

        let result = render(&tables);

        assert_eq!(result, expected_result);
    }
}
