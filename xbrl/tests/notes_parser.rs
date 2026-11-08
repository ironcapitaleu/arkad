//! # Notes Parser Integration Tests
//!
//! Runs [`xbrl::adapter::edgar::notes::parse`] on the `num` and `txt` rows of one real filing of the Notes file
//! `2026_10_notes.zip`: the 10-Q `0001628280-26-068246` of `FEDEX FREIGHT HOLDING COMPANY, INC.`. The
//! fixture files keep the header and the rows of the filing as the SEC wrote them. None of the rows
//! holds a quoted field, so the test splits each line on tabs.
//!
//! ## External Dependencies
//!
//! None. The fixture lives in `tests/fixtures/notes/0001628280-26-068246/`.

use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;

use chrono::{NaiveDate, TimeZone, Utc};
use pretty_assertions::assert_eq;
use xbrl::adapter::edgar::notes::row::{DimRow, NumRow, SubRow, TagRow, TxtRow};
use xbrl::adapter::edgar::notes::{NotesFiling, parse};
use xbrl::core::concept::{ConceptName, NamespaceFamily};
use xbrl::core::entity_name::EntityName;
use xbrl::core::fact::FactValue;
use xbrl::core::filing::{CompanyAttributes, EntityId, Filing, FilingId, IndustryCode};
use xbrl::core::fiscal_period::FiscalPeriod;
use xbrl::core::fiscal_year::FiscalYear;
use xbrl::core::form::Form;
use xbrl::core::release::{Sha256, TaxonomyRelease, TaxonomyReleaseId};
use xbrl::core::taxonomy::Taxonomy;
use xbrl::core::taxonomy::family::{FamilyDeclaration, UriPattern};
use xbrl::core::taxonomy::reference::{CalculationTree, ConceptAliases, ElementReference};
use xbrl::core::taxonomy::registry::Registry;

struct FakeUsGaap {
    release: TaxonomyRelease,
    families: Vec<FamilyDeclaration>,
    tree: CalculationTree,
}

impl Taxonomy for FakeUsGaap {
    fn release(&self) -> &TaxonomyRelease {
        &self.release
    }

    fn families(&self) -> &[FamilyDeclaration] {
        &self.families
    }

    fn aliases(&self) -> &[ConceptAliases] {
        &[]
    }

    fn calculation_tree(&self) -> &CalculationTree {
        &self.tree
    }

    fn element(&self, _concept: &ConceptName) -> Option<&ElementReference> {
        None
    }

    fn elements(&self) -> &[ElementReference] {
        &[]
    }
}

fn declaration(name: &str, pattern: &str) -> FamilyDeclaration {
    let family = NamespaceFamily::new(name)
        .expect("Given a hardcoded family without whitespace, the creation should always succeed");
    let pattern = UriPattern::new(pattern).expect(
        "Given a hardcoded pattern with one placeholder, the creation should always succeed",
    );
    FamilyDeclaration::new(family, vec![pattern], name, true)
        .expect("Given a hardcoded declaration, the creation should always succeed")
}

fn us_gaap() -> FakeUsGaap {
    FakeUsGaap {
        release: TaxonomyRelease {
            id: TaxonomyReleaseId::new("us-gaap/2026").expect(
                "Given a hardcoded release ID with a year, the creation should always succeed",
            ),
            checksum: Sha256::new([0; 32]),
        },
        families: vec![
            declaration("us-gaap", "http://fasb.org/us-gaap/{release}"),
            declaration("dei", "http://xbrl.sec.gov/dei/{release}"),
            declaration("srt", "http://fasb.org/srt/{release}"),
            declaration("ecd", "http://xbrl.sec.gov/ecd/{release}"),
        ],
        tree: CalculationTree::default(),
    }
}

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

#[test]
fn should_keep_every_row_except_the_repeated_copies_for_a_real_filing() {
    let rows = notes_filing();
    let plugin = us_gaap();
    let registry = Registry::new(vec![&plugin])
        .expect("Given one plug-in with four families, the creation should always succeed");

    let expected_result = (613, 0);

    let parsed = parse(&rows, &registry)
        .expect("Given the valid sub row of a real filing, the parse should always succeed");
    let result = (parsed.facts.len(), parsed.findings.len());

    assert_eq!(result, expected_result);
}

#[test]
fn should_map_the_sub_row_into_the_filing_for_a_real_filing() {
    let rows = notes_filing();
    let plugin = us_gaap();
    let registry = Registry::new(vec![&plugin])
        .expect("Given one plug-in with four families, the creation should always succeed");

    let expected_result =
        Filing {
            id: FilingId::new("sec", "0001628280-26-068246")
                .expect("Given a hardcoded source and ID, the creation should always succeed"),
            entity: EntityId::new("http://www.sec.gov/CIK", "0002082247")
                .expect("Given a hardcoded scheme and ID, the creation should always succeed"),
            form: Form::new("10-Q")
                .expect("Given a hardcoded form, the creation should always succeed"),
            is_amendment: false,
            filed: NaiveDate::from_ymd_opt(2026, 10, 29)
                .expect("Given a hardcoded valid date, the creation should always succeed"),
            accepted: Utc.with_ymd_and_hms(2026, 10, 29, 20, 15, 0).single(),
            period_of_report: NaiveDate::from_ymd_opt(2026, 9, 30),
            fiscal_year: Some(FiscalYear::new(2026)),
            fiscal_period: Some(
                FiscalPeriod::new("Q3")
                    .expect("Given a hardcoded fiscal period, the creation should always succeed"),
            ),
            company: CompanyAttributes {
                name: Some(EntityName::new("FEDEX FREIGHT HOLDING COMPANY, INC.")),
                industry: Some(IndustryCode::new("sic", "4513").expect(
                    "Given a hardcoded scheme and code, the creation should always succeed",
                )),
                tickers: vec![],
                fiscal_year_end: Some("1231".to_owned()),
            },
        };

    let result = parse(&rows, &registry)
        .expect("Given the valid sub row of a real filing, the parse should always succeed")
        .filing;

    assert_eq!(result, expected_result);
}

#[test]
fn should_restore_shares_and_rank_copies_of_basic_earnings_per_share_for_a_real_filing() {
    let rows = notes_filing();
    let plugin = us_gaap();
    let registry = Registry::new(vec![&plugin])
        .expect("Given one plug-in with four families, the creation should always succeed");

    let expected_result = vec![
        (
            "0 quarters to 2025-06-30".to_owned(),
            "USD/shares".to_owned(),
            "0.7600".to_owned(),
            0,
        ),
        (
            "1 quarters to 2025-09-30".to_owned(),
            "USD/shares".to_owned(),
            "1.9500".to_owned(),
            0,
        ),
        (
            "1 quarters to 2025-09-30".to_owned(),
            "USD/shares".to_owned(),
            "2.7000".to_owned(),
            1,
        ),
        (
            "0 quarters to 2026-06-30".to_owned(),
            "USD/shares".to_owned(),
            "0.5600".to_owned(),
            0,
        ),
        (
            "1 quarters to 2026-09-30".to_owned(),
            "USD/shares".to_owned(),
            "0.9800".to_owned(),
            0,
        ),
        (
            "1 quarters to 2026-09-30".to_owned(),
            "USD/shares".to_owned(),
            "1.5400".to_owned(),
            1,
        ),
    ];

    let parsed = parse(&rows, &registry)
        .expect("Given the valid sub row of a real filing, the parse should always succeed");
    let result: Vec<(String, String, String, u16)> = parsed
        .facts
        .iter()
        .filter(|fact| fact.concept.local_name() == "EarningsPerShareBasic")
        .map(|fact| {
            let unit = fact
                .unit
                .as_ref()
                .map(ToString::to_string)
                .unwrap_or_default();
            let value = match &fact.value {
                FactValue::Numeric(number) => number.to_string(),
                other => format!("{other:?}"),
            };
            (fact.context.period.to_string(), unit, value, fact.copy_rank)
        })
        .collect();

    assert_eq!(result, expected_result);
}

#[test]
fn should_map_the_document_type_into_a_text_fact_for_a_real_filing() {
    let rows = notes_filing();
    let plugin = us_gaap();
    let registry = Registry::new(vec![&plugin])
        .expect("Given one plug-in with four families, the creation should always succeed");

    let expected_result = vec![(
        "dei:DocumentType".to_owned(),
        "1 quarters to 2026-09-30".to_owned(),
        FactValue::Text("10-Q".to_owned()),
    )];

    let parsed = parse(&rows, &registry)
        .expect("Given the valid sub row of a real filing, the parse should always succeed");
    let result: Vec<(String, String, FactValue)> = parsed
        .facts
        .into_iter()
        .filter(|fact| fact.concept.local_name() == "DocumentType")
        .map(|fact| {
            (
                fact.concept.to_string(),
                fact.context.period.to_string(),
                fact.value,
            )
        })
        .collect();

    assert_eq!(result, expected_result);
}
