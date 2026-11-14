//! # Taxonomy
//!
//! Provides the [`Taxonomy`] trait that each plug-in implements, and the [`registry::Registry`] that
//! routes facts to the plug-ins.
//!
//! The core resolves facts through this trait only, so it knows no taxonomy. A plug-in implements
//! the trait with static data compiled into the binary.
//!
//! ## Modules
//!
//! - [`family`]: How a plug-in declares a namespace family.
//! - [`reference`](mod@reference): The alias lists, the calculation tree, and the element reference data.
//! - [`registry`]: Routes a fact to the plug-in that resolves its family, and splits namespaces.

use self::family::FamilyDeclaration;
use self::reference::{CalculationTree, ConceptAliases, ElementReference};
use super::concept::ConceptName;
use super::release::TaxonomyRelease;

pub mod family;
pub mod reference;
pub mod registry;

/// The knowledge of one taxonomy, for example US GAAP.
///
/// Accounting identities and period identities are not part of the trait. They are core rules, the
/// same for every taxonomy.
pub trait Taxonomy: Send + Sync {
    /// Returns the release that the tables of this plug-in come from, for example `us-gaap/2026`.
    fn release(&self) -> &TaxonomyRelease;

    /// Returns every family this plug-in declares: its URI patterns, its Notes `version` prefix,
    /// and whether the plug-in resolves its facts.
    fn families(&self) -> &[FamilyDeclaration];

    /// Returns, per standard concept, the taxonomy concepts in priority order, with the rule ID.
    fn aliases(&self) -> &[ConceptAliases];

    /// Returns the arcs of the calculation linkbase.
    fn calculation_tree(&self) -> &CalculationTree;

    /// Returns the reference data of one element, or `None` if the plug-in has none, as for an
    /// extension concept.
    fn element(&self, concept: &ConceptName) -> Option<&ElementReference>;

    /// Returns the reference data of every element of the release, and of every retired element.
    fn elements(&self) -> &[ElementReference];
}
