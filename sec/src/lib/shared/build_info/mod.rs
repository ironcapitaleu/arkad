//! # Build Info
//!
//! Provides [`producer`], which names the `sec` code that writes a stored document.
//!
//! ## Modules
//!
//! - [`constants`]: The [`CRATE_VERSION`] and the [`GIT_COMMIT`] of the build.

use storage::implementations::repository::filesystem::Producer;

pub mod constants;

pub use constants::{CRATE_VERSION, GIT_COMMIT};

/// Returns the [`Producer`] of a document that the given state writes with this build.
///
/// Pass the state's `STATE_NAME`. The crate version and the commit come from the build, so two
/// builds of one version differ in the commit.
///
/// # Examples
///
/// ```rust
/// use sec::shared::build_info::{CRATE_VERSION, producer};
///
/// let producer = producer("Store Company Facts");
///
/// let expected_result = CRATE_VERSION;
///
/// let result = producer.crate_version();
///
/// assert_eq!(result, expected_result);
/// ```
#[must_use]
pub fn producer(state_name: &str) -> Producer {
    Producer::new(state_name, CRATE_VERSION, GIT_COMMIT)
}

#[cfg(test)]
mod tests {
    use pretty_assertions::assert_eq;

    use super::*;

    #[test]
    fn should_name_the_state_when_building_a_producer() {
        let expected_result = "Store Company Facts";

        let result = producer("Store Company Facts").name().to_owned();

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_take_the_crate_version_from_cargo_when_building_a_producer() {
        let expected_result = env!("CARGO_PKG_VERSION");

        let result = producer("Store Company Facts").crate_version().to_owned();

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_take_the_commit_from_the_build_when_building_a_producer() {
        let expected_result = GIT_COMMIT;

        let result = producer("Store Company Facts").commit().to_owned();

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_hold_a_full_commit_hash_or_unknown_for_the_build() {
        let is_full_hash =
            GIT_COMMIT.len() == 40 && GIT_COMMIT.chars().all(|c| c.is_ascii_hexdigit());

        let result = is_full_hash || GIT_COMMIT == "unknown";

        assert!(result);
    }
}
