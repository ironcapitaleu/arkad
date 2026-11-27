//! # Required Check
//!
//! Provides [`check_required`], which reports each required standard concept that a filing leaves
//! without a value as a [`Finding::MissingRequired`]. The check never removes or derives a value.

use jiff::civil::Date;

use super::candidate::Candidate;
use super::chronology::{ANNUAL_FORMS, QUARTERLY_FORMS, is_form};
use super::dedup::Series;
use super::elements::CanonicalElementKind;
use super::fact::Dimensions;
use super::filing::{EntityId, Filing};
use super::finding::{Finding, MissingReason};
use super::period::is_near;
use super::standardized::StandardizedFact;

/// Returns a [`Finding::MissingRequired`] for each concept of `required` that has no value for the
/// period of report of a filing of `filings`.
///
/// The check reads only values without dimensions. It covers each filing that is an annual or a
/// quarterly report, is no amendment, and has a period of report. A value of the series or of
/// `added` fills the period if its entity and concept match and its end lies within 7 days of the
/// period of report. A value of any length fills it, as in the gap check of
/// [`check_chronology`](super::chronology::check_chronology). `added` holds the facts that later
/// rules give for the series, such as the values of Tier 3.
///
/// The finding has a reason only if a rule can tell it:
///
/// - [`MissingReason::Conflict`] if a blocked series key of the concept ends within 7 days of the
///   period of report.
/// - Otherwise [`MissingReason::Untagged`] if `candidates` holds no candidate of the filing for the
///   concept, for any period.
///
/// The findings follow the order of `filings`, then the order of `required`.
#[must_use]
pub fn check_required(
    series: &Series,
    added: &[StandardizedFact],
    candidates: &[Candidate],
    filings: &[Filing],
    required: &[CanonicalElementKind],
) -> Vec<Finding> {
    let mut findings = Vec::new();
    for filing in filings.iter().filter(|filing| is_checked(filing)) {
        let Some(period_end) = filing.period_of_report else {
            continue;
        };
        for &concept in required {
            let slot = Slot {
                entity: &filing.entity,
                concept,
                period_end,
            };
            let filled = series.facts.iter().chain(added).any(|fact| {
                slot.holds(
                    &fact.entity,
                    fact.concept,
                    &fact.dimensions,
                    fact.period.end(),
                )
            });
            if filled {
                continue;
            }
            let blocked = series
                .blocked
                .iter()
                .any(|key| slot.holds(&key.entity, key.concept, &key.dimensions, key.period_end));
            let tagged = candidates.iter().any(|candidate| {
                candidate.source.filing == filing.id
                    && candidate.concept == concept
                    && candidate.dimensions.is_empty()
            });
            let reason = if blocked {
                Some(MissingReason::Conflict)
            } else if tagged {
                None
            } else {
                Some(MissingReason::Untagged)
            };
            findings.push(Finding::MissingRequired {
                entity: filing.entity.clone(),
                filing: filing.id.clone(),
                concept,
                period_end,
                reason,
            });
        }
    }
    findings
}

/// One required concept of one entity at the period of report of one filing.
struct Slot<'a> {
    entity: &'a EntityId,
    concept: CanonicalElementKind,
    period_end: Date,
}

impl Slot<'_> {
    /// Returns `true` if a value or a key with these fields belongs to the slot: same entity and
    /// concept, no dimensions, and an end within 7 days of the period of report.
    fn holds(
        &self,
        entity: &EntityId,
        concept: CanonicalElementKind,
        dimensions: &Dimensions,
        end: Date,
    ) -> bool {
        entity == self.entity
            && concept == self.concept
            && dimensions.is_empty()
            && is_near(end, self.period_end)
    }
}

/// Returns `true` if `filing` is an annual or a quarterly report and no amendment.
fn is_checked(filing: &Filing) -> bool {
    !filing.is_amendment
        && (is_form(&filing.form, &ANNUAL_FORMS) || is_form(&filing.form, &QUARTERLY_FORMS))
}

#[cfg(test)]
mod tests {
    use pretty_assertions::assert_eq;

    use super::*;
    use crate::core::confidence::Confidence;
    use crate::core::elements::{BalanceSheetConcept, IncomeStatementConcept};
    use crate::core::filing::FilingId;
    use crate::core::period::{Duration, Period};
    use crate::tests::fixtures::sample_fact::{
        apple, candidate, date, filing, in_cloud_segment, source, standardized,
    };

    /// Returns the 10-K of Apple for the fiscal year that ends on 2024-09-28.
    fn annual_report() -> Filing {
        filing("10-K", date(2024, 9, 28), "0928")
    }

    /// Returns an empty series.
    fn no_series() -> Series {
        Series {
            facts: Vec::new(),
            blocked: Vec::new(),
        }
    }

    /// Returns the revenue candidate of [`annual_report`] for the fiscal year 2024.
    fn revenue_of_annual_report() -> Candidate {
        candidate(&source("2024-09-28-10-K", date(2024, 10, 28)), "391")
    }

    /// Returns the reason of each finding in `findings`.
    fn reasons(findings: &[Finding]) -> Vec<Option<MissingReason>> {
        findings
            .iter()
            .filter_map(|finding| match finding {
                Finding::MissingRequired { reason, .. } => Some(*reason),
                _ => None,
            })
            .collect()
    }

    #[test]
    fn should_report_the_concept_as_untagged_when_the_filing_has_no_value_of_it() {
        let filings = vec![annual_report()];
        let required = [IncomeStatementConcept::Revenue.element()];

        let expected_result = vec![Finding::MissingRequired {
            entity: apple(),
            filing: FilingId::new("sec", "2024-09-28-10-K")
                .expect("Given a hardcoded source and ID, the creation should always succeed"),
            concept: IncomeStatementConcept::Revenue.element(),
            period_end: date(2024, 9, 28),
            reason: Some(MissingReason::Untagged),
        }];

        let result = check_required(&no_series(), &[], &[], &filings, &required);

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_give_the_reason_conflict_when_dedup_blocks_the_value_of_the_period() {
        let blocked = revenue_of_annual_report();
        let series = Series {
            facts: Vec::new(),
            blocked: vec![blocked.series_key()],
        };
        let candidates = vec![blocked];
        let filings = vec![annual_report()];
        let required = [IncomeStatementConcept::Revenue.element()];

        let expected_result = vec![Some(MissingReason::Conflict)];

        let result = reasons(&check_required(
            &series,
            &[],
            &candidates,
            &filings,
            &required,
        ));

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_give_no_finding_when_tier_3_derives_the_value() {
        let derived = StandardizedFact {
            concept: BalanceSheetConcept::Liabilities.element(),
            confidence: Confidence::Derived,
            ..standardized(
                Period::Instant {
                    end: date(2024, 9, 28),
                },
                "7468",
            )
        };
        let added = vec![derived];
        let filings = vec![annual_report()];
        let required = [BalanceSheetConcept::Liabilities.element()];

        let expected_result: Vec<Finding> = Vec::new();

        let result = check_required(&no_series(), &added, &[], &filings, &required);

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_give_no_reason_when_the_filing_tags_the_concept_for_another_period_only() {
        let earlier_year = Candidate {
            period: Period::Duration(
                Duration::from_dates(date(2022, 9, 25), date(2023, 9, 30))
                    .expect("Given an end after the start, the creation should always succeed"),
            ),
            ..revenue_of_annual_report()
        };
        let candidates = vec![earlier_year];
        let filings = vec![annual_report()];
        let required = [IncomeStatementConcept::Revenue.element()];

        let expected_result = vec![None];

        let result = reasons(&check_required(
            &no_series(),
            &[],
            &candidates,
            &filings,
            &required,
        ));

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_give_a_finding_when_only_a_segment_value_covers_the_period() {
        let fiscal_year = Period::Duration(
            Duration::from_dates(date(2023, 10, 1), date(2024, 9, 28))
                .expect("Given an end after the start, the creation should always succeed"),
        );
        let added = in_cloud_segment(vec![standardized(fiscal_year, "100")]);
        let filings = vec![annual_report()];
        let required = [IncomeStatementConcept::Revenue.element()];

        let expected_result = vec![Some(MissingReason::Untagged)];

        let result = reasons(&check_required(
            &no_series(),
            &added,
            &[],
            &filings,
            &required,
        ));

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_give_no_finding_when_the_filing_is_an_amendment() {
        let filings = vec![filing("10-K/A", date(2024, 9, 28), "0928")];
        let required = [IncomeStatementConcept::Revenue.element()];

        let expected_result: Vec<Finding> = Vec::new();

        let result = check_required(&no_series(), &[], &[], &filings, &required);

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_give_no_finding_when_the_filing_is_no_periodic_report() {
        let filings = vec![filing("8-K", date(2024, 9, 28), "0928")];
        let required = [IncomeStatementConcept::Revenue.element()];

        let expected_result: Vec<Finding> = Vec::new();

        let result = check_required(&no_series(), &[], &[], &filings, &required);

        assert_eq!(result, expected_result);
    }
}
