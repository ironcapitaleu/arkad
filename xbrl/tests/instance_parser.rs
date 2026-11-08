//! # Instance Parser Integration Tests
//!
//! Runs [`xbrl::adapter::edgar::instance::parse`] on the instance document of one real filing: the
//! 10-Q `0001628280-26-068246` of `FEDEX FREIGHT HOLDING COMPANY, INC.`, the filing of the Notes
//! fixture. The fixture is the file `fdxf-20260930_htm.xml` as EDGAR serves it.
//!
//! ## External Dependencies
//!
//! None. The fixture lives in `tests/fixtures/instance/0001628280-26-068246/`.

use std::fs;
use std::path::PathBuf;

use bigdecimal::BigDecimal;
use chrono::NaiveDate;
use pretty_assertions::assert_eq;
use xbrl::adapter::edgar::instance::{InstanceFacts, InstanceFiling, parse};
use xbrl::core::concept::{ConceptName, NamespaceFamily};
use xbrl::core::entity_name::EntityName;
use xbrl::core::fact::FactValue;
use xbrl::core::filing::{CompanyAttributes, EntityId, Filing, FilingId, Ticker};
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

fn filing_id() -> FilingId {
    FilingId::new("sec", "0001628280-26-068246")
        .expect("Given a hardcoded source and ID, the creation should always succeed")
}

const fn filed() -> NaiveDate {
    NaiveDate::from_ymd_opt(2026, 10, 29)
        .expect("Given a hardcoded valid date, the creation should always succeed")
}

fn parse_fixture() -> InstanceFacts {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures/instance/0001628280-26-068246/fdxf-20260930_htm.xml");
    let document = fs::read_to_string(path)
        .expect("Given a fixture file in the repository, the read should always succeed");
    let plugin = us_gaap();
    let registry = Registry::new(vec![&plugin])
        .expect("Given one plug-in with four families, the creation should always succeed");
    let input = InstanceFiling {
        document: &document,
        id: filing_id(),
        filed: filed(),
    };
    parse(&input, &registry)
        .expect("Given the instance document of a real filing, the parse should always succeed")
}

#[test]
fn should_keep_every_fact_except_the_repeated_copies_for_a_real_filing() {
    let parsed = parse_fixture();

    let expected_result = (613, 0);

    let result = (parsed.facts.len(), parsed.findings.len());

    assert_eq!(result, expected_result);
}

#[test]
fn should_map_the_cover_facts_into_the_filing_for_a_real_filing() {
    let parsed = parse_fixture();

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
    let parsed = parse_fixture();

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
    let parsed = parse_fixture();

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
