//! # US-GAAP Taxonomy
//!
//! Provides [`UsGaap`], the plug-in for the US GAAP taxonomy of the FASB and the other SEC
//! taxonomies that US GAAP filings use.
//!
//! The plug-in builds its data from the tables of one FASB release, which the generator
//! writes into [`taxonomy`]. It resolves the families `us-gaap`, `srt`, and `dei`. It declares
//! the other SEC taxonomies without resolving them, so the registry can split their URIs.
//!
//! ## Modules
//!
//! - [`mappings`]: The alias lists of the standard concepts.
//! - [`table`]: The row types of the generated tables.
//! - [`taxonomy`]: The generated tables of the release: calculation arcs and element data.
//! - [`generator`]: Writes [`taxonomy`] from the vendored FASB files. Only with the `generator`
//!   feature.

use std::str::FromStr;

use bigdecimal::BigDecimal;

use crate::core::concept::{ConceptName, NamespaceFamily};
use crate::core::release::{Sha256, TaxonomyRelease, TaxonomyReleaseId};
use crate::core::rule::RuleId;
use crate::core::taxonomy::Taxonomy;
use crate::core::taxonomy::family::{FamilyDeclaration, UriPattern};
use crate::core::taxonomy::reference::{
    CalculationArc, CalculationTree, ConceptAliases, ElementReference,
};

// rustdoc sees the generator too, so the doc check covers it. A normal build leaves it out.
#[cfg(any(doc, feature = "generator"))]
pub mod generator;
pub mod mappings;
pub mod table;
// The generator writes this file. rustfmt must not reflow it, so a run of the generator gives the
// checked-in bytes.
#[rustfmt::skip]
pub mod taxonomy;

/// The family of the US GAAP concepts.
const US_GAAP_FAMILY: &str = "us-gaap";

/// One family that the plug-in declares.
struct FamilySpec {
    /// The family, which is also the prefix of its Notes `version`.
    name: &'static str,
    /// The URI patterns of the family.
    uri_patterns: &'static [&'static str],
    /// Whether the plug-in resolves the facts of the family.
    resolves: bool,
}

/// The families that the plug-in declares. The URI patterns name the hosts of the FASB, of the
/// SEC, and of XBRL US, which published the `us-gaap` and `dei` releases of 2009 and 2010.
const FAMILIES: &[FamilySpec] = &[
    FamilySpec {
        name: US_GAAP_FAMILY,
        uri_patterns: &[
            "http://fasb.org/us-gaap/{release}",
            "http://xbrl.us/us-gaap/{release}",
        ],
        resolves: true,
    },
    FamilySpec {
        name: "srt",
        uri_patterns: &["http://fasb.org/srt/{release}"],
        resolves: true,
    },
    FamilySpec {
        name: "dei",
        uri_patterns: &[
            "http://xbrl.sec.gov/dei/{release}",
            "http://xbrl.us/dei/{release}",
        ],
        resolves: true,
    },
    FamilySpec {
        name: "country",
        uri_patterns: &["http://xbrl.sec.gov/country/{release}"],
        resolves: false,
    },
    FamilySpec {
        name: "currency",
        uri_patterns: &["http://xbrl.sec.gov/currency/{release}"],
        resolves: false,
    },
    FamilySpec {
        name: "cyd",
        uri_patterns: &["http://xbrl.sec.gov/cyd/{release}"],
        resolves: false,
    },
    FamilySpec {
        name: "ecd",
        uri_patterns: &["http://xbrl.sec.gov/ecd/{release}"],
        resolves: false,
    },
    FamilySpec {
        name: "exch",
        uri_patterns: &["http://xbrl.sec.gov/exch/{release}"],
        resolves: false,
    },
    FamilySpec {
        name: "ffd",
        uri_patterns: &["http://xbrl.sec.gov/ffd/{release}"],
        resolves: false,
    },
    FamilySpec {
        name: "invest",
        uri_patterns: &["http://xbrl.sec.gov/invest/{release}"],
        resolves: false,
    },
    FamilySpec {
        name: "naics",
        uri_patterns: &["http://xbrl.sec.gov/naics/{release}"],
        resolves: false,
    },
    FamilySpec {
        name: "rxp",
        uri_patterns: &["http://xbrl.sec.gov/rxp/{release}"],
        resolves: false,
    },
    FamilySpec {
        name: "sic",
        uri_patterns: &["http://xbrl.sec.gov/sic/{release}"],
        resolves: false,
    },
    FamilySpec {
        name: "snj",
        uri_patterns: &["http://xbrl.sec.gov/snj/{release}"],
        resolves: false,
    },
    FamilySpec {
        name: "spac",
        uri_patterns: &["http://xbrl.sec.gov/spac/{release}"],
        resolves: false,
    },
    FamilySpec {
        name: "stpr",
        uri_patterns: &["http://xbrl.sec.gov/stpr/{release}"],
        resolves: false,
    },
];

/// The plug-in for US GAAP filings.
///
/// It holds the release, the family declarations, the alias lists of [`mappings`], the
/// calculation tree, and the element reference data of the generated tables.
///
/// [`Taxonomy::aliases`] returns one alias list per definition of
/// [`mappings::REQUIRED_CONCEPTS`] and [`mappings::OPTIONAL_CONCEPTS`], in that order.
/// [`Taxonomy::elements`] holds the elements of the `us-gaap`, `srt`, and `dei` 2026 schemas,
/// including the ones a schema marks as deprecated. It holds no element that an earlier release
/// removed, such as `SalesRevenueNet`.
///
/// # Examples
///
/// ```
/// use xbrl::core::taxonomy::Taxonomy;
/// use xbrl::us_gaap::UsGaap;
///
/// let us_gaap = UsGaap::new();
///
/// let expected_result = "us-gaap/2026";
///
/// let result = us_gaap.release().id.as_str();
///
/// assert_eq!(result, expected_result);
/// ```
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UsGaap {
    release: TaxonomyRelease,
    families: Vec<FamilyDeclaration>,
    aliases: Vec<ConceptAliases>,
    calculation_tree: CalculationTree,
    elements: Vec<ElementReference>,
}

impl UsGaap {
    /// Creates a [`UsGaap`] from the generated tables and the definitions of [`mappings`].
    ///
    /// # Panics
    ///
    /// Panics if a value of the generated tables, of the family declarations, or of the
    /// definitions breaks the rule of its type. The unit tests build the plug-in, so a bad value
    /// fails them.
    #[must_use]
    pub fn new() -> Self {
        let release_id = TaxonomyReleaseId::new(taxonomy::RELEASE_ID)
            .expect("Given the generated release ID, the creation should always succeed");
        let checksum = Sha256::from_str(taxonomy::RELEASE_CHECKSUM)
            .expect("Given the generated hex digest, the parse should always succeed");
        Self {
            release: TaxonomyRelease {
                id: release_id.clone(),
                checksum,
            },
            families: FAMILIES.iter().map(declaration).collect(),
            aliases: mappings::REQUIRED_CONCEPTS
                .iter()
                .chain(mappings::OPTIONAL_CONCEPTS)
                .map(concept_aliases)
                .collect(),
            calculation_tree: CalculationTree {
                arcs: taxonomy::CALCULATION_ARCS
                    .iter()
                    .map(calculation_arc)
                    .collect(),
            },
            elements: taxonomy::ELEMENTS
                .iter()
                .map(|row| element_reference(row, &release_id))
                .collect(),
        }
    }
}

impl Default for UsGaap {
    fn default() -> Self {
        Self::new()
    }
}

impl Taxonomy for UsGaap {
    fn release(&self) -> &TaxonomyRelease {
        &self.release
    }

    fn families(&self) -> &[FamilyDeclaration] {
        &self.families
    }

    fn aliases(&self) -> &[ConceptAliases] {
        &self.aliases
    }

    fn calculation_tree(&self) -> &CalculationTree {
        &self.calculation_tree
    }

    /// Finds the element by a binary search, because the generator sorts the elements by family
    /// and name.
    fn element(&self, concept: &ConceptName) -> Option<&ElementReference> {
        let key = (concept.family().as_str(), concept.local_name());
        self.elements
            .binary_search_by(|element| {
                (
                    element.concept.family().as_str(),
                    element.concept.local_name(),
                )
                    .cmp(&key)
            })
            .ok()
            .map(|index| &self.elements[index])
    }

    fn elements(&self) -> &[ElementReference] {
        &self.elements
    }
}

/// Returns the family of the text `name`.
fn family(name: &str) -> NamespaceFamily {
    NamespaceFamily::new(name)
        .expect("Given a family of the plug-in or the tables, the creation should always succeed")
}

/// Returns the concept `name` of the family `family_name`.
fn concept(family_name: &str, name: &str) -> ConceptName {
    ConceptName::new(family(family_name), name)
        .expect("Given a concept name of the tables, the creation should always succeed")
}

/// Returns the declaration of one family of [`FAMILIES`].
fn declaration(spec: &FamilySpec) -> FamilyDeclaration {
    let uri_patterns = spec
        .uri_patterns
        .iter()
        .map(|pattern| {
            UriPattern::new(pattern)
                .expect("Given a pattern of the plug-in, the creation should always succeed")
        })
        .collect();
    FamilyDeclaration::new(family(spec.name), uri_patterns, spec.name, spec.resolves)
        .expect("Given a declaration of the plug-in, the creation should always succeed")
}

/// Returns the alias list of one definition of [`mappings`].
fn concept_aliases(definition: &mappings::ConceptDefinition) -> ConceptAliases {
    ConceptAliases {
        element: definition.canonical_element(),
        rule: RuleId::from_str(definition.alias_rule())
            .expect("Given a rule ID of the definitions, the parse should always succeed"),
        concepts: definition
            .xbrl_keys()
            .iter()
            .map(|name| concept(US_GAAP_FAMILY, name))
            .collect(),
    }
}

/// Returns the calculation arc of one row of the tables.
fn calculation_arc(row: &table::CalculationRow) -> CalculationArc {
    CalculationArc {
        role: row.role.to_string(),
        parent: concept(row.parent_family, row.parent_name),
        child: concept(row.child_family, row.child_name),
        weight: BigDecimal::from_str(row.weight)
            .expect("Given a weight of the tables, the parse should always succeed"),
    }
}

/// Returns the element reference of one row of the tables.
fn element_reference(row: &table::ElementRow, release: &TaxonomyReleaseId) -> ElementReference {
    ElementReference {
        concept: concept(row.family, row.name),
        release: release.clone(),
        label: row.label.to_string(),
        data_type: row.data_type.to_string(),
        balance: row.balance,
        period_type: row.period_type,
    }
}

#[cfg(test)]
mod tests {
    use pretty_assertions::assert_eq;

    use super::*;
    use crate::core::alias::resolve_aliases;
    use crate::core::candidate::CandidateValue;
    use crate::core::confidence::Confidence;
    use crate::core::dedup::more_precise;
    use crate::core::elements::CanonicalElement;
    use crate::core::fact::{AsIsFact, Decimals, TaxonomyVersion};
    use crate::core::taxonomy::reference::{Balance, PeriodType};
    use crate::core::taxonomy::registry::Registry;
    use crate::tests::fixtures::sample_fact::{date, fact, source};

    const fn implements_auto_traits<T: Sized + Send + Sync + Unpin>() {}
    #[test]
    const fn should_implement_auto_traits_for_us_gaap() {
        implements_auto_traits::<UsGaap>();
    }

    fn release_id() -> TaxonomyReleaseId {
        TaxonomyReleaseId::new("us-gaap/2026")
            .expect("Given a hardcoded release ID with a year, the creation should always succeed")
    }

    #[test]
    fn should_return_release_with_checksum_of_joined_manifests_when_created() {
        let us_gaap = UsGaap::new();

        let expected_result = TaxonomyRelease {
            id: release_id(),
            checksum: Sha256::from_str(
                "a92083bfe2d99f604c258a3a192602c4e8c8292d08f51cd3d34db30b80353b3c",
            )
            .expect("Given 64 hardcoded hex digits, the parse should always succeed"),
        };

        let result = us_gaap.release().clone();

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_resolve_us_gaap_srt_and_dei_when_created() {
        let us_gaap = UsGaap::new();

        let expected_result = vec!["us-gaap", "srt", "dei"];

        let result: Vec<&str> = us_gaap
            .families()
            .iter()
            .filter(|declaration| declaration.resolves())
            .map(|declaration| declaration.family().as_str())
            .collect();

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_split_uri_of_old_release_when_registry_holds_us_gaap() {
        let us_gaap = UsGaap::new();
        let registry = Registry::new(vec![&us_gaap])
            .expect("Given one plug-in with distinct families, the registry should always start");
        let uri = "http://xbrl.us/us-gaap/2009-01-31";

        let expected_result = Some((
            family("us-gaap"),
            TaxonomyVersion::new("us-gaap/2009").expect(
                "Given a hardcoded version with a year, the creation should always succeed",
            ),
        ));

        let result = registry.split_uri(uri);

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_split_notes_version_of_declared_family_when_plugin_does_not_resolve_it() {
        let us_gaap = UsGaap::new();
        let registry = Registry::new(vec![&us_gaap])
            .expect("Given one plug-in with distinct families, the registry should always start");
        let notes_version = "ecd/2026";

        let expected_result = Some((
            family("ecd"),
            TaxonomyVersion::new("ecd/2026").expect(
                "Given a hardcoded version with a year, the creation should always succeed",
            ),
        ));

        let result = registry.split_notes_version(notes_version);

        assert_eq!(result, expected_result);
    }

    fn rule(text: &str) -> RuleId {
        RuleId::from_str(text).expect(
            "Given a hardcoded rule ID in the documented form, the parse should always succeed",
        )
    }

    /// Runs `core/dedup/more-precise@1` and tiers 1 and 2 on `facts`, and returns the standard
    /// concept, the rule ID, and the confidence of each candidate. A blocked candidate has no
    /// confidence.
    fn resolve(
        facts: &[AsIsFact],
        registry: &Registry<'_>,
    ) -> Vec<(CanonicalElement, RuleId, Option<Confidence>)> {
        resolve_aliases(&more_precise(facts), registry)
            .candidates
            .into_iter()
            .map(|candidate| {
                let confidence = match candidate.value {
                    CandidateValue::Reported { confidence, .. } => Some(confidence),
                    CandidateValue::Blocked => None,
                };
                (candidate.concept, candidate.rule, confidence)
            })
            .collect()
    }

    #[test]
    fn should_return_rule_id_of_each_alias_list_when_created() {
        let us_gaap = UsGaap::new();

        let expected_result = vec![
            (CanonicalElement::Revenue, rule("us-gaap/alias/revenue@1")),
            (
                CanonicalElement::OperatingIncome,
                rule("us-gaap/alias/operating-income@1"),
            ),
            (
                CanonicalElement::NetIncome,
                rule("us-gaap/alias/net-income@1"),
            ),
            (CanonicalElement::Assets, rule("us-gaap/alias/assets@1")),
            (
                CanonicalElement::Liabilities,
                rule("us-gaap/alias/liabilities@1"),
            ),
            (CanonicalElement::Equity, rule("us-gaap/alias/equity@1")),
            (
                CanonicalElement::OperatingCashFlow,
                rule("us-gaap/alias/operating-cash-flow@1"),
            ),
            (
                CanonicalElement::InvestingCashFlow,
                rule("us-gaap/alias/investing-cash-flow@1"),
            ),
            (
                CanonicalElement::FinancingCashFlow,
                rule("us-gaap/alias/financing-cash-flow@1"),
            ),
            (
                CanonicalElement::CurrentAssets,
                rule("us-gaap/alias/current-assets@1"),
            ),
            (
                CanonicalElement::NonCurrentAssets,
                rule("us-gaap/alias/non-current-assets@1"),
            ),
            (
                CanonicalElement::CashAndEquivalents,
                rule("us-gaap/alias/cash-and-equivalents@1"),
            ),
            (
                CanonicalElement::ShortTermInvestments,
                rule("us-gaap/alias/short-term-investments@1"),
            ),
            (
                CanonicalElement::AccountsReceivable,
                rule("us-gaap/alias/accounts-receivable@1"),
            ),
            (
                CanonicalElement::Inventory,
                rule("us-gaap/alias/inventory@1"),
            ),
            (
                CanonicalElement::PropertyPlantEquipment,
                rule("us-gaap/alias/property-plant-equipment@1"),
            ),
            (CanonicalElement::Goodwill, rule("us-gaap/alias/goodwill@1")),
            (
                CanonicalElement::IntangibleAssets,
                rule("us-gaap/alias/intangible-assets@1"),
            ),
            (
                CanonicalElement::CurrentLiabilities,
                rule("us-gaap/alias/current-liabilities@1"),
            ),
            (
                CanonicalElement::NonCurrentLiabilities,
                rule("us-gaap/alias/non-current-liabilities@1"),
            ),
            (
                CanonicalElement::LongTermDebt,
                rule("us-gaap/alias/long-term-debt@1"),
            ),
            (
                CanonicalElement::RetainedEarnings,
                rule("us-gaap/alias/retained-earnings@1"),
            ),
        ];

        let result: Vec<(CanonicalElement, RuleId)> = us_gaap
            .aliases()
            .iter()
            .map(|aliases| (aliases.element, aliases.rule.clone()))
            .collect();

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_find_reference_data_for_every_alias_except_the_removed_ones_when_created() {
        let us_gaap = UsGaap::new();

        // `xbrl/DESIGN.md`, "Release in Use Today", names these two as not in the 2026 release.
        let expected_result = vec![
            concept("us-gaap", "IncomeLossFromOperations"),
            concept("us-gaap", "SalesRevenueNet"),
        ];

        let mut result: Vec<ConceptName> = us_gaap
            .aliases()
            .iter()
            .flat_map(|aliases| aliases.concepts.iter())
            .filter(|concept| us_gaap.element(concept).is_none())
            .cloned()
            .collect();
        result.sort_by(|left, right| left.local_name().cmp(right.local_name()));

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_return_revenue_concepts_in_priority_order_when_created() {
        let us_gaap = UsGaap::new();

        let expected_result = Some(&ConceptAliases {
            element: CanonicalElement::Revenue,
            rule: rule("us-gaap/alias/revenue@1"),
            concepts: vec![
                concept("us-gaap", "Revenues"),
                concept(
                    "us-gaap",
                    "RevenueFromContractWithCustomerExcludingAssessedTax",
                ),
                concept("us-gaap", "SalesRevenueNet"),
                concept(
                    "us-gaap",
                    "RevenueFromContractWithCustomerIncludingAssessedTax",
                ),
            ],
        });

        let result = us_gaap
            .aliases()
            .iter()
            .find(|aliases| aliases.element == CanonicalElement::Revenue);

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_return_equity_concepts_in_priority_order_when_created() {
        let us_gaap = UsGaap::new();

        let expected_result = Some(&ConceptAliases {
            element: CanonicalElement::Equity,
            rule: rule("us-gaap/alias/equity@1"),
            concepts: vec![
                concept(
                    "us-gaap",
                    "StockholdersEquityIncludingPortionAttributableToNoncontrollingInterest",
                ),
                concept("us-gaap", "StockholdersEquity"),
            ],
        });

        let result = us_gaap
            .aliases()
            .iter()
            .find(|aliases| aliases.element == CanonicalElement::Equity);

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_give_revenue_rule_id_when_registry_with_us_gaap_resolves_revenues() {
        let us_gaap = UsGaap::new();
        let registry = Registry::new(vec![&us_gaap])
            .expect("Given one plug-in with distinct families, the registry should always start");
        let filing = source("0000320193-24-000123", date(2024, 11, 1));
        let facts = vec![fact(&filing, "Revenues", "391035000000")];

        let expected_result = vec![(
            CanonicalElement::Revenue,
            rule("us-gaap/alias/revenue@1"),
            Some(Confidence::Exact),
        )];

        let result = resolve(&facts, &registry);

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_give_synonym_with_revenue_rule_id_when_registry_with_us_gaap_resolves_later_alias() {
        let us_gaap = UsGaap::new();
        let registry = Registry::new(vec![&us_gaap])
            .expect("Given one plug-in with distinct families, the registry should always start");
        let filing = source("0000320193-24-000123", date(2024, 11, 1));
        let facts = vec![fact(
            &filing,
            "RevenueFromContractWithCustomerExcludingAssessedTax",
            "391035000000",
        )];

        let expected_result = vec![(
            CanonicalElement::Revenue,
            rule("us-gaap/alias/revenue@1"),
            Some(Confidence::Synonym),
        )];

        let result = resolve(&facts, &registry);

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_give_exact_equity_with_noncontrolling_interest_when_registry_with_us_gaap_resolves_both_equity_tags()
     {
        let us_gaap = UsGaap::new();
        let registry = Registry::new(vec![&us_gaap])
            .expect("Given one plug-in with distinct families, the registry should always start");
        let filing = source("0000320193-24-000123", date(2024, 11, 1));
        let facts = vec![
            fact(&filing, "StockholdersEquity", "56950000000"),
            fact(
                &filing,
                "StockholdersEquityIncludingPortionAttributableToNoncontrollingInterest",
                "57950000000",
            ),
        ];

        let expected_result = vec![CandidateValue::Reported {
            value: BigDecimal::from(57_950_000_000_i64),
            decimals: Some(Decimals::Finite(-6)),
            confidence: Confidence::Exact,
        }];

        let result: Vec<CandidateValue> = resolve_aliases(&more_precise(&facts), &registry)
            .candidates
            .into_iter()
            .map(|candidate| candidate.value)
            .collect();

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_return_reference_data_when_concept_is_assets() {
        let us_gaap = UsGaap::new();
        let assets = concept("us-gaap", "Assets");

        let expected_result = Some(ElementReference {
            concept: concept("us-gaap", "Assets"),
            release: release_id(),
            label: "Assets".to_string(),
            data_type: "xbrli:monetaryItemType".to_string(),
            balance: Some(Balance::Debit),
            period_type: PeriodType::Instant,
        });

        let result = us_gaap.element(&assets).cloned();

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_return_reference_data_when_concept_is_deprecated_in_release() {
        let us_gaap = UsGaap::new();
        let deprecated = concept(
            "us-gaap",
            "AccountingForCertainLoansAndDebtSecuritiesAcquiredInTransferDisclosureTextBlock",
        );

        let expected_result = Some(ElementReference {
            concept: concept(
                "us-gaap",
                "AccountingForCertainLoansAndDebtSecuritiesAcquiredInTransferDisclosureTextBlock",
            ),
            release: release_id(),
            label: "Accounting for Certain Loans and Debt Securities Acquired in Transfer \
                    Disclosure [Text Block] (Deprecated 2025)"
                .to_string(),
            data_type: "dtr-types:textBlockItemType".to_string(),
            balance: None,
            period_type: PeriodType::Duration,
        });

        let result = us_gaap.element(&deprecated).cloned();

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_return_no_reference_data_when_concept_was_removed_before_release() {
        let us_gaap = UsGaap::new();
        let removed = concept("us-gaap", "SalesRevenueNet");

        let expected_result = None;

        let result = us_gaap.element(&removed);

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_return_reference_data_when_concept_is_srt() {
        let us_gaap = UsGaap::new();
        let net_in_force = concept("srt", "LifeInsuranceInForceNet");

        let expected_result = Some(ElementReference {
            concept: concept("srt", "LifeInsuranceInForceNet"),
            release: release_id(),
            label: "SEC Schedule, 12-17, Insurance Companies, Reinsurance, Life Insurance in \
                    Force, Net"
                .to_string(),
            data_type: "xbrli:monetaryItemType".to_string(),
            balance: Some(Balance::Credit),
            period_type: PeriodType::Instant,
        });

        let result = us_gaap.element(&net_in_force).cloned();

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_return_reference_data_when_concept_is_dei() {
        let us_gaap = UsGaap::new();
        let trading_symbol = concept("dei", "TradingSymbol");

        let expected_result = Some(ElementReference {
            concept: concept("dei", "TradingSymbol"),
            release: release_id(),
            label: "Trading Symbol".to_string(),
            data_type: "dei:tradingSymbolItemType".to_string(),
            balance: None,
            period_type: PeriodType::Duration,
        });

        let result = us_gaap.element(&trading_symbol).cloned();

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_return_no_reference_data_when_name_exists_only_in_another_family() {
        let us_gaap = UsGaap::new();
        let srt_concept = concept("srt", "Assets");

        let expected_result = None;

        let result = us_gaap.element(&srt_concept);

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_return_every_concept_of_the_three_schemas_when_listing_elements() {
        let us_gaap = UsGaap::new();

        let expected_result = 18_006;

        let result = us_gaap.elements().len();

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_return_arc_with_negative_weight_when_child_is_subtracted() {
        let us_gaap = UsGaap::new();
        let role = "http://fasb.org/us-gaap/role/statement/StatementOfFinancialPositionClassifiedFirstAlternative";
        let parent = concept("srt", "LifeInsuranceInForceNet");
        let child = concept("srt", "LifeInsuranceInForceCeded");

        let expected_result = Some(&CalculationArc {
            role: role.to_string(),
            parent: parent.clone(),
            child: child.clone(),
            weight: BigDecimal::from_str("-1.0")
                .expect("Given a hardcoded decimal, the parse should always succeed"),
        });

        let result = us_gaap
            .calculation_tree()
            .arcs
            .iter()
            .find(|arc| arc.role == role && arc.parent == parent && arc.child == child);

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_find_every_listed_element_when_looking_up_its_concept() {
        let us_gaap = UsGaap::new();

        // The list has 18,006 elements, too many to write out, so the test compares it with a
        // lookup of each concept.
        let expected_result: Vec<Option<&ElementReference>> =
            us_gaap.elements().iter().map(Some).collect();

        let result: Vec<Option<&ElementReference>> = us_gaap
            .elements()
            .iter()
            .map(|element| us_gaap.element(&element.concept))
            .collect();

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_find_reference_data_for_every_concept_of_calculation_tree() {
        let us_gaap = UsGaap::new();

        let expected_result: Vec<&ConceptName> = Vec::new();

        let result: Vec<&ConceptName> = us_gaap
            .calculation_tree()
            .arcs
            .iter()
            .flat_map(|arc| [&arc.parent, &arc.child])
            .filter(|concept| us_gaap.element(concept).is_none())
            .collect();

        assert_eq!(result, expected_result);
    }
}
