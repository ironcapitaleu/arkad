//! # Notes Fixture
//!
//! Reads and parses the `sub`, `num`, `txt`, `dim`, and `tag` rows of the 10-Q `0001628280-26-068246` from
//! `tests/fixtures/notes/`. The files keep the header and the rows as the SEC wrote them. None of
//! the rows holds a quoted field, so each line splits on tabs.

use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;

use xbrl::adapter::edgar::notes::row::{DimRow, NumRow, SubRow, TagRow, TxtRow};
use xbrl::adapter::edgar::notes::{NotesFacts, NotesFiling, parse};
use xbrl::core::taxonomy::registry::Registry;

use super::taxonomy::us_gaap;

fn table(name: &str) -> Vec<HashMap<String, String>> {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures/notes/0001628280-26-068246")
        .join(name);
    let text = fs::read_to_string(path)
        .expect("Given a fixture file in the repository, the read should always succeed");
    let mut lines = text.lines();
    let header: Vec<&str> = lines
        .next()
        .expect("Given a fixture file with a header, the first line should always exist")
        .split('\t')
        .collect();
    lines
        .map(|line| {
            header
                .iter()
                .zip(line.split('\t'))
                .map(|(column, field)| ((*column).to_owned(), field.to_owned()))
                .collect()
        })
        .collect()
}

/// Parses the rows of the fixture filing.
pub fn parse_notes_fixture() -> NotesFacts {
    let rows = notes_filing();
    let plugin = us_gaap();
    let registry = Registry::new(vec![&plugin])
        .expect("Given one plug-in with four families, the creation should always succeed");
    parse(&rows, &registry)
        .expect("Given the valid sub row of a real filing, the parse should always succeed")
}

/// Returns the rows of the fixture filing.
fn notes_filing() -> NotesFiling {
    let field = |row: &HashMap<String, String>, column: &str| row[column].clone();
    let sub = &table("sub.tsv")[0];
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
        num: table("num.tsv")
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
        txt: table("txt.tsv")
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
        dim: table("dim.tsv")
            .iter()
            .map(|row| DimRow {
                dimh: field(row, "dimhash"),
                segments: field(row, "segments"),
                segt: field(row, "segt"),
            })
            .collect(),
        tag: table("tag.tsv")
            .iter()
            .map(|row| TagRow {
                tag: field(row, "tag"),
                version: field(row, "version"),
                datatype: field(row, "datatype"),
            })
            .collect(),
    }
}
