//! # Store Full Index Output
//!
//! Provides the [`StoreFullIndexOutput`] produced by the
//! [`StoreFullIndex`](crate::implementations::states::extract::store_full_index::StoreFullIndex)
//! state, together with its updater and builder.

use std::fmt;

use serde::Serialize;
use state_maschine::prelude::StateData as SMStateData;

use crate::error::State as StateError;
use crate::shared::stored_form_index::StoredFormIndex;
use crate::traits::state_machine::state::StateData;

/// Output data of the [`StoreFullIndex`](super::super::StoreFullIndex) state.
#[derive(Debug, Clone, PartialEq, PartialOrd, Hash, Eq, Ord, Serialize)]
pub struct StoreFullIndexOutput {
    /// The stored full index, with its text.
    pub index: StoredFormIndex,
}

impl StoreFullIndexOutput {
    /// Creates a new [`StoreFullIndexOutput`] from a stored full index.
    #[must_use]
    pub const fn new(index: StoredFormIndex) -> Self {
        Self { index }
    }
}

impl StateData for StoreFullIndexOutput {
    /// Replaces the stored index if the updater sets it.
    fn update_state(&mut self, updates: Self::UpdateType) -> Result<(), StateError> {
        if let Some(index) = updates.index {
            self.index = index;
        }
        Ok(())
    }
}

impl SMStateData for StoreFullIndexOutput {
    type UpdateType = StoreFullIndexOutputUpdater;

    fn state(&self) -> &Self {
        self
    }

    /// Delegates to the SEC [`StateData::update_state`] implementation.
    ///
    /// # Panics
    ///
    /// Panics if the fallible SEC update returns an error.
    fn update_state(&mut self, updates: Self::UpdateType) {
        if let Err(e) = <Self as StateData>::update_state(self, updates) {
            panic!("StateData::update_state failed: {e}")
        }
    }
}

impl fmt::Display for StoreFullIndexOutput {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "\tKey: {}, Size: {} bytes",
            self.index.key.as_path().display(),
            self.index.text.len()
        )
    }
}

/// Updater for [`StoreFullIndexOutput`].
///
/// Fields set to `None` stay unchanged when the updater is applied.
#[derive(Debug)]
pub struct StoreFullIndexOutputUpdater {
    /// The new stored index.
    pub index: Option<StoredFormIndex>,
}

impl StoreFullIndexOutputUpdater {
    /// Returns a [`StoreFullIndexOutputUpdaterBuilder`] with no field set.
    #[must_use]
    pub const fn builder() -> StoreFullIndexOutputUpdaterBuilder {
        StoreFullIndexOutputUpdaterBuilder::new()
    }
}

/// Fluent builder for a [`StoreFullIndexOutputUpdater`].
#[derive(Debug, Default)]
pub struct StoreFullIndexOutputUpdaterBuilder {
    index: Option<StoredFormIndex>,
}

impl StoreFullIndexOutputUpdaterBuilder {
    /// Creates a [`StoreFullIndexOutputUpdaterBuilder`] with no field set.
    #[must_use]
    pub const fn new() -> Self {
        Self { index: None }
    }

    /// Sets the stored index.
    #[must_use]
    #[allow(clippy::missing_const_for_fn)]
    pub fn index(mut self, index: StoredFormIndex) -> Self {
        self.index = Some(index);
        self
    }

    /// Builds the [`StoreFullIndexOutputUpdater`].
    #[must_use]
    pub fn build(self) -> StoreFullIndexOutputUpdater {
        StoreFullIndexOutputUpdater { index: self.index }
    }
}

#[cfg(test)]
mod tests {
    use std::{fmt::Debug, hash::Hash};

    use bytes::Bytes;
    use pretty_assertions::assert_eq;
    use state_maschine::prelude::StateData as SMStateData;
    use storage::Sha256Digest;
    use storage::implementations::repository::filesystem::DocumentKey;

    use super::*;

    fn stored_index(folder: &str, text: &'static [u8]) -> StoredFormIndex {
        StoredFormIndex {
            key: DocumentKey::new(format!("sec/full-index/2026/{folder}/form.idx"))
                .expect("Given a hardcoded valid relative path, the key should always build"),
            sha256: Sha256Digest::of(text),
            text: Bytes::from_static(text),
        }
    }

    #[test]
    fn should_display_the_key_and_the_size_for_store_full_index_output() {
        let output = StoreFullIndexOutput::new(stored_index("QTR4", b"Form Type"));

        let expected_result = "\tKey: sec/full-index/2026/QTR4/form.idx, Size: 9 bytes";

        let result = output.to_string();

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_update_index_when_update_contains_index() {
        let mut output = StoreFullIndexOutput::new(stored_index("QTR3", b"Form Type"));
        let update = StoreFullIndexOutputUpdater::builder()
            .index(stored_index("QTR4", b"Form Type"))
            .build();

        let expected_result = &StoreFullIndexOutput::new(stored_index("QTR4", b"Form Type"));

        SMStateData::update_state(&mut output, update);
        let result = output.state();

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_leave_output_unchanged_when_update_is_empty() {
        let mut output = StoreFullIndexOutput::new(stored_index("QTR3", b"Form Type"));
        let update = StoreFullIndexOutputUpdater::builder().build();

        let expected_result = &StoreFullIndexOutput::new(stored_index("QTR3", b"Form Type"));

        SMStateData::update_state(&mut output, update);
        let result = output.state();

        assert_eq!(result, expected_result);
    }

    const fn implements_value_traits<
        T: Sized + Send + Sync + Unpin + Clone + Debug + Hash + Eq + Ord,
    >() {
    }
    #[test]
    const fn should_implement_value_traits_for_store_full_index_output() {
        implements_value_traits::<StoreFullIndexOutput>();
    }
}
