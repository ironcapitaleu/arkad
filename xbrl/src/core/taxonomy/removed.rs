//! # Removed Elements
//!
//! Provides the [`RemovedElements`] trait, which a plug-in implements next to [`Taxonomy`] to give
//! the reference data of the elements that its release removed.

use super::Taxonomy;
use super::reference::ElementReference;
use crate::core::concept::ConceptName;
use crate::core::release::TaxonomyReleaseId;

/// The elements that an earlier release defines and the release of the plug-in does not.
///
/// The as-is facts of old filings use them. [`Taxonomy::element`] returns `None` for such an
/// element, and [`RemovedElements::removed_element`] returns its data.
///
/// # Required Traits
///
/// - [`Taxonomy`]: an element counts as removed against [`Taxonomy::release`].
pub trait RemovedElements: Taxonomy {
    /// Returns the data of the removed element `concept`, or `None` if no earlier release defines
    /// it or the release of the plug-in defines it.
    fn removed_element(&self, concept: &ConceptName) -> Option<&RemovedElement>;

    /// Returns every removed element.
    fn removed_elements(&self) -> &[RemovedElement];
}

/// One element that the release of a plug-in removed.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct RemovedElement {
    /// The data of the last release that defines the element. Its `release` names that release.
    pub reference: ElementReference,
    /// The release that the standard label names as the deprecation date, or `None` if the label
    /// names none.
    pub deprecated_in: Option<TaxonomyReleaseId>,
}

#[cfg(test)]
mod tests {
    use super::*;

    const fn implements_auto_traits<T: Sized + Send + Sync + Unpin>() {}
    #[test]
    const fn should_implement_auto_traits_for_removed_element() {
        implements_auto_traits::<RemovedElement>();
    }
}
