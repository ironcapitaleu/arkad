//! # Registry
//!
//! Provides [`Registry`], which routes each fact to the plug-in that resolves its namespace family,
//! and splits a namespace URI or a Notes `version` into a family and a taxonomy version.

use std::collections::BTreeMap;

use thiserror::Error;

use super::Taxonomy;
use super::family::FamilyDeclaration;
use crate::core::concept::NamespaceFamily;
use crate::core::fact::TaxonomyVersion;
use crate::core::release::TaxonomyReleaseId;

/// The plug-ins of one run, keyed by the namespace families they declare.
///
/// A filing can mix families, for example a 20-F with `ifrs-full` and `dei`, so the registry routes
/// per fact. Two rules hold for every family:
///
/// - At most one plug-in resolves the family.
/// - Every declaration of the family agrees: the same URI patterns and the same Notes `version`
///   prefix. So a plug-in can declare, without resolving, a family that another plug-in resolves.
///
/// Two different families must not overlap: they share no Notes `version` prefix, and no URI can
/// match a pattern of each. So a URI or a Notes `version` splits into at most one family.
///
/// A family that no plug-in declares has no split. The parser keeps its facts with the full URI or
/// Notes `version` as family and version, and returns a finding that names the family. Such facts
/// do not join between sources or across releases until a plug-in declares the family.
pub struct Registry<'a> {
    plugins: Vec<&'a dyn Taxonomy>,
    families: BTreeMap<NamespaceFamily, Entry>,
}

struct Entry {
    declaration: FamilyDeclaration,
    first_declarer: usize,
    resolver: Option<usize>,
}

impl<'a> Registry<'a> {
    /// Creates a [`Registry`] from the plug-ins of one run.
    ///
    /// # Errors
    ///
    /// Returns [`RegistryError`] if a plug-in declares one family twice, two plug-ins resolve one
    /// family, two declarations of one family disagree, or two families overlap.
    pub fn new(plugins: Vec<&'a dyn Taxonomy>) -> Result<Self, RegistryError> {
        let mut families: BTreeMap<NamespaceFamily, Entry> = BTreeMap::new();
        for (index, plugin) in plugins.iter().enumerate() {
            reject_repeated_family(*plugin)?;
            for declaration in plugin.families() {
                add_declaration(&mut families, &plugins, index, declaration)?;
            }
        }
        reject_overlap(&families)?;
        Ok(Self { plugins, families })
    }

    /// Returns the plug-in that resolves the family, or `None` if no plug-in resolves it.
    #[must_use]
    pub fn resolver(&self, family: &NamespaceFamily) -> Option<&'a dyn Taxonomy> {
        let index = self.families.get(family)?.resolver?;
        Some(self.plugins[index])
    }

    /// Returns the declaration of the family, or `None` if no plug-in declares it.
    #[must_use]
    pub fn declaration(&self, family: &NamespaceFamily) -> Option<&FamilyDeclaration> {
        Some(&self.families.get(family)?.declaration)
    }

    /// Splits a namespace URI into a declared family and a taxonomy version, for example
    /// `http://fasb.org/us-gaap/2017-01-31` into `us-gaap` and `us-gaap/2017`.
    ///
    /// Returns `None` if no declared pattern matches. The parser then decides whether the URI is an
    /// extension of the filing or an undeclared family.
    #[must_use]
    pub fn split_uri(&self, uri: &str) -> Option<(NamespaceFamily, TaxonomyVersion)> {
        self.families.iter().find_map(|(family, entry)| {
            entry
                .declaration
                .version_of_uri(uri)
                .map(|version| (family.clone(), version))
        })
    }

    /// Splits a Notes `version`, for example `us-gaap/2017`, into a declared family and a
    /// taxonomy version.
    ///
    /// Returns `None` if no declared prefix matches. The parser then decides whether the `version`
    /// marks an extension tag or an undeclared family.
    #[must_use]
    pub fn split_notes_version(
        &self,
        notes_version: &str,
    ) -> Option<(NamespaceFamily, TaxonomyVersion)> {
        self.families.iter().find_map(|(family, entry)| {
            entry
                .declaration
                .version_of_notes(notes_version)
                .map(|version| (family.clone(), version))
        })
    }
}

fn reject_repeated_family(plugin: &dyn Taxonomy) -> Result<(), RegistryError> {
    let declarations = plugin.families();
    for (index, declaration) in declarations.iter().enumerate() {
        if declarations[..index]
            .iter()
            .any(|earlier| earlier.family() == declaration.family())
        {
            return Err(RegistryError::RepeatedFamilyDeclaration {
                family: declaration.family().clone(),
                release: plugin.release().id.clone(),
            });
        }
    }
    Ok(())
}

fn add_declaration(
    families: &mut BTreeMap<NamespaceFamily, Entry>,
    plugins: &[&dyn Taxonomy],
    index: usize,
    declaration: &FamilyDeclaration,
) -> Result<(), RegistryError> {
    let resolver = declaration.resolves().then_some(index);
    let Some(entry) = families.get_mut(declaration.family()) else {
        families.insert(
            declaration.family().clone(),
            Entry {
                declaration: declaration.clone(),
                first_declarer: index,
                resolver,
            },
        );
        return Ok(());
    };
    if !entry.declaration.agrees_with(declaration) {
        return Err(RegistryError::InconsistentFamilyDeclaration {
            family: declaration.family().clone(),
            first: release_id(plugins, entry.first_declarer),
            second: release_id(plugins, index),
        });
    }
    match (entry.resolver, resolver) {
        (Some(first), Some(_)) => Err(RegistryError::DuplicateFamilyResolver {
            family: declaration.family().clone(),
            first: release_id(plugins, first),
            second: release_id(plugins, index),
        }),
        (None, Some(_)) => {
            entry.resolver = resolver;
            Ok(())
        }
        _ => Ok(()),
    }
}

fn reject_overlap(families: &BTreeMap<NamespaceFamily, Entry>) -> Result<(), RegistryError> {
    let entries: Vec<&Entry> = families.values().collect();
    for (index, entry) in entries.iter().enumerate() {
        if let Some(other) = entries[index + 1..]
            .iter()
            .find(|other| entry.declaration.overlaps(&other.declaration))
        {
            return Err(RegistryError::OverlappingFamilyDeclaration {
                first: entry.declaration.family().clone(),
                second: other.declaration.family().clone(),
            });
        }
    }
    Ok(())
}

fn release_id(plugins: &[&dyn Taxonomy], index: usize) -> TaxonomyReleaseId {
    plugins[index].release().id.clone()
}

/// The error of [`Registry::new`] when the declarations of the plug-ins conflict.
#[non_exhaustive]
#[derive(Debug, Error, Clone, PartialEq, Eq)]
pub enum RegistryError {
    /// Two plug-ins resolve one family, so a fact of the family has no single resolver.
    #[error(
        "[DuplicateFamilyResolver] Plug-ins '{first}' and '{second}' both resolve the family '{family}'"
    )]
    DuplicateFamilyResolver {
        /// The family.
        family: NamespaceFamily,
        /// The release of the first plug-in that resolves the family.
        first: TaxonomyReleaseId,
        /// The release of the second plug-in that resolves the family.
        second: TaxonomyReleaseId,
    },

    /// Two plug-ins declare one family with other URI patterns or another Notes `version` prefix,
    /// so the split of a URI would depend on the order of the plug-ins.
    #[error(
        "[InconsistentFamilyDeclaration] Plug-ins '{first}' and '{second}' declare the family '{family}' with different URI patterns or Notes version prefixes"
    )]
    InconsistentFamilyDeclaration {
        /// The family.
        family: NamespaceFamily,
        /// The release of the plug-in that declared the family first.
        first: TaxonomyReleaseId,
        /// The release of the plug-in whose declaration disagrees.
        second: TaxonomyReleaseId,
    },

    /// Two families share a Notes `version` prefix, or have URI patterns whose text before and after
    /// the release overlaps, so the split would depend on the order of the families.
    #[error(
        "[OverlappingFamilyDeclaration] The families '{first}' and '{second}' share a Notes version prefix or have URI patterns whose text before and after the release overlaps"
    )]
    OverlappingFamilyDeclaration {
        /// The first family, in sort order.
        first: NamespaceFamily,
        /// The second family, in sort order.
        second: NamespaceFamily,
    },

    /// One plug-in declares one family twice.
    #[error("[RepeatedFamilyDeclaration] Plug-in '{release}' declares the family '{family}' twice")]
    RepeatedFamilyDeclaration {
        /// The family.
        family: NamespaceFamily,
        /// The release of the plug-in.
        release: TaxonomyReleaseId,
    },
}

#[cfg(test)]
mod tests {
    use pretty_assertions::assert_eq;

    use super::*;
    use crate::core::concept::ConceptName;
    use crate::core::release::{Sha256, TaxonomyRelease};
    use crate::core::taxonomy::family::UriPattern;
    use crate::core::taxonomy::reference::{CalculationTree, ConceptAliases, ElementReference};

    struct FakeTaxonomy {
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

    fn family(text: &str) -> NamespaceFamily {
        NamespaceFamily::new(text).expect(
            "Given a hardcoded family without whitespace, the creation should always succeed",
        )
    }

    fn release_id(text: &str) -> TaxonomyReleaseId {
        TaxonomyReleaseId::new(text)
            .expect("Given a hardcoded release ID with a year, the creation should always succeed")
    }

    fn declaration(name: &str, pattern: &str, resolves: bool) -> FamilyDeclaration {
        let pattern = UriPattern::new(pattern).expect(
            "Given a hardcoded pattern with one placeholder, the creation should always succeed",
        );
        FamilyDeclaration::new(family(name), vec![pattern], name, resolves)
            .expect("Given a hardcoded declaration, the creation should always succeed")
    }

    fn plugin(release: &str, families: Vec<FamilyDeclaration>) -> FakeTaxonomy {
        FakeTaxonomy {
            release: TaxonomyRelease {
                id: release_id(release),
                checksum: Sha256::new([0; 32]),
            },
            families,
            tree: CalculationTree::default(),
        }
    }

    fn us_gaap() -> FakeTaxonomy {
        plugin(
            "us-gaap/2026",
            vec![
                declaration("us-gaap", "http://fasb.org/us-gaap/{release}", true),
                declaration("dei", "http://xbrl.sec.gov/dei/{release}", true),
                declaration("country", "http://xbrl.sec.gov/country/{release}", false),
            ],
        )
    }

    fn version(text: &str) -> TaxonomyVersion {
        TaxonomyVersion::new(text).expect(
            "Given a hardcoded version without whitespace, the creation should always succeed",
        )
    }

    fn resolver_release(registry: &Registry<'_>, name: &str) -> Option<TaxonomyReleaseId> {
        registry
            .resolver(&family(name))
            .map(|plugin| plugin.release().id.clone())
    }

    const fn implements_auto_traits<T: Sized + Send + Sync + Unpin>() {}
    #[test]
    const fn should_implement_auto_traits_for_registry() {
        implements_auto_traits::<Registry<'static>>();
    }

    #[test]
    const fn should_implement_auto_traits_for_registry_error() {
        implements_auto_traits::<RegistryError>();
    }

    #[test]
    fn should_reject_registry_when_two_plugins_resolve_one_family() {
        let first = us_gaap();
        let second = plugin(
            "ifrs-full/2025",
            vec![declaration(
                "dei",
                "http://xbrl.sec.gov/dei/{release}",
                true,
            )],
        );
        let plugins: Vec<&dyn Taxonomy> = vec![&first, &second];

        let expected_result = Some(RegistryError::DuplicateFamilyResolver {
            family: family("dei"),
            first: release_id("us-gaap/2026"),
            second: release_id("ifrs-full/2025"),
        });

        let result = Registry::new(plugins).err();

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_accept_registry_when_two_plugins_declare_one_family_they_do_not_resolve() {
        let first = us_gaap();
        let second = plugin(
            "ifrs-full/2025",
            vec![declaration(
                "country",
                "http://xbrl.sec.gov/country/{release}",
                false,
            )],
        );
        let plugins: Vec<&dyn Taxonomy> = vec![&first, &second];

        Registry::new(plugins).expect("Given two agreeing declarations without a second resolver, the creation should always succeed");
    }

    #[test]
    fn should_route_to_resolver_when_another_plugin_only_declares_the_family() {
        let declaring = plugin(
            "ifrs-full/2025",
            vec![declaration(
                "dei",
                "http://xbrl.sec.gov/dei/{release}",
                false,
            )],
        );
        let resolving = us_gaap();
        let plugins: Vec<&dyn Taxonomy> = vec![&declaring, &resolving];

        let expected_result = Some(release_id("us-gaap/2026"));

        let registry = Registry::new(plugins).expect(
            "Given agreeing declarations and one resolver, the creation should always succeed",
        );
        let result = resolver_release(&registry, "dei");

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_reject_registry_when_two_declarations_of_one_family_disagree() {
        let first = us_gaap();
        let second = plugin(
            "ifrs-full/2025",
            vec![declaration(
                "country",
                "https://xbrl.sec.gov/country/{release}",
                false,
            )],
        );
        let plugins: Vec<&dyn Taxonomy> = vec![&first, &second];

        let expected_result = Some(RegistryError::InconsistentFamilyDeclaration {
            family: family("country"),
            first: release_id("us-gaap/2026"),
            second: release_id("ifrs-full/2025"),
        });

        let result = Registry::new(plugins).err();

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_route_no_fact_when_family_is_only_declared() {
        let us_gaap = us_gaap();
        let plugins: Vec<&dyn Taxonomy> = vec![&us_gaap];

        let expected_result = None;

        let registry =
            Registry::new(plugins).expect("Given one plug-in, the creation should always succeed");
        let result = resolver_release(&registry, "country");

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_split_uri_when_a_declared_pattern_matches() {
        let us_gaap = us_gaap();
        let plugins: Vec<&dyn Taxonomy> = vec![&us_gaap];
        let uri = "http://fasb.org/us-gaap/2017-01-31";

        let expected_result = Some((family("us-gaap"), version("us-gaap/2017")));

        let registry =
            Registry::new(plugins).expect("Given one plug-in, the creation should always succeed");
        let result = registry.split_uri(uri);

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_split_notes_version_when_family_is_only_declared() {
        let us_gaap = us_gaap();
        let plugins: Vec<&dyn Taxonomy> = vec![&us_gaap];
        let notes_version = "country/2024";

        let expected_result = Some((family("country"), version("country/2024")));

        let registry =
            Registry::new(plugins).expect("Given one plug-in, the creation should always succeed");
        let result = registry.split_notes_version(notes_version);

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_not_split_uri_when_no_plugin_declares_its_family() {
        let us_gaap = us_gaap();
        let plugins: Vec<&dyn Taxonomy> = vec![&us_gaap];
        let uri = "http://xbrl.sec.gov/ecd/2023";

        let expected_result = None;

        let registry =
            Registry::new(plugins).expect("Given one plug-in, the creation should always succeed");
        let result = registry.split_uri(uri);

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_print_both_releases_when_two_plugins_resolve_one_family() {
        let error = RegistryError::DuplicateFamilyResolver {
            family: family("dei"),
            first: release_id("us-gaap/2026"),
            second: release_id("ifrs-full/2025"),
        };

        let expected_result = "[DuplicateFamilyResolver] Plug-ins 'us-gaap/2026' and 'ifrs-full/2025' both resolve the family 'dei'";

        let result = error.to_string();

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_reject_registry_when_two_families_share_a_notes_prefix() {
        let first = us_gaap();
        let second = plugin(
            "ifrs-full/2025",
            vec![
                FamilyDeclaration::new(
                    family("ifrs-full"),
                    vec![UriPattern::new("https://xbrl.ifrs.org/taxonomy/{release}/ifrs-full")
                        .expect("Given a hardcoded pattern with one placeholder, the creation should always succeed")],
                    "dei",
                    true,
                )
                .expect("Given a hardcoded declaration, the creation should always succeed"),
            ],
        );
        let plugins: Vec<&dyn Taxonomy> = vec![&first, &second];

        let expected_result = Some(RegistryError::OverlappingFamilyDeclaration {
            first: family("dei"),
            second: family("ifrs-full"),
        });

        let result = Registry::new(plugins).err();

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_reject_registry_when_one_uri_matches_patterns_of_two_families() {
        let first = us_gaap();
        let second = plugin(
            "us-gaap-full/2026",
            vec![declaration(
                "us-gaap-full",
                "http://fasb.org/us-gaap/{release}-full",
                false,
            )],
        );
        let plugins: Vec<&dyn Taxonomy> = vec![&first, &second];

        let expected_result = Some(RegistryError::OverlappingFamilyDeclaration {
            first: family("us-gaap"),
            second: family("us-gaap-full"),
        });

        let result = Registry::new(plugins).err();

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_reject_registry_when_one_plugin_declares_a_family_twice() {
        let repeating = plugin(
            "us-gaap/2026",
            vec![
                declaration("dei", "http://xbrl.sec.gov/dei/{release}", true),
                declaration("dei", "http://xbrl.sec.gov/dei/{release}", false),
            ],
        );
        let plugins: Vec<&dyn Taxonomy> = vec![&repeating];

        let expected_result = Some(RegistryError::RepeatedFamilyDeclaration {
            family: family("dei"),
            release: release_id("us-gaap/2026"),
        });

        let result = Registry::new(plugins).err();

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_print_both_releases_when_two_declarations_disagree() {
        let error = RegistryError::InconsistentFamilyDeclaration {
            family: family("country"),
            first: release_id("us-gaap/2026"),
            second: release_id("ifrs-full/2025"),
        };

        let expected_result = "[InconsistentFamilyDeclaration] Plug-ins 'us-gaap/2026' and 'ifrs-full/2025' declare the family 'country' with different URI patterns or Notes version prefixes";

        let result = error.to_string();

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_return_declaration_when_family_is_declared() {
        let us_gaap = us_gaap();
        let plugins: Vec<&dyn Taxonomy> = vec![&us_gaap];

        let dei = family("dei");

        let expected_result = Some(&declaration(
            "dei",
            "http://xbrl.sec.gov/dei/{release}",
            true,
        ));

        let registry =
            Registry::new(plugins).expect("Given one plug-in, the creation should always succeed");
        let result = registry.declaration(&dei);

        assert_eq!(result, expected_result);
    }
}
