//! # Notes Parser Integration Tests
//!
//! Runs [`xbrl::adapter::edgar::notes::parse`] on the `num` and `txt` rows of one real filing of the Notes file
//! `2026_10_notes.zip`: the 10-Q `0001628280-26-068246` of `FEDEX FREIGHT HOLDING COMPANY, INC.`. The
//! fixture files keep the header and the rows of the filing as the SEC wrote them.
//!
//! ## External Dependencies
//!
//! None. The fixture lives in `tests/fixtures/notes/0001628280-26-068246/`.

use chrono::{NaiveDate, TimeZone, Utc};
use pretty_assertions::assert_eq;
use xbrl::core::entity_name::EntityName;
use xbrl::core::fact::FactValue;
use xbrl::core::filing::{CompanyAttributes, EntityId, Filing, FilingId, IndustryCode};
use xbrl::core::fiscal_period::FiscalPeriod;
use xbrl::core::fiscal_year::FiscalYear;
use xbrl::core::form::Form;

use self::notes_fixture::parse_notes_fixture;

#[path = "support/notes_fixture.rs"]
mod notes_fixture;
#[path = "support/taxonomy.rs"]
mod taxonomy;

#[test]
fn should_keep_every_row_except_the_repeated_copies_for_a_real_filing() {
    let parsed = parse_notes_fixture();

    let expected_result = (613, 0);

    let result = (parsed.facts.len(), parsed.findings.len());

    assert_eq!(result, expected_result);
}

#[test]
fn should_map_the_sub_row_into_the_filing_for_a_real_filing() {
    let parsed = parse_notes_fixture();

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

    let result = parsed.filing;

    assert_eq!(result, expected_result);
}

#[test]
fn should_restore_shares_and_rank_copies_of_basic_earnings_per_share_for_a_real_filing() {
    let parsed = parse_notes_fixture();

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
    let parsed = parse_notes_fixture();

    let expected_result = vec![(
        "dei:DocumentType".to_owned(),
        "1 quarters to 2026-09-30".to_owned(),
        FactValue::Text("10-Q".to_owned()),
    )];

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
