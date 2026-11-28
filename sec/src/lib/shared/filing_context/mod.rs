//! # Filing Context
//!
//! Provides [`FilingContext`], the context that every state of the filing machine takes, and
//! [`RegistryHandle`], the taxonomy registry of the task inside it.

use std::cmp::Ordering;
use std::fmt;
use std::hash::{Hash, Hasher};

use state_maschine::prelude::Context as SMContext;
use uuid::Uuid;
use xbrl::core::taxonomy::registry::Registry;

use crate::traits::state_machine::state::Context;

/// The taxonomy registry of one transform task.
///
/// The task builds one [`Registry`] at its start and keeps it for its whole run, so the handle
/// holds a `&'static` reference. Its `Eq`, `Ord`, and `Hash` compare the release IDs of the
/// plug-ins, in the order of the registry. So two registries of the same releases are equal.
#[derive(Clone, Copy)]
pub struct RegistryHandle(&'static Registry<'static>);

impl RegistryHandle {
    /// Creates a handle on `registry`.
    #[must_use]
    pub const fn new(registry: &'static Registry<'static>) -> Self {
        Self(registry)
    }

    /// Returns the registry.
    #[must_use]
    pub const fn registry(&self) -> &'static Registry<'static> {
        self.0
    }
}

impl fmt::Debug for RegistryHandle {
    /// Writes the release IDs of the plug-ins, since a plug-in implements no `Debug`.
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_tuple("RegistryHandle")
            .field(&self.0.release_ids().collect::<Vec<_>>())
            .finish()
    }
}

impl PartialEq for RegistryHandle {
    fn eq(&self, other: &Self) -> bool {
        self.0.release_ids().eq(other.0.release_ids())
    }
}

impl Eq for RegistryHandle {}

impl Hash for RegistryHandle {
    fn hash<H: Hasher>(&self, state: &mut H) {
        for release_id in self.0.release_ids() {
            release_id.hash(state);
        }
    }
}

impl PartialOrd for RegistryHandle {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for RegistryHandle {
    fn cmp(&self, other: &Self) -> Ordering {
        self.0.release_ids().cmp(other.0.release_ids())
    }
}

/// The context of every state of the filing machine: the registry, the `transform_version`, and
/// the `execution_id` of the task.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct FilingContext {
    /// The taxonomy registry of the task.
    pub registry: RegistryHandle,
    /// The version of the output that this build writes.
    pub transform_version: u32,
    /// The ID of the job run.
    pub execution_id: Uuid,
}

impl SMContext for FilingContext {
    type UpdateType = ();

    fn context(&self) -> &Self {
        self
    }

    fn update_context(&mut self, (): Self::UpdateType) {}
}

impl Context for FilingContext {
    /// Returns 0: the filing machine does no I/O, so a second try gives the same result.
    fn max_retries(&self) -> u32 {
        0
    }
}

#[cfg(test)]
mod tests {
    use std::collections::hash_map::DefaultHasher;

    use pretty_assertions::assert_eq;
    use xbrl::tests::fixtures::sample_taxonomy::{declaration, plugin};

    use super::*;

    /// Returns a handle on a registry with one plug-in of each release, leaked for the
    /// `'static` lifetime of the handle.
    fn handle(releases: &[&str]) -> RegistryHandle {
        let plugins = releases
            .iter()
            .map(|release| {
                let family = release
                    .split('/')
                    .next()
                    .expect("Given a hardcoded release ID, the split should always give a part");
                let plugin: &'static _ = Box::leak(Box::new(plugin(
                    release,
                    vec![declaration(
                        family,
                        &format!("http://example.com/{family}/{{release}}"),
                        true,
                    )],
                )));
                plugin as &'static dyn xbrl::core::taxonomy::Taxonomy
            })
            .collect();
        let registry = Registry::new(plugins)
            .expect("Given plug-ins with distinct families, the creation should always succeed");
        RegistryHandle::new(Box::leak(Box::new(registry)))
    }

    fn hash_of(handle: RegistryHandle) -> u64 {
        let mut hasher = DefaultHasher::new();
        handle.hash(&mut hasher);
        hasher.finish()
    }

    const fn implements_auto_traits<T: Sized + Send + Sync + Unpin>() {}
    #[test]
    const fn should_implement_auto_traits_for_filing_context() {
        implements_auto_traits::<FilingContext>();
    }

    #[test]
    fn should_equal_when_two_registries_hold_the_same_releases() {
        let handle = handle(&["us-gaap/2026", "srt/2026"]);
        let other = self::handle(&["us-gaap/2026", "srt/2026"]);

        let expected_result = true;

        let result = handle == other;

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_differ_when_a_release_differs() {
        let handle = handle(&["us-gaap/2026"]);
        let other = self::handle(&["us-gaap/2025"]);

        let expected_result = false;

        let result = handle == other;

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_order_by_release_ids_when_releases_differ() {
        let handle = handle(&["us-gaap/2025"]);
        let other = self::handle(&["us-gaap/2026"]);

        let expected_result = Ordering::Less;

        let result = handle.cmp(&other);

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_hash_equal_when_two_registries_hold_the_same_releases() {
        let handle = handle(&["us-gaap/2026"]);
        let other = self::handle(&["us-gaap/2026"]);

        let expected_result = hash_of(other);

        let result = hash_of(handle);

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_allow_no_retry_for_filing_context() {
        let context = FilingContext {
            registry: handle(&["us-gaap/2026"]),
            transform_version: 1,
            execution_id: Uuid::nil(),
        };

        let expected_result = false;

        let result = context.can_retry();

        assert_eq!(result, expected_result);
    }
}
