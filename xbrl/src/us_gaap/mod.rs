//! # US-GAAP Taxonomy
//!
//! Provides [`UsGaap`], the plug-in for the US GAAP taxonomy of the FASB and the other SEC
//! taxonomies that US GAAP filings use.
//!
//! The plug-in builds its data from the tables of one FASB release, which `us_gaap_generator`
//! writes into [`taxonomy`]. It resolves the families `us-gaap`, `srt`, and `dei`. It declares
//! the other SEC taxonomies without resolving them, so the registry can split their URIs.
//!
//! ## Modules
//!
//! - [`mappings`]: The alias lists of the standard concepts.
//! - [`table`]: The row types of the generated tables.
//! - [`taxonomy`]: The generated tables of the release: calculation arcs and element data.

use std::str::FromStr;

use bigdecimal::BigDecimal;

use crate::core::concept::{ConceptName, NamespaceFamily};
use crate::core::release::{Sha256, TaxonomyRelease, TaxonomyReleaseId};
use crate::core::taxonomy::Taxonomy;
use crate::core::taxonomy::family::{FamilyDeclaration, UriPattern};
use crate::core::taxonomy::reference::{
    CalculationArc, CalculationTree, ConceptAliases, ElementReference,
};

pub mod mappings;
pub mod table;
// `us_gaap_generator` writes this file. rustfmt must not reflow it, so a run of the generator
// gives the checked-in bytes.
#[rustfmt::skip]
pub mod taxonomy;

/// The family of the concepts of the element schema.
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
/// SEC, and of XBRL US, which published the releases of 2009 and 2010.
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
        name: "ecd",
        uri_patterns: &["http://xbrl.sec.gov/ecd/{release}"],
        resolves: false,
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
        name: "exch",
        uri_patterns: &["http://xbrl.sec.gov/exch/{release}"],
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
/// It holds the release, the family declarations, the calculation tree, and the element
/// reference data of the generated tables. [`Taxonomy::aliases`] returns no alias list yet.
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
    calculation_tree: CalculationTree,
    elements: Vec<ElementReference>,
}

impl UsGaap {
    /// Creates a [`UsGaap`] from the generated tables.
    ///
    /// # Panics
    ///
    /// Panics if a value of the generated tables or of the family declarations breaks the rule of
    /// its type. The unit tests build the plug-in, so a bad table fails them.
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
        &[]
    }

    fn calculation_tree(&self) -> &CalculationTree {
        &self.calculation_tree
    }

    /// Finds the element by a binary search, because the generator sorts the elements by name.
    fn element(&self, concept: &ConceptName) -> Option<&ElementReference> {
        if concept.family().as_str() != US_GAAP_FAMILY {
            return None;
        }
        self.elements
            .binary_search_by(|element| element.concept.local_name().cmp(concept.local_name()))
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

/// Returns the calculation arc of one row of the tables.
fn calculation_arc(row: &table::CalculationRow) -> CalculationArc {
    CalculationArc {
        parent: concept(row.parent_family, row.parent_name),
        child: concept(row.child_family, row.child_name),
        weight: BigDecimal::from_str(row.weight)
            .expect("Given a weight of the tables, the parse should always succeed"),
    }
}

/// Returns the element reference of one row of the tables.
fn element_reference(row: &table::ElementRow, release: &TaxonomyReleaseId) -> ElementReference {
    ElementReference {
        concept: concept(US_GAAP_FAMILY, row.name),
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
    use crate::core::fact::TaxonomyVersion;
    use crate::core::taxonomy::reference::{Balance, PeriodType};
    use crate::core::taxonomy::registry::Registry;

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
    fn should_return_release_with_checksum_of_manifest_when_created() {
        let us_gaap = UsGaap::new();

        let expected_result = TaxonomyRelease {
            id: release_id(),
            checksum: Sha256::from_str(
                "e9b4878ac485e7591bd24fb9e5ca156fe81273afd293f9e592d147af555f7f35",
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
    fn should_return_no_reference_data_when_family_is_not_us_gaap() {
        let us_gaap = UsGaap::new();
        let srt_concept = concept("srt", "Assets");

        let expected_result = None;

        let result = us_gaap.element(&srt_concept);

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_return_every_concept_of_schema_when_listing_elements() {
        let us_gaap = UsGaap::new();

        let expected_result = 17_179;

        let result = us_gaap.elements().len();

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_return_arc_with_negative_weight_when_child_is_subtracted() {
        let us_gaap = UsGaap::new();

        let expected_result = Some(&CalculationArc {
            parent: concept("srt", "LifeInsuranceInForceNet"),
            child: concept("srt", "LifeInsuranceInForceCeded"),
            weight: BigDecimal::from_str("-1.0")
                .expect("Given a hardcoded decimal, the parse should always succeed"),
        });

        let result = us_gaap.calculation_tree().arcs.get(1);

        assert_eq!(result, expected_result);
    }
}
