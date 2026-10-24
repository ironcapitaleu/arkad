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
/// Pass the state's `STATE_NAME`.
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

        let producer = producer("Store Company Facts");
        let result = producer.name();

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_take_the_crate_version_from_cargo_when_building_a_producer() {
        let expected_result = env!("CARGO_PKG_VERSION");

        let producer = producer("Store Company Facts");
        let result = producer.crate_version();

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_take_the_commit_from_the_build_when_building_a_producer() {
        let expected_result = GIT_COMMIT;

        let producer = producer("Store Company Facts");
        let result = producer.commit();

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_hold_a_full_commit_hash_or_unknown_for_the_build() {
        let is_full_hash =
            GIT_COMMIT.len() == 40 && GIT_COMMIT.chars().all(|c| c.is_ascii_hexdigit());

        let result = is_full_hash || GIT_COMMIT == "unknown";

        assert!(result);
    }

    #[test]
    fn should_match_the_checkout_head_when_built_inside_a_git_checkout() {
        // A packaged crate records `unknown`, even inside another git repository, whose HEAD this
        // test would read.
        let manifest_dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR"));
        if manifest_dir.join(".cargo_vcs_info.json").exists() {
            return;
        }

        let expected_result = std::process::Command::new("git")
            .args(["rev-parse", "HEAD"])
            .output()
            .ok()
            .filter(|output| output.status.success())
            .and_then(|output| String::from_utf8(output.stdout).ok())
            .map_or_else(|| "unknown".to_owned(), |commit| commit.trim().to_owned());

        let result = GIT_COMMIT;

        assert_eq!(result, expected_result);
    }
}
