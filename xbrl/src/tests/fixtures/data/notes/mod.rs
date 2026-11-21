//! # Notes Fixture
//!
//! The `sub`, `num`, `txt`, `dim`, and `tag` rows of the fixture filings, and helpers that parse
//! them. Each directory named after an accession holds the rows of one filing, with the header and
//! the rows as the SEC wrote them. The 10-Q `0001628280-26-068246` keeps plain `.tsv` files and is
//! compiled in. Every other filing keeps `.tsv.gz` files, which [`parse_notes_fixture_filing`]
//! reads at run time. [`manifest`](super::manifest) lists the filings and their source.
//!
//! A field with a quote is quoted, and a quote inside it is doubled, as in CSV. [`table`] removes
//! the quoting. It splits each line on tabs, and panics if a row has more or fewer fields than the
//! header, for example after a quoted field that holds a tab.

use std::collections::HashMap;

use super::{fixture_content, fixture_taxonomy};
use crate::adapter::edgar::notes::row::{DimRow, NumRow, SubRow, TagRow, TxtRow};
use crate::adapter::edgar::notes::{NotesFacts, NotesFiling, parse};
use crate::core::taxonomy::registry::Registry;

/// The `sub.tsv` rows of the compiled-in fixture filing.
const SUB_TSV: &str = include_str!("0001628280-26-068246/sub.tsv");

/// The `num.tsv` rows of the compiled-in fixture filing.
const NUM_TSV: &str = include_str!("0001628280-26-068246/num.tsv");

/// The `txt.tsv` rows of the compiled-in fixture filing.
const TXT_TSV: &str = include_str!("0001628280-26-068246/txt.tsv");

/// The `dim.tsv` rows that the `num` and `txt` rows of the compiled-in fixture filing name.
const DIM_TSV: &str = include_str!("0001628280-26-068246/dim.tsv");

/// The `tag.tsv` rows that the `num` rows of the compiled-in fixture filing name.
const TAG_TSV: &str = include_str!("0001628280-26-068246/tag.tsv");

/// The character between the fields of a row.
const FIELD_SEPARATOR: char = '\t';

/// The character that quotes a field.
const QUOTE: char = '"';

/// The text of the five tables of one filing.
struct NotesTables {
    sub: String,
    num: String,
    txt: String,
    dim: String,
    tag: String,
}

/// Parses the rows of the compiled-in 10-Q `0001628280-26-068246` with a registry of
/// [`fixture_taxonomy`].
///
/// # Panics
///
/// Never panics: the `sub` row of the fixture filing is valid.
#[must_use]
pub fn parse_notes_fixture() -> NotesFacts {
    parse_with_fixture_taxonomy(&notes_filing(&NotesTables {
        sub: SUB_TSV.to_owned(),
        num: NUM_TSV.to_owned(),
        txt: TXT_TSV.to_owned(),
        dim: DIM_TSV.to_owned(),
        tag: TAG_TSV.to_owned(),
    }))
}

/// Parses the rows of the fixture filing `accession` with a registry of [`fixture_taxonomy`].
///
/// # Panics
///
/// Panics if no compressed fixture filing has the accession `accession`, or if its `sub` row is
/// invalid.
#[must_use]
pub fn parse_notes_fixture_filing(accession: &str) -> NotesFacts {
    let read = |table: &str| {
        String::from_utf8(fixture_content(&format!(
            "notes/{accession}/{table}.tsv.gz"
        )))
        .expect("Given a fixture file the SEC wrote in UTF-8, the decoding should always succeed")
    };
    parse_with_fixture_taxonomy(&notes_filing(&NotesTables {
        sub: read("sub"),
        num: read("num"),
        txt: read("txt"),
        dim: read("dim"),
        tag: read("tag"),
    }))
}

/// Parses `rows` with a registry of [`fixture_taxonomy`].
fn parse_with_fixture_taxonomy(rows: &NotesFiling) -> NotesFacts {
    let plugin = fixture_taxonomy();
    let registry = Registry::new(vec![&plugin])
        .expect("Given one plug-in with four families, the creation should always succeed");
    parse(rows, &registry)
        .expect("Given the valid sub row of a real filing, the parse should always succeed")
}

/// Returns the rows of the filing in `tables`.
fn notes_filing(tables: &NotesTables) -> NotesFiling {
    let field = |row: &HashMap<String, String>, column: &str| row[column].clone();
    let sub = &table(&tables.sub)[0];
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
        num: table(&tables.num)
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
        txt: table(&tables.txt)
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
        dim: table(&tables.dim)
            .iter()
            .map(|row| DimRow {
                dimh: field(row, "dimhash"),
                segments: field(row, "segments"),
                segt: field(row, "segt"),
            })
            .collect(),
        tag: table(&tables.tag)
            .iter()
            .map(|row| TagRow {
                tag: field(row, "tag"),
                version: field(row, "version"),
                datatype: field(row, "datatype"),
            })
            .collect(),
    }
}

/// Returns each row of a table as a map from column to unquoted field.
///
/// # Panics
///
/// Panics if a row has more or fewer fields than the header.
fn table(text: &str) -> Vec<HashMap<String, String>> {
    let mut lines = text.lines();
    let header: Vec<&str> = lines
        .next()
        .expect("Given a fixture file with a header, the first line should always exist")
        .split(FIELD_SEPARATOR)
        .collect();
    lines
        .map(|line| {
            let fields: Vec<&str> = line.split(FIELD_SEPARATOR).collect();
            assert_eq!(
                fields.len(),
                header.len(),
                "Given a fixture row with no tab inside a field, the row should have one field per column: '{line}'"
            );
            header
                .iter()
                .zip(fields)
                .map(|(column, field)| ((*column).to_owned(), unquote(field)))
                .collect()
        })
        .collect()
}

/// Returns `field` without its quotes, with each doubled quote inside it as one quote.
fn unquote(field: &str) -> String {
    field
        .strip_prefix(QUOTE)
        .and_then(|rest| rest.strip_suffix(QUOTE))
        .map_or_else(|| field.to_owned(), |inner| inner.replace("\"\"", "\""))
}

#[cfg(test)]
mod tests {
    use pretty_assertions::assert_eq;

    use super::{table, unquote};

    #[test]
    fn should_keep_the_field_when_the_field_has_no_quotes() {
        let field = "Revenues";

        let expected_result = "Revenues".to_owned();

        let result = unquote(field);

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_remove_the_quotes_and_undouble_inner_quotes_when_the_field_is_quoted() {
        let field = r#""Includes the ""VIE"" balances""#;

        let expected_result = r#"Includes the "VIE" balances"#.to_owned();

        let result = unquote(field);

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_keep_the_field_when_the_field_has_only_an_opening_quote() {
        let field = r#""Includes"#;

        let expected_result = r#""Includes"#.to_owned();

        let result = unquote(field);

        assert_eq!(result, expected_result);
    }

    #[test]
    #[should_panic(expected = "the row should have one field per column")]
    fn should_panic_when_a_row_has_more_fields_than_the_header() {
        let text = "dimhash\tsegments\tsegt\n0x1\t\"a\tb\"\t0\n";

        let _rows = table(text);
    }

    #[test]
    #[should_panic(expected = "the row should have one field per column")]
    fn should_panic_when_a_row_has_fewer_fields_than_the_header() {
        let text = "dimhash\tsegments\tsegt\n0x1\t0\n";

        let _rows = table(text);
    }
}
