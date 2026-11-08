//! # Fake Taxonomy
//!
//! Provides [`FakeTaxonomy`], a plug-in that declares families and holds no tables, for unit tests
//! that need a [`Registry`](super::registry::Registry).

use super::Taxonomy;
use super::family::{FamilyDeclaration, UriPattern};
use super::reference::{CalculationTree, ConceptAliases, ElementReference};
use crate::core::concept::{ConceptName, NamespaceFamily};
use crate::core::release::{Sha256, TaxonomyRelease, TaxonomyReleaseId};

/// A plug-in with a release and family declarations, and with no aliases, tree, or elements.
pub struct FakeTaxonomy {
    release: TaxonomyRelease,
    families: Vec<FamilyDeclaration>,
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

/// Returns a declaration of the family `name` with one URI pattern and `name` as Notes prefix.
pub fn declaration(name: &str, pattern: &str, resolves: bool) -> FamilyDeclaration {
    let family = NamespaceFamily::new(name)
        .expect("Given a hardcoded family without whitespace, the creation should always succeed");
    let pattern = UriPattern::new(pattern).expect(
        "Given a hardcoded pattern with one placeholder, the creation should always succeed",
    );
    FamilyDeclaration::new(family, vec![pattern], name, resolves)
        .expect("Given a hardcoded declaration, the creation should always succeed")
}

/// Returns a [`FakeTaxonomy`] of the release `release` that declares `families`.
pub fn plugin(release: &str, families: Vec<FamilyDeclaration>) -> FakeTaxonomy {
    FakeTaxonomy {
        release: TaxonomyRelease {
            id: TaxonomyReleaseId::new(release).expect(
                "Given a hardcoded release ID with a year, the creation should always succeed",
            ),
            checksum: Sha256::new([0; 32]),
        },
        families,
        tree: CalculationTree::default(),
    }
}
