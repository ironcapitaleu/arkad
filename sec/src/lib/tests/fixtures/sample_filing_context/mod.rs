//! # Sample Filing Context
//!
//! Provides [`fixture_context`], a [`FilingContext`] on a registry of the families of the `xbrl`
//! fixture filing, and [`us_gaap_context`], a [`FilingContext`] on the `us-gaap` plug-in.

use jiff::Timestamp;
use uuid::Uuid;
use xbrl::core::taxonomy::Taxonomy;
use xbrl::core::taxonomy::registry::Registry;
use xbrl::tests::fixtures::data::fixture_taxonomy;
use xbrl::us_gaap::UsGaap;

use crate::shared::filing_context::{FilingContext, RegistryHandle};

/// Returns a [`FilingContext`] whose registry declares `us-gaap`, `dei`, `srt`, and `ecd`. The
/// function leaks the plug-in and the registry, so the handle lives for `'static`.
///
/// # Panics
///
/// Never panics: one plug-in with four distinct families always gives a registry.
#[must_use]
pub fn fixture_context() -> FilingContext {
    let plugin: &'static dyn Taxonomy = Box::leak(Box::new(fixture_taxonomy()));
    context_of(plugin)
}

/// Returns a [`FilingContext`] whose registry holds the `us-gaap` plug-in with its alias lists. The
/// function leaks the plug-in and the registry, so the handle lives for `'static`.
///
/// # Panics
///
/// Never panics: the `us-gaap` plug-in alone always gives a registry.
#[must_use]
pub fn us_gaap_context() -> FilingContext {
    let plugin: &'static dyn Taxonomy = Box::leak(Box::new(UsGaap::new()));
    context_of(plugin)
}

fn context_of(plugin: &'static dyn Taxonomy) -> FilingContext {
    let registry = Registry::new(vec![plugin])
        .expect("Given one plug-in with distinct families, the creation should always succeed");
    FilingContext {
        registry: RegistryHandle::new(Box::leak(Box::new(registry))),
        transform_version: 1,
        execution_id: Uuid::nil(),
        loaded_at: Timestamp::UNIX_EPOCH,
    }
}
