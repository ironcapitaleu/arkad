//! # Instance Parser Integration Tests
//!
//! Runs [`xbrl::adapter::edgar::instance::parse`] on the instance document of one real filing: the
//! 10-Q `0001628280-26-068246` of `FEDEX FREIGHT HOLDING COMPANY, INC.`, the filing of the Notes
//! fixture. The fixture is the file `fdxf-20260930_htm.xml` as EDGAR serves it.
//!
//! ## External Dependencies
//!
//! None. The fixture lives in `src/tests/fixtures/data/instance/0001628280-26-068246/`.

use bigdecimal::BigDecimal;
use chrono::NaiveDate;
use pretty_assertions::assert_eq;
use xbrl::core::entity_name::EntityName;
use xbrl::core::fact::FactValue;
use xbrl::core::filing::{CompanyAttributes, EntityId, Filing, Ticker};
use xbrl::core::fiscal_period::FiscalPeriod;
use xbrl::core::fiscal_year::FiscalYear;
use xbrl::core::form::Form;
use xbrl::tests::fixtures::data::instance::{filed, filing_id, parse_instance_fixture};

#[test]
fn should_keep_every_fact_except_the_repeated_copies_for_a_real_filing() {
    let parsed = parse_instance_fixture();

    let expected_result = (613, 0);

    let result = (parsed.facts.len(), parsed.findings.len());

    assert_eq!(result, expected_result);
}

#[test]
fn should_map_the_cover_facts_into_the_filing_for_a_real_filing() {
    let parsed = parse_instance_fixture();

    let expected_result = Filing {
        id: filing_id(),
        entity: EntityId::new("http://www.sec.gov/CIK", "0002082247")
            .expect("Given a hardcoded scheme and ID, the creation should always succeed"),
        form: Form::new("10-Q")
            .expect("Given a hardcoded form, the creation should always succeed"),
        is_amendment: false,
        filed: filed(),
        accepted: None,
        period_of_report: NaiveDate::from_ymd_opt(2026, 9, 30),
        fiscal_year: Some(FiscalYear::new(2026)),
        fiscal_period: Some(
            FiscalPeriod::new("Q3")
                .expect("Given a hardcoded fiscal period, the creation should always succeed"),
        ),
        company: CompanyAttributes {
            name: Some(EntityName::new("FedEx Freight Holding Company, Inc.")),
            industry: None,
            tickers: vec![Ticker {
                symbol: "FDXF".to_owned(),
                exchange: Some("NYSE".to_owned()),
            }],
            fiscal_year_end: Some("--12-31".to_owned()),
        },
    };

    let result = parsed.filing;

    assert_eq!(result, expected_result);
}

#[test]
fn should_rank_copies_of_basic_earnings_per_share_in_the_sec_order_for_a_real_filing() {
    let parsed = parse_instance_fixture();

    let expected_result = vec![
        (
            "2026-07-01 to 2026-09-30".to_owned(),
            "USD/shares".to_owned(),
            "0.98".to_owned(),
            0,
        ),
        (
            "2025-07-01 to 2025-09-30".to_owned(),
            "USD/shares".to_owned(),
            "1.95".to_owned(),
            0,
        ),
        (
            "2026-06-01 to 2026-09-30".to_owned(),
            "USD/shares".to_owned(),
            "1.54".to_owned(),
            1,
        ),
        (
            "2025-06-01 to 2025-09-30".to_owned(),
            "USD/shares".to_owned(),
            "2.70".to_owned(),
            1,
        ),
        (
            "2026-06-01 to 2026-06-30".to_owned(),
            "USD/shares".to_owned(),
            "0.56".to_owned(),
            0,
        ),
        (
            "2025-06-01 to 2025-06-30".to_owned(),
            "USD/shares".to_owned(),
            "0.76".to_owned(),
            0,
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
fn should_attach_the_footnote_of_the_link_for_a_real_filing() {
    let parsed = parse_instance_fixture();

    let expected_result = vec![
        "Refer to Note 12, Related Party Transactions, for additional information on Net transfers from (to) Parent.".to_owned(),
    ];

    let result = parsed
        .facts
        .iter()
        .find(|fact| {
            fact.concept.local_name() == "StockholdersEquityNetTransfersFromToParent"
                && fact.value == FactValue::Numeric(BigDecimal::from(-5_000_000))
        })
        .map(|fact| fact.footnotes.clone())
        .unwrap_or_default();

    assert_eq!(result, expected_result);
}
