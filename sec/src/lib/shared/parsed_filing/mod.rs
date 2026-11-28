//! # Parsed Filing
//!
//! Provides [`ParsedFiling`], the as-is output of one filing from either parser, which both parse
//! states of the filing machine return.

use std::cmp::Ordering;
use std::hash::{Hash, Hasher};

use state_maschine::prelude::StateData as SMStateData;
use xbrl::core::fact::AsIsFact;
use xbrl::core::filing::Filing;
use xbrl::core::finding::Finding;

use crate::error::State as StateError;
use crate::shared::filing_input::StoredIndustry;
use crate::shared::source_counts::SourceCounts;
use crate::shared::source_document::SourceDocument;
use crate::traits::state_machine::state::StateData;

/// The as-is output of one filing, from either parser.
#[derive(Debug, Clone)]
pub struct ParsedFiling {
    /// The as-is facts of the filing.
    pub facts: Vec<AsIsFact>,
    /// The filing and its attributes as filed.
    pub filing: Filing,
    /// The parser findings: `UnmappedRow` and `UndeclaredFamily`.
    pub findings: Vec<Finding>,
    /// The raw document of the facts and the filing.
    pub source: SourceDocument,
    /// The stored `submissions` file that gave the industry of an instance filing, or `None`.
    pub industry_source: Option<StoredIndustry>,
    /// The source rows of the filing and the rows the parse dropped.
    pub counts: SourceCounts,
}

// Deviation: `AsIsFact`, `Filing`, and `Finding` implement no `Ord`, and `Finding` no `Hash`, but a
// state data type must implement both. So equality, hashing, and order use the raw document and
// the filing ID only, and agree. The pair names one parse of one filing. A test of a parse state
// compares `facts`, `findings`, and `counts` field by field, since `==` does not compare them.
impl PartialEq for ParsedFiling {
    fn eq(&self, other: &Self) -> bool {
        self.source == other.source && self.filing.id == other.filing.id
    }
}

impl Eq for ParsedFiling {}

impl Hash for ParsedFiling {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.source.hash(state);
        self.filing.id.hash(state);
    }
}

impl PartialOrd for ParsedFiling {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for ParsedFiling {
    fn cmp(&self, other: &Self) -> Ordering {
        self.source
            .cmp(&other.source)
            .then_with(|| self.filing.id.cmp(&other.filing.id))
    }
}

// Deviation: the update is `()`, not a field-wise updater, because a value of this type is never
// changed in place.
impl SMStateData for ParsedFiling {
    type UpdateType = ();

    fn state(&self) -> &Self {
        self
    }

    fn update_state(&mut self, (): Self::UpdateType) {}
}

impl StateData for ParsedFiling {
    /// Changes nothing: the update holds no field.
    fn update_state(&mut self, (): Self::UpdateType) -> Result<(), StateError> {
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use std::collections::hash_map::DefaultHasher;
    use std::fmt::Debug;

    use jiff::civil::date;
    use pretty_assertions::assert_eq;
    use xbrl::core::concept::NamespaceFamily;
    use xbrl::core::filing::{CompanyAttributes, EntityId, FilingId};
    use xbrl::core::form::Form;

    use super::*;
    use crate::tests::fixtures::sample_source_document::{instance_source, notes_source};

    fn parsed_filing(accession_number: &str, findings: Vec<Finding>) -> ParsedFiling {
        ParsedFiling {
            facts: Vec::new(),
            filing: Filing {
                id: FilingId::new("sec", accession_number).expect(
                    "Given a hardcoded source and accession number, the creation should always succeed",
                ),
                entity: EntityId::new("sec", "320193").expect(
                    "Given a hardcoded source and CIK, the creation should always succeed",
                ),
                form: Form::new("10-K")
                    .expect("Given a hardcoded form type, the creation should always succeed"),
                is_amendment: false,
                filed: date(2026, 11, 2),
                accepted: None,
                period_of_report: None,
                fiscal_year: None,
                fiscal_period: None,
                company: CompanyAttributes::default(),
            },
            findings,
            source: notes_source(),
            industry_source: None,
            counts: SourceCounts::default(),
        }
    }

    fn hash_of<T: Hash>(value: &T) -> u64 {
        let mut hasher = DefaultHasher::new();
        value.hash(&mut hasher);
        hasher.finish()
    }

    const fn implements_auto_traits<T: Sized + Send + Sync + Unpin>() {}
    #[test]
    const fn should_implement_auto_traits_for_parsed_filing() {
        implements_auto_traits::<ParsedFiling>();
    }

    const fn implements_value_traits<
        T: Debug + Clone + PartialEq + Eq + PartialOrd + Ord + Hash,
    >() {
    }
    #[test]
    const fn should_implement_value_traits_for_parsed_filing() {
        implements_value_traits::<ParsedFiling>();
    }

    #[test]
    fn should_equal_when_parsed_filings_share_source_and_filing_id() {
        let parsed = parsed_filing("0000320193-26-000001", Vec::new());
        let other = parsed_filing(
            "0000320193-26-000001",
            vec![Finding::UndeclaredFamily {
                filing: parsed.filing.id.clone(),
                family: NamespaceFamily::new("xyz/2027").expect(
                    "Given a hardcoded family without whitespace, the creation should always succeed",
                ),
            }],
        );

        let expected_result = true;

        let result = parsed == other;

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_differ_when_parsed_filings_share_the_filing_id_but_not_the_source() {
        let parsed = parsed_filing("0000320193-26-000001", Vec::new());
        let other = ParsedFiling {
            source: instance_source(),
            ..parsed_filing("0000320193-26-000001", Vec::new())
        };

        let expected_result = false;

        let result = parsed == other;

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_hash_equal_when_parsed_filings_differ_only_in_findings() {
        let parsed = parsed_filing("0000320193-26-000001", Vec::new());
        let other = parsed_filing(
            "0000320193-26-000001",
            vec![Finding::UndeclaredFamily {
                filing: parsed.filing.id.clone(),
                family: NamespaceFamily::new("xyz/2027").expect(
                    "Given a hardcoded family without whitespace, the creation should always succeed",
                ),
            }],
        );

        let expected_result = hash_of(&other);

        let result = hash_of(&parsed);

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_order_parsed_filings_by_filing_id_when_they_share_the_source() {
        let parsed = parsed_filing("0000320193-26-000001", Vec::new());
        let other = parsed_filing("0000320193-26-000002", Vec::new());

        let expected_result = Ordering::Less;

        let result = parsed.cmp(&other);

        assert_eq!(result, expected_result);
    }
}
