//! # Fake Taxonomy
//!
//! Provides [`us_gaap`], a plug-in that declares the families of the fixture filing, for the
//! integration tests. Unit tests use the fake in `src/tests/fixtures`, which `tests/` cannot see.

use xbrl::core::concept::{ConceptName, NamespaceFamily};
use xbrl::core::release::{Sha256, TaxonomyRelease, TaxonomyReleaseId};
use xbrl::core::taxonomy::Taxonomy;
use xbrl::core::taxonomy::family::{FamilyDeclaration, UriPattern};
use xbrl::core::taxonomy::reference::{CalculationTree, ConceptAliases, ElementReference};

/// A plug-in with family declarations and no tables.
pub struct FakeUsGaap {
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

/// Returns a plug-in that declares `us-gaap`, `dei`, `srt`, and `ecd`, the families of the
/// fixture filing, and holds no tables.
pub fn us_gaap() -> FakeUsGaap {
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
