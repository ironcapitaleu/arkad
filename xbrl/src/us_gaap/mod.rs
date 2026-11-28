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
use crate::core::elements::CanonicalElementKind;
use crate::core::release::{Sha256, TaxonomyRelease, TaxonomyReleaseId};
use crate::core::rule::RuleId;
use crate::core::taxonomy::Taxonomy;
use crate::core::taxonomy::family::{FamilyDeclaration, UriPattern};
use crate::core::taxonomy::reference::{
    CalculationArc, CalculationTree, ConceptAliases, ElementReference, LineTotal, PartAliases,
};
use crate::core::taxonomy::segment::{
    MemberRole, ReconciliationRole, SegmentAxes, SegmentAxis, SegmentAxisKind,
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

/// The axis of kind [`SegmentAxisKind::ConsolidationItems`].
const CONSOLIDATION_ITEMS_AXIS: &str = "ConsolidationItems";

/// The segment axes of the plug-in, as written in `dimensions`. `SegmentReportingInformationBySegment`
/// is the business-segment axis of filings of 2011.
const SEGMENT_AXES: &[SegmentAxis] = &[
    SegmentAxis::new("BusinessSegments", SegmentAxisKind::BusinessSegment),
    SegmentAxis::new(
        "SegmentReportingInformationBySegment",
        SegmentAxisKind::BusinessSegment,
    ),
    SegmentAxis::new("Geographical", SegmentAxisKind::Geography),
    SegmentAxis::new("ProductOrService", SegmentAxisKind::Product),
    SegmentAxis::new(
        CONSOLIDATION_ITEMS_AXIS,
        SegmentAxisKind::ConsolidationItems,
    ),
    SegmentAxis::new("Subsegments", SegmentAxisKind::Subsegment),
    SegmentAxis::new(
        "SubsegmentsConsolidationItems",
        SegmentAxisKind::SubsegmentConsolidationItems,
    ),
];

/// The members of [`CONSOLIDATION_ITEMS_AXIS`] that have a role in the reconciliation.
const MEMBER_ROLES: &[MemberRole] = &[
    MemberRole::new(
        CONSOLIDATION_ITEMS_AXIS,
        "OperatingSegments",
        ReconciliationRole::Segment,
    ),
    MemberRole::new(
        CONSOLIDATION_ITEMS_AXIS,
        "IntersegmentElimination",
        ReconciliationRole::Elimination,
    ),
    MemberRole::new(
        CONSOLIDATION_ITEMS_AXIS,
        "ConsolidationEliminations",
        ReconciliationRole::Elimination,
    ),
    MemberRole::new(
        CONSOLIDATION_ITEMS_AXIS,
        "CorporateNonSegment",
        ReconciliationRole::Corporate,
    ),
    MemberRole::new(
        CONSOLIDATION_ITEMS_AXIS,
        "MaterialReconcilingItems",
        ReconciliationRole::Corporate,
    ),
];

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
            aliases: alias_lists(mappings::LINE_TOTAL_CONCEPTS, &calculation_tree),
            calculation_tree,
            elements: taxonomy::ELEMENTS
                .iter()
                .map(|row| element_reference(row, &release_id))
                .collect(),
        }
    }
}

impl UsGaap {
    /// Returns the standard concepts whose definition in [`mappings`] has the flag `required`, in
    /// the order of [`mappings::REQUIRED_CONCEPTS`], then [`mappings::OPTIONAL_CONCEPTS`].
    ///
    /// [`standardize_series`](crate::core::series::standardize_series) takes them to report a
    /// [`Finding::MissingRequired`](crate::core::finding::Finding::MissingRequired).
    #[must_use]
    pub fn required_concepts(&self) -> Vec<CanonicalElementKind> {
        mappings::REQUIRED_CONCEPTS
            .iter()
            .chain(mappings::OPTIONAL_CONCEPTS)
            .filter(|definition| definition.required())
            .map(mappings::ConceptDefinition::canonical_element)
            .collect()
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

impl SegmentAxes for UsGaap {
    fn segment_axes(&self) -> &[SegmentAxis] {
        SEGMENT_AXES
    }

    fn member_roles(&self) -> &[MemberRole] {
        MEMBER_ROLES
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

/// Returns one alias list per definition of [`mappings::REQUIRED_CONCEPTS`] and
/// [`mappings::OPTIONAL_CONCEPTS`], each with its parts of [`mappings::PART_CONCEPTS`] and its line
/// total of `line_totals`.
fn alias_lists(
    line_totals: &[mappings::LineTotalDefinition],
    tree: &CalculationTree,
) -> Vec<ConceptAliases> {
    mappings::REQUIRED_CONCEPTS
        .iter()
        .chain(mappings::OPTIONAL_CONCEPTS)
        .map(|definition| {
            let element = definition.canonical_element();
            let parts = mappings::PART_CONCEPTS
                .iter()
                .find(|parts| parts.canonical_element() == element);
            let line_total = line_totals
                .iter()
                .find(|total| total.canonical_element() == element);
            concept_aliases(definition, parts, line_total, tree)
        })
        .collect()
}

/// Returns the alias list of one definition of [`mappings`], with the parts of `part_definition`
/// and the lines of `line_total_definition`.
///
/// The scopes hold the scopes of the definition, then the scopes of the parts.
fn concept_aliases(
    definition: &mappings::ConceptDefinition,
    part_definition: Option<&mappings::PartDefinition>,
    line_total_definition: Option<&mappings::LineTotalDefinition>,
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
        parts: part_definition.map(|parts| part_aliases(parts, tree)),
        line_total: line_total_definition.map(line_total),
        scopes: definition
            .scopes()
            .iter()
            .chain(
                part_definition
                    .into_iter()
                    .flat_map(mappings::PartDefinition::scopes),
            )
            .map(|(name, scope)| (concept(US_GAAP_FAMILY, name), *scope))
            .collect(),
    }
}

/// Returns the parts of one definition of [`mappings::PART_CONCEPTS`].
///
/// The other parts hold each concept of `other_parts` in table order. Then each root of
/// `other_part_trees` follows, with the concepts below it in `tree` in name order. Each concept
/// appears once. The own other parts of each part count alone, without the concepts below them.
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
        own_other_parts: definition
            .own_other_parts()
            .iter()
            .map(|(part, others)| (concept(US_GAAP_FAMILY, part), concepts(others)))
            .collect(),
        other_part_totals: concepts(definition.other_part_totals()),
    }
}

/// Returns the line total of one definition of [`mappings::LINE_TOTAL_CONCEPTS`].
fn line_total(definition: &mappings::LineTotalDefinition) -> LineTotal {
    LineTotal {
        rule: RuleId::from_str(definition.total_rule())
            .expect("Given a rule ID of the definitions, the parse should always succeed"),
        lines: definition
            .lines()
            .iter()
            .map(|name| concept(US_GAAP_FAMILY, name))
            .collect(),
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
    use crate::core::elements::{
        BalanceSheetConcept, CanonicalElementKind, CashFlowConcept, DebtMaturityConcept,
        IncomeStatementConcept, PerShareConcept,
    };
    use crate::core::fact::{AsIsFact, Decimals, TaxonomyVersion};
    use crate::core::finding::Finding;
    use crate::core::scope::Scope;
    use crate::core::taxonomy::reference::{Balance, PeriodType};
    use crate::core::taxonomy::registry::{Plugin, Registry};
    use crate::tests::fixtures::sample_fact::{apple, date, fact, source};
    use crate::tests::fixtures::sample_taxonomy::{declaration, plugin};

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
    fn should_return_seven_axes_with_kinds_for_us_gaap() {
        let us_gaap = UsGaap::new();

        let expected_result = vec![
            ("BusinessSegments", SegmentAxisKind::BusinessSegment),
            (
                "SegmentReportingInformationBySegment",
                SegmentAxisKind::BusinessSegment,
            ),
            ("Geographical", SegmentAxisKind::Geography),
            ("ProductOrService", SegmentAxisKind::Product),
            ("ConsolidationItems", SegmentAxisKind::ConsolidationItems),
            ("Subsegments", SegmentAxisKind::Subsegment),
            (
                "SubsegmentsConsolidationItems",
                SegmentAxisKind::SubsegmentConsolidationItems,
            ),
        ];

        let result: Vec<(&str, SegmentAxisKind)> = us_gaap
            .segment_axes()
            .iter()
            .map(|axis| (axis.name(), axis.kind()))
            .collect();

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_return_five_member_roles_for_us_gaap() {
        let us_gaap = UsGaap::new();

        let expected_result = vec![
            ("OperatingSegments", ReconciliationRole::Segment),
            ("IntersegmentElimination", ReconciliationRole::Elimination),
            ("ConsolidationEliminations", ReconciliationRole::Elimination),
            ("CorporateNonSegment", ReconciliationRole::Corporate),
            ("MaterialReconcilingItems", ReconciliationRole::Corporate),
        ];

        let result: Vec<(&str, ReconciliationRole)> = us_gaap
            .member_roles()
            .iter()
            .map(|member_role| (member_role.member(), member_role.role()))
            .collect();

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_name_an_axis_of_kind_consolidation_items_for_each_member_role_of_us_gaap() {
        let us_gaap = UsGaap::new();

        let expected_result: Vec<&str> = Vec::new();

        let result: Vec<&str> = us_gaap
            .member_roles()
            .iter()
            .filter(|member_role| {
                !us_gaap.segment_axes().iter().any(|axis| {
                    axis.name() == member_role.axis()
                        && axis.kind() == SegmentAxisKind::ConsolidationItems
                })
            })
            .map(MemberRole::member)
            .collect();

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_route_us_gaap_concept_to_us_gaap_when_registered_with_segment_axes() {
        let us_gaap = UsGaap::new();
        let registry = Registry::with_plugins(vec![Plugin::WithSegmentAxes(&us_gaap)])
            .expect("Given one plug-in with distinct families, the registry should always start");
        let concept = concept("us-gaap", "Revenues");

        let expected_result = Some(release_id());

        let result = registry
            .resolver(concept.family())
            .map(|plugin| plugin.release().id.clone());

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_return_segment_axes_of_us_gaap_when_registered_with_segment_axes() {
        let us_gaap = UsGaap::new();
        let registry = Registry::with_plugins(vec![Plugin::WithSegmentAxes(&us_gaap)])
            .expect("Given one plug-in with distinct families, the registry should always start");
        let us_gaap_family = family("us-gaap");

        let expected_result = Some(SEGMENT_AXES);

        let result = registry
            .segment_axes(&us_gaap_family)
            .map(SegmentAxes::segment_axes);

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_return_segment_axes_of_us_gaap_when_another_plugin_declares_the_family_first() {
        let declarer = plugin(
            "declarer/2025",
            vec![declaration("srt", "http://fasb.org/srt/{release}", false)],
        );
        let us_gaap = UsGaap::new();
        let registry = Registry::with_plugins(vec![
            Plugin::Taxonomy(&declarer),
            Plugin::WithSegmentAxes(&us_gaap),
        ])
        .expect(
            "Given a declaration of srt that agrees with us_gaap and does not resolve, the registry should always start",
        );
        let srt_family = family("srt");

        let expected_result = Some(SEGMENT_AXES);

        let result = registry
            .segment_axes(&srt_family)
            .map(SegmentAxes::segment_axes);

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
    ) -> Vec<(CanonicalElementKind, RuleId, Option<Confidence>)> {
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

    /// Runs `core/dedup/more-precise@1` and tiers 1 and 2 on `facts`, and returns the value of
    /// each candidate of [`BalanceSheetConcept::TemporaryEquity`].
    fn temporary_equity_values(facts: &[AsIsFact], registry: &Registry<'_>) -> Vec<CandidateValue> {
        resolve_aliases(&more_precise(facts), registry)
            .candidates
            .into_iter()
            .filter(|candidate| candidate.concept == BalanceSheetConcept::TemporaryEquity.element())
            .map(|candidate| candidate.value)
            .collect()
    }

    #[test]
    fn should_return_the_11_roll_ups_as_required_concepts_for_us_gaap() {
        let us_gaap = UsGaap::new();

        let expected_result = vec![
            IncomeStatementConcept::Revenue.element(),
            IncomeStatementConcept::NetIncome.element(),
            BalanceSheetConcept::Assets.element(),
            BalanceSheetConcept::Liabilities.element(),
            BalanceSheetConcept::Equity.element(),
            CashFlowConcept::OperatingCashFlow.element(),
            CashFlowConcept::InvestingCashFlow.element(),
            CashFlowConcept::FinancingCashFlow.element(),
            BalanceSheetConcept::LiabilitiesAndEquity.element(),
            IncomeStatementConcept::NetIncomeAttributableToParent.element(),
            CashFlowConcept::NetChangeInCash.element(),
        ];

        let result = us_gaap.required_concepts();

        assert_eq!(result, expected_result);
    }

    // The test writes out every alias list in one place, so a list that goes missing fails it. A
    // split by statement would need a filter, and a list outside every filter would pass.
    #[allow(clippy::too_many_lines)]
    #[test]
    fn should_return_rule_id_of_each_alias_list_when_created() {
        let us_gaap = UsGaap::new();

        let expected_result = vec![
            (
                IncomeStatementConcept::Revenue.element(),
                "us-gaap/alias/revenue@1",
            ),
            (
                IncomeStatementConcept::OperatingIncome.element(),
                "us-gaap/alias/operating-income@1",
            ),
            (
                IncomeStatementConcept::NetIncome.element(),
                "us-gaap/alias/net-income@1",
            ),
            (
                BalanceSheetConcept::Assets.element(),
                "us-gaap/alias/assets@1",
            ),
            (
                BalanceSheetConcept::Liabilities.element(),
                "us-gaap/alias/liabilities@1",
            ),
            (
                BalanceSheetConcept::Equity.element(),
                "us-gaap/alias/equity@1",
            ),
            (
                CashFlowConcept::OperatingCashFlow.element(),
                "us-gaap/alias/operating-cash-flow@1",
            ),
            (
                CashFlowConcept::InvestingCashFlow.element(),
                "us-gaap/alias/investing-cash-flow@1",
            ),
            (
                CashFlowConcept::FinancingCashFlow.element(),
                "us-gaap/alias/financing-cash-flow@1",
            ),
            (
                BalanceSheetConcept::CurrentAssets.element(),
                "us-gaap/alias/current-assets@1",
            ),
            (
                BalanceSheetConcept::NonCurrentAssets.element(),
                "us-gaap/alias/non-current-assets@1",
            ),
            (
                BalanceSheetConcept::CashAndEquivalents.element(),
                "us-gaap/alias/cash-and-equivalents@1",
            ),
            (
                BalanceSheetConcept::ShortTermInvestments.element(),
                "us-gaap/alias/short-term-investments@1",
            ),
            (
                BalanceSheetConcept::AccountsReceivable.element(),
                "us-gaap/alias/accounts-receivable@1",
            ),
            (
                BalanceSheetConcept::Inventory.element(),
                "us-gaap/alias/inventory@1",
            ),
            (
                BalanceSheetConcept::PropertyPlantEquipment.element(),
                "us-gaap/alias/property-plant-equipment@1",
            ),
            (
                BalanceSheetConcept::Goodwill.element(),
                "us-gaap/alias/goodwill@1",
            ),
            (
                BalanceSheetConcept::IntangibleAssets.element(),
                "us-gaap/alias/intangible-assets@1",
            ),
            (
                BalanceSheetConcept::LiabilitiesAndEquity.element(),
                "us-gaap/alias/liabilities-and-equity@1",
            ),
            (
                BalanceSheetConcept::TemporaryEquity.element(),
                "us-gaap/alias/temporary-equity@1",
            ),
            (
                BalanceSheetConcept::CurrentLiabilities.element(),
                "us-gaap/alias/current-liabilities@1",
            ),
            (
                BalanceSheetConcept::NonCurrentLiabilities.element(),
                "us-gaap/alias/non-current-liabilities@1",
            ),
            (
                BalanceSheetConcept::AccountsPayable.element(),
                "us-gaap/alias/accounts-payable@1",
            ),
            (
                BalanceSheetConcept::ShortTermDebt.element(),
                "us-gaap/alias/short-term-debt@1",
            ),
            (
                BalanceSheetConcept::LongTermDebt.element(),
                "us-gaap/alias/long-term-debt@1",
            ),
            (
                BalanceSheetConcept::EquityAttributableToParent.element(),
                "us-gaap/alias/equity-attributable-to-parent@1",
            ),
            (
                BalanceSheetConcept::NoncontrollingInterest.element(),
                "us-gaap/alias/noncontrolling-interest@1",
            ),
            (
                BalanceSheetConcept::AdditionalPaidInCapital.element(),
                "us-gaap/alias/additional-paid-in-capital@1",
            ),
            (
                BalanceSheetConcept::RetainedEarnings.element(),
                "us-gaap/alias/retained-earnings@1",
            ),
            (
                BalanceSheetConcept::TreasuryStock.element(),
                "us-gaap/alias/treasury-stock@1",
            ),
            (
                BalanceSheetConcept::AccumulatedOtherComprehensiveIncome.element(),
                "us-gaap/alias/accumulated-other-comprehensive-income@1",
            ),
            (
                IncomeStatementConcept::CostOfRevenue.element(),
                "us-gaap/alias/cost-of-revenue@1",
            ),
            (
                IncomeStatementConcept::GrossProfit.element(),
                "us-gaap/alias/gross-profit@1",
            ),
            (
                IncomeStatementConcept::SellingGeneralAdmin.element(),
                "us-gaap/alias/selling-general-admin@1",
            ),
            (
                IncomeStatementConcept::ResearchDevelopment.element(),
                "us-gaap/alias/research-development@1",
            ),
            (
                IncomeStatementConcept::InterestExpense.element(),
                "us-gaap/alias/interest-expense@1",
            ),
            (
                IncomeStatementConcept::OperatingExpenses.element(),
                "us-gaap/alias/operating-expenses@1",
            ),
            (
                IncomeStatementConcept::PretaxIncome.element(),
                "us-gaap/alias/pretax-income@1",
            ),
            (
                IncomeStatementConcept::IncomeTaxExpense.element(),
                "us-gaap/alias/income-tax-expense@1",
            ),
            (
                IncomeStatementConcept::NetIncomeAttributableToParent.element(),
                "us-gaap/alias/net-income-attributable-to-parent@1",
            ),
            (
                IncomeStatementConcept::NetIncomeAttributableToNoncontrollingInterest.element(),
                "us-gaap/alias/net-income-attributable-to-noncontrolling-interest@1",
            ),
            (
                PerShareConcept::EarningsPerShareBasic.element(),
                "us-gaap/alias/earnings-per-share-basic@1",
            ),
            (
                PerShareConcept::EarningsPerShareDiluted.element(),
                "us-gaap/alias/earnings-per-share-diluted@1",
            ),
            (
                DebtMaturityConcept::LongTermDebtIncludingCurrentMaturities.element(),
                "us-gaap/alias/long-term-debt-including-current-maturities@1",
            ),
            (
                DebtMaturityConcept::DebtDueInYearOne.element(),
                "us-gaap/alias/debt-due-in-year-one@1",
            ),
            (
                DebtMaturityConcept::DebtDueInYearTwo.element(),
                "us-gaap/alias/debt-due-in-year-two@1",
            ),
            (
                DebtMaturityConcept::DebtDueInYearThree.element(),
                "us-gaap/alias/debt-due-in-year-three@1",
            ),
            (
                DebtMaturityConcept::DebtDueInYearFour.element(),
                "us-gaap/alias/debt-due-in-year-four@1",
            ),
            (
                DebtMaturityConcept::DebtDueInYearFive.element(),
                "us-gaap/alias/debt-due-in-year-five@1",
            ),
            (
                DebtMaturityConcept::DebtDueAfterYearFive.element(),
                "us-gaap/alias/debt-due-after-year-five@1",
            ),
            (
                CashFlowConcept::DepreciationAmortization.element(),
                "us-gaap/alias/depreciation-amortization@1",
            ),
            (
                CashFlowConcept::CapitalExpenditures.element(),
                "us-gaap/alias/capital-expenditures@1",
            ),
            (
                CashFlowConcept::DividendsPaid.element(),
                "us-gaap/alias/dividends-paid@1",
            ),
            (
                CashFlowConcept::ShareRepurchases.element(),
                "us-gaap/alias/share-repurchases@1",
            ),
            (
                CashFlowConcept::ShareBasedCompensation.element(),
                "us-gaap/alias/share-based-compensation@1",
            ),
            (
                CashFlowConcept::NetChangeInCash.element(),
                "us-gaap/alias/net-change-in-cash@1",
            ),
            (
                CashFlowConcept::ExchangeRateEffect.element(),
                "us-gaap/alias/exchange-rate-effect@1",
            ),
            (
                CashFlowConcept::CashAndRestrictedCash.element(),
                "us-gaap/alias/cash-and-restricted-cash@1",
            ),
            (
                IncomeStatementConcept::OtherComprehensiveIncome.element(),
                "us-gaap/alias/other-comprehensive-income@1",
            ),
            (
                IncomeStatementConcept::ComprehensiveIncome.element(),
                "us-gaap/alias/comprehensive-income@1",
            ),
        ];

        let result: Vec<(CanonicalElementKind, &str)> = us_gaap
            .aliases()
            .iter()
            .map(|aliases| (aliases.element, aliases.rule.as_str()))
            .collect();

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_find_reference_data_for_every_alias_part_and_line_except_the_removed_ones_when_created()
     {
        let us_gaap = UsGaap::new();

        // The 2026 release does not hold these concepts. The EPS concept and the two revenue parts
        // are in two lists each.
        let expected_result = vec![
            concept("us-gaap", "CashAndCashEquivalentsPeriodIncreaseDecrease"),
            concept("us-gaap", "CashPeriodIncreaseDecrease"),
            concept("us-gaap", "EarningsPerShareBasicAndDiluted"),
            concept("us-gaap", "EarningsPerShareBasicAndDiluted"),
            concept("us-gaap", "EffectOfExchangeRateOnCashAndCashEquivalents"),
            concept(
                "us-gaap",
                "EffectOfExchangeRateOnCashAndCashEquivalentsContinuingOperations",
            ),
            concept(
                "us-gaap",
                "EffectOfExchangeRateOnCashAndCashEquivalentsDiscontinuedOperations",
            ),
            concept("us-gaap", "IncomeLossFromOperations"),
            concept("us-gaap", "SalesRevenueGoodsNet"),
            concept("us-gaap", "SalesRevenueGoodsNet"),
            concept("us-gaap", "SalesRevenueNet"),
            concept("us-gaap", "SalesRevenueServicesNet"),
            concept("us-gaap", "SalesRevenueServicesNet"),
        ];

        let mut result: Vec<ConceptName> = us_gaap
            .aliases()
            .iter()
            .flat_map(|aliases| {
                let parts = aliases.parts.iter().flat_map(|parts| {
                    let own = parts.own_other_parts.iter().flat_map(|(_, others)| others);
                    parts.parts.iter().chain(&parts.other_parts).chain(own)
                });
                let lines = aliases.line_total.iter().flat_map(|total| &total.lines);
                aliases.concepts.iter().chain(parts).chain(lines)
            })
            .filter(|concept| us_gaap.element(concept).is_none())
            .cloned()
            .collect();
        result.sort_by(|left, right| left.local_name().cmp(right.local_name()));

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_list_each_part_that_is_an_other_part_of_another_part_when_created() {
        let us_gaap = UsGaap::new();

        let expected_result = vec![
            concept("us-gaap", "SalesRevenueGoodsNet"),
            concept("us-gaap", "SalesRevenueServicesNet"),
            concept(
                "us-gaap",
                "TemporaryEquityCarryingAmountAttributableToParent",
            ),
            concept(
                "us-gaap",
                "RedeemableNoncontrollingInterestEquityCarryingAmount",
            ),
            concept("us-gaap", "LongTermDebtAndCapitalLeaseObligationsCurrent"),
            concept("us-gaap", "ShortTermBorrowings"),
        ];

        let result: Vec<ConceptName> = us_gaap
            .aliases()
            .iter()
            .filter_map(|aliases| aliases.parts.as_ref())
            .flat_map(|parts| {
                parts.parts.iter().filter(|part| {
                    parts.other_parts.contains(part)
                        || parts
                            .own_other_parts
                            .iter()
                            .any(|(_, others)| others.contains(part))
                })
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

        let expected_result: Vec<(CanonicalElementKind, RuleId, Option<Confidence>)> = Vec::new();

        let result = resolve(&facts, &registry);

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_attach_each_line_total_to_the_alias_list_of_its_standard_concept_for_a_table_of_line_totals()
     {
        let line_totals = [mappings::LineTotalDefinition::new(
            IncomeStatementConcept::Revenue.element(),
            &["InterestIncomeExpenseNet", "NoninterestIncome"],
            "us-gaap/total/revenue@1",
        )];
        let tree = CalculationTree::default();

        let expected_result = vec![(
            IncomeStatementConcept::Revenue.element(),
            LineTotal {
                rule: RuleId::from_str("us-gaap/total/revenue@1")
                    .expect("Given a hardcoded rule ID, the parse should always succeed"),
                lines: vec![
                    concept("us-gaap", "InterestIncomeExpenseNet"),
                    concept("us-gaap", "NoninterestIncome"),
                ],
            },
        )];

        let result: Vec<(CanonicalElementKind, LineTotal)> = alias_lists(&line_totals, &tree)
            .into_iter()
            .filter_map(|aliases| Some((aliases.element, aliases.line_total?)))
            .collect();

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_give_the_rule_and_the_us_gaap_lines_for_a_line_total_definition() {
        let definition = mappings::LineTotalDefinition::new(
            IncomeStatementConcept::Revenue.element(),
            &["InterestIncomeExpenseNet", "NoninterestIncome"],
            "us-gaap/total/revenue@1",
        );

        let expected_result = LineTotal {
            rule: RuleId::from_str("us-gaap/total/revenue@1")
                .expect("Given a hardcoded rule ID, the parse should always succeed"),
            lines: vec![
                concept("us-gaap", "InterestIncomeExpenseNet"),
                concept("us-gaap", "NoninterestIncome"),
            ],
        };

        let result = line_total(&definition);

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_attach_the_parts_of_each_part_definition_to_its_alias_list_when_created() {
        let us_gaap = UsGaap::new();

        let expected_result = vec![
            IncomeStatementConcept::Revenue.element(),
            IncomeStatementConcept::NetIncome.element(),
            BalanceSheetConcept::Equity.element(),
            CashFlowConcept::OperatingCashFlow.element(),
            CashFlowConcept::InvestingCashFlow.element(),
            CashFlowConcept::FinancingCashFlow.element(),
            BalanceSheetConcept::CashAndEquivalents.element(),
            BalanceSheetConcept::ShortTermInvestments.element(),
            BalanceSheetConcept::IntangibleAssets.element(),
            BalanceSheetConcept::TemporaryEquity.element(),
            BalanceSheetConcept::ShortTermDebt.element(),
            BalanceSheetConcept::AdditionalPaidInCapital.element(),
            BalanceSheetConcept::TreasuryStock.element(),
            IncomeStatementConcept::CostOfRevenue.element(),
            IncomeStatementConcept::SellingGeneralAdmin.element(),
            IncomeStatementConcept::ResearchDevelopment.element(),
            IncomeStatementConcept::InterestExpense.element(),
            IncomeStatementConcept::PretaxIncome.element(),
            CashFlowConcept::DepreciationAmortization.element(),
            CashFlowConcept::CapitalExpenditures.element(),
            CashFlowConcept::DividendsPaid.element(),
            CashFlowConcept::ShareRepurchases.element(),
            CashFlowConcept::NetChangeInCash.element(),
            CashFlowConcept::ExchangeRateEffect.element(),
            CashFlowConcept::CashAndRestrictedCash.element(),
            IncomeStatementConcept::OtherComprehensiveIncome.element(),
            IncomeStatementConcept::ComprehensiveIncome.element(),
        ];

        let result: Vec<CanonicalElementKind> = us_gaap
            .aliases()
            .iter()
            .filter(|aliases| aliases.parts.is_some())
            .map(|aliases| aliases.element)
            .collect();

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_name_the_scope_of_each_cash_concept_before_2018_when_created() {
        let us_gaap = UsGaap::new();

        let expected_result = vec![
            (
                CashFlowConcept::NetChangeInCash.element(),
                concept("us-gaap", "CashAndCashEquivalentsPeriodIncreaseDecrease"),
                Scope::ExcludesRestrictedCash,
            ),
            (
                CashFlowConcept::NetChangeInCash.element(),
                concept("us-gaap", "CashPeriodIncreaseDecrease"),
                Scope::ExcludesRestrictedCash,
            ),
            (
                CashFlowConcept::ExchangeRateEffect.element(),
                concept("us-gaap", "EffectOfExchangeRateOnCashAndCashEquivalents"),
                Scope::ExcludesRestrictedCash,
            ),
            (
                CashFlowConcept::ExchangeRateEffect.element(),
                concept(
                    "us-gaap",
                    "EffectOfExchangeRateOnCashAndCashEquivalentsContinuingOperations",
                ),
                Scope::ExcludesRestrictedCash,
            ),
        ];

        let result: Vec<(CanonicalElementKind, ConceptName, Scope)> = us_gaap
            .aliases()
            .iter()
            .flat_map(|aliases| {
                aliases
                    .scopes
                    .iter()
                    .map(|(concept, scope)| (aliases.element, concept.clone(), *scope))
            })
            .collect();

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_name_the_scopes_of_the_definition_and_then_of_the_parts_when_both_name_scopes() {
        let definition = mappings::ConceptDefinition::new(
            CashFlowConcept::ExchangeRateEffect.element(),
            &[
                "EffectOfExchangeRateOnCashCashEquivalentsRestrictedCashAndRestrictedCashEquivalents",
                "EffectOfExchangeRateOnCashAndCashEquivalents",
            ],
            "us-gaap/alias/exchange-rate-effect@1",
            false,
        )
        .with_scopes(&[(
            "EffectOfExchangeRateOnCashAndCashEquivalents",
            Scope::ExcludesRestrictedCash,
        )]);
        let parts = mappings::PartDefinition::new(
            CashFlowConcept::ExchangeRateEffect.element(),
            &["EffectOfExchangeRateOnCashAndCashEquivalentsContinuingOperations"],
            &["EffectOfExchangeRateOnCashAndCashEquivalentsDiscontinuedOperations"],
            &[],
            "us-gaap/part/exchange-rate-effect@1",
        )
        .with_scopes(&[(
            "EffectOfExchangeRateOnCashAndCashEquivalentsContinuingOperations",
            Scope::ExcludesRestrictedCash,
        )]);
        let tree = CalculationTree { arcs: Vec::new() };

        let expected_result = vec![
            (
                concept("us-gaap", "EffectOfExchangeRateOnCashAndCashEquivalents"),
                Scope::ExcludesRestrictedCash,
            ),
            (
                concept(
                    "us-gaap",
                    "EffectOfExchangeRateOnCashAndCashEquivalentsContinuingOperations",
                ),
                Scope::ExcludesRestrictedCash,
            ),
        ];

        let result = concept_aliases(&definition, Some(&parts), None, &tree).scopes;

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_return_revenue_concepts_and_parts_in_priority_order_when_created() {
        let us_gaap = UsGaap::new();

        let expected_result = Some(&ConceptAliases {
            element: IncomeStatementConcept::Revenue.element(),
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
                    concept("us-gaap", "SalesRevenueGoodsNet"),
                    concept("us-gaap", "SalesRevenueServicesNet"),
                ],
                own_other_parts: vec![
                    (
                        concept("us-gaap", "SalesRevenueGoodsNet"),
                        vec![concept("us-gaap", "SalesRevenueServicesNet")],
                    ),
                    (
                        concept("us-gaap", "SalesRevenueServicesNet"),
                        vec![concept("us-gaap", "SalesRevenueGoodsNet")],
                    ),
                ],
                other_part_totals: vec![],
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
            line_total: None,
            scopes: Vec::new(),
        });

        let result = us_gaap
            .aliases()
            .iter()
            .find(|aliases| aliases.element == IncomeStatementConcept::Revenue.element());

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_return_net_income_concepts_and_parts_in_priority_order_when_created() {
        let us_gaap = UsGaap::new();

        let expected_result = Some(&ConceptAliases {
            element: IncomeStatementConcept::NetIncome.element(),
            rule: rule("us-gaap/alias/net-income@1"),
            concepts: vec![concept("us-gaap", "ProfitLoss")],
            parts: Some(PartAliases {
                rule: rule("us-gaap/part/net-income@1"),
                parts: vec![concept("us-gaap", "NetIncomeLoss")],
                own_other_parts: vec![],
                other_part_totals: vec![],
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
            line_total: None,
            scopes: Vec::new(),
        });

        let result = us_gaap
            .aliases()
            .iter()
            .find(|aliases| aliases.element == IncomeStatementConcept::NetIncome.element());

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_return_equity_concepts_and_parts_in_priority_order_when_created() {
        let us_gaap = UsGaap::new();

        let expected_result = Some(&ConceptAliases {
            element: BalanceSheetConcept::Equity.element(),
            rule: rule("us-gaap/alias/equity@1"),
            concepts: vec![concept(
                "us-gaap",
                "StockholdersEquityIncludingPortionAttributableToNoncontrollingInterest",
            )],
            parts: Some(PartAliases {
                rule: rule("us-gaap/part/equity@1"),
                parts: vec![concept("us-gaap", "StockholdersEquity")],
                own_other_parts: vec![],
                other_part_totals: vec![],
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
            line_total: None,
            scopes: Vec::new(),
        });

        let result = us_gaap
            .aliases()
            .iter()
            .find(|aliases| aliases.element == BalanceSheetConcept::Equity.element());

        assert_eq!(result, expected_result);
    }

    /// The balance sheet concepts of [`mappings::OPTIONAL_CONCEPTS`].
    const BALANCE_SHEET: [CanonicalElementKind; 22] = [
        BalanceSheetConcept::CurrentAssets.element(),
        BalanceSheetConcept::NonCurrentAssets.element(),
        BalanceSheetConcept::CashAndEquivalents.element(),
        BalanceSheetConcept::ShortTermInvestments.element(),
        BalanceSheetConcept::AccountsReceivable.element(),
        BalanceSheetConcept::Inventory.element(),
        BalanceSheetConcept::PropertyPlantEquipment.element(),
        BalanceSheetConcept::Goodwill.element(),
        BalanceSheetConcept::IntangibleAssets.element(),
        BalanceSheetConcept::LiabilitiesAndEquity.element(),
        BalanceSheetConcept::TemporaryEquity.element(),
        BalanceSheetConcept::CurrentLiabilities.element(),
        BalanceSheetConcept::NonCurrentLiabilities.element(),
        BalanceSheetConcept::AccountsPayable.element(),
        BalanceSheetConcept::ShortTermDebt.element(),
        BalanceSheetConcept::LongTermDebt.element(),
        BalanceSheetConcept::EquityAttributableToParent.element(),
        BalanceSheetConcept::NoncontrollingInterest.element(),
        BalanceSheetConcept::AdditionalPaidInCapital.element(),
        BalanceSheetConcept::RetainedEarnings.element(),
        BalanceSheetConcept::TreasuryStock.element(),
        BalanceSheetConcept::AccumulatedOtherComprehensiveIncome.element(),
    ];

    /// The income statement concepts of [`mappings::OPTIONAL_CONCEPTS`].
    const INCOME_STATEMENT: [CanonicalElementKind; 12] = [
        IncomeStatementConcept::CostOfRevenue.element(),
        IncomeStatementConcept::GrossProfit.element(),
        IncomeStatementConcept::SellingGeneralAdmin.element(),
        IncomeStatementConcept::ResearchDevelopment.element(),
        IncomeStatementConcept::InterestExpense.element(),
        IncomeStatementConcept::OperatingExpenses.element(),
        IncomeStatementConcept::PretaxIncome.element(),
        IncomeStatementConcept::IncomeTaxExpense.element(),
        IncomeStatementConcept::NetIncomeAttributableToParent.element(),
        IncomeStatementConcept::NetIncomeAttributableToNoncontrollingInterest.element(),
        PerShareConcept::EarningsPerShareBasic.element(),
        PerShareConcept::EarningsPerShareDiluted.element(),
    ];

    /// The cash flow concepts of [`mappings::OPTIONAL_CONCEPTS`].
    const CASH_FLOW: [CanonicalElementKind; 8] = [
        CashFlowConcept::DepreciationAmortization.element(),
        CashFlowConcept::CapitalExpenditures.element(),
        CashFlowConcept::DividendsPaid.element(),
        CashFlowConcept::ShareRepurchases.element(),
        CashFlowConcept::ShareBasedCompensation.element(),
        CashFlowConcept::NetChangeInCash.element(),
        CashFlowConcept::ExchangeRateEffect.element(),
        CashFlowConcept::CashAndRestrictedCash.element(),
    ];

    #[test]
    fn should_return_cash_flow_concepts_in_priority_order_when_created() {
        let us_gaap = UsGaap::new();

        let expected_result = vec![
            (
                CashFlowConcept::DepreciationAmortization.element(),
                rule("us-gaap/alias/depreciation-amortization@1"),
                vec![concept("us-gaap", "DepreciationDepletionAndAmortization")],
            ),
            (
                CashFlowConcept::CapitalExpenditures.element(),
                rule("us-gaap/alias/capital-expenditures@1"),
                vec![concept("us-gaap", "PaymentsToAcquireProductiveAssets")],
            ),
            (
                CashFlowConcept::DividendsPaid.element(),
                rule("us-gaap/alias/dividends-paid@1"),
                vec![concept("us-gaap", "PaymentsOfDividends")],
            ),
            (
                CashFlowConcept::ShareRepurchases.element(),
                rule("us-gaap/alias/share-repurchases@1"),
                vec![concept("us-gaap", "PaymentsForRepurchaseOfEquity")],
            ),
            (
                CashFlowConcept::ShareBasedCompensation.element(),
                rule("us-gaap/alias/share-based-compensation@1"),
                vec![concept("us-gaap", "ShareBasedCompensation")],
            ),
            (
                CashFlowConcept::NetChangeInCash.element(),
                rule("us-gaap/alias/net-change-in-cash@1"),
                vec![
                    concept(
                        "us-gaap",
                        "CashCashEquivalentsRestrictedCashAndRestrictedCashEquivalentsPeriodIncreaseDecreaseIncludingExchangeRateEffect",
                    ),
                    concept("us-gaap", "CashAndCashEquivalentsPeriodIncreaseDecrease"),
                    concept("us-gaap", "CashPeriodIncreaseDecrease"),
                ],
            ),
            (
                CashFlowConcept::ExchangeRateEffect.element(),
                rule("us-gaap/alias/exchange-rate-effect@1"),
                vec![
                    concept(
                        "us-gaap",
                        "EffectOfExchangeRateOnCashCashEquivalentsRestrictedCashAndRestrictedCashEquivalentsIncludingDisposalGroupAndDiscontinuedOperations",
                    ),
                    concept("us-gaap", "EffectOfExchangeRateOnCashAndCashEquivalents"),
                ],
            ),
            (
                CashFlowConcept::CashAndRestrictedCash.element(),
                rule("us-gaap/alias/cash-and-restricted-cash@1"),
                vec![concept(
                    "us-gaap",
                    "CashCashEquivalentsRestrictedCashAndRestrictedCashEquivalentsIncludingDisposalGroupAndDiscontinuedOperations",
                )],
            ),
        ];

        let result: Vec<(CanonicalElementKind, RuleId, Vec<ConceptName>)> = us_gaap
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
            own_other_parts: vec![],
            other_part_totals: vec![],
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
            .find(|aliases| aliases.element == CashFlowConcept::DepreciationAmortization.element())
            .and_then(|aliases| aliases.parts.clone());

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_return_parts_of_cash_flow_concepts_in_priority_order_when_created() {
        let us_gaap = UsGaap::new();

        let expected_result = vec![
            (
                CashFlowConcept::CapitalExpenditures.element(),
                PartAliases {
                    rule: rule("us-gaap/part/capital-expenditures@1"),
                    parts: vec![concept(
                        "us-gaap",
                        "PaymentsToAcquirePropertyPlantAndEquipment",
                    )],
                    own_other_parts: vec![],
                    other_part_totals: vec![],
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
                CashFlowConcept::DividendsPaid.element(),
                PartAliases {
                    rule: rule("us-gaap/part/dividends-paid@1"),
                    parts: vec![concept("us-gaap", "PaymentsOfDividendsCommonStock")],
                    own_other_parts: vec![],
                    other_part_totals: vec![],
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
                CashFlowConcept::ShareRepurchases.element(),
                PartAliases {
                    rule: rule("us-gaap/part/share-repurchases@1"),
                    parts: vec![concept("us-gaap", "PaymentsForRepurchaseOfCommonStock")],
                    own_other_parts: vec![],
                    other_part_totals: vec![],
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
            (
                CashFlowConcept::CashAndRestrictedCash.element(),
                PartAliases {
                    rule: rule("us-gaap/part/cash-and-restricted-cash@1"),
                    parts: vec![concept(
                        "us-gaap",
                        "CashCashEquivalentsRestrictedCashAndRestrictedCashEquivalents",
                    )],
                    own_other_parts: vec![],
                    other_part_totals: vec![],
                    other_parts: vec![concept(
                        "us-gaap",
                        "CashCashEquivalentsRestrictedCashAndRestrictedCashEquivalentsDisposalGroupIncludingDiscontinuedOperations",
                    )],
                },
            ),
        ];

        let result: Vec<(CanonicalElementKind, PartAliases)> = us_gaap
            .aliases()
            .iter()
            .filter(|aliases| {
                CASH_FLOW.contains(&aliases.element)
                    && !CASH_CHANGE.contains(&aliases.element)
                    && aliases.element != CashFlowConcept::DepreciationAmortization.element()
            })
            .filter_map(|aliases| Some((aliases.element, aliases.parts.clone()?)))
            .collect();

        assert_eq!(result, expected_result);
    }

    /// The cash flow concepts of the change in cash.
    const CASH_CHANGE: [CanonicalElementKind; 2] = [
        CashFlowConcept::NetChangeInCash.element(),
        CashFlowConcept::ExchangeRateEffect.element(),
    ];

    #[test]
    fn should_return_parts_of_the_change_in_cash_in_priority_order_when_created() {
        let us_gaap = UsGaap::new();

        let expected_result = vec![
            (
                CashFlowConcept::NetChangeInCash.element(),
                PartAliases {
                    rule: rule("us-gaap/part/net-change-in-cash@1"),
                    parts: vec![concept(
                        "us-gaap",
                        "CashCashEquivalentsRestrictedCashAndRestrictedCashEquivalentsPeriodIncreaseDecreaseExcludingExchangeRateEffect",
                    )],
                    own_other_parts: vec![],
                    other_part_totals: vec![],
                    other_parts: vec![
                        concept(
                            "us-gaap",
                            "EffectOfExchangeRateOnCashCashEquivalentsRestrictedCashAndRestrictedCashEquivalentsIncludingDisposalGroupAndDiscontinuedOperations",
                        ),
                        concept(
                            "us-gaap",
                            "EffectOfExchangeRateOnCashCashEquivalentsRestrictedCashAndRestrictedCashEquivalents",
                        ),
                        concept(
                            "us-gaap",
                            "EffectOfExchangeRateOnCashCashEquivalentsRestrictedCashAndRestrictedCashEquivalentsDisposalGroupIncludingDiscontinuedOperations",
                        ),
                    ],
                },
            ),
            (
                CashFlowConcept::ExchangeRateEffect.element(),
                PartAliases {
                    rule: rule("us-gaap/part/exchange-rate-effect@1"),
                    parts: vec![
                        concept(
                            "us-gaap",
                            "EffectOfExchangeRateOnCashCashEquivalentsRestrictedCashAndRestrictedCashEquivalents",
                        ),
                        concept(
                            "us-gaap",
                            "EffectOfExchangeRateOnCashAndCashEquivalentsContinuingOperations",
                        ),
                    ],
                    own_other_parts: vec![],
                    other_part_totals: vec![],
                    other_parts: vec![
                        concept(
                            "us-gaap",
                            "EffectOfExchangeRateOnCashCashEquivalentsRestrictedCashAndRestrictedCashEquivalentsDisposalGroupIncludingDiscontinuedOperations",
                        ),
                        concept(
                            "us-gaap",
                            "EffectOfExchangeRateOnCashAndCashEquivalentsDiscontinuedOperations",
                        ),
                    ],
                },
            ),
        ];

        let result: Vec<(CanonicalElementKind, PartAliases)> = us_gaap
            .aliases()
            .iter()
            .filter(|aliases| CASH_CHANGE.contains(&aliases.element))
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
            element: IncomeStatementConcept::OtherComprehensiveIncome.element(),
            rule: rule("us-gaap/alias/other-comprehensive-income@1"),
            concepts: vec![concept("us-gaap", "OtherComprehensiveIncomeLossNetOfTax")],
            parts: Some(PartAliases {
                rule: rule("us-gaap/part/other-comprehensive-income@1"),
                parts: vec![concept(
                    "us-gaap",
                    "OtherComprehensiveIncomeLossNetOfTaxPortionAttributableToParent",
                )],
                own_other_parts: vec![],
                other_part_totals: vec![],
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
            line_total: None,
            scopes: Vec::new(),
        });

        let result = us_gaap.aliases().iter().find(|aliases| {
            aliases.element == IncomeStatementConcept::OtherComprehensiveIncome.element()
        });

        assert_eq!(result, expected_result);
    }

    // The test writes out the expanded other parts, so it passes 100 lines.
    #[allow(clippy::too_many_lines)]
    #[test]
    fn should_return_comprehensive_income_concepts_and_parts_in_priority_order_when_created() {
        let us_gaap = UsGaap::new();

        let expected_result = Some(&ConceptAliases {
            element: IncomeStatementConcept::ComprehensiveIncome.element(),
            rule: rule("us-gaap/alias/comprehensive-income@1"),
            concepts: vec![concept(
                "us-gaap",
                "ComprehensiveIncomeNetOfTaxIncludingPortionAttributableToNoncontrollingInterest",
            )],
            parts: Some(PartAliases {
                rule: rule("us-gaap/part/comprehensive-income@1"),
                parts: vec![concept("us-gaap", "ComprehensiveIncomeNetOfTax")],
                own_other_parts: vec![],
                other_part_totals: vec![],
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
            line_total: None,
            scopes: Vec::new(),
        });

        let result = us_gaap.aliases().iter().find(|aliases| {
            aliases.element == IncomeStatementConcept::ComprehensiveIncome.element()
        });

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_give_synonym_with_debt_due_in_year_two_rule_id_when_the_filing_reports_only_the_rolling_concept()
     {
        let us_gaap = UsGaap::new();
        let registry = Registry::new(vec![&us_gaap])
            .expect("Given one plug-in with distinct families, the registry should always start");
        let filing = source("0000320193-24-000123", date(2024, 11, 1));
        let facts = vec![fact(
            &filing,
            "LongTermDebtMaturitiesRepaymentsOfPrincipalInRollingYearTwo",
            "500000000",
        )];

        let expected_result = vec![(
            DebtMaturityConcept::DebtDueInYearTwo.element(),
            rule("us-gaap/alias/debt-due-in-year-two@1"),
            Some(Confidence::Synonym),
        )];

        let result = resolve(&facts, &registry);

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_give_exact_with_debt_due_in_year_two_rule_id_when_the_filing_reports_both_the_fiscal_year_and_the_rolling_concept()
     {
        let us_gaap = UsGaap::new();
        let registry = Registry::new(vec![&us_gaap])
            .expect("Given one plug-in with distinct families, the registry should always start");
        let filing = source("0000320193-24-000123", date(2024, 11, 1));
        let facts = vec![
            fact(
                &filing,
                "LongTermDebtMaturitiesRepaymentsOfPrincipalInRollingYearTwo",
                "500000000",
            ),
            fact(
                &filing,
                "LongTermDebtMaturitiesRepaymentsOfPrincipalInYearTwo",
                "500000000",
            ),
        ];

        let expected_result = vec![(
            DebtMaturityConcept::DebtDueInYearTwo.element(),
            rule("us-gaap/alias/debt-due-in-year-two@1"),
            Some(Confidence::Exact),
        )];

        let result = resolve(&facts, &registry);

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_give_long_term_debt_including_current_maturities_when_the_filing_reports_long_term_debt()
     {
        let us_gaap = UsGaap::new();
        let registry = Registry::new(vec![&us_gaap])
            .expect("Given one plug-in with distinct families, the registry should always start");
        let filing = source("0000320193-24-000123", date(2024, 11, 1));
        let facts = vec![fact(&filing, "LongTermDebt", "2100000000")];

        let expected_result = vec![(
            DebtMaturityConcept::LongTermDebtIncludingCurrentMaturities.element(),
            rule("us-gaap/alias/long-term-debt-including-current-maturities@1"),
            Some(Confidence::Exact),
        )];

        let result = resolve(&facts, &registry);

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
            IncomeStatementConcept::ComprehensiveIncome.element(),
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
            IncomeStatementConcept::OtherComprehensiveIncome.element(),
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

        // The noncontrolling net income resolves to its own concept, and no comprehensive income.
        let expected_result = vec![(
            IncomeStatementConcept::NetIncomeAttributableToNoncontrollingInterest.element(),
            rule("us-gaap/alias/net-income-attributable-to-noncontrolling-interest@1"),
            Some(Confidence::Exact),
        )];

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

        let expected_result: Vec<(CanonicalElementKind, RuleId, Option<Confidence>)> = Vec::new();

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

        let expected_result: Vec<(CanonicalElementKind, RuleId, Option<Confidence>)> = Vec::new();

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

        let expected_result: Vec<(CanonicalElementKind, RuleId, Option<Confidence>)> = Vec::new();

        let result = resolve(&facts, &registry);

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_return_income_statement_concepts_in_priority_order_when_created() {
        let us_gaap = UsGaap::new();

        let expected_result = vec![
            (
                IncomeStatementConcept::CostOfRevenue.element(),
                rule("us-gaap/alias/cost-of-revenue@1"),
                vec![concept("us-gaap", "CostOfRevenue")],
            ),
            (
                IncomeStatementConcept::GrossProfit.element(),
                rule("us-gaap/alias/gross-profit@1"),
                vec![concept("us-gaap", "GrossProfit")],
            ),
            (
                IncomeStatementConcept::SellingGeneralAdmin.element(),
                rule("us-gaap/alias/selling-general-admin@1"),
                vec![concept("us-gaap", "SellingGeneralAndAdministrativeExpense")],
            ),
            (
                IncomeStatementConcept::ResearchDevelopment.element(),
                rule("us-gaap/alias/research-development@1"),
                vec![concept("us-gaap", "ResearchAndDevelopmentExpense")],
            ),
            (
                IncomeStatementConcept::InterestExpense.element(),
                rule("us-gaap/alias/interest-expense@1"),
                vec![concept("us-gaap", "InterestExpense")],
            ),
            (
                IncomeStatementConcept::OperatingExpenses.element(),
                rule("us-gaap/alias/operating-expenses@1"),
                vec![concept("us-gaap", "OperatingExpenses")],
            ),
            (
                IncomeStatementConcept::PretaxIncome.element(),
                rule("us-gaap/alias/pretax-income@1"),
                vec![concept(
                    "us-gaap",
                    "IncomeLossFromContinuingOperationsBeforeIncomeTaxesExtraordinaryItemsNoncontrollingInterest",
                )],
            ),
            (
                IncomeStatementConcept::IncomeTaxExpense.element(),
                rule("us-gaap/alias/income-tax-expense@1"),
                vec![concept("us-gaap", "IncomeTaxExpenseBenefit")],
            ),
            (
                IncomeStatementConcept::NetIncomeAttributableToParent.element(),
                rule("us-gaap/alias/net-income-attributable-to-parent@1"),
                vec![concept("us-gaap", "NetIncomeLoss")],
            ),
            (
                IncomeStatementConcept::NetIncomeAttributableToNoncontrollingInterest.element(),
                rule("us-gaap/alias/net-income-attributable-to-noncontrolling-interest@1"),
                vec![concept(
                    "us-gaap",
                    "NetIncomeLossAttributableToNoncontrollingInterest",
                )],
            ),
            (
                PerShareConcept::EarningsPerShareBasic.element(),
                rule("us-gaap/alias/earnings-per-share-basic@1"),
                vec![
                    concept("us-gaap", "EarningsPerShareBasic"),
                    concept("us-gaap", "EarningsPerShareBasicAndDiluted"),
                ],
            ),
            (
                PerShareConcept::EarningsPerShareDiluted.element(),
                rule("us-gaap/alias/earnings-per-share-diluted@1"),
                vec![
                    concept("us-gaap", "EarningsPerShareDiluted"),
                    concept("us-gaap", "EarningsPerShareBasicAndDiluted"),
                ],
            ),
        ];

        let result: Vec<(CanonicalElementKind, RuleId, Vec<ConceptName>)> = us_gaap
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
            own_other_parts: vec![],
            other_part_totals: vec![],
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
            .find(|aliases| aliases.element == IncomeStatementConcept::InterestExpense.element())
            .and_then(|aliases| aliases.parts.clone());

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_return_parts_of_income_statement_concepts_in_priority_order_when_created() {
        let us_gaap = UsGaap::new();

        let expected_result = vec![
            (
                IncomeStatementConcept::CostOfRevenue.element(),
                PartAliases {
                    rule: rule("us-gaap/part/cost-of-revenue@1"),
                    parts: vec![concept("us-gaap", "CostOfGoodsAndServicesSold")],
                    own_other_parts: vec![],
                    other_part_totals: vec![],
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
                IncomeStatementConcept::SellingGeneralAdmin.element(),
                PartAliases {
                    rule: rule("us-gaap/part/selling-general-admin@1"),
                    parts: vec![concept("us-gaap", "GeneralAndAdministrativeExpense")],
                    own_other_parts: vec![],
                    other_part_totals: vec![],
                    other_parts: vec![
                        concept("us-gaap", "MarketingExpense"),
                        concept("us-gaap", "OtherSellingGeneralAndAdministrativeExpense"),
                        concept("us-gaap", "SellingAndMarketingExpense"),
                        concept("us-gaap", "SellingExpense"),
                    ],
                },
            ),
            (
                IncomeStatementConcept::ResearchDevelopment.element(),
                PartAliases {
                    rule: rule("us-gaap/part/research-development@1"),
                    parts: vec![concept(
                        "us-gaap",
                        "ResearchAndDevelopmentExpenseExcludingAcquiredInProcessCost",
                    )],
                    own_other_parts: vec![],
                    other_part_totals: vec![],
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
            (
                IncomeStatementConcept::PretaxIncome.element(),
                PartAliases {
                    rule: rule("us-gaap/part/pretax-income@1"),
                    parts: vec![concept(
                        "us-gaap",
                        "IncomeLossFromContinuingOperationsBeforeIncomeTaxesMinorityInterestAndIncomeLossFromEquityMethodInvestments",
                    )],
                    own_other_parts: vec![],
                    other_part_totals: vec![],
                    other_parts: vec![
                        concept("us-gaap", "IncomeLossFromEquityMethodInvestments"),
                        concept(
                            "us-gaap",
                            "IncomeLossFromAffordableHousingProjectsEquityMethodInvestments",
                        ),
                    ],
                },
            ),
        ];

        let result: Vec<(CanonicalElementKind, PartAliases)> = us_gaap
            .aliases()
            .iter()
            .filter(|aliases| {
                INCOME_STATEMENT.contains(&aliases.element)
                    && aliases.element != IncomeStatementConcept::InterestExpense.element()
            })
            .filter_map(|aliases| Some((aliases.element, aliases.parts.clone()?)))
            .collect();

        assert_eq!(result, expected_result);
    }

    // The test writes out every list of the statement in one place, so a list that goes missing
    // fails it.
    #[allow(clippy::too_many_lines)]
    #[test]
    fn should_return_balance_sheet_concepts_in_priority_order_when_created() {
        let us_gaap = UsGaap::new();

        let expected_result = vec![
            (
                BalanceSheetConcept::CurrentAssets.element(),
                rule("us-gaap/alias/current-assets@1"),
                vec![concept("us-gaap", "AssetsCurrent")],
            ),
            (
                BalanceSheetConcept::NonCurrentAssets.element(),
                rule("us-gaap/alias/non-current-assets@1"),
                vec![concept("us-gaap", "AssetsNoncurrent")],
            ),
            (
                BalanceSheetConcept::CashAndEquivalents.element(),
                rule("us-gaap/alias/cash-and-equivalents@1"),
                vec![concept("us-gaap", "CashAndCashEquivalentsAtCarryingValue")],
            ),
            (
                BalanceSheetConcept::ShortTermInvestments.element(),
                rule("us-gaap/alias/short-term-investments@1"),
                vec![concept("us-gaap", "ShortTermInvestments")],
            ),
            (
                BalanceSheetConcept::AccountsReceivable.element(),
                rule("us-gaap/alias/accounts-receivable@1"),
                vec![concept("us-gaap", "AccountsReceivableNetCurrent")],
            ),
            (
                BalanceSheetConcept::Inventory.element(),
                rule("us-gaap/alias/inventory@1"),
                vec![concept("us-gaap", "InventoryNet")],
            ),
            (
                BalanceSheetConcept::PropertyPlantEquipment.element(),
                rule("us-gaap/alias/property-plant-equipment@1"),
                vec![concept("us-gaap", "PropertyPlantAndEquipmentNet")],
            ),
            (
                BalanceSheetConcept::Goodwill.element(),
                rule("us-gaap/alias/goodwill@1"),
                vec![concept("us-gaap", "Goodwill")],
            ),
            (
                BalanceSheetConcept::IntangibleAssets.element(),
                rule("us-gaap/alias/intangible-assets@1"),
                vec![concept("us-gaap", "IntangibleAssetsNetExcludingGoodwill")],
            ),
            (
                BalanceSheetConcept::LiabilitiesAndEquity.element(),
                rule("us-gaap/alias/liabilities-and-equity@1"),
                vec![concept("us-gaap", "LiabilitiesAndStockholdersEquity")],
            ),
            (
                BalanceSheetConcept::TemporaryEquity.element(),
                rule("us-gaap/alias/temporary-equity@1"),
                vec![concept(
                    "us-gaap",
                    "TemporaryEquityCarryingAmountIncludingPortionAttributableToNoncontrollingInterests",
                )],
            ),
            (
                BalanceSheetConcept::CurrentLiabilities.element(),
                rule("us-gaap/alias/current-liabilities@1"),
                vec![concept("us-gaap", "LiabilitiesCurrent")],
            ),
            (
                BalanceSheetConcept::NonCurrentLiabilities.element(),
                rule("us-gaap/alias/non-current-liabilities@1"),
                vec![concept("us-gaap", "LiabilitiesNoncurrent")],
            ),
            (
                BalanceSheetConcept::AccountsPayable.element(),
                rule("us-gaap/alias/accounts-payable@1"),
                vec![concept("us-gaap", "AccountsPayableCurrent")],
            ),
            (
                BalanceSheetConcept::ShortTermDebt.element(),
                rule("us-gaap/alias/short-term-debt@1"),
                vec![concept("us-gaap", "DebtCurrent")],
            ),
            (
                BalanceSheetConcept::LongTermDebt.element(),
                rule("us-gaap/alias/long-term-debt@1"),
                vec![concept("us-gaap", "LongTermDebtNoncurrent")],
            ),
            (
                BalanceSheetConcept::EquityAttributableToParent.element(),
                rule("us-gaap/alias/equity-attributable-to-parent@1"),
                vec![concept("us-gaap", "StockholdersEquity")],
            ),
            (
                BalanceSheetConcept::NoncontrollingInterest.element(),
                rule("us-gaap/alias/noncontrolling-interest@1"),
                vec![concept("us-gaap", "MinorityInterest")],
            ),
            (
                BalanceSheetConcept::AdditionalPaidInCapital.element(),
                rule("us-gaap/alias/additional-paid-in-capital@1"),
                vec![concept("us-gaap", "AdditionalPaidInCapital")],
            ),
            (
                BalanceSheetConcept::RetainedEarnings.element(),
                rule("us-gaap/alias/retained-earnings@1"),
                vec![concept("us-gaap", "RetainedEarningsAccumulatedDeficit")],
            ),
            (
                BalanceSheetConcept::TreasuryStock.element(),
                rule("us-gaap/alias/treasury-stock@1"),
                vec![concept("us-gaap", "TreasuryStockValue")],
            ),
            (
                BalanceSheetConcept::AccumulatedOtherComprehensiveIncome.element(),
                rule("us-gaap/alias/accumulated-other-comprehensive-income@1"),
                vec![concept(
                    "us-gaap",
                    "AccumulatedOtherComprehensiveIncomeLossNetOfTax",
                )],
            ),
        ];

        let result: Vec<(CanonicalElementKind, RuleId, Vec<ConceptName>)> = us_gaap
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

    // The test writes out every list of the statement in one place, so a list that goes missing
    // fails it.
    #[allow(clippy::too_many_lines)]
    #[test]
    fn should_return_parts_of_balance_sheet_concepts_in_priority_order_when_created() {
        let us_gaap = UsGaap::new();

        let expected_result = vec![
            (
                BalanceSheetConcept::CashAndEquivalents.element(),
                PartAliases {
                    rule: rule("us-gaap/part/cash-and-equivalents@1"),
                    parts: vec![concept("us-gaap", "Cash")],
                    own_other_parts: vec![],
                    other_part_totals: vec![],
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
                BalanceSheetConcept::ShortTermInvestments.element(),
                PartAliases {
                    rule: rule("us-gaap/part/short-term-investments@1"),
                    parts: vec![concept("us-gaap", "MarketableSecuritiesCurrent")],
                    own_other_parts: vec![],
                    other_part_totals: vec![],
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
                BalanceSheetConcept::IntangibleAssets.element(),
                PartAliases {
                    rule: rule("us-gaap/part/intangible-assets@1"),
                    parts: vec![concept("us-gaap", "FiniteLivedIntangibleAssetsNet")],
                    own_other_parts: vec![],
                    other_part_totals: vec![],
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
            (
                BalanceSheetConcept::TemporaryEquity.element(),
                PartAliases {
                    rule: rule("us-gaap/part/temporary-equity@1"),
                    parts: vec![
                        concept(
                            "us-gaap",
                            "TemporaryEquityCarryingAmountAttributableToParent",
                        ),
                        concept(
                            "us-gaap",
                            "RedeemableNoncontrollingInterestEquityCarryingAmount",
                        ),
                        concept(
                            "us-gaap",
                            "RedeemableNoncontrollingInterestEquityCommonCarryingAmount",
                        ),
                        concept(
                            "us-gaap",
                            "RedeemableNoncontrollingInterestEquityOtherCarryingAmount",
                        ),
                        concept(
                            "us-gaap",
                            "RedeemableNoncontrollingInterestEquityPreferredCarryingAmount",
                        ),
                        concept(
                            "us-gaap",
                            "RedeemableNoncontrollingInterestEquityOtherFairValue",
                        ),
                    ],
                    own_other_parts: vec![],
                    other_part_totals: vec![],
                    other_parts: vec![
                        concept(
                            "us-gaap",
                            "TemporaryEquityCarryingAmountAttributableToParent",
                        ),
                        concept(
                            "us-gaap",
                            "RedeemableNoncontrollingInterestEquityCarryingAmount",
                        ),
                    ],
                },
            ),
            (
                BalanceSheetConcept::ShortTermDebt.element(),
                PartAliases {
                    rule: rule("us-gaap/part/short-term-debt@1"),
                    parts: vec![
                        concept("us-gaap", "LongTermDebtAndCapitalLeaseObligationsCurrent"),
                        concept("us-gaap", "ShortTermBorrowings"),
                    ],
                    own_other_parts: vec![],
                    other_part_totals: vec![],
                    other_parts: vec![
                        concept("us-gaap", "LongTermDebtAndCapitalLeaseObligationsCurrent"),
                        concept("us-gaap", "ShortTermBorrowings"),
                    ],
                },
            ),
            (
                BalanceSheetConcept::AdditionalPaidInCapital.element(),
                PartAliases {
                    rule: rule("us-gaap/part/additional-paid-in-capital@1"),
                    parts: vec![concept("us-gaap", "AdditionalPaidInCapitalCommonStock")],
                    own_other_parts: vec![],
                    other_part_totals: vec![],
                    other_parts: vec![concept("us-gaap", "AdditionalPaidInCapitalPreferredStock")],
                },
            ),
            (
                BalanceSheetConcept::TreasuryStock.element(),
                PartAliases {
                    rule: rule("us-gaap/part/treasury-stock@1"),
                    parts: vec![concept("us-gaap", "TreasuryStockCommonValue")],
                    own_other_parts: vec![],
                    other_part_totals: vec![],
                    other_parts: vec![concept("us-gaap", "TreasuryStockPreferredValue")],
                },
            ),
        ];

        let result: Vec<(CanonicalElementKind, PartAliases)> = us_gaap
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
            IncomeStatementConcept::Revenue.element(),
            rule("us-gaap/alias/revenue@1"),
            Some(Confidence::Exact),
        )];

        let result = resolve(&facts, &registry);

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_give_operating_expenses_rule_id_when_registry_with_us_gaap_resolves_operating_expenses()
     {
        let us_gaap = UsGaap::new();
        let registry = Registry::new(vec![&us_gaap])
            .expect("Given one plug-in with distinct families, the registry should always start");
        let filing = source("0000320193-24-000123", date(2024, 11, 1));
        let facts = vec![fact(&filing, "OperatingExpenses", "120000000")];

        let expected_result = vec![(
            IncomeStatementConcept::OperatingExpenses.element(),
            rule("us-gaap/alias/operating-expenses@1"),
            Some(Confidence::Exact),
        )];

        let result = resolve(&facts, &registry);

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_give_pretax_income_rule_id_when_registry_with_us_gaap_resolves_its_first_concept() {
        let us_gaap = UsGaap::new();
        let registry = Registry::new(vec![&us_gaap])
            .expect("Given one plug-in with distinct families, the registry should always start");
        let filing = source("0000320193-24-000123", date(2024, 11, 1));
        let facts = vec![fact(
            &filing,
            "IncomeLossFromContinuingOperationsBeforeIncomeTaxesExtraordinaryItemsNoncontrollingInterest",
            "123000000",
        )];

        let expected_result = vec![(
            IncomeStatementConcept::PretaxIncome.element(),
            rule("us-gaap/alias/pretax-income@1"),
            Some(Confidence::Exact),
        )];

        let result = resolve(&facts, &registry);

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_give_synonym_with_pretax_income_part_rule_id_when_registry_with_us_gaap_resolves_a_part_alone()
     {
        let us_gaap = UsGaap::new();
        let registry = Registry::new(vec![&us_gaap])
            .expect("Given one plug-in with distinct families, the registry should always start");
        let filing = source("0000320193-24-000123", date(2024, 11, 1));
        let facts = vec![fact(
            &filing,
            "IncomeLossFromContinuingOperationsBeforeIncomeTaxesMinorityInterestAndIncomeLossFromEquityMethodInvestments",
            "123000000",
        )];

        let expected_result = vec![(
            IncomeStatementConcept::PretaxIncome.element(),
            rule("us-gaap/part/pretax-income@1"),
            Some(Confidence::Synonym),
        )];

        let result = resolve(&facts, &registry);

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_give_income_tax_expense_rule_id_when_registry_with_us_gaap_resolves_income_tax_expense_benefit()
     {
        let us_gaap = UsGaap::new();
        let registry = Registry::new(vec![&us_gaap])
            .expect("Given one plug-in with distinct families, the registry should always start");
        let filing = source("0000320193-24-000123", date(2024, 11, 1));
        let facts = vec![fact(&filing, "IncomeTaxExpenseBenefit", "29000000")];

        let expected_result = vec![(
            IncomeStatementConcept::IncomeTaxExpense.element(),
            rule("us-gaap/alias/income-tax-expense@1"),
            Some(Confidence::Exact),
        )];

        let result = resolve(&facts, &registry);

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_give_net_income_attributable_to_noncontrolling_interest_rule_id_when_registry_with_us_gaap_resolves_its_first_concept()
     {
        let us_gaap = UsGaap::new();
        let registry = Registry::new(vec![&us_gaap])
            .expect("Given one plug-in with distinct families, the registry should always start");
        let filing = source("0000320193-24-000123", date(2024, 11, 1));
        let facts = vec![fact(
            &filing,
            "NetIncomeLossAttributableToNoncontrollingInterest",
            "5000000",
        )];

        let expected_result = vec![(
            IncomeStatementConcept::NetIncomeAttributableToNoncontrollingInterest.element(),
            rule("us-gaap/alias/net-income-attributable-to-noncontrolling-interest@1"),
            Some(Confidence::Exact),
        )];

        let result = resolve(&facts, &registry);

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_give_net_change_in_cash_rule_id_when_registry_with_us_gaap_resolves_the_change_including_exchange_rate_effect()
     {
        let us_gaap = UsGaap::new();
        let registry = Registry::new(vec![&us_gaap])
            .expect("Given one plug-in with distinct families, the registry should always start");
        let filing = source("0000320193-24-000123", date(2024, 11, 1));
        let facts = vec![fact(
            &filing,
            "CashCashEquivalentsRestrictedCashAndRestrictedCashEquivalentsPeriodIncreaseDecreaseIncludingExchangeRateEffect",
            "-7000000",
        )];

        let expected_result = vec![(
            CashFlowConcept::NetChangeInCash.element(),
            rule("us-gaap/alias/net-change-in-cash@1"),
            Some(Confidence::Exact),
        )];

        let result = resolve(&facts, &registry);

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_give_synonym_with_net_change_in_cash_part_rule_id_when_registry_with_us_gaap_resolves_a_part_alone()
     {
        let us_gaap = UsGaap::new();
        let registry = Registry::new(vec![&us_gaap])
            .expect("Given one plug-in with distinct families, the registry should always start");
        let filing = source("0000320193-24-000123", date(2024, 11, 1));
        let facts = vec![fact(
            &filing,
            "CashCashEquivalentsRestrictedCashAndRestrictedCashEquivalentsPeriodIncreaseDecreaseExcludingExchangeRateEffect",
            "-7000000",
        )];

        let expected_result = vec![(
            CashFlowConcept::NetChangeInCash.element(),
            rule("us-gaap/part/net-change-in-cash@1"),
            Some(Confidence::Synonym),
        )];

        let result = resolve(&facts, &registry);

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_give_exchange_rate_effect_rule_id_when_registry_with_us_gaap_resolves_the_effect_including_disposal_groups()
     {
        let us_gaap = UsGaap::new();
        let registry = Registry::new(vec![&us_gaap])
            .expect("Given one plug-in with distinct families, the registry should always start");
        let filing = source("0000320193-24-000123", date(2024, 11, 1));
        let facts = vec![fact(
            &filing,
            "EffectOfExchangeRateOnCashCashEquivalentsRestrictedCashAndRestrictedCashEquivalentsIncludingDisposalGroupAndDiscontinuedOperations",
            "-2000000",
        )];

        let expected_result = vec![(
            CashFlowConcept::ExchangeRateEffect.element(),
            rule("us-gaap/alias/exchange-rate-effect@1"),
            Some(Confidence::Exact),
        )];

        let result = resolve(&facts, &registry);

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_give_synonym_with_exchange_rate_effect_part_rule_id_when_registry_with_us_gaap_resolves_a_part_alone()
     {
        let us_gaap = UsGaap::new();
        let registry = Registry::new(vec![&us_gaap])
            .expect("Given one plug-in with distinct families, the registry should always start");
        let filing = source("0000320193-24-000123", date(2024, 11, 1));
        let facts = vec![fact(
            &filing,
            "EffectOfExchangeRateOnCashCashEquivalentsRestrictedCashAndRestrictedCashEquivalents",
            "-2000000",
        )];

        let expected_result = vec![(
            CashFlowConcept::ExchangeRateEffect.element(),
            rule("us-gaap/part/exchange-rate-effect@1"),
            Some(Confidence::Synonym),
        )];

        let result = resolve(&facts, &registry);

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_give_cash_and_restricted_cash_rule_id_when_registry_with_us_gaap_resolves_the_cash_including_disposal_groups()
     {
        let us_gaap = UsGaap::new();
        let registry = Registry::new(vec![&us_gaap])
            .expect("Given one plug-in with distinct families, the registry should always start");
        let filing = source("0000320193-24-000123", date(2024, 11, 1));
        let facts = vec![fact(
            &filing,
            "CashCashEquivalentsRestrictedCashAndRestrictedCashEquivalentsIncludingDisposalGroupAndDiscontinuedOperations",
            "388000000",
        )];

        let expected_result = vec![(
            CashFlowConcept::CashAndRestrictedCash.element(),
            rule("us-gaap/alias/cash-and-restricted-cash@1"),
            Some(Confidence::Exact),
        )];

        let result = resolve(&facts, &registry);

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_give_synonym_with_cash_and_restricted_cash_part_rule_id_when_registry_with_us_gaap_resolves_a_part_alone()
     {
        let us_gaap = UsGaap::new();
        let registry = Registry::new(vec![&us_gaap])
            .expect("Given one plug-in with distinct families, the registry should always start");
        let filing = source("0000320193-24-000123", date(2024, 11, 1));
        let facts = vec![fact(
            &filing,
            "CashCashEquivalentsRestrictedCashAndRestrictedCashEquivalents",
            "388000000",
        )];

        let expected_result = vec![(
            CashFlowConcept::CashAndRestrictedCash.element(),
            rule("us-gaap/part/cash-and-restricted-cash@1"),
            Some(Confidence::Synonym),
        )];

        let result = resolve(&facts, &registry);

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_give_no_cash_and_restricted_cash_when_the_filing_reports_the_part_next_to_the_cash_of_disposal_groups()
     {
        let us_gaap = UsGaap::new();
        let registry = Registry::new(vec![&us_gaap])
            .expect("Given one plug-in with distinct families, the registry should always start");
        let filing = source("0000320193-24-000123", date(2024, 11, 1));
        let facts = vec![
            fact(
                &filing,
                "CashCashEquivalentsRestrictedCashAndRestrictedCashEquivalents",
                "388000000",
            ),
            fact(
                &filing,
                "CashCashEquivalentsRestrictedCashAndRestrictedCashEquivalentsDisposalGroupIncludingDiscontinuedOperations",
                "12000000",
            ),
        ];

        let expected_result: Vec<(CanonicalElementKind, RuleId, Option<Confidence>)> = Vec::new();

        let result = resolve(&facts, &registry);

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_return_parts_of_the_cash_flow_totals_in_priority_order_when_created() {
        let us_gaap = UsGaap::new();
        let totals = [
            CashFlowConcept::OperatingCashFlow.element(),
            CashFlowConcept::InvestingCashFlow.element(),
            CashFlowConcept::FinancingCashFlow.element(),
        ];

        let expected_result = vec![
            (
                CashFlowConcept::OperatingCashFlow.element(),
                PartAliases {
                    rule: rule("us-gaap/part/operating-cash-flow@1"),
                    parts: vec![concept(
                        "us-gaap",
                        "NetCashProvidedByUsedInOperatingActivitiesContinuingOperations",
                    )],
                    other_parts: vec![concept(
                        "us-gaap",
                        "CashProvidedByUsedInOperatingActivitiesDiscontinuedOperations",
                    )],
                    own_other_parts: vec![],
                    other_part_totals: vec![concept(
                        "us-gaap",
                        "NetCashProvidedByUsedInDiscontinuedOperations",
                    )],
                },
            ),
            (
                CashFlowConcept::InvestingCashFlow.element(),
                PartAliases {
                    rule: rule("us-gaap/part/investing-cash-flow@1"),
                    parts: vec![concept(
                        "us-gaap",
                        "NetCashProvidedByUsedInInvestingActivitiesContinuingOperations",
                    )],
                    other_parts: vec![concept(
                        "us-gaap",
                        "CashProvidedByUsedInInvestingActivitiesDiscontinuedOperations",
                    )],
                    own_other_parts: vec![],
                    other_part_totals: vec![concept(
                        "us-gaap",
                        "NetCashProvidedByUsedInDiscontinuedOperations",
                    )],
                },
            ),
            (
                CashFlowConcept::FinancingCashFlow.element(),
                PartAliases {
                    rule: rule("us-gaap/part/financing-cash-flow@1"),
                    parts: vec![concept(
                        "us-gaap",
                        "NetCashProvidedByUsedInFinancingActivitiesContinuingOperations",
                    )],
                    other_parts: vec![concept(
                        "us-gaap",
                        "CashProvidedByUsedInFinancingActivitiesDiscontinuedOperations",
                    )],
                    own_other_parts: vec![],
                    other_part_totals: vec![concept(
                        "us-gaap",
                        "NetCashProvidedByUsedInDiscontinuedOperations",
                    )],
                },
            ),
        ];

        let result: Vec<(CanonicalElementKind, PartAliases)> = us_gaap
            .aliases()
            .iter()
            .filter(|aliases| totals.contains(&aliases.element))
            .filter_map(|aliases| Some((aliases.element, aliases.parts.clone()?)))
            .collect();

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_give_synonym_with_operating_cash_flow_part_rule_id_when_the_filing_reports_no_discontinued_operations_flow()
     {
        let us_gaap = UsGaap::new();
        let registry = Registry::new(vec![&us_gaap])
            .expect("Given one plug-in with distinct families, the registry should always start");
        let filing = source("0000320193-24-000123", date(2024, 11, 1));
        let facts = vec![fact(
            &filing,
            "NetCashProvidedByUsedInOperatingActivitiesContinuingOperations",
            "118000000",
        )];

        let expected_result = vec![(
            CashFlowConcept::OperatingCashFlow.element(),
            rule("us-gaap/part/operating-cash-flow@1"),
            Some(Confidence::Synonym),
        )];

        let result = resolve(&facts, &registry);

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_give_synonym_with_investing_cash_flow_part_rule_id_when_the_filing_reports_no_discontinued_operations_flow()
     {
        let us_gaap = UsGaap::new();
        let registry = Registry::new(vec![&us_gaap])
            .expect("Given one plug-in with distinct families, the registry should always start");
        let filing = source("0000320193-24-000123", date(2024, 11, 1));
        let facts = vec![fact(
            &filing,
            "NetCashProvidedByUsedInInvestingActivitiesContinuingOperations",
            "-46000000",
        )];

        let expected_result = vec![(
            CashFlowConcept::InvestingCashFlow.element(),
            rule("us-gaap/part/investing-cash-flow@1"),
            Some(Confidence::Synonym),
        )];

        let result = resolve(&facts, &registry);

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_give_synonym_with_financing_cash_flow_part_rule_id_when_the_filing_reports_no_discontinued_operations_flow()
     {
        let us_gaap = UsGaap::new();
        let registry = Registry::new(vec![&us_gaap])
            .expect("Given one plug-in with distinct families, the registry should always start");
        let filing = source("0000320193-24-000123", date(2024, 11, 1));
        let facts = vec![fact(
            &filing,
            "NetCashProvidedByUsedInFinancingActivitiesContinuingOperations",
            "-61000000",
        )];

        let expected_result = vec![(
            CashFlowConcept::FinancingCashFlow.element(),
            rule("us-gaap/part/financing-cash-flow@1"),
            Some(Confidence::Synonym),
        )];

        let result = resolve(&facts, &registry);

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_give_the_operating_cash_flow_part_when_the_discontinued_operations_flow_is_zero() {
        let us_gaap = UsGaap::new();
        let registry = Registry::new(vec![&us_gaap])
            .expect("Given one plug-in with distinct families, the registry should always start");
        let filing = source("0000320193-24-000123", date(2024, 11, 1));
        let facts = vec![
            fact(
                &filing,
                "NetCashProvidedByUsedInOperatingActivitiesContinuingOperations",
                "118000000",
            ),
            fact(
                &filing,
                "CashProvidedByUsedInOperatingActivitiesDiscontinuedOperations",
                "0",
            ),
        ];

        let expected_result = vec![(
            CashFlowConcept::OperatingCashFlow.element(),
            rule("us-gaap/part/operating-cash-flow@1"),
            Some(Confidence::Synonym),
        )];

        let result = resolve(&facts, &registry);

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_give_no_operating_cash_flow_when_the_discontinued_operations_flow_is_not_zero() {
        let us_gaap = UsGaap::new();
        let registry = Registry::new(vec![&us_gaap])
            .expect("Given one plug-in with distinct families, the registry should always start");
        let filing = source("0000320193-24-000123", date(2024, 11, 1));
        let facts = vec![
            fact(
                &filing,
                "NetCashProvidedByUsedInOperatingActivitiesContinuingOperations",
                "118000000",
            ),
            fact(
                &filing,
                "CashProvidedByUsedInOperatingActivitiesDiscontinuedOperations",
                "14000000",
            ),
        ];

        let expected_result = vec![];

        let result = resolve(&facts, &registry);

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_give_no_investing_cash_flow_when_the_discontinued_operations_flow_is_not_zero() {
        let us_gaap = UsGaap::new();
        let registry = Registry::new(vec![&us_gaap])
            .expect("Given one plug-in with distinct families, the registry should always start");
        let filing = source("0000320193-24-000123", date(2024, 11, 1));
        let facts = vec![
            fact(
                &filing,
                "NetCashProvidedByUsedInInvestingActivitiesContinuingOperations",
                "-46000000",
            ),
            fact(
                &filing,
                "CashProvidedByUsedInInvestingActivitiesDiscontinuedOperations",
                "-1000000",
            ),
        ];

        let expected_result = vec![];

        let result = resolve(&facts, &registry);

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_give_no_financing_cash_flow_when_the_discontinued_operations_flow_is_not_zero() {
        let us_gaap = UsGaap::new();
        let registry = Registry::new(vec![&us_gaap])
            .expect("Given one plug-in with distinct families, the registry should always start");
        let filing = source("0000320193-24-000123", date(2024, 11, 1));
        let facts = vec![
            fact(
                &filing,
                "NetCashProvidedByUsedInFinancingActivitiesContinuingOperations",
                "-61000000",
            ),
            fact(
                &filing,
                "CashProvidedByUsedInFinancingActivitiesDiscontinuedOperations",
                "2000000",
            ),
        ];

        let expected_result = vec![];

        let result = resolve(&facts, &registry);

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_give_no_operating_cash_flow_when_the_filing_reports_discontinued_operations_only_as_one_line()
     {
        let us_gaap = UsGaap::new();
        let registry = Registry::new(vec![&us_gaap])
            .expect("Given one plug-in with distinct families, the registry should always start");
        let filing = source("0000320193-24-000123", date(2024, 11, 1));
        let facts = vec![
            fact(
                &filing,
                "NetCashProvidedByUsedInOperatingActivitiesContinuingOperations",
                "118000000",
            ),
            fact(
                &filing,
                "NetCashProvidedByUsedInDiscontinuedOperations",
                "13000000",
            ),
        ];

        let expected_result = vec![];

        let result = resolve(&facts, &registry);

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_give_the_financing_cash_flow_part_when_the_filing_reports_the_one_line_total_next_to_a_financing_flow_of_zero()
     {
        let us_gaap = UsGaap::new();
        let registry = Registry::new(vec![&us_gaap])
            .expect("Given one plug-in with distinct families, the registry should always start");
        let filing = source("0000320193-24-000123", date(2024, 11, 1));
        let facts = vec![
            fact(
                &filing,
                "NetCashProvidedByUsedInFinancingActivitiesContinuingOperations",
                "115400000",
            ),
            fact(
                &filing,
                "NetCashProvidedByUsedInDiscontinuedOperations",
                "13100000",
            ),
            fact(
                &filing,
                "CashProvidedByUsedInFinancingActivitiesDiscontinuedOperations",
                "0",
            ),
        ];

        let expected_result = vec![(
            CashFlowConcept::FinancingCashFlow.element(),
            rule("us-gaap/part/financing-cash-flow@1"),
            Some(Confidence::Synonym),
        )];

        let result = resolve(&facts, &registry);

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_give_net_income_attributable_to_parent_and_the_net_income_part_when_registry_with_us_gaap_resolves_net_income_loss()
     {
        let us_gaap = UsGaap::new();
        let registry = Registry::new(vec![&us_gaap])
            .expect("Given one plug-in with distinct families, the registry should always start");
        let filing = source("0000320193-24-000123", date(2024, 11, 1));
        let facts = vec![fact(&filing, "NetIncomeLoss", "94000000")];

        let expected_result = vec![
            (
                IncomeStatementConcept::NetIncome.element(),
                rule("us-gaap/part/net-income@1"),
                Some(Confidence::Synonym),
            ),
            (
                IncomeStatementConcept::NetIncomeAttributableToParent.element(),
                rule("us-gaap/alias/net-income-attributable-to-parent@1"),
                Some(Confidence::Exact),
            ),
        ];

        let result = resolve(&facts, &registry);

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_give_no_net_change_in_cash_when_the_filing_reports_the_change_excluding_exchange_rate_effect_next_to_the_effect()
     {
        let us_gaap = UsGaap::new();
        let registry = Registry::new(vec![&us_gaap])
            .expect("Given one plug-in with distinct families, the registry should always start");
        let filing = source("0000320193-24-000123", date(2024, 11, 1));
        let facts = vec![
            fact(
                &filing,
                "CashCashEquivalentsRestrictedCashAndRestrictedCashEquivalentsPeriodIncreaseDecreaseExcludingExchangeRateEffect",
                "-7000000",
            ),
            fact(
                &filing,
                "EffectOfExchangeRateOnCashCashEquivalentsRestrictedCashAndRestrictedCashEquivalents",
                "-2000000",
            ),
        ];

        // The effect resolves to its own concept by its part rule, and no net change in cash.
        let expected_result = vec![(
            CashFlowConcept::ExchangeRateEffect.element(),
            rule("us-gaap/part/exchange-rate-effect@1"),
            Some(Confidence::Synonym),
        )];

        let result = resolve(&facts, &registry);

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_give_no_pretax_income_when_the_filing_reports_the_part_next_to_affordable_housing_income()
     {
        let us_gaap = UsGaap::new();
        let registry = Registry::new(vec![&us_gaap])
            .expect("Given one plug-in with distinct families, the registry should always start");
        let filing = source("0000320193-24-000123", date(2024, 11, 1));
        let facts = vec![
            fact(
                &filing,
                "IncomeLossFromContinuingOperationsBeforeIncomeTaxesMinorityInterestAndIncomeLossFromEquityMethodInvestments",
                "100000000",
            ),
            fact(
                &filing,
                "IncomeLossFromAffordableHousingProjectsEquityMethodInvestments",
                "8000000",
            ),
        ];

        let expected_result: Vec<(CanonicalElementKind, RuleId, Option<Confidence>)> = Vec::new();

        let result = resolve(&facts, &registry);

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_give_no_exchange_rate_effect_when_the_filing_reports_the_part_next_to_the_disposal_group_effect()
     {
        let us_gaap = UsGaap::new();
        let registry = Registry::new(vec![&us_gaap])
            .expect("Given one plug-in with distinct families, the registry should always start");
        let filing = source("0000320193-24-000123", date(2024, 11, 1));
        let facts = vec![
            fact(
                &filing,
                "EffectOfExchangeRateOnCashCashEquivalentsRestrictedCashAndRestrictedCashEquivalents",
                "-2000000",
            ),
            fact(
                &filing,
                "EffectOfExchangeRateOnCashCashEquivalentsRestrictedCashAndRestrictedCashEquivalentsDisposalGroupIncludingDiscontinuedOperations",
                "-100000",
            ),
        ];

        let expected_result: Vec<(CanonicalElementKind, RuleId, Option<Confidence>)> = Vec::new();

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
            IncomeStatementConcept::Revenue.element(),
            rule("us-gaap/part/revenue@1"),
            Some(Confidence::Synonym),
        )];

        let result = resolve(&facts, &registry);

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_give_synonym_with_revenue_part_rule_id_when_the_filing_reports_only_sales_revenue_of_goods()
     {
        let us_gaap = UsGaap::new();
        let registry = Registry::new(vec![&us_gaap])
            .expect("Given one plug-in with distinct families, the registry should always start");
        let filing = source("0000000001-14-000001", date(2014, 2, 27));
        let facts = vec![fact(&filing, "SalesRevenueGoodsNet", "46854000000")];

        let expected_result = vec![(
            IncomeStatementConcept::Revenue.element(),
            rule("us-gaap/part/revenue@1"),
            Some(Confidence::Synonym),
        )];

        let result = resolve(&facts, &registry);

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_give_synonym_with_revenue_part_rule_id_when_the_filing_reports_only_sales_revenue_of_services()
     {
        let us_gaap = UsGaap::new();
        let registry = Registry::new(vec![&us_gaap])
            .expect("Given one plug-in with distinct families, the registry should always start");
        let filing = source("0000000001-14-000001", date(2014, 2, 27));
        let facts = vec![fact(&filing, "SalesRevenueServicesNet", "1250000000")];

        let expected_result = vec![(
            IncomeStatementConcept::Revenue.element(),
            rule("us-gaap/part/revenue@1"),
            Some(Confidence::Synonym),
        )];

        let result = resolve(&facts, &registry);

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_give_no_revenue_when_the_filing_reports_sales_revenue_of_goods_and_services_without_a_total()
     {
        let us_gaap = UsGaap::new();
        let registry = Registry::new(vec![&us_gaap])
            .expect("Given one plug-in with distinct families, the registry should always start");
        let filing = source("0000000001-14-000001", date(2014, 2, 27));
        let facts = vec![
            fact(&filing, "SalesRevenueGoodsNet", "830000000"),
            fact(&filing, "SalesRevenueServicesNet", "1250000000"),
        ];

        let expected_result: Vec<(CanonicalElementKind, RuleId, Option<Confidence>)> = Vec::new();

        let result = resolve(&facts, &registry);

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_give_contract_revenue_as_revenue_when_the_filing_reports_contract_revenue_next_to_sales_revenue_of_goods()
     {
        let us_gaap = UsGaap::new();
        let registry = Registry::new(vec![&us_gaap])
            .expect("Given one plug-in with distinct families, the registry should always start");
        let filing = source("0000000001-19-000001", date(2019, 2, 21));
        let facts = vec![
            fact(
                &filing,
                "RevenueFromContractWithCustomerExcludingAssessedTax",
                "100000000",
            ),
            fact(&filing, "SalesRevenueGoodsNet", "60000000"),
        ];

        let expected_result = vec![CandidateValue::Reported {
            value: BigDecimal::from(100_000_000_i64),
            decimals: Some(Decimals::Finite(-6)),
            confidence: Confidence::Synonym,
        }];

        let result: Vec<CandidateValue> = resolve_aliases(&more_precise(&facts), &registry)
            .candidates
            .into_iter()
            .filter(|candidate| candidate.concept == IncomeStatementConcept::Revenue.element())
            .map(|candidate| candidate.value)
            .collect();

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

        let expected_result: Vec<(CanonicalElementKind, RuleId, Option<Confidence>)> = Vec::new();

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

        let expected_result: Vec<(CanonicalElementKind, RuleId, Option<Confidence>)> = Vec::new();

        let result = resolve(&facts, &registry);

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_give_the_members_of_equity_but_no_equity_when_the_filing_reports_stockholders_equity_next_to_minority_interest()
     {
        let us_gaap = UsGaap::new();
        let registry = Registry::new(vec![&us_gaap])
            .expect("Given one plug-in with distinct families, the registry should always start");
        let filing = source("0000320193-24-000123", date(2024, 11, 1));
        let facts = vec![
            fact(&filing, "StockholdersEquity", "56950000000"),
            fact(&filing, "MinorityInterest", "1000000000"),
        ];

        let expected_result = vec![
            (
                BalanceSheetConcept::EquityAttributableToParent.element(),
                rule("us-gaap/alias/equity-attributable-to-parent@1"),
                Some(Confidence::Exact),
            ),
            (
                BalanceSheetConcept::NoncontrollingInterest.element(),
                rule("us-gaap/alias/noncontrolling-interest@1"),
                Some(Confidence::Exact),
            ),
        ];

        let result = resolve(&facts, &registry);

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_give_liabilities_and_equity_rule_id_when_registry_with_us_gaap_resolves_liabilities_and_stockholders_equity()
     {
        let us_gaap = UsGaap::new();
        let registry = Registry::new(vec![&us_gaap])
            .expect("Given one plug-in with distinct families, the registry should always start");
        let filing = source("0000320193-24-000123", date(2024, 11, 1));
        let facts = vec![fact(
            &filing,
            "LiabilitiesAndStockholdersEquity",
            "7216000000",
        )];

        let expected_result = vec![(
            BalanceSheetConcept::LiabilitiesAndEquity.element(),
            rule("us-gaap/alias/liabilities-and-equity@1"),
            Some(Confidence::Exact),
        )];

        let result = resolve(&facts, &registry);

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_give_temporary_equity_rule_id_when_registry_with_us_gaap_resolves_temporary_equity_carrying_amount_including_portion_attributable_to_noncontrolling_interests()
     {
        let us_gaap = UsGaap::new();
        let registry = Registry::new(vec![&us_gaap])
            .expect("Given one plug-in with distinct families, the registry should always start");
        let filing = source("0000320193-24-000123", date(2024, 11, 1));
        let facts = vec![fact(
            &filing,
            "TemporaryEquityCarryingAmountIncludingPortionAttributableToNoncontrollingInterests",
            "311000000",
        )];

        let expected_result = vec![(
            BalanceSheetConcept::TemporaryEquity.element(),
            rule("us-gaap/alias/temporary-equity@1"),
            Some(Confidence::Exact),
        )];

        let result = resolve(&facts, &registry);

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_give_accounts_payable_rule_id_when_registry_with_us_gaap_resolves_accounts_payable_current()
     {
        let us_gaap = UsGaap::new();
        let registry = Registry::new(vec![&us_gaap])
            .expect("Given one plug-in with distinct families, the registry should always start");
        let filing = source("0000320193-24-000123", date(2024, 11, 1));
        let facts = vec![fact(&filing, "AccountsPayableCurrent", "210000000")];

        let expected_result = vec![(
            BalanceSheetConcept::AccountsPayable.element(),
            rule("us-gaap/alias/accounts-payable@1"),
            Some(Confidence::Exact),
        )];

        let result = resolve(&facts, &registry);

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_give_short_term_debt_rule_id_when_registry_with_us_gaap_resolves_debt_current() {
        let us_gaap = UsGaap::new();
        let registry = Registry::new(vec![&us_gaap])
            .expect("Given one plug-in with distinct families, the registry should always start");
        let filing = source("0000320193-24-000123", date(2024, 11, 1));
        let facts = vec![fact(&filing, "DebtCurrent", "45000000")];

        let expected_result = vec![(
            BalanceSheetConcept::ShortTermDebt.element(),
            rule("us-gaap/alias/short-term-debt@1"),
            Some(Confidence::Exact),
        )];

        let result = resolve(&facts, &registry);

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_give_equity_by_the_part_rule_and_equity_attributable_to_parent_by_the_alias_when_registry_with_us_gaap_resolves_stockholders_equity()
     {
        let us_gaap = UsGaap::new();
        let registry = Registry::new(vec![&us_gaap])
            .expect("Given one plug-in with distinct families, the registry should always start");
        let filing = source("0000320193-24-000123", date(2024, 11, 1));
        let facts = vec![fact(&filing, "StockholdersEquity", "56950000000")];

        let expected_result = vec![
            (
                BalanceSheetConcept::Equity.element(),
                rule("us-gaap/part/equity@1"),
                Some(Confidence::Synonym),
            ),
            (
                BalanceSheetConcept::EquityAttributableToParent.element(),
                rule("us-gaap/alias/equity-attributable-to-parent@1"),
                Some(Confidence::Exact),
            ),
        ];

        let result = resolve(&facts, &registry);

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_give_noncontrolling_interest_rule_id_when_registry_with_us_gaap_resolves_minority_interest()
     {
        let us_gaap = UsGaap::new();
        let registry = Registry::new(vec![&us_gaap])
            .expect("Given one plug-in with distinct families, the registry should always start");
        let filing = source("0000320193-24-000123", date(2024, 11, 1));
        let facts = vec![fact(&filing, "MinorityInterest", "1000000000")];

        let expected_result = vec![(
            BalanceSheetConcept::NoncontrollingInterest.element(),
            rule("us-gaap/alias/noncontrolling-interest@1"),
            Some(Confidence::Exact),
        )];

        let result = resolve(&facts, &registry);

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_give_additional_paid_in_capital_rule_id_when_registry_with_us_gaap_resolves_additional_paid_in_capital()
     {
        let us_gaap = UsGaap::new();
        let registry = Registry::new(vec![&us_gaap])
            .expect("Given one plug-in with distinct families, the registry should always start");
        let filing = source("0000320193-24-000123", date(2024, 11, 1));
        let facts = vec![fact(&filing, "AdditionalPaidInCapital", "19000000")];

        let expected_result = vec![(
            BalanceSheetConcept::AdditionalPaidInCapital.element(),
            rule("us-gaap/alias/additional-paid-in-capital@1"),
            Some(Confidence::Exact),
        )];

        let result = resolve(&facts, &registry);

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_give_treasury_stock_rule_id_when_registry_with_us_gaap_resolves_treasury_stock_value()
    {
        let us_gaap = UsGaap::new();
        let registry = Registry::new(vec![&us_gaap])
            .expect("Given one plug-in with distinct families, the registry should always start");
        let filing = source("0000320193-24-000123", date(2024, 11, 1));
        let facts = vec![fact(&filing, "TreasuryStockValue", "854000000")];

        let expected_result = vec![(
            BalanceSheetConcept::TreasuryStock.element(),
            rule("us-gaap/alias/treasury-stock@1"),
            Some(Confidence::Exact),
        )];

        let result = resolve(&facts, &registry);

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_give_accumulated_other_comprehensive_income_rule_id_when_registry_with_us_gaap_resolves_accumulated_other_comprehensive_income_loss_net_of_tax()
     {
        let us_gaap = UsGaap::new();
        let registry = Registry::new(vec![&us_gaap])
            .expect("Given one plug-in with distinct families, the registry should always start");
        let filing = source("0000320193-24-000123", date(2024, 11, 1));
        let facts = vec![fact(
            &filing,
            "AccumulatedOtherComprehensiveIncomeLossNetOfTax",
            "6000000",
        )];

        let expected_result = vec![(
            BalanceSheetConcept::AccumulatedOtherComprehensiveIncome.element(),
            rule("us-gaap/alias/accumulated-other-comprehensive-income@1"),
            Some(Confidence::Exact),
        )];

        let result = resolve(&facts, &registry);

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_give_synonym_with_temporary_equity_part_rule_id_when_the_filing_reports_only_the_redeemable_noncontrolling_interest()
     {
        let us_gaap = UsGaap::new();
        let registry = Registry::new(vec![&us_gaap])
            .expect("Given one plug-in with distinct families, the registry should always start");
        let filing = source("0000320193-24-000123", date(2024, 11, 1));
        let facts = vec![fact(
            &filing,
            "RedeemableNoncontrollingInterestEquityCarryingAmount",
            "311000000",
        )];

        let expected_result = vec![(
            BalanceSheetConcept::TemporaryEquity.element(),
            rule("us-gaap/part/temporary-equity@1"),
            Some(Confidence::Synonym),
        )];

        let result = resolve(&facts, &registry);

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_give_no_short_term_debt_from_the_us_gaap_lists_when_the_filing_reports_both_parts_of_debt_current()
     {
        let us_gaap = UsGaap::new();
        let registry = Registry::new(vec![&us_gaap])
            .expect("Given one plug-in with distinct families, the registry should always start");
        let filing = source("0000320193-24-000123", date(2024, 11, 1));
        let facts = vec![
            fact(
                &filing,
                "LongTermDebtAndCapitalLeaseObligationsCurrent",
                "45000000",
            ),
            fact(&filing, "ShortTermBorrowings", "756000000"),
        ];

        let expected_result = Vec::<(CanonicalElementKind, RuleId, Option<Confidence>)>::new();

        let result = resolve(&facts, &registry);

        assert_eq!(result, expected_result);
    }

    // A known gap: `LongTermDebtCurrent` sits below a part, so it blocks nothing, and the value
    // leaves out the current maturities.
    #[test]
    fn should_give_short_term_borrowings_alone_when_the_filing_reports_long_term_debt_current_next_to_short_term_borrowings()
     {
        let us_gaap = UsGaap::new();
        let registry = Registry::new(vec![&us_gaap])
            .expect("Given one plug-in with distinct families, the registry should always start");
        let filing = source("0000320193-24-000123", date(2024, 11, 1));
        let facts = vec![
            fact(&filing, "LongTermDebtCurrent", "45000000"),
            fact(&filing, "ShortTermBorrowings", "756000000"),
        ];

        let expected_result = vec![(
            BalanceSheetConcept::ShortTermDebt.element(),
            rule("us-gaap/part/short-term-debt@1"),
            Some(Confidence::Synonym),
        )];

        let result = resolve(&facts, &registry);

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_give_no_temporary_equity_when_the_filing_reports_both_parts_of_temporary_equity() {
        let us_gaap = UsGaap::new();
        let registry = Registry::new(vec![&us_gaap])
            .expect("Given one plug-in with distinct families, the registry should always start");
        let filing = source("0000320193-24-000123", date(2024, 11, 1));
        let facts = vec![
            fact(
                &filing,
                "TemporaryEquityCarryingAmountAttributableToParent",
                "448000000",
            ),
            fact(
                &filing,
                "RedeemableNoncontrollingInterestEquityCarryingAmount",
                "311000000",
            ),
        ];

        let expected_result: Vec<(CanonicalElementKind, RuleId, Option<Confidence>)> = Vec::new();

        let result = resolve(&facts, &registry);

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_give_synonym_with_temporary_equity_part_rule_id_when_the_filing_reports_only_the_common_redeemable_noncontrolling_interest()
     {
        let us_gaap = UsGaap::new();
        let registry = Registry::new(vec![&us_gaap])
            .expect("Given one plug-in with distinct families, the registry should always start");
        let filing = source("0000320193-24-000123", date(2024, 11, 1));
        let facts = vec![fact(
            &filing,
            "RedeemableNoncontrollingInterestEquityCommonCarryingAmount",
            "156000000",
        )];

        let expected_result = vec![(
            BalanceSheetConcept::TemporaryEquity.element(),
            rule("us-gaap/part/temporary-equity@1"),
            Some(Confidence::Synonym),
        )];

        let result = resolve(&facts, &registry);

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_give_synonym_with_temporary_equity_part_rule_id_when_the_filing_reports_only_the_other_redeemable_noncontrolling_interest()
     {
        let us_gaap = UsGaap::new();
        let registry = Registry::new(vec![&us_gaap])
            .expect("Given one plug-in with distinct families, the registry should always start");
        let filing = source("0000320193-24-000123", date(2024, 11, 1));
        let facts = vec![fact(
            &filing,
            "RedeemableNoncontrollingInterestEquityOtherCarryingAmount",
            "180000000",
        )];

        let expected_result = vec![(
            BalanceSheetConcept::TemporaryEquity.element(),
            rule("us-gaap/part/temporary-equity@1"),
            Some(Confidence::Synonym),
        )];

        let result = resolve(&facts, &registry);

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_give_synonym_with_temporary_equity_part_rule_id_when_the_filing_reports_only_the_preferred_redeemable_noncontrolling_interest()
     {
        let us_gaap = UsGaap::new();
        let registry = Registry::new(vec![&us_gaap])
            .expect("Given one plug-in with distinct families, the registry should always start");
        let filing = source("0000320193-24-000123", date(2024, 11, 1));
        let facts = vec![fact(
            &filing,
            "RedeemableNoncontrollingInterestEquityPreferredCarryingAmount",
            "50000000",
        )];

        let expected_result = vec![(
            BalanceSheetConcept::TemporaryEquity.element(),
            rule("us-gaap/part/temporary-equity@1"),
            Some(Confidence::Synonym),
        )];

        let result = resolve(&facts, &registry);

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_give_synonym_with_temporary_equity_part_rule_id_when_the_filing_reports_only_the_fair_value_of_the_other_redeemable_noncontrolling_interest()
     {
        let us_gaap = UsGaap::new();
        let registry = Registry::new(vec![&us_gaap])
            .expect("Given one plug-in with distinct families, the registry should always start");
        let filing = source("0000320193-24-000123", date(2024, 11, 1));
        let facts = vec![fact(
            &filing,
            "RedeemableNoncontrollingInterestEquityOtherFairValue",
            "967500000",
        )];

        let expected_result = vec![(
            BalanceSheetConcept::TemporaryEquity.element(),
            rule("us-gaap/part/temporary-equity@1"),
            Some(Confidence::Synonym),
        )];

        let result = resolve(&facts, &registry);

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_give_the_redeemable_noncontrolling_interest_when_the_filing_reports_it_next_to_the_common_part()
     {
        let us_gaap = UsGaap::new();
        let registry = Registry::new(vec![&us_gaap])
            .expect("Given one plug-in with distinct families, the registry should always start");
        let filing = source("0000320193-24-000123", date(2024, 11, 1));
        let facts = vec![
            fact(
                &filing,
                "RedeemableNoncontrollingInterestEquityCarryingAmount",
                "311000000",
            ),
            fact(
                &filing,
                "RedeemableNoncontrollingInterestEquityCommonCarryingAmount",
                "156000000",
            ),
        ];

        let expected_result = vec![CandidateValue::Reported {
            value: BigDecimal::from(311_000_000_i64),
            decimals: Some(Decimals::Finite(-6)),
            confidence: Confidence::Synonym,
        }];

        let result = temporary_equity_values(&facts, &registry);

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_give_the_carrying_amount_when_the_filing_reports_it_next_to_the_fair_value_of_the_redeemable_noncontrolling_interest()
     {
        let us_gaap = UsGaap::new();
        let registry = Registry::new(vec![&us_gaap])
            .expect("Given one plug-in with distinct families, the registry should always start");
        let filing = source("0000320193-24-000123", date(2024, 11, 1));
        let facts = vec![
            fact(
                &filing,
                "RedeemableNoncontrollingInterestEquityCarryingAmount",
                "311000000",
            ),
            fact(
                &filing,
                "RedeemableNoncontrollingInterestEquityOtherFairValue",
                "967500000",
            ),
        ];

        let expected_result = vec![CandidateValue::Reported {
            value: BigDecimal::from(311_000_000_i64),
            decimals: Some(Decimals::Finite(-6)),
            confidence: Confidence::Synonym,
        }];

        let result = temporary_equity_values(&facts, &registry);

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_give_the_carrying_amount_when_the_filing_reports_it_as_zero_next_to_the_fair_value_of_the_redeemable_noncontrolling_interest()
     {
        let us_gaap = UsGaap::new();
        let registry = Registry::new(vec![&us_gaap])
            .expect("Given one plug-in with distinct families, the registry should always start");
        let filing = source("0000320193-24-000123", date(2024, 11, 1));
        let facts = vec![
            fact(
                &filing,
                "RedeemableNoncontrollingInterestEquityCarryingAmount",
                "0",
            ),
            fact(
                &filing,
                "RedeemableNoncontrollingInterestEquityOtherFairValue",
                "967500000",
            ),
        ];

        let expected_result = vec![CandidateValue::Reported {
            value: BigDecimal::from(0),
            decimals: Some(Decimals::Finite(-6)),
            confidence: Confidence::Synonym,
        }];

        let result = temporary_equity_values(&facts, &registry);

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_give_no_temporary_equity_when_the_filing_reports_both_parts_of_temporary_equity_next_to_the_fair_value()
     {
        let us_gaap = UsGaap::new();
        let registry = Registry::new(vec![&us_gaap])
            .expect("Given one plug-in with distinct families, the registry should always start");
        let filing = source("0000320193-24-000123", date(2024, 11, 1));
        let facts = vec![
            fact(
                &filing,
                "TemporaryEquityCarryingAmountAttributableToParent",
                "448000000",
            ),
            fact(
                &filing,
                "RedeemableNoncontrollingInterestEquityCarryingAmount",
                "311000000",
            ),
            fact(
                &filing,
                "RedeemableNoncontrollingInterestEquityOtherFairValue",
                "967500000",
            ),
        ];

        let expected_result: Vec<CandidateValue> = Vec::new();

        let result = temporary_equity_values(&facts, &registry);

        assert_eq!(result, expected_result);
    }

    // A known gap: a part after the first two is no other part, so it blocks nothing, and the value
    // leaves out the redeemable noncontrolling interest.
    #[test]
    fn should_give_the_parent_part_alone_when_the_filing_reports_it_next_to_the_fair_value_of_the_redeemable_noncontrolling_interest()
     {
        let us_gaap = UsGaap::new();
        let registry = Registry::new(vec![&us_gaap])
            .expect("Given one plug-in with distinct families, the registry should always start");
        let filing = source("0000320193-24-000123", date(2024, 11, 1));
        let facts = vec![
            fact(
                &filing,
                "TemporaryEquityCarryingAmountAttributableToParent",
                "448000000",
            ),
            fact(
                &filing,
                "RedeemableNoncontrollingInterestEquityOtherFairValue",
                "967500000",
            ),
        ];

        let expected_result = vec![CandidateValue::Reported {
            value: BigDecimal::from(448_000_000_i64),
            decimals: Some(Decimals::Finite(-6)),
            confidence: Confidence::Synonym,
        }];

        let result = temporary_equity_values(&facts, &registry);

        assert_eq!(result, expected_result);
    }

    // A known gap: the parts below the redeemable noncontrolling interest block nothing, so the
    // first counts alone.
    #[test]
    fn should_give_the_common_part_alone_when_the_filing_reports_it_next_to_the_preferred_part() {
        let us_gaap = UsGaap::new();
        let registry = Registry::new(vec![&us_gaap])
            .expect("Given one plug-in with distinct families, the registry should always start");
        let filing = source("0000320193-24-000123", date(2024, 11, 1));
        let facts = vec![
            fact(
                &filing,
                "RedeemableNoncontrollingInterestEquityCommonCarryingAmount",
                "100000000",
            ),
            fact(
                &filing,
                "RedeemableNoncontrollingInterestEquityPreferredCarryingAmount",
                "50000000",
            ),
        ];

        let expected_result = vec![CandidateValue::Reported {
            value: BigDecimal::from(100_000_000_i64),
            decimals: Some(Decimals::Finite(-6)),
            confidence: Confidence::Synonym,
        }];

        let result = temporary_equity_values(&facts, &registry);

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
                PerShareConcept::EarningsPerShareBasic.element(),
                rule("us-gaap/alias/earnings-per-share-basic@1"),
                Some(Confidence::Synonym),
            ),
            (
                PerShareConcept::EarningsPerShareDiluted.element(),
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
            entity: apple(),
            key: eps.key(),
            ranks: vec![1],
            standard_concept: Some(PerShareConcept::EarningsPerShareBasic.element()),
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

        let expected_result: Vec<(CanonicalElementKind, RuleId, Option<Confidence>)> = Vec::new();

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

        let expected_result: Vec<(CanonicalElementKind, RuleId, Option<Confidence>)> = Vec::new();

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

        let expected_result: Vec<(CanonicalElementKind, RuleId, Option<Confidence>)> = Vec::new();

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

        let expected_result: Vec<(CanonicalElementKind, RuleId, Option<Confidence>)> = Vec::new();

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
            CashFlowConcept::DepreciationAmortization.element(),
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

        let expected_result: Vec<(CanonicalElementKind, RuleId, Option<Confidence>)> = Vec::new();

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
            .filter(|candidate| candidate.concept == BalanceSheetConcept::Equity.element())
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
            role: "http://fasb.org/us-gaap/role/statement/StatementOfFinancialPositionClassifiedFirstAlternative".to_string(),
            parent: concept("srt", "LifeInsuranceInForceNet"),
            child: concept("srt", "LifeInsuranceInForceCeded"),
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
