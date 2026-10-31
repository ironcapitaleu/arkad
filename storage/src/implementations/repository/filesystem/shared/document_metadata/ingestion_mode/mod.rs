//! # Ingestion Mode
//!
//! Provides [`IngestionMode`], the mode of the run that produced a stored document.

use std::fmt::{self, Display, Formatter};

use serde::{Deserialize, Serialize};

/// The mode of the run that produced a stored document.
///
/// Each variant's [`Display`] text is the `ingestion_mode` the store writes into a document's
/// metadata. Serde writes and reads the same text, so a stored mode that is not one of these
/// texts fails to read.
///
/// # Examples
///
/// ```rust
/// use storage::implementations::repository::filesystem::IngestionMode;
///
/// let ingestion_mode = IngestionMode::Backfill;
///
/// let expected_result = "backfill";
///
/// let result = ingestion_mode.to_string();
///
/// assert_eq!(result, expected_result);
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum IngestionMode {
    /// A run that loads the history from the SEC bulk files.
    Backfill,
    /// A run that loads the filings the SEC published since the last run.
    Incremental,
}

impl Display for IngestionMode {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        match self {
            Self::Backfill => write!(f, "backfill"),
            Self::Incremental => write!(f, "incremental"),
        }
    }
}

#[cfg(test)]
mod tests {
    use std::fmt::Debug;
    use std::hash::Hash;

    use pretty_assertions::assert_eq;

    use super::*;

    const fn implements_auto_traits<T: Sized + Send + Sync + Unpin>() {}
    #[test]
    const fn should_implement_auto_traits_for_ingestion_mode() {
        implements_auto_traits::<IngestionMode>();
    }

    const fn implements_value_traits<
        T: Debug + Clone + Copy + PartialEq + Eq + PartialOrd + Ord + Hash,
    >() {
    }
    #[test]
    const fn should_implement_value_traits_for_ingestion_mode() {
        implements_value_traits::<IngestionMode>();
    }

    #[test]
    fn should_display_backfill_for_the_backfill_ingestion_mode() {
        let ingestion_mode = IngestionMode::Backfill;

        let expected_result = "backfill";

        let result = ingestion_mode.to_string();

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_display_incremental_for_the_incremental_ingestion_mode() {
        let ingestion_mode = IngestionMode::Incremental;

        let expected_result = "incremental";

        let result = ingestion_mode.to_string();

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_serialize_the_display_text_for_the_backfill_ingestion_mode() {
        let ingestion_mode = IngestionMode::Backfill;

        let expected_result = serde_json::json!(IngestionMode::Backfill.to_string());

        let result = serde_json::to_value(ingestion_mode)
            .expect("Given a unit variant, serialization should always succeed");

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_serialize_the_display_text_for_the_incremental_ingestion_mode() {
        let ingestion_mode = IngestionMode::Incremental;

        let expected_result = serde_json::json!(IngestionMode::Incremental.to_string());

        let result = serde_json::to_value(ingestion_mode)
            .expect("Given a unit variant, serialization should always succeed");

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_read_the_ingestion_mode_when_the_text_is_backfill() {
        let json = "\"backfill\"";

        let expected_result = IngestionMode::Backfill;

        let result: IngestionMode = serde_json::from_str(json)
            .expect("Given the text of a variant, deserialization should always succeed");

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_fail_to_read_the_ingestion_mode_when_the_text_is_unknown() {
        let json = "\"foo\"";

        let expected_result = true;

        let result = serde_json::from_str::<IngestionMode>(json).is_err();

        assert_eq!(result, expected_result);
    }
}
