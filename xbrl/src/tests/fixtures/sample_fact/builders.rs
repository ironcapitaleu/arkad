//! # Builders
//!
//! Provides helpers that build a [`FilingSource`], an [`AsIsFact`], and a [`Candidate`] with fixed
//! defaults. A test changes the public fields it needs.

use std::str::FromStr;

use bigdecimal::BigDecimal;
use chrono::{DateTime, NaiveDate, Utc};

use crate::core::candidate::{Candidate, CandidateValue};
use crate::core::concept::{ConceptName, NamespaceFamily};
use crate::core::confidence::Confidence;
use crate::core::elements::CanonicalElement;
use crate::core::fact::{
    AsIsFact, Context, Decimals, Dimensions, FactValue, FilingSource, TaxonomyVersion,
};
use crate::core::filing::{EntityId, FilingId};
use crate::core::period::{Duration, Period};
use crate::core::release::{Sha256, TaxonomyRelease, TaxonomyReleaseId};
use crate::core::rule::RuleId;
use crate::core::taxonomy::reference::ConceptAliases;
use crate::core::unit::Unit;
use crate::tests::fixtures::sample_taxonomy::{FakeTaxonomy, declaration, plugin_with_aliases};

/// The release of the fake US GAAP plug-in.
const RELEASE: &str = "us-gaap/2026";

/// The family of every fact the builders make.
const FAMILY: &str = "us-gaap";

/// Returns the date `year`-`month`-`day`.
///
/// # Panics
///
/// Panics if the date does not exist.
#[must_use]
pub fn date(year: i32, month: u32, day: u32) -> NaiveDate {
    NaiveDate::from_ymd_opt(year, month, day)
        .expect("Given a hardcoded valid date, the creation should always succeed")
}

/// Returns the source of the SEC filing `accession`, filed on `filed`, with no acceptance time.
///
/// # Panics
///
/// Panics if `accession` is not a filing ID.
#[must_use]
pub fn source(accession: &str, filed: NaiveDate) -> FilingSource {
    FilingSource {
        filing: FilingId::new("sec", accession)
            .expect("Given a hardcoded source and ID, the creation should always succeed"),
        filed,
        accepted: None::<DateTime<Utc>>,
    }
}

fn concept(local_name: &str) -> ConceptName {
    let family = NamespaceFamily::new(FAMILY)
        .expect("Given a hardcoded family without whitespace, the creation should always succeed");
    ConceptName::new(family, local_name)
        .expect("Given a hardcoded local name without a colon, the creation should always succeed")
}

fn number(text: &str) -> BigDecimal {
    BigDecimal::from_str(text)
        .expect("Given a hardcoded decimal text, the parse should always succeed")
}

fn apple() -> EntityId {
    EntityId::new("cik", "0000320193")
        .expect("Given a hardcoded scheme and ID, the creation should always succeed")
}

fn usd() -> Unit {
    Unit::from_str("USD")
        .expect("Given a hardcoded unit with one measure, the parse should always succeed")
}

fn fiscal_year_2024() -> Period {
    Period::Duration(Duration::from_end_and_quarters(date(2024, 9, 28), 4))
}

fn release() -> TaxonomyRelease {
    TaxonomyRelease {
        id: TaxonomyReleaseId::new(RELEASE)
            .expect("Given a hardcoded release ID with a year, the creation should always succeed"),
        checksum: Sha256::new([0; 32]),
    }
}

/// Returns the as-is fact `us-gaap:<local_name>` = `value` of Apple in USD, for the fiscal year
/// that ends on 2024-09-28, at `decimals` -6 and copy rank 0.
///
/// # Panics
///
/// Panics if `local_name` is not a local name or `value` is not a decimal.
#[must_use]
pub fn fact(source: &FilingSource, local_name: &str, value: &str) -> AsIsFact {
    AsIsFact {
        source: source.clone(),
        context: Context {
            entity: apple(),
            period: fiscal_year_2024(),
            dimensions: Dimensions::default(),
        },
        concept: concept(local_name),
        taxonomy_version: Some(
            TaxonomyVersion::new(RELEASE)
                .expect("Given a hardcoded version, the creation should always succeed"),
        ),
        unit: Some(usd()),
        value: FactValue::Numeric(number(value)),
        decimals: Some(Decimals::Finite(-6)),
        copy_rank: 0,
        footnotes: Vec::new(),
    }
}

/// Returns a candidate for revenue = `value` of Apple in USD from `source`, for the fiscal year
/// that ends on 2024-09-28, at `decimals` -6 and confidence [`Confidence::Exact`].
///
/// # Panics
///
/// Panics if `value` is not a decimal.
#[must_use]
pub fn candidate(source: &FilingSource, value: &str) -> Candidate {
    let input = fact(source, "Revenues", value);
    Candidate {
        source: source.clone(),
        entity: apple(),
        concept: CanonicalElement::Revenue,
        period: fiscal_year_2024(),
        unit: usd(),
        dimensions: Dimensions::default(),
        value: CandidateValue::Reported {
            value: number(value),
            decimals: Some(Decimals::Finite(-6)),
            confidence: Confidence::Exact,
        },
        rule: RuleId::from_str("us-gaap/alias/revenue@1").expect(
            "Given a hardcoded rule ID in the documented form, the parse should always succeed",
        ),
        release: release(),
        inputs: vec![input.key()],
    }
}

/// Returns the alias list of revenue: `Revenues`, then `SalesRevenueNet`, then
/// `RevenueFromContractWithCustomerExcludingAssessedTax`.
///
/// # Panics
///
/// Panics if a hardcoded concept or rule ID breaks its rule.
#[must_use]
pub fn revenue_aliases() -> ConceptAliases {
    ConceptAliases {
        element: CanonicalElement::Revenue,
        rule: RuleId::from_str("us-gaap/alias/revenue@1").expect(
            "Given a hardcoded rule ID in the documented form, the parse should always succeed",
        ),
        concepts: vec![
            concept("Revenues"),
            concept("SalesRevenueNet"),
            concept("RevenueFromContractWithCustomerExcludingAssessedTax"),
        ],
    }
}

/// Returns a fake US GAAP plug-in of the release `us-gaap/2026` that resolves `us-gaap` with the
/// alias list of revenue.
#[must_use]
pub fn us_gaap_plugin() -> FakeTaxonomy {
    plugin_with_aliases(
        RELEASE,
        vec![declaration(
            FAMILY,
            "http://fasb.org/us-gaap/{release}",
            true,
        )],
        vec![revenue_aliases()],
    )
}
