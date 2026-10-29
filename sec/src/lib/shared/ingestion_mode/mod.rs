//! # Ingestion Mode
//!
//! Provides [`IngestionMode`], the mode of the run that produced a stored document.

use std::fmt::{self, Display, Formatter};

/// The mode of the run that produced a stored document.
///
/// Each variant's [`Display`] text is the `ingestion_mode` the store writes into a document's
/// metadata. The [`From`] impl for [`String`] gives the same text, so a store state can pass the
/// mode straight to the metadata builder.
///
/// # Examples
///
/// ```rust
/// use sec::shared::ingestion_mode::IngestionMode;
///
/// let expected_result = "backfill";
///
/// let result = IngestionMode::Backfill.to_string();
///
/// assert_eq!(result, expected_result);
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
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

impl From<IngestionMode> for String {
    fn from(ingestion_mode: IngestionMode) -> Self {
        ingestion_mode.to_string()
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
        let expected_result = "backfill";

        let result = IngestionMode::Backfill.to_string();

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_convert_to_the_display_text_for_the_incremental_ingestion_mode() {
        let expected_result = "incremental".to_owned();

        let result = String::from(IngestionMode::Incremental);

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_display_incremental_for_the_incremental_ingestion_mode() {
        let expected_result = "incremental";

        let result = IngestionMode::Incremental.to_string();

        assert_eq!(result, expected_result);
    }
}
