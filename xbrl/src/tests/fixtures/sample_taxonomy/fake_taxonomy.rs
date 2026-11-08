//! # Fake Taxonomy
//!
//! Provides [`FakeTaxonomy`], a plug-in that declares families and holds alias lists, for unit tests
//! that need a [`Registry`](crate::core::taxonomy::registry::Registry).

use crate::core::concept::{ConceptName, NamespaceFamily};
use crate::core::release::{Sha256, TaxonomyRelease, TaxonomyReleaseId};
use crate::core::taxonomy::Taxonomy;
use crate::core::taxonomy::family::{FamilyDeclaration, UriPattern};
use crate::core::taxonomy::reference::{CalculationTree, ConceptAliases, ElementReference};

/// A plug-in with a release, family declarations, and alias lists, and with no tree or elements.
pub struct FakeTaxonomy {
    release: TaxonomyRelease,
    families: Vec<FamilyDeclaration>,
    aliases: Vec<ConceptAliases>,
    tree: CalculationTree,
}

impl Taxonomy for FakeTaxonomy {
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
        &self.tree
    }

    fn element(&self, _concept: &ConceptName) -> Option<&ElementReference> {
        None
    }

    fn elements(&self) -> &[ElementReference] {
        &[]
    }
}

/// Returns a declaration of the family `name` with one URI pattern and `name` as Notes prefix.
///
/// # Panics
///
/// Panics if `name` is not a family or a Notes `version` prefix, or if `pattern` is not a URI pattern.
#[must_use]
pub fn declaration(name: &str, pattern: &str, resolves: bool) -> FamilyDeclaration {
    let family = NamespaceFamily::new(name)
        .expect("Given a hardcoded family without whitespace, the creation should always succeed");
    let pattern = UriPattern::new(pattern).expect(
        "Given a hardcoded pattern with one placeholder, the creation should always succeed",
    );
    FamilyDeclaration::new(family, vec![pattern], name, resolves)
        .expect("Given a hardcoded declaration, the creation should always succeed")
}

/// Returns a [`FakeTaxonomy`] of the release `release` that declares `families` and has no alias
/// lists.
///
/// # Panics
///
/// Panics if `release` is not a release ID.
#[must_use]
pub fn plugin(release: &str, families: Vec<FamilyDeclaration>) -> FakeTaxonomy {
    plugin_with_aliases(release, families, Vec::new())
}

/// Returns a [`FakeTaxonomy`] of the release `release` that declares `families` and has the alias
/// lists `aliases`.
///
/// # Panics
///
/// Panics if `release` is not a release ID.
#[must_use]
pub fn plugin_with_aliases(
    release: &str,
    families: Vec<FamilyDeclaration>,
    aliases: Vec<ConceptAliases>,
) -> FakeTaxonomy {
    FakeTaxonomy {
        release: TaxonomyRelease {
            id: TaxonomyReleaseId::new(release).expect(
                "Given a hardcoded release ID with a year, the creation should always succeed",
            ),
            checksum: Sha256::new([0; 32]),
        },
        families,
        aliases,
        tree: CalculationTree::default(),
    }
}
