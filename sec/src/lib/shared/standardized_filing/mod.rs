//! # Standardized Filing
//!
//! Provides [`StandardizedFiling`], the as-is output of one filing together with its candidates and
//! its `Conflict` findings.

use std::cmp::Ordering;
use std::hash::{Hash, Hasher};

use state_maschine::prelude::StateData as SMStateData;
use xbrl::core::candidate::Candidate;
use xbrl::core::finding::Finding;

use crate::error::State as StateError;
use crate::shared::parsed_filing::ParsedFiling;
use crate::traits::state_machine::state::StateData;

/// The as-is output, the candidates, and the `Conflict` findings of one source of one filing.
#[derive(Debug, Clone)]
pub struct StandardizedFiling {
    /// The as-is output of the filing.
    pub parsed: ParsedFiling,
    /// The candidates of the filing, the ones a conflict blocks included.
    pub candidates: Vec<Candidate>,
    /// One `Finding::Conflict` per value key whose copy ranks disagree.
    pub conflicts: Vec<Finding>,
}

// Deviation: `Candidate` and `Finding` implement no `Ord`, and `Finding` no `Hash`, but a state data
// type must implement both. So equality, hashing, and order use `parsed` only, which compares by the
// raw document and the filing ID. A test of a state compares `candidates` and `conflicts` field by
// field, since `==` does not compare them.
impl PartialEq for StandardizedFiling {
    fn eq(&self, other: &Self) -> bool {
        self.parsed == other.parsed
    }
}

impl Eq for StandardizedFiling {}

impl Hash for StandardizedFiling {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.parsed.hash(state);
    }
}

impl PartialOrd for StandardizedFiling {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for StandardizedFiling {
    fn cmp(&self, other: &Self) -> Ordering {
        self.parsed.cmp(&other.parsed)
    }
}

// Deviation: the update is `()`. No code changes a value of this type in place, so it needs no
// field-wise updater.
impl SMStateData for StandardizedFiling {
    type UpdateType = ();

    fn state(&self) -> &Self {
        self
    }

    fn update_state(&mut self, (): Self::UpdateType) {}
}

impl StateData for StandardizedFiling {
    /// Changes nothing: the update holds no field.
    fn update_state(&mut self, (): Self::UpdateType) -> Result<(), StateError> {
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use std::fmt::Debug;

    use jiff::civil::date;
    use pretty_assertions::assert_eq;
    use xbrl::core::concept::NamespaceFamily;
    use xbrl::core::filing::{CompanyAttributes, EntityId, Filing, FilingId};
    use xbrl::core::form::Form;

    use super::*;
    use crate::shared::source_counts::SourceCounts;
    use crate::tests::fixtures::sample_source_document::{instance_source, notes_source};

    fn parsed_filing(accession_number: &str) -> ParsedFiling {
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
            findings: Vec::new(),
            source: notes_source(),
            industry_source: None,
            counts: SourceCounts::default(),
        }
    }

    fn standardized_filing(parsed: ParsedFiling) -> StandardizedFiling {
        StandardizedFiling {
            parsed,
            candidates: Vec::new(),
            conflicts: Vec::new(),
        }
    }

    const fn implements_auto_traits<T: Sized + Send + Sync + Unpin>() {}
    #[test]
    const fn should_implement_auto_traits_for_standardized_filing() {
        implements_auto_traits::<StandardizedFiling>();
    }

    const fn implements_value_traits<
        T: Debug + Clone + PartialEq + Eq + PartialOrd + Ord + Hash,
    >() {
    }
    #[test]
    const fn should_implement_value_traits_for_standardized_filing() {
        implements_value_traits::<StandardizedFiling>();
    }

    #[test]
    fn should_equal_when_standardized_filings_differ_only_in_conflicts() {
        let standardized = standardized_filing(parsed_filing("0000320193-26-000001"));
        let other = StandardizedFiling {
            conflicts: vec![Finding::UndeclaredFamily {
                filing: standardized.parsed.filing.id.clone(),
                family: NamespaceFamily::new("xyz/2027").expect(
                    "Given a hardcoded family without whitespace, the creation should always succeed",
                ),
            }],
            ..standardized_filing(parsed_filing("0000320193-26-000001"))
        };

        let expected_result = true;

        let result = standardized == other;

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_differ_when_the_parsed_filings_differ_in_the_source() {
        let standardized = standardized_filing(parsed_filing("0000320193-26-000001"));
        let other = standardized_filing(ParsedFiling {
            source: instance_source(),
            ..parsed_filing("0000320193-26-000001")
        });

        let expected_result = false;

        let result = standardized == other;

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_order_standardized_filings_by_the_parsed_filing() {
        let standardized = standardized_filing(parsed_filing("0000320193-26-000001"));
        let other = standardized_filing(parsed_filing("0000320193-26-000002"));

        let expected_result = Ordering::Less;

        let result = standardized.cmp(&other);

        assert_eq!(result, expected_result);
    }
}
