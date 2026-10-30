//! # Raw Notes Archive
//!
//! Provides [`RawNotesArchive`], the [`RawSource`] marker for one SEC Financial Statement and
//! Notes ZIP.

use crate::shared::document_type::SecDocumentType;
use crate::shared::raw_source::traits::{RawFileSource, RawSource};

/// Marks one SEC Financial Statement and Notes ZIP, as the SEC returns it.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct RawNotesArchive;

impl RawSource for RawNotesArchive {
    const DOCUMENT_TYPE: SecDocumentType = SecDocumentType::RawNotesArchive;
    const DOCUMENT_TYPE_VERSION: u32 = 1;
}

impl RawFileSource for RawNotesArchive {}

#[cfg(test)]
mod tests {
    use pretty_assertions::assert_eq;

    use super::*;

    const fn implements_raw_file_source<T: RawFileSource>() {}
    #[test]
    const fn should_implement_raw_file_source_for_raw_notes_archive() {
        implements_raw_file_source::<RawNotesArchive>();
    }

    const fn implements_raw_source<T: RawSource>() {}
    #[test]
    const fn should_implement_raw_source_for_raw_notes_archive() {
        implements_raw_source::<RawNotesArchive>();
    }

    #[test]
    fn should_name_the_raw_notes_archive_document_type_for_raw_notes_archive() {
        let expected_result = SecDocumentType::RawNotesArchive;

        let result = RawNotesArchive::DOCUMENT_TYPE;

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_name_document_type_version_one_for_raw_notes_archive() {
        let expected_result = 1;

        let result = RawNotesArchive::DOCUMENT_TYPE_VERSION;

        assert_eq!(result, expected_result);
    }
}
