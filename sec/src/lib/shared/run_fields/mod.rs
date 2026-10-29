//! # Run Fields
//!
//! Provides [`RunFields`], the metadata fields that name the run that produced a stored document.

use chrono::{DateTime, Utc};
use storage::implementations::repository::filesystem::Producer;
use uuid::Uuid;

use crate::shared::ingestion_mode::IngestionMode;

/// The metadata fields that name the run that produced a stored document.
///
/// Every stored document has these four fields, so a constructor takes them as one value.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct RunFields {
    /// When the producing state made the document bytes.
    pub produced_at: DateTime<Utc>,
    /// The id of the run that produced the document bytes.
    pub execution_id: Uuid,
    /// The code that wrote the document.
    pub producer: Producer,
    /// The ingestion mode of the run that produced the document bytes.
    pub ingestion_mode: IngestionMode,
}

#[cfg(test)]
mod tests {
    use std::fmt::Debug;
    use std::hash::Hash;

    use super::*;

    const fn implements_auto_traits<T: Sized + Send + Sync + Unpin>() {}
    #[test]
    const fn should_implement_auto_traits_for_run_fields() {
        implements_auto_traits::<RunFields>();
    }

    const fn implements_value_traits<
        T: Debug + Clone + PartialEq + Eq + PartialOrd + Ord + Hash,
    >() {
    }
    #[test]
    const fn should_implement_value_traits_for_run_fields() {
        implements_value_traits::<RunFields>();
    }
}
