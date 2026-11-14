//! # Raw Source Implementations
//!
//! One marker type per SEC file, each implementing the [`RawSource`](super::traits::RawSource)
//! contract.
//!
//! ## Modules
//!
//! - [`raw_company_facts`]: The [`RawCompanyFacts`] marker for the Company Facts JSON.
//! - [`raw_daily_index`]: The [`RawDailyIndex`] marker for the EDGAR daily index of one day.
//! - [`raw_filing_index`]: The [`RawFilingIndex`] marker for the folder `index.json` of one filing.
//! - [`raw_full_index`]: The [`RawFullIndex`] marker for the EDGAR full index of one quarter.
//! - [`raw_notes_archive`]: The [`RawNotesArchive`] marker for a Financial Statement and Notes ZIP.
//! - [`raw_submissions`]: The [`RawSubmissions`] marker for the `submissions` JSON.
//! - [`raw_xbrl_instance`]: The [`RawXbrlInstance`] marker for the XBRL instance of one filing.
//! - [`raw_xbrl_labels`]: The [`RawXbrlLabels`] marker for the label file of one filing.

pub mod raw_company_facts;
pub mod raw_daily_index;
pub mod raw_filing_index;
pub mod raw_full_index;
pub mod raw_notes_archive;
pub mod raw_submissions;
pub mod raw_xbrl_instance;
pub mod raw_xbrl_labels;

pub use raw_company_facts::RawCompanyFacts;
pub use raw_daily_index::RawDailyIndex;
pub use raw_filing_index::RawFilingIndex;
pub use raw_full_index::RawFullIndex;
pub use raw_notes_archive::RawNotesArchive;
pub use raw_submissions::RawSubmissions;
pub use raw_xbrl_instance::RawXbrlInstance;
pub use raw_xbrl_labels::RawXbrlLabels;

#[cfg(test)]
mod tests {
    use pretty_assertions::assert_eq;

    use super::*;
    use crate::shared::raw_source::traits::RawSource;

    #[test]
    fn should_hold_a_different_document_type_for_each_raw_source() {
        let mut document_types = [
            RawCompanyFacts::DOCUMENT_TYPE,
            RawDailyIndex::DOCUMENT_TYPE,
            RawFilingIndex::DOCUMENT_TYPE,
            RawFullIndex::DOCUMENT_TYPE,
            RawNotesArchive::DOCUMENT_TYPE,
            RawSubmissions::DOCUMENT_TYPE,
            RawXbrlInstance::DOCUMENT_TYPE,
            RawXbrlLabels::DOCUMENT_TYPE,
        ];
        document_types.sort_unstable();

        let expected_result: Vec<&str> = Vec::new();

        let result: Vec<&str> = document_types
            .windows(2)
            .filter(|pair| pair[0] == pair[1])
            .map(|pair| pair[0])
            .collect();

        assert_eq!(result, expected_result);
    }
}
