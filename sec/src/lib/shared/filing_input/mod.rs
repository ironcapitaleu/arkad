//! # Filing Input
//!
//! Provides [`FilingInput`], the input of the filing machine for one filing, and its two forms:
//! [`NotesFilingInput`] from a Notes file and [`InstanceFilingInput`] from a filing day.
//!
//! A unit reader fills these values, and the parse state of each source reads them.

use std::cmp::Ordering;
use std::hash::{Hash, Hasher};

use jiff::Timestamp;
use xbrl::adapter::edgar::notes::NotesFiling;
use xbrl::core::filing::IndustryCode;

use crate::shared::filing_ref::FilingRef;
use crate::shared::source_document::SourceDocument;

/// The input of the filing machine for one filing, from either source.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum FilingInput {
    /// One filing of a Notes file.
    Notes(NotesFilingInput),
    /// The main instance of one filing of a filing day.
    Instance(InstanceFilingInput),
}

/// The rows of one selected Notes filing and the raw document they came from.
#[derive(Debug, Clone)]
pub struct NotesFilingInput {
    /// The rows of the filing, from each table of the Notes file.
    pub rows: NotesFiling,
    /// The Notes file.
    pub source: SourceDocument,
}

impl NotesFilingInput {
    /// Returns the accession number of the filing, from its `sub` row.
    #[must_use]
    pub fn adsh(&self) -> &str {
        &self.rows.sub.adsh
    }
}

// Deviation: `NotesFiling` implements no `Hash` and no `Ord`, and a state data type must implement
// both. So equality, hashing, and order use the Notes file and the `adsh` only, and agree. One
// Notes file holds one `sub` row per `adsh`, so the pair names one filing.
impl PartialEq for NotesFilingInput {
    fn eq(&self, other: &Self) -> bool {
        self.source == other.source && self.adsh() == other.adsh()
    }
}

impl Eq for NotesFilingInput {}

impl Hash for NotesFilingInput {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.source.hash(state);
        self.adsh().hash(state);
    }
}

impl PartialOrd for NotesFilingInput {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for NotesFilingInput {
    fn cmp(&self, other: &Self) -> Ordering {
        self.source
            .cmp(&other.source)
            .then_with(|| self.adsh().cmp(other.adsh()))
    }
}

/// The main instance of one filing, its company data, and the raw documents they came from.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct InstanceFilingInput {
    /// The filing, from the daily index of the day.
    pub filing: FilingRef,
    /// The bytes of the stored main instance.
    pub document: Vec<u8>,
    /// The stored main instance.
    pub source: SourceDocument,
    /// The SIC of the company, or `None` if the store holds no `submissions` file for the CIK.
    pub industry: Option<StoredIndustry>,
}

/// The SIC of the company from its stored `submissions` file.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct StoredIndustry {
    /// The industry code, for example `sic:3571`.
    pub code: IndustryCode,
    /// The stored `submissions` file.
    pub source: SourceDocument,
    /// When extract fetched the `submissions` file.
    pub retrieved_at: Timestamp,
}

#[cfg(test)]
mod tests {
    use std::collections::hash_map::DefaultHasher;
    use std::fmt::Debug;

    use pretty_assertions::assert_eq;
    use xbrl::adapter::edgar::notes::row::{NumRow, SubRow};

    use super::*;
    use crate::tests::fixtures::sample_source_document::{instance_source, notes_source};

    fn notes_input(adsh: &str, num: Vec<NumRow>) -> NotesFilingInput {
        NotesFilingInput {
            rows: NotesFiling {
                sub: SubRow {
                    adsh: adsh.to_owned(),
                    ..SubRow::default()
                },
                num,
                ..NotesFiling::default()
            },
            source: notes_source(),
        }
    }

    fn hash_of<T: Hash>(value: &T) -> u64 {
        let mut hasher = DefaultHasher::new();
        value.hash(&mut hasher);
        hasher.finish()
    }

    const fn implements_auto_traits<T: Sized + Send + Sync + Unpin>() {}
    #[test]
    const fn should_implement_auto_traits_for_filing_input() {
        implements_auto_traits::<FilingInput>();
    }

    const fn implements_value_traits<
        T: Debug + Clone + PartialEq + Eq + PartialOrd + Ord + Hash,
    >() {
    }
    #[test]
    const fn should_implement_value_traits_for_filing_input() {
        implements_value_traits::<FilingInput>();
    }

    #[test]
    const fn should_implement_value_traits_for_stored_industry() {
        implements_value_traits::<StoredIndustry>();
    }

    #[test]
    fn should_equal_when_notes_inputs_share_source_and_adsh() {
        let input = notes_input("0000320193-26-000001", Vec::new());
        let other = notes_input("0000320193-26-000001", vec![NumRow::default()]);

        let expected_result = true;

        let result = input == other;

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_differ_when_notes_inputs_share_the_adsh_but_not_the_source() {
        let input = notes_input("0000320193-26-000001", Vec::new());
        let other = NotesFilingInput {
            source: instance_source(),
            ..notes_input("0000320193-26-000001", Vec::new())
        };

        let expected_result = false;

        let result = input == other;

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_hash_equal_when_notes_inputs_differ_only_in_num_rows() {
        let input = notes_input("0000320193-26-000001", Vec::new());
        let other = notes_input("0000320193-26-000001", vec![NumRow::default()]);

        let expected_result = hash_of(&other);

        let result = hash_of(&input);

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_order_notes_inputs_by_adsh_when_they_share_the_source() {
        let input = notes_input("0000320193-26-000001", Vec::new());
        let other = notes_input("0000320193-26-000002", Vec::new());

        let expected_result = Ordering::Less;

        let result = input.cmp(&other);

        assert_eq!(result, expected_result);
    }
}
