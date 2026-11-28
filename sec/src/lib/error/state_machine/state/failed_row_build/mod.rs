//! # Failed Row Build Error
//!
//! Provides the [`FailedRowBuild`] error: the rows of a filing do not fit the Arrow form of their
//! dataset.

use std::error::Error;
use std::fmt;

use super::State as StateError;
use crate::datasets::ArrowRowError;

/// Error for the rows of a filing that do not fit their dataset, tagged with the state it occurred
/// in.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct FailedRowBuild {
    /// The name of the state where the error occurred.
    pub state_name: String,
    /// The filing ID, for example `sec:0000320193-26-000001`.
    pub filing: String,
    /// The error of the row type. It is boxed, because it is larger than the other state errors.
    pub source: Box<ArrowRowError>,
}

// Deviation: `Display` and `source()` are written by hand, because the derive of `thiserror`
// makes `source()` return the `Box` and not the `ArrowRowError` in it.
impl fmt::Display for FailedRowBuild {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "[FailedRowBuild] Failed to build the rows of the filing '{}', Caused by: {}",
            self.filing, self.source
        )
    }
}

impl Error for FailedRowBuild {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        Some(self.source.as_ref())
    }
}

impl FailedRowBuild {
    /// Creates a new [`FailedRowBuild`] error.
    #[must_use]
    pub fn new(
        state_name: impl Into<String>,
        filing: impl Into<String>,
        source: ArrowRowError,
    ) -> Self {
        Self {
            state_name: state_name.into(),
            filing: filing.into(),
            source: Box::new(source),
        }
    }
}

impl From<FailedRowBuild> for StateError {
    /// Wraps the error in the [`StateError::FailedRowBuild`] variant.
    fn from(error: FailedRowBuild) -> Self {
        Self::FailedRowBuild(error)
    }
}

#[cfg(test)]
mod tests {
    use std::{fmt::Debug, hash::Hash};

    use pretty_assertions::assert_eq;

    use super::*;

    const STATE_NAME: &str = "Build Filing Rows";

    fn too_many_digits() -> ArrowRowError {
        ArrowRowError::UnexpectedColumnValue {
            dataset: "as_is_facts",
            column: "value_numeric",
            row: 3,
            reason: "'1e40' has more than 38 digits".to_owned(),
        }
    }

    #[test]
    fn should_chain_the_row_error_after_caused_by_when_displayed() {
        let error = FailedRowBuild::new(STATE_NAME, "sec:0000320193-26-000001", too_many_digits());

        let expected_result = "[FailedRowBuild] Failed to build the rows of the filing 'sec:0000320193-26-000001', Caused by: [UnexpectedColumnValue] Row '3' of column 'value_numeric' in dataset 'as_is_facts' holds an unexpected value, Reason: ''1e40' has more than 38 digits'";

        let result = error.to_string();

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_expose_the_row_error_as_source_when_wrapping_it() {
        let error = FailedRowBuild::new(STATE_NAME, "sec:0000320193-26-000001", too_many_digits());

        let expected_result = Some(&too_many_digits());

        let result = error
            .source()
            .and_then(|source| source.downcast_ref::<ArrowRowError>());

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_convert_to_state_error_when_from_is_called() {
        let error = FailedRowBuild::new(STATE_NAME, "sec:0000320193-26-000001", too_many_digits());

        let expected_result = StateError::FailedRowBuild(FailedRowBuild {
            state_name: "Build Filing Rows".to_owned(),
            filing: "sec:0000320193-26-000001".to_owned(),
            source: Box::new(too_many_digits()),
        });

        let result = StateError::from(error);

        assert_eq!(result, expected_result);
    }

    const fn implements_auto_traits<T: Sized + Send + Sync + Unpin>() {}
    #[test]
    const fn should_implement_auto_traits_for_failed_row_build() {
        implements_auto_traits::<FailedRowBuild>();
    }

    const fn implements_value_traits<
        T: Debug + Clone + PartialEq + Eq + PartialOrd + Ord + Hash,
    >() {
    }
    #[test]
    const fn should_implement_value_traits_for_failed_row_build() {
        implements_value_traits::<FailedRowBuild>();
    }
}
