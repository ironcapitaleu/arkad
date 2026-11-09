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

use std::collections::BTreeSet;
use std::str::FromStr;

use bigdecimal::BigDecimal;

use crate::core::concept::{ConceptName, NamespaceFamily};
use crate::core::release::{Sha256, TaxonomyRelease, TaxonomyReleaseId};
use crate::core::rule::RuleId;
use crate::core::taxonomy::Taxonomy;
use crate::core::taxonomy::family::{FamilyDeclaration, UriPattern};
use crate::core::taxonomy::reference::{
    CalculationArc, CalculationTree, ConceptAliases, ElementReference, PartAliases,
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
        let calculation_tree = CalculationTree {
            arcs: taxonomy::CALCULATION_ARCS
                .iter()
                .map(calculation_arc)
                .collect(),
        };
        Self {
            release: TaxonomyRelease {
                id: release_id.clone(),
                checksum,
            },
            families: FAMILIES.iter().map(declaration).collect(),
            aliases: mappings::REQUIRED_CONCEPTS
                .iter()
                .chain(mappings::OPTIONAL_CONCEPTS)
                .map(|definition| concept_aliases(definition, &calculation_tree))
                .collect(),
            calculation_tree,
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
fn concept_aliases(
    definition: &mappings::ConceptDefinition,
    tree: &CalculationTree,
) -> ConceptAliases {
    ConceptAliases {
        element: definition.canonical_element(),
        rule: RuleId::from_str(definition.alias_rule())
            .expect("Given a rule ID of the definitions, the parse should always succeed"),
        concepts: definition
            .xbrl_keys()
            .iter()
            .map(|name| concept(US_GAAP_FAMILY, name))
            .collect(),
        parts: mappings::PART_CONCEPTS
            .iter()
            .find(|parts| parts.canonical_element() == definition.canonical_element())
            .map(|parts| part_aliases(parts, tree)),
    }
}

/// Returns the parts of one definition of [`mappings::PART_CONCEPTS`].
///
/// The other parts hold each concept of `other_parts` in table order. Then each root of
/// `other_part_trees` follows, with the concepts below it in `tree` in name order. Each concept
/// appears once.
fn part_aliases(definition: &mappings::PartDefinition, tree: &CalculationTree) -> PartAliases {
    let concepts = |names: &[&str]| -> Vec<ConceptName> {
        names
            .iter()
            .map(|name| concept(US_GAAP_FAMILY, name))
            .collect()
    };
    let mut other_parts = concepts(definition.other_parts());
    for root in concepts(definition.other_part_trees()) {
        let below = below(&root, tree);
        other_parts.push(root);
        other_parts.extend(below);
    }
    let mut seen = BTreeSet::new();
    other_parts.retain(|concept| seen.insert(concept.clone()));
    PartAliases {
        rule: RuleId::from_str(definition.part_rule())
            .expect("Given a rule ID of the definitions, the parse should always succeed"),
        parts: concepts(definition.parts()),
        other_parts,
    }
}

/// Returns each concept below `root` in `tree`, in every role, in name order.
fn below(root: &ConceptName, tree: &CalculationTree) -> BTreeSet<ConceptName> {
    let mut found = BTreeSet::new();
    let mut open = vec![root];
    while let Some(parent) = open.pop() {
        for arc in tree.arcs.iter().filter(|arc| &arc.parent == parent) {
            if arc.child != *root && found.insert(arc.child.clone()) {
                open.push(&arc.child);
            }
        }
    }
    found
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
    use crate::core::finding::Finding;
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

    // The test writes out every alias list in one place, so a list that goes missing fails it. A
    // split by statement would need a filter, and a list outside every filter would pass.
    #[allow(clippy::too_many_lines)]
    #[test]
    fn should_return_rule_id_of_each_alias_list_when_created() {
        let us_gaap = UsGaap::new();

        let expected_result = vec![
            (CanonicalElement::Revenue, "us-gaap/alias/revenue@1"),
            (
                CanonicalElement::OperatingIncome,
                "us-gaap/alias/operating-income@1",
            ),
            (CanonicalElement::NetIncome, "us-gaap/alias/net-income@1"),
            (CanonicalElement::Assets, "us-gaap/alias/assets@1"),
            (CanonicalElement::Liabilities, "us-gaap/alias/liabilities@1"),
            (CanonicalElement::Equity, "us-gaap/alias/equity@1"),
            (
                CanonicalElement::OperatingCashFlow,
                "us-gaap/alias/operating-cash-flow@1",
            ),
            (
                CanonicalElement::InvestingCashFlow,
                "us-gaap/alias/investing-cash-flow@1",
            ),
            (
                CanonicalElement::FinancingCashFlow,
                "us-gaap/alias/financing-cash-flow@1",
            ),
            (
                CanonicalElement::CurrentAssets,
                "us-gaap/alias/current-assets@1",
            ),
            (
                CanonicalElement::NonCurrentAssets,
                "us-gaap/alias/non-current-assets@1",
            ),
            (
                CanonicalElement::CashAndEquivalents,
                "us-gaap/alias/cash-and-equivalents@1",
            ),
            (
                CanonicalElement::ShortTermInvestments,
                "us-gaap/alias/short-term-investments@1",
            ),
            (
                CanonicalElement::AccountsReceivable,
                "us-gaap/alias/accounts-receivable@1",
            ),
            (CanonicalElement::Inventory, "us-gaap/alias/inventory@1"),
            (
                CanonicalElement::PropertyPlantEquipment,
                "us-gaap/alias/property-plant-equipment@1",
            ),
            (CanonicalElement::Goodwill, "us-gaap/alias/goodwill@1"),
            (
                CanonicalElement::IntangibleAssets,
                "us-gaap/alias/intangible-assets@1",
            ),
            (
                CanonicalElement::CurrentLiabilities,
                "us-gaap/alias/current-liabilities@1",
            ),
            (
                CanonicalElement::NonCurrentLiabilities,
                "us-gaap/alias/non-current-liabilities@1",
            ),
            (
                CanonicalElement::LongTermDebt,
                "us-gaap/alias/long-term-debt@1",
            ),
            (
                CanonicalElement::RetainedEarnings,
                "us-gaap/alias/retained-earnings@1",
            ),
            (
                CanonicalElement::CostOfRevenue,
                "us-gaap/alias/cost-of-revenue@1",
            ),
            (
                CanonicalElement::GrossProfit,
                "us-gaap/alias/gross-profit@1",
            ),
            (
                CanonicalElement::SellingGeneralAdmin,
                "us-gaap/alias/selling-general-admin@1",
            ),
            (
                CanonicalElement::ResearchDevelopment,
                "us-gaap/alias/research-development@1",
            ),
            (
                CanonicalElement::InterestExpense,
                "us-gaap/alias/interest-expense@1",
            ),
            (
                CanonicalElement::EarningsPerShareBasic,
                "us-gaap/alias/earnings-per-share-basic@1",
            ),
            (
                CanonicalElement::EarningsPerShareDiluted,
                "us-gaap/alias/earnings-per-share-diluted@1",
            ),
            (
                CanonicalElement::DepreciationAmortization,
                "us-gaap/alias/depreciation-amortization@1",
            ),
            (
                CanonicalElement::CapitalExpenditures,
                "us-gaap/alias/capital-expenditures@1",
            ),
            (
                CanonicalElement::DividendsPaid,
                "us-gaap/alias/dividends-paid@1",
            ),
            (
                CanonicalElement::ShareRepurchases,
                "us-gaap/alias/share-repurchases@1",
            ),
            (
                CanonicalElement::ShareBasedCompensation,
                "us-gaap/alias/share-based-compensation@1",
            ),
            (
                CanonicalElement::OtherComprehensiveIncome,
                "us-gaap/alias/other-comprehensive-income@1",
            ),
            (
                CanonicalElement::ComprehensiveIncome,
                "us-gaap/alias/comprehensive-income@1",
            ),
        ];

        let result: Vec<(CanonicalElement, &str)> = us_gaap
            .aliases()
            .iter()
            .map(|aliases| (aliases.element, aliases.rule.as_str()))
            .collect();

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_find_reference_data_for_every_alias_and_part_except_the_removed_ones_when_created() {
        let us_gaap = UsGaap::new();

        // `xbrl/DESIGN.md`, "Release in Use Today", names these as not in the 2026 release. The
        // EPS concept is in two lists.
        let expected_result = vec![
            concept("us-gaap", "EarningsPerShareBasicAndDiluted"),
            concept("us-gaap", "EarningsPerShareBasicAndDiluted"),
            concept("us-gaap", "IncomeLossFromOperations"),
            concept("us-gaap", "SalesRevenueNet"),
        ];

        let mut result: Vec<ConceptName> = us_gaap
            .aliases()
            .iter()
            .flat_map(|aliases| {
                let parts = aliases
                    .parts
                    .iter()
                    .flat_map(|parts| parts.parts.iter().chain(&parts.other_parts));
                aliases.concepts.iter().chain(parts)
            })
            .filter(|concept| us_gaap.element(concept).is_none())
            .cloned()
            .collect();
        result.sort_by(|left, right| left.local_name().cmp(right.local_name()));

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_keep_each_part_out_of_its_other_parts_when_created() {
        let us_gaap = UsGaap::new();

        let expected_result: Vec<ConceptName> = Vec::new();

        let result: Vec<ConceptName> = us_gaap
            .aliases()
            .iter()
            .filter_map(|aliases| aliases.parts.as_ref())
            .flat_map(|parts| {
                parts
                    .parts
                    .iter()
                    .filter(|part| parts.other_parts.contains(part))
            })
            .cloned()
            .collect();

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_give_no_revenue_when_a_bank_reports_contract_revenue_next_to_other_noninterest_revenue()
     {
        let us_gaap = UsGaap::new();
        let registry = Registry::new(vec![&us_gaap])
            .expect("Given one plug-in with distinct families, the registry should always start");
        let filing = source("0000320193-24-000123", date(2024, 11, 1));
        let facts = vec![
            fact(
                &filing,
                "RevenueFromContractWithCustomerExcludingAssessedTax",
                "120000000",
            ),
            fact(
                &filing,
                "RevenueNotFromContractWithCustomerExcludingInterestIncome",
                "80000000",
            ),
        ];

        let expected_result: Vec<(CanonicalElement, RuleId, Option<Confidence>)> = Vec::new();

        let result = resolve(&facts, &registry);

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_attach_the_parts_of_each_part_definition_to_its_alias_list_when_created() {
        let us_gaap = UsGaap::new();

        let expected_result = vec![
            CanonicalElement::Revenue,
            CanonicalElement::NetIncome,
            CanonicalElement::Equity,
            CanonicalElement::CashAndEquivalents,
            CanonicalElement::ShortTermInvestments,
            CanonicalElement::IntangibleAssets,
            CanonicalElement::CostOfRevenue,
            CanonicalElement::SellingGeneralAdmin,
            CanonicalElement::ResearchDevelopment,
            CanonicalElement::InterestExpense,
            CanonicalElement::DepreciationAmortization,
            CanonicalElement::CapitalExpenditures,
            CanonicalElement::DividendsPaid,
            CanonicalElement::ShareRepurchases,
            CanonicalElement::OtherComprehensiveIncome,
            CanonicalElement::ComprehensiveIncome,
        ];

        let result: Vec<CanonicalElement> = us_gaap
            .aliases()
            .iter()
            .filter(|aliases| aliases.parts.is_some())
            .map(|aliases| aliases.element)
            .collect();

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_return_revenue_concepts_and_parts_in_priority_order_when_created() {
        let us_gaap = UsGaap::new();

        let expected_result = Some(&ConceptAliases {
            element: CanonicalElement::Revenue,
            rule: rule("us-gaap/alias/revenue@1"),
            concepts: vec![
                concept("us-gaap", "Revenues"),
                concept("us-gaap", "SalesRevenueNet"),
            ],
            parts: Some(PartAliases {
                rule: rule("us-gaap/part/revenue@1"),
                parts: vec![
                    concept(
                        "us-gaap",
                        "RevenueFromContractWithCustomerExcludingAssessedTax",
                    ),
                    concept(
                        "us-gaap",
                        "RevenueFromContractWithCustomerIncludingAssessedTax",
                    ),
                ],
                other_parts: vec![
                    concept(
                        "us-gaap",
                        "ContractuallySpecifiedServicingFeesLateFeesAndAncillaryFeesEarnedInExchangeForServicingFinancialAssets",
                    ),
                    concept("us-gaap", "DirectFinancingLeaseInterestIncome"),
                    concept("us-gaap", "DirectFinancingLeaseRevenue"),
                    concept("us-gaap", "DirectFinancingLeaseVariableLeaseIncome"),
                    concept("us-gaap", "FeeIncome"),
                    concept("us-gaap", "InsuranceAgencyManagementFee"),
                    concept("us-gaap", "InsuranceCommissionsAndFees"),
                    concept("us-gaap", "InterestIncomeExpenseAfterProvisionForLoanLoss"),
                    concept("us-gaap", "NetInvestmentIncome"),
                    concept("us-gaap", "NoninterestIncome"),
                    concept("us-gaap", "RevenueNotFromContractWithCustomer"),
                    concept(
                        "us-gaap",
                        "RevenueNotFromContractWithCustomerExcludingInterestIncome",
                    ),
                    concept("us-gaap", "RevenuesExcludingInterestAndDividends"),
                    concept("us-gaap", "SaleOfTrustAssetsToPayExpenses"),
                    concept("us-gaap", "SalesTypeLeaseInterestIncome"),
                    concept("us-gaap", "SalesTypeLeaseRevenue"),
                    concept("us-gaap", "SalesTypeLeaseVariableLeaseIncome"),
                    concept("us-gaap", "OperatingLeaseLeaseIncome"),
                    concept("us-gaap", "OperatingLeaseLeaseIncomeLeasePayments"),
                    concept("us-gaap", "OperatingLeaseVariableLeaseIncome"),
                    concept("us-gaap", "PremiumsEarnedNet"),
                    concept("us-gaap", "AssumedPremiumsEarned"),
                    concept("us-gaap", "CededPremiumsEarned"),
                    concept("us-gaap", "DirectPremiumsEarned"),
                ],
            }),
        });

        let result = us_gaap
            .aliases()
            .iter()
            .find(|aliases| aliases.element == CanonicalElement::Revenue);

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_return_net_income_concepts_and_parts_in_priority_order_when_created() {
        let us_gaap = UsGaap::new();

        let expected_result = Some(&ConceptAliases {
            element: CanonicalElement::NetIncome,
            rule: rule("us-gaap/alias/net-income@1"),
            concepts: vec![concept("us-gaap", "ProfitLoss")],
            parts: Some(PartAliases {
                rule: rule("us-gaap/part/net-income@1"),
                parts: vec![concept("us-gaap", "NetIncomeLoss")],
                other_parts: vec![
                    concept("us-gaap", "ProfitLoss"),
                    concept(
                        "us-gaap",
                        "NetIncomeLossAttributableToNoncontrollingInterest",
                    ),
                    concept(
                        "us-gaap",
                        "NetIncomeLossAttributableToNonredeemableNoncontrollingInterest",
                    ),
                    concept(
                        "us-gaap",
                        "NetIncomeLossAttributableToRedeemableNoncontrollingInterest",
                    ),
                    concept(
                        "us-gaap",
                        "NoncontrollingInterestInNetIncomeLossJointVenturePartnersNonredeemable",
                    ),
                    concept(
                        "us-gaap",
                        "NoncontrollingInterestInNetIncomeLossJointVenturePartnersRedeemable",
                    ),
                    concept(
                        "us-gaap",
                        "NoncontrollingInterestInNetIncomeLossLimitedPartnershipsNonredeemable",
                    ),
                    concept(
                        "us-gaap",
                        "NoncontrollingInterestInNetIncomeLossLimitedPartnershipsRedeemable",
                    ),
                    concept(
                        "us-gaap",
                        "NoncontrollingInterestInNetIncomeLossOperatingPartnershipsNonredeemable",
                    ),
                    concept(
                        "us-gaap",
                        "NoncontrollingInterestInNetIncomeLossOperatingPartnershipsRedeemable",
                    ),
                    concept(
                        "us-gaap",
                        "NoncontrollingInterestInNetIncomeLossOtherNoncontrollingInterestsNonredeemable",
                    ),
                    concept(
                        "us-gaap",
                        "NoncontrollingInterestInNetIncomeLossOtherNoncontrollingInterestsRedeemable",
                    ),
                    concept(
                        "us-gaap",
                        "NoncontrollingInterestInNetIncomeLossPreferredUnitHoldersNonredeemable",
                    ),
                    concept(
                        "us-gaap",
                        "NoncontrollingInterestInNetIncomeLossPreferredUnitHoldersRedeemable",
                    ),
                ],
            }),
        });

        let result = us_gaap
            .aliases()
            .iter()
            .find(|aliases| aliases.element == CanonicalElement::NetIncome);

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_return_equity_concepts_and_parts_in_priority_order_when_created() {
        let us_gaap = UsGaap::new();

        let expected_result = Some(&ConceptAliases {
            element: CanonicalElement::Equity,
            rule: rule("us-gaap/alias/equity@1"),
            concepts: vec![concept(
                "us-gaap",
                "StockholdersEquityIncludingPortionAttributableToNoncontrollingInterest",
            )],
            parts: Some(PartAliases {
                rule: rule("us-gaap/part/equity@1"),
                parts: vec![concept("us-gaap", "StockholdersEquity")],
                other_parts: vec![
                    concept(
                        "us-gaap",
                        "StockholdersEquityIncludingPortionAttributableToNoncontrollingInterest",
                    ),
                    concept("us-gaap", "MinorityInterest"),
                    concept("us-gaap", "MinorityInterestInJointVentures"),
                    concept("us-gaap", "MinorityInterestInLimitedPartnerships"),
                    concept("us-gaap", "MinorityInterestInOperatingPartnerships"),
                    concept("us-gaap", "MinorityInterestInPreferredUnitHolders"),
                    concept("us-gaap", "NoncontrollingInterestInVariableInterestEntity"),
                    concept("us-gaap", "NonredeemableNoncontrollingInterest"),
                    concept("us-gaap", "OtherMinorityInterests"),
                ],
            }),
        });

        let result = us_gaap
            .aliases()
            .iter()
            .find(|aliases| aliases.element == CanonicalElement::Equity);

        assert_eq!(result, expected_result);
    }

    /// The balance sheet concepts of [`mappings::OPTIONAL_CONCEPTS`].
    const BALANCE_SHEET: [CanonicalElement; 13] = [
        CanonicalElement::CurrentAssets,
        CanonicalElement::NonCurrentAssets,
        CanonicalElement::CashAndEquivalents,
        CanonicalElement::ShortTermInvestments,
        CanonicalElement::AccountsReceivable,
        CanonicalElement::Inventory,
        CanonicalElement::PropertyPlantEquipment,
        CanonicalElement::Goodwill,
        CanonicalElement::IntangibleAssets,
        CanonicalElement::CurrentLiabilities,
        CanonicalElement::NonCurrentLiabilities,
        CanonicalElement::LongTermDebt,
        CanonicalElement::RetainedEarnings,
    ];

    /// The income statement concepts of [`mappings::OPTIONAL_CONCEPTS`].
    const INCOME_STATEMENT: [CanonicalElement; 7] = [
        CanonicalElement::CostOfRevenue,
        CanonicalElement::GrossProfit,
        CanonicalElement::SellingGeneralAdmin,
        CanonicalElement::ResearchDevelopment,
        CanonicalElement::InterestExpense,
        CanonicalElement::EarningsPerShareBasic,
        CanonicalElement::EarningsPerShareDiluted,
    ];

    /// The cash flow concepts of [`mappings::OPTIONAL_CONCEPTS`].
    const CASH_FLOW: [CanonicalElement; 5] = [
        CanonicalElement::DepreciationAmortization,
        CanonicalElement::CapitalExpenditures,
        CanonicalElement::DividendsPaid,
        CanonicalElement::ShareRepurchases,
        CanonicalElement::ShareBasedCompensation,
    ];

    #[test]
    fn should_return_cash_flow_concepts_in_priority_order_when_created() {
        let us_gaap = UsGaap::new();

        let expected_result = vec![
            (
                CanonicalElement::DepreciationAmortization,
                rule("us-gaap/alias/depreciation-amortization@1"),
                vec![concept("us-gaap", "DepreciationDepletionAndAmortization")],
            ),
            (
                CanonicalElement::CapitalExpenditures,
                rule("us-gaap/alias/capital-expenditures@1"),
                vec![concept("us-gaap", "PaymentsToAcquireProductiveAssets")],
            ),
            (
                CanonicalElement::DividendsPaid,
                rule("us-gaap/alias/dividends-paid@1"),
                vec![concept("us-gaap", "PaymentsOfDividends")],
            ),
            (
                CanonicalElement::ShareRepurchases,
                rule("us-gaap/alias/share-repurchases@1"),
                vec![concept("us-gaap", "PaymentsForRepurchaseOfEquity")],
            ),
            (
                CanonicalElement::ShareBasedCompensation,
                rule("us-gaap/alias/share-based-compensation@1"),
                vec![concept("us-gaap", "ShareBasedCompensation")],
            ),
        ];

        let result: Vec<(CanonicalElement, RuleId, Vec<ConceptName>)> = us_gaap
            .aliases()
            .iter()
            .filter(|aliases| CASH_FLOW.contains(&aliases.element))
            .map(|aliases| {
                (
                    aliases.element,
                    aliases.rule.clone(),
                    aliases.concepts.clone(),
                )
            })
            .collect();

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_return_parts_of_depreciation_amortization_in_priority_order_when_created() {
        let us_gaap = UsGaap::new();

        let expected_result = Some(PartAliases {
            rule: rule("us-gaap/part/depreciation-amortization@1"),
            parts: vec![concept("us-gaap", "Depreciation")],
            other_parts: vec![
                concept("us-gaap", "AdjustmentForAmortization"),
                concept(
                    "us-gaap",
                    "AmortizationAndDepreciationOfDecontaminatingAndDecommissioningAssets",
                ),
                concept("us-gaap", "AmortizationOfAcquisitionCosts"),
                concept("us-gaap", "AmortizationOfAdvanceRoyalty"),
                concept("us-gaap", "AmortizationOfDeferredPropertyTaxes"),
                concept("us-gaap", "AmortizationOfDeferredSalesCommissions"),
                concept("us-gaap", "AmortizationOfIntangibleAssets"),
                concept("us-gaap", "AmortizationOfMortgageServicingRightsMSRs"),
                concept("us-gaap", "AmortizationOfNuclearFuelLease"),
                concept("us-gaap", "AmortizationOfPowerContractsEmissionCredits"),
                concept("us-gaap", "AmortizationOfRateDeferral"),
                concept("us-gaap", "AmortizationOfRegulatoryAsset"),
                concept("us-gaap", "AmortizationOfValueOfBusinessAcquiredVOBA"),
                concept("us-gaap", "CostOfGoodsAndServicesSoldAmortization"),
                concept(
                    "us-gaap",
                    "DeferredPolicyAcquisitionCostAmortizationExpense",
                ),
                concept("us-gaap", "DeferredSalesInducementsAmortizationExpense"),
                concept("us-gaap", "FinanceLeaseRightOfUseAssetAmortization"),
                concept("us-gaap", "OtherAmortizationOfDeferredCharges"),
                concept(
                    "us-gaap",
                    "PresentValueOfFutureInsuranceProfitsAmortizationExpense1",
                ),
                concept("us-gaap", "OtherDepreciationAndAmortization"),
                concept("us-gaap", "Depletion"),
                concept("us-gaap", "CostDepletion"),
                concept("us-gaap", "DepletionOfOilAndGasProperties"),
            ],
        });

        let result = us_gaap
            .aliases()
            .iter()
            .find(|aliases| aliases.element == CanonicalElement::DepreciationAmortization)
            .and_then(|aliases| aliases.parts.clone());

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_return_parts_of_cash_flow_concepts_in_priority_order_when_created() {
        let us_gaap = UsGaap::new();

        let expected_result = vec![
            (
                CanonicalElement::CapitalExpenditures,
                PartAliases {
                    rule: rule("us-gaap/part/capital-expenditures@1"),
                    parts: vec![concept(
                        "us-gaap",
                        "PaymentsToAcquirePropertyPlantAndEquipment",
                    )],
                    other_parts: vec![
                        concept("us-gaap", "PaymentForAcquisitionCryptoAsset"),
                        concept("us-gaap", "PaymentsForDelayedTaxExemptExchange"),
                        concept("us-gaap", "PaymentsForSoftware"),
                        concept("us-gaap", "PaymentsToAcquireEquipmentOnLease"),
                        concept("us-gaap", "PaymentsToAcquireIntangibleAssets"),
                        concept("us-gaap", "PaymentsToAcquireMineralRights"),
                        concept("us-gaap", "PaymentsToAcquireOtherProductiveAssets"),
                    ],
                },
            ),
            (
                CanonicalElement::DividendsPaid,
                PartAliases {
                    rule: rule("us-gaap/part/dividends-paid@1"),
                    parts: vec![concept("us-gaap", "PaymentsOfDividendsCommonStock")],
                    other_parts: vec![
                        concept("us-gaap", "PaymentsOfCapitalDistribution"),
                        concept("us-gaap", "PaymentsOfDividendsMinorityInterest"),
                        concept(
                            "us-gaap",
                            "PaymentsOfDividendsPreferredStockAndPreferenceStock",
                        ),
                    ],
                },
            ),
            (
                CanonicalElement::ShareRepurchases,
                PartAliases {
                    rule: rule("us-gaap/part/share-repurchases@1"),
                    parts: vec![concept("us-gaap", "PaymentsForRepurchaseOfCommonStock")],
                    other_parts: vec![
                        concept(
                            "us-gaap",
                            "PaymentsForRepurchaseOfConvertiblePreferredStock",
                        ),
                        concept("us-gaap", "PaymentsForRepurchaseOfInitialPublicOffering"),
                        concept("us-gaap", "PaymentsForRepurchaseOfOtherEquity"),
                        concept(
                            "us-gaap",
                            "PaymentsForRepurchaseOfPreferredStockAndPreferenceStock",
                        ),
                        concept("us-gaap", "PaymentsForRepurchaseOfPrivatePlacement"),
                        concept("us-gaap", "PaymentsForRepurchaseOfWarrants"),
                    ],
                },
            ),
        ];

        let result: Vec<(CanonicalElement, PartAliases)> = us_gaap
            .aliases()
            .iter()
            .filter(|aliases| {
                CASH_FLOW.contains(&aliases.element)
                    && aliases.element != CanonicalElement::DepreciationAmortization
            })
            .filter_map(|aliases| Some((aliases.element, aliases.parts.clone()?)))
            .collect();

        assert_eq!(result, expected_result);
    }

    // The test writes out the expanded other parts, so it passes 100 lines.
    #[allow(clippy::too_many_lines)]
    #[test]
    fn should_return_other_comprehensive_income_concepts_and_parts_in_priority_order_when_created()
    {
        let us_gaap = UsGaap::new();

        let expected_result = Some(&ConceptAliases {
            element: CanonicalElement::OtherComprehensiveIncome,
            rule: rule("us-gaap/alias/other-comprehensive-income@1"),
            concepts: vec![concept("us-gaap", "OtherComprehensiveIncomeLossNetOfTax")],
            parts: Some(PartAliases {
                rule: rule("us-gaap/part/other-comprehensive-income@1"),
                parts: vec![concept(
                    "us-gaap",
                    "OtherComprehensiveIncomeLossNetOfTaxPortionAttributableToParent",
                )],
                other_parts: vec![
                    concept("us-gaap", "OtherComprehensiveIncomeLossNetOfTax"),
                    concept(
                        "us-gaap",
                        "OtherComprehensiveIncomeLossNetOfTaxPortionAttributableToNoncontrollingInterest",
                    ),
                    concept(
                        "us-gaap",
                        "OciEquityMethodInvestmentAfterTaxNoncontrollingInterest",
                    ),
                    concept(
                        "us-gaap",
                        "OciEquityMethodInvestmentBeforeTaxNoncontrollingInterest",
                    ),
                    concept(
                        "us-gaap",
                        "OciEquityMethodInvestmentTaxNoncontrollingInterest",
                    ),
                    concept(
                        "us-gaap",
                        "OciLiabilityForFuturePolicyBenefitGainLossAfterReclassificationAdjustmentAndTaxNoncontrollingInterest",
                    ),
                    concept(
                        "us-gaap",
                        "OciLiabilityForFuturePolicyBenefitGainLossAfterReclassificationAdjustmentBeforeTaxNoncontrollingInterest",
                    ),
                    concept(
                        "us-gaap",
                        "OciLiabilityForFuturePolicyBenefitGainLossAfterReclassificationAdjustmentTaxNoncontrollingInterest",
                    ),
                    concept(
                        "us-gaap",
                        "OciMarketRiskBenefitInstrumentSpecificCreditRiskGainLossAfterAdjustmentsAndTaxNoncontrollingInterest",
                    ),
                    concept(
                        "us-gaap",
                        "OciMarketRiskBenefitInstrumentSpecificCreditRiskGainLossAfterAdjustmentsBeforeTaxNoncontrollingInterest",
                    ),
                    concept(
                        "us-gaap",
                        "OciMarketRiskBenefitInstrumentSpecificCreditRiskGainLossAfterAdjustmentsTaxNoncontrollingInterest",
                    ),
                    concept(
                        "us-gaap",
                        "OtherComprehensiveIncomeAvailableforsaleSecuritiesAdjustmentBeforeTaxPortionAttributableToNoncontrollingInterest",
                    ),
                    concept(
                        "us-gaap",
                        "OtherComprehensiveIncomeAvailableforsaleSecuritiesAdjustmentNetOfTaxPortionAttributableToNoncontrollingInterest",
                    ),
                    concept(
                        "us-gaap",
                        "OtherComprehensiveIncomeAvailableforsaleSecuritiesTaxPortionAttributableToNoncontrollingInterest",
                    ),
                    concept(
                        "us-gaap",
                        "OtherComprehensiveIncomeDefinedBenefitPlansAdjustmentBeforeTaxPortionAttributableToNoncontrollingInterest",
                    ),
                    concept(
                        "us-gaap",
                        "OtherComprehensiveIncomeDefinedBenefitPlansAdjustmentNetOfTaxPortionAttributableToNoncontrollingInterest",
                    ),
                    concept(
                        "us-gaap",
                        "OtherComprehensiveIncomeDefinedBenefitPlansTaxPortionAttributableToNoncontrollingInterest",
                    ),
                    concept(
                        "us-gaap",
                        "OtherComprehensiveIncomeForeignCurrencyTransactionAndTranslationAdjustmentBeforeTaxPortionAttributableToNoncontrollingInterest",
                    ),
                    concept(
                        "us-gaap",
                        "OtherComprehensiveIncomeForeignCurrencyTransactionAndTranslationAdjustmentNetOfTaxPortionAttributableToNoncontrollingInterest",
                    ),
                    concept(
                        "us-gaap",
                        "OtherComprehensiveIncomeForeignCurrencyTranslationAdjustmentTaxPortionAttributableToNoncontrollingInterest",
                    ),
                    concept(
                        "us-gaap",
                        "OtherComprehensiveIncomeLossBeforeTaxPortionAttributableToNoncontrollingInterest",
                    ),
                    concept(
                        "us-gaap",
                        "OtherComprehensiveIncomeLossCashFlowHedgeGainLossAfterReclassificationAndTaxNoncontrollingInterest",
                    ),
                    concept(
                        "us-gaap",
                        "OtherComprehensiveIncomeLossCashFlowHedgeGainLossAfterReclassificationBeforeTaxNoncontrollingInterest",
                    ),
                    concept(
                        "us-gaap",
                        "OtherComprehensiveIncomeLossCashFlowHedgeGainLossAfterReclassificationTaxNoncontrollingInterest",
                    ),
                    concept(
                        "us-gaap",
                        "OtherComprehensiveIncomeLossDerivativeExcludedComponentIncreaseDecreaseAfterAdjustmentsAndTaxNoncontrollingInterest1",
                    ),
                    concept(
                        "us-gaap",
                        "OtherComprehensiveIncomeLossDerivativeExcludedComponentIncreaseDecreaseAfterAdjustmentsBeforeTaxNoncontrollingInterest1",
                    ),
                    concept(
                        "us-gaap",
                        "OtherComprehensiveIncomeLossDerivativeExcludedComponentIncreaseDecreaseAfterAdjustmentsTaxNoncontrollingInterest1",
                    ),
                    concept(
                        "us-gaap",
                        "OtherComprehensiveIncomeLossFinancialLiabilityFairValueOptionAfterReclassificationAdjustmentTaxAttributableToNoncontrollingInterest",
                    ),
                    concept(
                        "us-gaap",
                        "OtherComprehensiveIncomeLossFinancialLiabilityFairValueOptionAfterTaxAndReclassificationAdjustmentAttributableToNoncontrollingInterest",
                    ),
                    concept(
                        "us-gaap",
                        "OtherComprehensiveIncomeLossFinancialLiabilityFairValueOptionBeforeTaxAfterReclassificationAdjustmentAttributableToNoncontrollingInterest",
                    ),
                    concept(
                        "us-gaap",
                        "OtherComprehensiveIncomeLossTaxPortionAttributableToNoncontrollingInterest",
                    ),
                ],
            }),
        });

        let result = us_gaap
            .aliases()
            .iter()
            .find(|aliases| aliases.element == CanonicalElement::OtherComprehensiveIncome);

        assert_eq!(result, expected_result);
    }

    // The test writes out the expanded other parts, so it passes 100 lines.
    #[allow(clippy::too_many_lines)]
    #[test]
    fn should_return_comprehensive_income_concepts_and_parts_in_priority_order_when_created() {
        let us_gaap = UsGaap::new();

        let expected_result = Some(&ConceptAliases {
            element: CanonicalElement::ComprehensiveIncome,
            rule: rule("us-gaap/alias/comprehensive-income@1"),
            concepts: vec![concept(
                "us-gaap",
                "ComprehensiveIncomeNetOfTaxIncludingPortionAttributableToNoncontrollingInterest",
            )],
            parts: Some(PartAliases {
                rule: rule("us-gaap/part/comprehensive-income@1"),
                parts: vec![concept("us-gaap", "ComprehensiveIncomeNetOfTax")],
                other_parts: vec![
                    concept(
                        "us-gaap",
                        "ComprehensiveIncomeNetOfTaxIncludingPortionAttributableToNoncontrollingInterest",
                    ),
                    concept(
                        "us-gaap",
                        "ComprehensiveIncomeNetOfTaxAttributableToNoncontrollingInterest",
                    ),
                    concept(
                        "us-gaap",
                        "NetIncomeLossAttributableToNoncontrollingInterest",
                    ),
                    concept(
                        "us-gaap",
                        "NetIncomeLossAttributableToNonredeemableNoncontrollingInterest",
                    ),
                    concept(
                        "us-gaap",
                        "NetIncomeLossAttributableToRedeemableNoncontrollingInterest",
                    ),
                    concept(
                        "us-gaap",
                        "NoncontrollingInterestInNetIncomeLossJointVenturePartnersNonredeemable",
                    ),
                    concept(
                        "us-gaap",
                        "NoncontrollingInterestInNetIncomeLossJointVenturePartnersRedeemable",
                    ),
                    concept(
                        "us-gaap",
                        "NoncontrollingInterestInNetIncomeLossLimitedPartnershipsNonredeemable",
                    ),
                    concept(
                        "us-gaap",
                        "NoncontrollingInterestInNetIncomeLossLimitedPartnershipsRedeemable",
                    ),
                    concept(
                        "us-gaap",
                        "NoncontrollingInterestInNetIncomeLossOperatingPartnershipsNonredeemable",
                    ),
                    concept(
                        "us-gaap",
                        "NoncontrollingInterestInNetIncomeLossOperatingPartnershipsRedeemable",
                    ),
                    concept(
                        "us-gaap",
                        "NoncontrollingInterestInNetIncomeLossOtherNoncontrollingInterestsNonredeemable",
                    ),
                    concept(
                        "us-gaap",
                        "NoncontrollingInterestInNetIncomeLossOtherNoncontrollingInterestsRedeemable",
                    ),
                    concept(
                        "us-gaap",
                        "NoncontrollingInterestInNetIncomeLossPreferredUnitHoldersNonredeemable",
                    ),
                    concept(
                        "us-gaap",
                        "NoncontrollingInterestInNetIncomeLossPreferredUnitHoldersRedeemable",
                    ),
                    concept(
                        "us-gaap",
                        "OciEquityMethodInvestmentAfterTaxNoncontrollingInterest",
                    ),
                    concept(
                        "us-gaap",
                        "OciEquityMethodInvestmentBeforeTaxNoncontrollingInterest",
                    ),
                    concept(
                        "us-gaap",
                        "OciEquityMethodInvestmentTaxNoncontrollingInterest",
                    ),
                    concept(
                        "us-gaap",
                        "OciLiabilityForFuturePolicyBenefitGainLossAfterReclassificationAdjustmentAndTaxNoncontrollingInterest",
                    ),
                    concept(
                        "us-gaap",
                        "OciLiabilityForFuturePolicyBenefitGainLossAfterReclassificationAdjustmentBeforeTaxNoncontrollingInterest",
                    ),
                    concept(
                        "us-gaap",
                        "OciLiabilityForFuturePolicyBenefitGainLossAfterReclassificationAdjustmentTaxNoncontrollingInterest",
                    ),
                    concept(
                        "us-gaap",
                        "OciMarketRiskBenefitInstrumentSpecificCreditRiskGainLossAfterAdjustmentsAndTaxNoncontrollingInterest",
                    ),
                    concept(
                        "us-gaap",
                        "OciMarketRiskBenefitInstrumentSpecificCreditRiskGainLossAfterAdjustmentsBeforeTaxNoncontrollingInterest",
                    ),
                    concept(
                        "us-gaap",
                        "OciMarketRiskBenefitInstrumentSpecificCreditRiskGainLossAfterAdjustmentsTaxNoncontrollingInterest",
                    ),
                    concept(
                        "us-gaap",
                        "OtherComprehensiveIncomeAvailableforsaleSecuritiesAdjustmentBeforeTaxPortionAttributableToNoncontrollingInterest",
                    ),
                    concept(
                        "us-gaap",
                        "OtherComprehensiveIncomeAvailableforsaleSecuritiesAdjustmentNetOfTaxPortionAttributableToNoncontrollingInterest",
                    ),
                    concept(
                        "us-gaap",
                        "OtherComprehensiveIncomeAvailableforsaleSecuritiesTaxPortionAttributableToNoncontrollingInterest",
                    ),
                    concept(
                        "us-gaap",
                        "OtherComprehensiveIncomeDefinedBenefitPlansAdjustmentBeforeTaxPortionAttributableToNoncontrollingInterest",
                    ),
                    concept(
                        "us-gaap",
                        "OtherComprehensiveIncomeDefinedBenefitPlansAdjustmentNetOfTaxPortionAttributableToNoncontrollingInterest",
                    ),
                    concept(
                        "us-gaap",
                        "OtherComprehensiveIncomeDefinedBenefitPlansTaxPortionAttributableToNoncontrollingInterest",
                    ),
                    concept(
                        "us-gaap",
                        "OtherComprehensiveIncomeForeignCurrencyTransactionAndTranslationAdjustmentBeforeTaxPortionAttributableToNoncontrollingInterest",
                    ),
                    concept(
                        "us-gaap",
                        "OtherComprehensiveIncomeForeignCurrencyTransactionAndTranslationAdjustmentNetOfTaxPortionAttributableToNoncontrollingInterest",
                    ),
                    concept(
                        "us-gaap",
                        "OtherComprehensiveIncomeForeignCurrencyTranslationAdjustmentTaxPortionAttributableToNoncontrollingInterest",
                    ),
                    concept(
                        "us-gaap",
                        "OtherComprehensiveIncomeLossBeforeTaxPortionAttributableToNoncontrollingInterest",
                    ),
                    concept(
                        "us-gaap",
                        "OtherComprehensiveIncomeLossCashFlowHedgeGainLossAfterReclassificationAndTaxNoncontrollingInterest",
                    ),
                    concept(
                        "us-gaap",
                        "OtherComprehensiveIncomeLossCashFlowHedgeGainLossAfterReclassificationBeforeTaxNoncontrollingInterest",
                    ),
                    concept(
                        "us-gaap",
                        "OtherComprehensiveIncomeLossCashFlowHedgeGainLossAfterReclassificationTaxNoncontrollingInterest",
                    ),
                    concept(
                        "us-gaap",
                        "OtherComprehensiveIncomeLossDerivativeExcludedComponentIncreaseDecreaseAfterAdjustmentsAndTaxNoncontrollingInterest1",
                    ),
                    concept(
                        "us-gaap",
                        "OtherComprehensiveIncomeLossDerivativeExcludedComponentIncreaseDecreaseAfterAdjustmentsBeforeTaxNoncontrollingInterest1",
                    ),
                    concept(
                        "us-gaap",
                        "OtherComprehensiveIncomeLossDerivativeExcludedComponentIncreaseDecreaseAfterAdjustmentsTaxNoncontrollingInterest1",
                    ),
                    concept(
                        "us-gaap",
                        "OtherComprehensiveIncomeLossFinancialLiabilityFairValueOptionAfterReclassificationAdjustmentTaxAttributableToNoncontrollingInterest",
                    ),
                    concept(
                        "us-gaap",
                        "OtherComprehensiveIncomeLossFinancialLiabilityFairValueOptionAfterTaxAndReclassificationAdjustmentAttributableToNoncontrollingInterest",
                    ),
                    concept(
                        "us-gaap",
                        "OtherComprehensiveIncomeLossFinancialLiabilityFairValueOptionBeforeTaxAfterReclassificationAdjustmentAttributableToNoncontrollingInterest",
                    ),
                    concept(
                        "us-gaap",
                        "OtherComprehensiveIncomeLossNetOfTaxPortionAttributableToNoncontrollingInterest",
                    ),
                    concept(
                        "us-gaap",
                        "OtherComprehensiveIncomeLossTaxPortionAttributableToNoncontrollingInterest",
                    ),
                ],
            }),
        });

        let result = us_gaap
            .aliases()
            .iter()
            .find(|aliases| aliases.element == CanonicalElement::ComprehensiveIncome);

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_give_synonym_with_comprehensive_income_part_rule_id_when_registry_with_us_gaap_resolves_a_part_alone()
     {
        let us_gaap = UsGaap::new();
        let registry = Registry::new(vec![&us_gaap])
            .expect("Given one plug-in with distinct families, the registry should always start");
        let filing = source("0000320193-24-000123", date(2024, 11, 1));
        let facts = vec![fact(&filing, "ComprehensiveIncomeNetOfTax", "900000000")];

        let expected_result = vec![(
            CanonicalElement::ComprehensiveIncome,
            rule("us-gaap/part/comprehensive-income@1"),
            Some(Confidence::Synonym),
        )];

        let result = resolve(&facts, &registry);

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_give_synonym_with_other_comprehensive_income_part_rule_id_when_registry_with_us_gaap_resolves_a_part_alone()
     {
        let us_gaap = UsGaap::new();
        let registry = Registry::new(vec![&us_gaap])
            .expect("Given one plug-in with distinct families, the registry should always start");
        let filing = source("0000320193-24-000123", date(2024, 11, 1));
        let facts = vec![fact(
            &filing,
            "OtherComprehensiveIncomeLossNetOfTaxPortionAttributableToParent",
            "-30000000",
        )];

        let expected_result = vec![(
            CanonicalElement::OtherComprehensiveIncome,
            rule("us-gaap/part/other-comprehensive-income@1"),
            Some(Confidence::Synonym),
        )];

        let result = resolve(&facts, &registry);

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_give_no_comprehensive_income_when_the_filing_reports_the_parent_part_next_to_the_noncontrolling_net_income()
     {
        let us_gaap = UsGaap::new();
        let registry = Registry::new(vec![&us_gaap])
            .expect("Given one plug-in with distinct families, the registry should always start");
        let filing = source("0000320193-24-000123", date(2024, 11, 1));
        let facts = vec![
            fact(&filing, "ComprehensiveIncomeNetOfTax", "900000000"),
            fact(
                &filing,
                "NetIncomeLossAttributableToNoncontrollingInterest",
                "50000000",
            ),
        ];

        let expected_result: Vec<(CanonicalElement, RuleId, Option<Confidence>)> = Vec::new();

        let result = resolve(&facts, &registry);

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_give_no_other_comprehensive_income_when_the_filing_reports_the_parent_part_next_to_a_noncontrolling_component()
     {
        let us_gaap = UsGaap::new();
        let registry = Registry::new(vec![&us_gaap])
            .expect("Given one plug-in with distinct families, the registry should always start");
        let filing = source("0000320193-24-000123", date(2024, 11, 1));
        let facts = vec![
            fact(
                &filing,
                "OtherComprehensiveIncomeLossNetOfTaxPortionAttributableToParent",
                "-30000000",
            ),
            fact(
                &filing,
                "OtherComprehensiveIncomeForeignCurrencyTransactionAndTranslationAdjustmentNetOfTaxPortionAttributableToNoncontrollingInterest",
                "-2000000",
            ),
        ];

        let expected_result: Vec<(CanonicalElement, RuleId, Option<Confidence>)> = Vec::new();

        let result = resolve(&facts, &registry);

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_give_no_other_comprehensive_income_when_the_filing_reports_the_parent_part_next_to_the_noncontrolling_share()
     {
        let us_gaap = UsGaap::new();
        let registry = Registry::new(vec![&us_gaap])
            .expect("Given one plug-in with distinct families, the registry should always start");
        let filing = source("0000320193-24-000123", date(2024, 11, 1));
        let facts = vec![
            fact(
                &filing,
                "OtherComprehensiveIncomeLossNetOfTaxPortionAttributableToParent",
                "-30000000",
            ),
            fact(
                &filing,
                "OtherComprehensiveIncomeLossNetOfTaxPortionAttributableToNoncontrollingInterest",
                "-2000000",
            ),
        ];

        let expected_result: Vec<(CanonicalElement, RuleId, Option<Confidence>)> = Vec::new();

        let result = resolve(&facts, &registry);

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_give_no_comprehensive_income_when_the_filing_reports_the_parent_part_next_to_the_noncontrolling_share()
     {
        let us_gaap = UsGaap::new();
        let registry = Registry::new(vec![&us_gaap])
            .expect("Given one plug-in with distinct families, the registry should always start");
        let filing = source("0000320193-24-000123", date(2024, 11, 1));
        let facts = vec![
            fact(&filing, "ComprehensiveIncomeNetOfTax", "900000000"),
            fact(
                &filing,
                "ComprehensiveIncomeNetOfTaxAttributableToNoncontrollingInterest",
                "100000000",
            ),
        ];

        let expected_result: Vec<(CanonicalElement, RuleId, Option<Confidence>)> = Vec::new();

        let result = resolve(&facts, &registry);

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_return_income_statement_concepts_in_priority_order_when_created() {
        let us_gaap = UsGaap::new();

        let expected_result = vec![
            (
                CanonicalElement::CostOfRevenue,
                rule("us-gaap/alias/cost-of-revenue@1"),
                vec![concept("us-gaap", "CostOfRevenue")],
            ),
            (
                CanonicalElement::GrossProfit,
                rule("us-gaap/alias/gross-profit@1"),
                vec![concept("us-gaap", "GrossProfit")],
            ),
            (
                CanonicalElement::SellingGeneralAdmin,
                rule("us-gaap/alias/selling-general-admin@1"),
                vec![concept("us-gaap", "SellingGeneralAndAdministrativeExpense")],
            ),
            (
                CanonicalElement::ResearchDevelopment,
                rule("us-gaap/alias/research-development@1"),
                vec![concept("us-gaap", "ResearchAndDevelopmentExpense")],
            ),
            (
                CanonicalElement::InterestExpense,
                rule("us-gaap/alias/interest-expense@1"),
                vec![concept("us-gaap", "InterestExpense")],
            ),
            (
                CanonicalElement::EarningsPerShareBasic,
                rule("us-gaap/alias/earnings-per-share-basic@1"),
                vec![
                    concept("us-gaap", "EarningsPerShareBasic"),
                    concept("us-gaap", "EarningsPerShareBasicAndDiluted"),
                ],
            ),
            (
                CanonicalElement::EarningsPerShareDiluted,
                rule("us-gaap/alias/earnings-per-share-diluted@1"),
                vec![
                    concept("us-gaap", "EarningsPerShareDiluted"),
                    concept("us-gaap", "EarningsPerShareBasicAndDiluted"),
                ],
            ),
        ];

        let result: Vec<(CanonicalElement, RuleId, Vec<ConceptName>)> = us_gaap
            .aliases()
            .iter()
            .filter(|aliases| INCOME_STATEMENT.contains(&aliases.element))
            .map(|aliases| {
                (
                    aliases.element,
                    aliases.rule.clone(),
                    aliases.concepts.clone(),
                )
            })
            .collect();

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_return_parts_of_interest_expense_in_priority_order_when_created() {
        let us_gaap = UsGaap::new();

        let expected_result = Some(PartAliases {
            rule: rule("us-gaap/part/interest-expense@1"),
            parts: vec![concept("us-gaap", "InterestExpenseNonoperating")],
            other_parts: vec![
                concept("us-gaap", "InterestExpenseOperating"),
                concept(
                    "us-gaap",
                    "InterestExpenseBeneficialInterestsIssuedByConsolidatedVariableInterestEntities",
                ),
                concept("us-gaap", "InterestExpenseTradingLiabilities"),
                concept("us-gaap", "InterestExpenseTrustPreferredSecurities"),
                concept("us-gaap", "InterestExpenseDeposits"),
                concept("us-gaap", "InterestExpenseDemandDepositAccounts"),
                concept("us-gaap", "InterestExpenseDomesticDeposits"),
                concept("us-gaap", "InterestExpenseForeignDeposits"),
                concept("us-gaap", "InterestExpenseMoneyMarketDeposits"),
                concept(
                    "us-gaap",
                    "InterestExpenseNOWAccountsMoneyMarketAccountsAndSavingsDeposits",
                ),
                concept(
                    "us-gaap",
                    "InterestExpenseNegotiableOrderOfWithdrawalNOWDeposits",
                ),
                concept("us-gaap", "InterestExpenseOtherDomesticDeposits"),
                concept("us-gaap", "InterestExpenseSavingsDeposits"),
                concept("us-gaap", "InterestExpenseTimeDeposits"),
                concept("us-gaap", "InterestExpenseTimeDeposits100000OrMore"),
                concept("us-gaap", "InterestExpenseTimeDepositsLessThan100000"),
            ],
        });

        let result = us_gaap
            .aliases()
            .iter()
            .find(|aliases| aliases.element == CanonicalElement::InterestExpense)
            .and_then(|aliases| aliases.parts.clone());

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_return_parts_of_income_statement_concepts_in_priority_order_when_created() {
        let us_gaap = UsGaap::new();

        let expected_result = vec![
            (
                CanonicalElement::CostOfRevenue,
                PartAliases {
                    rule: rule("us-gaap/part/cost-of-revenue@1"),
                    parts: vec![concept("us-gaap", "CostOfGoodsAndServicesSold")],
                    other_parts: vec![
                        concept("us-gaap", "AmortizationOfMortgageServicingRightsMSRs"),
                        concept("us-gaap", "AmortizationOfValueOfBusinessAcquiredVOBA"),
                        concept(
                            "us-gaap",
                            "DeferredPolicyAcquisitionCostAmortizationExpense",
                        ),
                        concept("us-gaap", "DeferredSalesInducementsAmortizationExpense"),
                        concept("us-gaap", "FinancingInterestExpense"),
                        concept("us-gaap", "InterestCreditedToPolicyholdersAccountBalances"),
                        concept("us-gaap", "MerchantMarineOperatingDifferentialSubsidy"),
                        concept("us-gaap", "OtherCostOfOperatingRevenue"),
                        concept("us-gaap", "PolicyholderBenefitsAndClaimsIncurredNet"),
                        concept("us-gaap", "PolicyholderDividends"),
                        concept(
                            "us-gaap",
                            "PresentValueOfFutureInsuranceProfitsAmortizationExpense1",
                        ),
                    ],
                },
            ),
            (
                CanonicalElement::SellingGeneralAdmin,
                PartAliases {
                    rule: rule("us-gaap/part/selling-general-admin@1"),
                    parts: vec![concept("us-gaap", "GeneralAndAdministrativeExpense")],
                    other_parts: vec![
                        concept("us-gaap", "MarketingExpense"),
                        concept("us-gaap", "OtherSellingGeneralAndAdministrativeExpense"),
                        concept("us-gaap", "SellingAndMarketingExpense"),
                        concept("us-gaap", "SellingExpense"),
                    ],
                },
            ),
            (
                CanonicalElement::ResearchDevelopment,
                PartAliases {
                    rule: rule("us-gaap/part/research-development@1"),
                    parts: vec![concept(
                        "us-gaap",
                        "ResearchAndDevelopmentExpenseExcludingAcquiredInProcessCost",
                    )],
                    other_parts: vec![
                        concept(
                            "us-gaap",
                            "ResearchAndDevelopmentAssetAcquiredOtherThanThroughBusinessCombinationWrittenOff",
                        ),
                        concept(
                            "us-gaap",
                            "ResearchAndDevelopmentExpenseSoftwareExcludingAcquiredInProcessCost",
                        ),
                    ],
                },
            ),
        ];

        let result: Vec<(CanonicalElement, PartAliases)> = us_gaap
            .aliases()
            .iter()
            .filter(|aliases| {
                INCOME_STATEMENT.contains(&aliases.element)
                    && aliases.element != CanonicalElement::InterestExpense
            })
            .filter_map(|aliases| Some((aliases.element, aliases.parts.clone()?)))
            .collect();

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_return_balance_sheet_concepts_in_priority_order_when_created() {
        let us_gaap = UsGaap::new();

        let expected_result = vec![
            (
                CanonicalElement::CurrentAssets,
                rule("us-gaap/alias/current-assets@1"),
                vec![concept("us-gaap", "AssetsCurrent")],
            ),
            (
                CanonicalElement::NonCurrentAssets,
                rule("us-gaap/alias/non-current-assets@1"),
                vec![concept("us-gaap", "AssetsNoncurrent")],
            ),
            (
                CanonicalElement::CashAndEquivalents,
                rule("us-gaap/alias/cash-and-equivalents@1"),
                vec![concept("us-gaap", "CashAndCashEquivalentsAtCarryingValue")],
            ),
            (
                CanonicalElement::ShortTermInvestments,
                rule("us-gaap/alias/short-term-investments@1"),
                vec![concept("us-gaap", "ShortTermInvestments")],
            ),
            (
                CanonicalElement::AccountsReceivable,
                rule("us-gaap/alias/accounts-receivable@1"),
                vec![concept("us-gaap", "AccountsReceivableNetCurrent")],
            ),
            (
                CanonicalElement::Inventory,
                rule("us-gaap/alias/inventory@1"),
                vec![concept("us-gaap", "InventoryNet")],
            ),
            (
                CanonicalElement::PropertyPlantEquipment,
                rule("us-gaap/alias/property-plant-equipment@1"),
                vec![concept("us-gaap", "PropertyPlantAndEquipmentNet")],
            ),
            (
                CanonicalElement::Goodwill,
                rule("us-gaap/alias/goodwill@1"),
                vec![concept("us-gaap", "Goodwill")],
            ),
            (
                CanonicalElement::IntangibleAssets,
                rule("us-gaap/alias/intangible-assets@1"),
                vec![concept("us-gaap", "IntangibleAssetsNetExcludingGoodwill")],
            ),
            (
                CanonicalElement::CurrentLiabilities,
                rule("us-gaap/alias/current-liabilities@1"),
                vec![concept("us-gaap", "LiabilitiesCurrent")],
            ),
            (
                CanonicalElement::NonCurrentLiabilities,
                rule("us-gaap/alias/non-current-liabilities@1"),
                vec![concept("us-gaap", "LiabilitiesNoncurrent")],
            ),
            (
                CanonicalElement::LongTermDebt,
                rule("us-gaap/alias/long-term-debt@1"),
                vec![concept("us-gaap", "LongTermDebtNoncurrent")],
            ),
            (
                CanonicalElement::RetainedEarnings,
                rule("us-gaap/alias/retained-earnings@1"),
                vec![concept("us-gaap", "RetainedEarningsAccumulatedDeficit")],
            ),
        ];

        let result: Vec<(CanonicalElement, RuleId, Vec<ConceptName>)> = us_gaap
            .aliases()
            .iter()
            .filter(|aliases| BALANCE_SHEET.contains(&aliases.element))
            .map(|aliases| {
                (
                    aliases.element,
                    aliases.rule.clone(),
                    aliases.concepts.clone(),
                )
            })
            .collect();

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_return_parts_of_balance_sheet_concepts_in_priority_order_when_created() {
        let us_gaap = UsGaap::new();

        let expected_result = vec![
            (
                CanonicalElement::CashAndEquivalents,
                PartAliases {
                    rule: rule("us-gaap/part/cash-and-equivalents@1"),
                    parts: vec![concept("us-gaap", "Cash")],
                    other_parts: vec![
                        concept("us-gaap", "DueFromBanks"),
                        concept("us-gaap", "InterestBearingDepositsInBanks"),
                        concept("us-gaap", "CashEquivalentsAtCarryingValue"),
                        concept("us-gaap", "AssetBackedSecuritiesAtCarryingValue"),
                        concept("us-gaap", "CertificatesOfDepositAtCarryingValue"),
                        concept("us-gaap", "CommercialPaperAtCarryingValue"),
                        concept("us-gaap", "CreditAndDebitCardReceivablesAtCarryingValue"),
                        concept("us-gaap", "MoneyMarketFundsAtCarryingValue"),
                        concept("us-gaap", "MunicipalDebtSecuritiesAtCarryingValue"),
                        concept("us-gaap", "OtherCashEquivalentsAtCarryingValue"),
                        concept("us-gaap", "TimeDepositsAtCarryingValue"),
                        concept("us-gaap", "USGovernmentAgenciesSecuritiesAtCarryingValue"),
                        concept("us-gaap", "USGovernmentSecuritiesAtCarryingValue"),
                    ],
                },
            ),
            (
                CanonicalElement::ShortTermInvestments,
                PartAliases {
                    rule: rule("us-gaap/part/short-term-investments@1"),
                    parts: vec![concept("us-gaap", "MarketableSecuritiesCurrent")],
                    other_parts: vec![
                        concept("us-gaap", "EquitySecuritiesFvNi"),
                        concept("us-gaap", "OtherShortTermInvestments"),
                        concept("us-gaap", "DebtSecuritiesCurrent"),
                        concept("us-gaap", "AvailableForSaleSecuritiesDebtSecuritiesCurrent"),
                        concept(
                            "us-gaap",
                            "DebtSecuritiesHeldToMaturityAllowanceForCreditLossCurrent",
                        ),
                        concept(
                            "us-gaap",
                            "DebtSecuritiesHeldToMaturityAmortizedCostAfterAllowanceForCreditLossCurrent",
                        ),
                        concept("us-gaap", "HeldToMaturitySecuritiesCurrent"),
                        concept("us-gaap", "TradingSecuritiesDebt"),
                    ],
                },
            ),
            (
                CanonicalElement::IntangibleAssets,
                PartAliases {
                    rule: rule("us-gaap/part/intangible-assets@1"),
                    parts: vec![concept("us-gaap", "FiniteLivedIntangibleAssetsNet")],
                    other_parts: vec![
                        concept(
                            "us-gaap",
                            "IndefiniteLivedIntangibleAssetsExcludingGoodwill",
                        ),
                        concept("us-gaap", "IndefiniteLivedContractualRights"),
                        concept("us-gaap", "IndefiniteLivedFranchiseRights"),
                        concept("us-gaap", "IndefiniteLivedLicenseAgreements"),
                        concept("us-gaap", "IndefiniteLivedTradeNames"),
                        concept("us-gaap", "IndefiniteLivedTrademarks"),
                        concept("us-gaap", "OtherIndefiniteLivedIntangibleAssets"),
                    ],
                },
            ),
        ];

        let result: Vec<(CanonicalElement, PartAliases)> = us_gaap
            .aliases()
            .iter()
            .filter(|aliases| BALANCE_SHEET.contains(&aliases.element))
            .filter_map(|aliases| Some((aliases.element, aliases.parts.clone()?)))
            .collect();

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
    fn should_give_synonym_with_revenue_part_rule_id_when_registry_with_us_gaap_resolves_a_part_alone()
     {
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
            rule("us-gaap/part/revenue@1"),
            Some(Confidence::Synonym),
        )];

        let result = resolve(&facts, &registry);

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_give_no_revenue_when_a_lessor_reports_contract_revenue_next_to_lease_income() {
        let us_gaap = UsGaap::new();
        let registry = Registry::new(vec![&us_gaap])
            .expect("Given one plug-in with distinct families, the registry should always start");
        let filing = source("0000320193-24-000123", date(2024, 11, 1));
        let facts = vec![
            fact(
                &filing,
                "RevenueFromContractWithCustomerExcludingAssessedTax",
                "40000000",
            ),
            fact(&filing, "OperatingLeaseLeaseIncome", "960000000"),
        ];

        let expected_result: Vec<(CanonicalElement, RuleId, Option<Confidence>)> = Vec::new();

        let result = resolve(&facts, &registry);

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_give_no_short_term_investments_when_a_concept_below_an_other_part_is_reported() {
        let us_gaap = UsGaap::new();
        let registry = Registry::new(vec![&us_gaap])
            .expect("Given one plug-in with distinct families, the registry should always start");
        let filing = source("0000320193-24-000123", date(2024, 11, 1));
        let facts = vec![
            fact(&filing, "MarketableSecuritiesCurrent", "300000000"),
            fact(
                &filing,
                "AvailableForSaleSecuritiesDebtSecuritiesCurrent",
                "700000000",
            ),
        ];

        let expected_result: Vec<(CanonicalElement, RuleId, Option<Confidence>)> = Vec::new();

        let result = resolve(&facts, &registry);

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_give_no_equity_when_the_filing_reports_stockholders_equity_next_to_minority_interest()
    {
        let us_gaap = UsGaap::new();
        let registry = Registry::new(vec![&us_gaap])
            .expect("Given one plug-in with distinct families, the registry should always start");
        let filing = source("0000320193-24-000123", date(2024, 11, 1));
        let facts = vec![
            fact(&filing, "StockholdersEquity", "56950000000"),
            fact(&filing, "MinorityInterest", "1000000000"),
        ];

        let expected_result: Vec<(CanonicalElement, RuleId, Option<Confidence>)> = Vec::new();

        let result = resolve(&facts, &registry);

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_give_both_eps_concepts_when_the_filing_reports_only_eps_basic_and_diluted() {
        let us_gaap = UsGaap::new();
        let registry = Registry::new(vec![&us_gaap])
            .expect("Given one plug-in with distinct families, the registry should always start");
        let filing = source("0000320193-15-000123", date(2015, 11, 1));
        let facts = vec![fact(&filing, "EarningsPerShareBasicAndDiluted", "1.25")];

        let expected_result = vec![
            (
                CanonicalElement::EarningsPerShareBasic,
                rule("us-gaap/alias/earnings-per-share-basic@1"),
                Some(Confidence::Synonym),
            ),
            (
                CanonicalElement::EarningsPerShareDiluted,
                rule("us-gaap/alias/earnings-per-share-diluted@1"),
                Some(Confidence::Synonym),
            ),
        ];

        let result = resolve(&facts, &registry);

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_name_the_first_eps_concept_in_the_conflict_when_eps_basic_and_diluted_has_a_conflict()
    {
        let us_gaap = UsGaap::new();
        let registry = Registry::new(vec![&us_gaap])
            .expect("Given one plug-in with distinct families, the registry should always start");
        let filing = source("0000320193-15-000123", date(2015, 11, 1));
        let mut eps = fact(&filing, "EarningsPerShareBasicAndDiluted", "1.25");
        eps.decimals = Some(Decimals::Finite(2));
        let mut other_eps = fact(&filing, "EarningsPerShareBasicAndDiluted", "1.30");
        other_eps.decimals = Some(Decimals::Finite(2));
        other_eps.copy_rank = 1;
        let facts = vec![eps.clone(), other_eps];

        let expected_result = vec![Finding::Conflict {
            entity: eps.context.entity.clone(),
            key: eps.key(),
            ranks: vec![1],
            standard_concept: Some(CanonicalElement::EarningsPerShareBasic),
        }];

        let result = resolve_aliases(&more_precise(&facts), &registry).findings;

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_give_no_selling_general_admin_when_the_filing_reports_g_and_a_next_to_selling_and_marketing()
     {
        let us_gaap = UsGaap::new();
        let registry = Registry::new(vec![&us_gaap])
            .expect("Given one plug-in with distinct families, the registry should always start");
        let filing = source("0000320193-24-000123", date(2024, 11, 1));
        let facts = vec![
            fact(&filing, "GeneralAndAdministrativeExpense", "100000000"),
            fact(&filing, "SellingAndMarketingExpense", "80000000"),
        ];

        let expected_result: Vec<(CanonicalElement, RuleId, Option<Confidence>)> = Vec::new();

        let result = resolve(&facts, &registry);

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_give_no_selling_general_admin_when_the_filing_reports_g_and_a_next_to_a_marketing_line()
     {
        let us_gaap = UsGaap::new();
        let registry = Registry::new(vec![&us_gaap])
            .expect("Given one plug-in with distinct families, the registry should always start");
        let filing = source("0000320193-24-000123", date(2024, 11, 1));
        let facts = vec![
            fact(&filing, "GeneralAndAdministrativeExpense", "100000000"),
            fact(&filing, "MarketingExpense", "80000000"),
        ];

        let expected_result: Vec<(CanonicalElement, RuleId, Option<Confidence>)> = Vec::new();

        let result = resolve(&facts, &registry);

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_give_no_interest_expense_when_a_bank_concept_below_the_operating_part_is_reported() {
        let us_gaap = UsGaap::new();
        let registry = Registry::new(vec![&us_gaap])
            .expect("Given one plug-in with distinct families, the registry should always start");
        let filing = source("0000320193-24-000123", date(2024, 11, 1));
        let facts = vec![
            fact(&filing, "InterestExpenseNonoperating", "40000000"),
            fact(&filing, "InterestExpenseDeposits", "300000000"),
        ];

        let expected_result: Vec<(CanonicalElement, RuleId, Option<Confidence>)> = Vec::new();

        let result = resolve(&facts, &registry);

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_give_no_depreciation_amortization_when_the_filing_reports_depreciation_next_to_oil_and_gas_depletion()
     {
        let us_gaap = UsGaap::new();
        let registry = Registry::new(vec![&us_gaap])
            .expect("Given one plug-in with distinct families, the registry should always start");
        let filing = source("0000320193-24-000123", date(2024, 11, 1));
        let facts = vec![
            fact(&filing, "Depreciation", "100000000"),
            fact(&filing, "DepletionOfOilAndGasProperties", "400000000"),
        ];

        let expected_result: Vec<(CanonicalElement, RuleId, Option<Confidence>)> = Vec::new();

        let result = resolve(&facts, &registry);

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_give_synonym_with_depreciation_amortization_part_rule_id_when_the_filing_reports_depreciation_next_to_financing_cost_amortization()
     {
        let us_gaap = UsGaap::new();
        let registry = Registry::new(vec![&us_gaap])
            .expect("Given one plug-in with distinct families, the registry should always start");
        let filing = source("0000320193-24-000123", date(2024, 11, 1));
        let facts = vec![
            fact(&filing, "Depreciation", "100000000"),
            fact(
                &filing,
                "AmortizationOfFinancingCostsAndDiscounts",
                "5000000",
            ),
        ];

        let expected_result = vec![(
            CanonicalElement::DepreciationAmortization,
            rule("us-gaap/part/depreciation-amortization@1"),
            Some(Confidence::Synonym),
        )];

        let result = resolve(&facts, &registry);

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_give_no_depreciation_amortization_when_the_filing_reports_depreciation_next_to_lease_amortization()
     {
        let us_gaap = UsGaap::new();
        let registry = Registry::new(vec![&us_gaap])
            .expect("Given one plug-in with distinct families, the registry should always start");
        let filing = source("0000320193-24-000123", date(2024, 11, 1));
        let facts = vec![
            fact(&filing, "Depreciation", "100000000"),
            fact(
                &filing,
                "FinanceLeaseRightOfUseAssetAmortization",
                "20000000",
            ),
        ];

        let expected_result: Vec<(CanonicalElement, RuleId, Option<Confidence>)> = Vec::new();

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
