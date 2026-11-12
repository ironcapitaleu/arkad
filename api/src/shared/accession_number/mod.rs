//! # Accession Number
//!
//! Provides the [`AccessionNumber`] of a filing in the format of the `/v1` contract.

use serde::Serialize;
use utoipa::ToSchema;
use xbrl::core::filing::FilingId;

use crate::traits::from_domain::FromDomain;

/// The accession number of a filing, for example `0000320193-24-000123`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, ToSchema)]
#[schema(example = "0000320193-24-000123")]
pub struct AccessionNumber(String);

impl AccessionNumber {
    /// Returns the accession number as text.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl From<FilingId> for AccessionNumber {
    /// Converts a [`FilingId`] into an [`AccessionNumber`]: the ID without its source.
    fn from(id: FilingId) -> Self {
        Self(id.id().to_owned())
    }
}

impl FromDomain<FilingId> for AccessionNumber {}

#[cfg(test)]
mod tests {
    use pretty_assertions::assert_eq;

    use super::*;

    const fn implements_auto_traits<T: Sized + Send + Sync + Unpin>() {}

    #[test]
    const fn should_implement_auto_traits_for_accession_number() {
        implements_auto_traits::<AccessionNumber>();
    }

    #[test]
    fn should_drop_the_source_when_converting_a_filing_id() {
        let id = FilingId::new("sec", "0000320193-24-000123")
            .expect("Given a hardcoded source and ID, the creation should always succeed");

        let expected_result = "0000320193-24-000123";

        let accession_number = AccessionNumber::from(id);
        let result = accession_number.as_str();

        assert_eq!(result, expected_result);
    }
}
