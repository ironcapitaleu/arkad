//! # Notes Fixture
//!
//! The `sub`, `num`, `txt`, `dim`, and `tag` rows of the 10-Q `0001628280-26-068246` from the Notes
//! file `2026_10_notes.zip`, and helpers that parse them. The files keep the header and the rows as
//! the SEC wrote them. None of the rows holds a quoted field, so each line splits on tabs.

use std::collections::HashMap;

use super::fixture_taxonomy;
use crate::adapter::edgar::notes::row::{DimRow, NumRow, SubRow, TagRow, TxtRow};
use crate::adapter::edgar::notes::{NotesFacts, NotesFiling, parse};
use crate::core::taxonomy::registry::Registry;

/// The `sub.tsv` rows of the fixture filing.
const SUB_TSV: &str = include_str!("0001628280-26-068246/sub.tsv");

/// The `num.tsv` rows of the fixture filing.
const NUM_TSV: &str = include_str!("0001628280-26-068246/num.tsv");

/// The `txt.tsv` rows of the fixture filing.
const TXT_TSV: &str = include_str!("0001628280-26-068246/txt.tsv");

/// The `dim.tsv` rows that the `num` and `txt` rows of the fixture filing name.
const DIM_TSV: &str = include_str!("0001628280-26-068246/dim.tsv");

/// The `tag.tsv` rows that the `num` rows of the fixture filing name.
const TAG_TSV: &str = include_str!("0001628280-26-068246/tag.tsv");

/// The character between the fields of a row.
const FIELD_SEPARATOR: char = '\t';

/// Parses the rows of the fixture filing with a registry of [`fixture_taxonomy`].
///
/// # Panics
///
/// Never panics: the `sub` row of the fixture filing is valid.
#[must_use]
pub fn parse_notes_fixture() -> NotesFacts {
    let rows = notes_filing();
    let plugin = fixture_taxonomy();
    let registry = Registry::new(vec![&plugin])
        .expect("Given one plug-in with four families, the creation should always succeed");
    parse(&rows, &registry)
        .expect("Given the valid sub row of a real filing, the parse should always succeed")
}

/// Returns the rows of the fixture filing.
///
fn notes_filing() -> NotesFiling {
    let field = |row: &HashMap<String, String>, column: &str| row[column].clone();
    let sub = &table(SUB_TSV)[0];
    NotesFiling {
        sub: SubRow {
            adsh: field(sub, "adsh"),
            cik: field(sub, "cik"),
            name: field(sub, "name"),
            sic: field(sub, "sic"),
            fye: field(sub, "fye"),
            form: field(sub, "form"),
            period: field(sub, "period"),
            fy: field(sub, "fy"),
            fp: field(sub, "fp"),
            filed: field(sub, "filed"),
            accepted: field(sub, "accepted"),
        },
        num: table(NUM_TSV)
            .iter()
            .map(|row| NumRow {
                tag: field(row, "tag"),
                version: field(row, "version"),
                ddate: field(row, "ddate"),
                qtrs: field(row, "qtrs"),
                uom: field(row, "uom"),
                dimh: field(row, "dimh"),
                iprx: field(row, "iprx"),
                value: field(row, "value"),
                footnote: field(row, "footnote"),
                coreg: field(row, "coreg"),
                durp: field(row, "durp"),
                datp: field(row, "datp"),
                dcml: field(row, "dcml"),
            })
            .collect(),
        txt: table(TXT_TSV)
            .iter()
            .map(|row| TxtRow {
                tag: field(row, "tag"),
                version: field(row, "version"),
                ddate: field(row, "ddate"),
                qtrs: field(row, "qtrs"),
                dimh: field(row, "dimh"),
                iprx: field(row, "iprx"),
                value: field(row, "value"),
                srclen: field(row, "srclen"),
                footnote: field(row, "footnote"),
                coreg: field(row, "coreg"),
                durp: field(row, "durp"),
                datp: field(row, "datp"),
            })
            .collect(),
        dim: table(DIM_TSV)
            .iter()
            .map(|row| DimRow {
                dimh: field(row, "dimhash"),
                segments: field(row, "segments"),
                segt: field(row, "segt"),
            })
            .collect(),
        tag: table(TAG_TSV)
            .iter()
            .map(|row| TagRow {
                tag: field(row, "tag"),
                version: field(row, "version"),
                datatype: field(row, "datatype"),
            })
            .collect(),
    }
}

/// Returns each row of a table as a map from column to field.
fn table(text: &str) -> Vec<HashMap<String, String>> {
    let mut lines = text.lines();
    let header: Vec<&str> = lines
        .next()
        .expect("Given a fixture file with a header, the first line should always exist")
        .split(FIELD_SEPARATOR)
        .collect();
    lines
        .map(|line| {
            header
                .iter()
                .zip(line.split(FIELD_SEPARATOR))
                .map(|(column, field)| ((*column).to_owned(), field.to_owned()))
                .collect()
        })
        .collect()
}
