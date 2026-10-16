//! # Producer
//!
//! Provides [`Producer`], the code that wrote a stored document.

use serde::{Deserialize, Serialize};

/// Names the code that wrote a stored document: the step, the crate version, and the build.
///
/// A derived document changes when the code that derives it changes, even if its input does not.
/// [`Producer::commit`] tells two builds apart. [`Producer::crate_version`] alone does not, because
/// a crate version stays the same across many commits.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct Producer {
    name: String,
    crate_version: String,
    commit: String,
}

impl Producer {
    /// Creates a new [`Producer`] from the step name, the crate version, and the git commit.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use storage::Producer;
    ///
    /// let producer = Producer::new("Store Company Facts", "0.1.0", "63c1f59");
    ///
    /// let expected_result = "63c1f59";
    ///
    /// let result = producer.commit();
    ///
    /// assert_eq!(result, expected_result);
    /// ```
    #[must_use]
    pub fn new(
        name: impl Into<String>,
        crate_version: impl Into<String>,
        commit: impl Into<String>,
    ) -> Self {
        Self {
            name: name.into(),
            crate_version: crate_version.into(),
            commit: commit.into(),
        }
    }

    /// Returns the name of the step that wrote the document.
    #[must_use]
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Returns the version of the crate that wrote the document.
    #[must_use]
    pub fn crate_version(&self) -> &str {
        &self.crate_version
    }

    /// Returns the git commit of the build that wrote the document.
    #[must_use]
    pub fn commit(&self) -> &str {
        &self.commit
    }
}

#[cfg(test)]
mod tests {
    use std::fmt::Debug;
    use std::hash::Hash;

    use pretty_assertions::assert_eq;

    use super::*;

    fn sample_producer() -> Producer {
        Producer::new("Store Company Facts", "0.1.0", "63c1f59")
    }

    const fn implements_auto_traits<T: Sized + Send + Sync + Unpin>() {}
    #[test]
    const fn should_implement_auto_traits_for_producer() {
        implements_auto_traits::<Producer>();
    }

    const fn implements_send<T: Send>() {}
    const fn implements_sync<T: Sync>() {}

    #[test]
    const fn should_implement_send_for_producer() {
        implements_send::<Producer>();
    }

    #[test]
    const fn should_implement_sync_for_producer() {
        implements_sync::<Producer>();
    }

    const fn implements_debug<T: Debug>() {}
    #[test]
    const fn should_be_able_to_rely_on_debug_implementation_for_producer() {
        implements_debug::<Producer>();
    }

    const fn implements_clone<T: Clone>() {}
    #[test]
    const fn should_be_able_to_rely_on_clone_implementation_for_producer() {
        implements_clone::<Producer>();
    }

    const fn implements_hash<T: Hash>() {}
    #[test]
    const fn should_be_able_to_rely_on_hash_implementation_for_producer() {
        implements_hash::<Producer>();
    }

    const fn implements_eq<T: Eq>() {}
    #[test]
    const fn should_be_able_to_rely_on_eq_implementation_for_producer() {
        implements_eq::<Producer>();
    }

    const fn implements_ord<T: Ord>() {}
    #[test]
    const fn should_be_able_to_rely_on_ord_implementation_for_producer() {
        implements_ord::<Producer>();
    }

    const fn implements_sized<T: Sized>() {}
    #[test]
    const fn should_be_sized_for_producer() {
        implements_sized::<Producer>();
    }

    const fn implements_partial_eq<T: PartialEq>() {}
    #[test]
    const fn should_be_able_to_rely_on_partial_eq_implementation_for_producer() {
        implements_partial_eq::<Producer>();
    }

    const fn implements_partial_ord<T: PartialOrd>() {}
    #[test]
    const fn should_be_able_to_rely_on_partial_ord_implementation_for_producer() {
        implements_partial_ord::<Producer>();
    }

    const fn implements_unpin<T: Unpin>() {}
    #[test]
    const fn should_be_able_to_rely_on_unpin_implementation_for_producer() {
        implements_unpin::<Producer>();
    }

    #[test]
    fn should_return_the_name_when_building_a_producer() {
        let producer = sample_producer();

        let expected_result = "Store Company Facts";

        let result = producer.name();

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_return_the_crate_version_when_building_a_producer() {
        let producer = sample_producer();

        let expected_result = "0.1.0";

        let result = producer.crate_version();

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_serialize_the_three_fields_under_their_names_when_writing_json() {
        let producer = sample_producer();

        let expected_result = serde_json::json!({
            "name": "Store Company Facts",
            "crate_version": "0.1.0",
            "commit": "63c1f59",
        });

        let result = serde_json::to_value(producer)
            .expect("Given a producer of owned strings, serialization should always succeed");

        assert_eq!(result, expected_result);
    }
}
