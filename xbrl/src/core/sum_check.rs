//! # Sum Check
//!
//! Provides [`check_sums`], the check of one sum of a disclosure on the standardized series of one
//! entity, and [`check_all_sums`], which runs it for every enum with [`Summed`]. Each failed check
//! is a [`Finding::SumMismatch`]. A check never removes or derives a value.

use std::collections::BTreeMap;
use std::str::FromStr;

use bigdecimal::BigDecimal;
use chrono::NaiveDate;

use super::check_tolerance::{exceeds_tolerance, weakest};
use super::dedup::Series;
use super::elements::{CanonicalElementKind, SumRole, Summed};
use super::fact::Dimensions;
use super::filing::EntityId;
use super::finding::Finding;
use super::rule::RuleId;
use super::standardized::StandardizedFact;
use super::taxonomy::reference::PeriodType;
use super::unit::Unit;

pub use super::elements::check_all_sums;

/// Runs the check of the sum of `S` on the series of one entity, and returns a
/// [`Finding::SumMismatch`] for each failed check.
///
/// `added` holds the facts that later rules give for the series, as in
/// [`check_roll_ups`](super::roll_up_check::check_roll_ups). The check reads only values without
/// dimensions. It runs on each period and unit: the sum of the parts must equal the total within
/// [`Summed::TOLERANCE_BASIS_POINTS`] of the largest absolute value of the check.
///
/// The check runs only if every concept of `S` has a value for the period and the unit. A missing
/// part never counts as zero. So values of mixed units get no check.
#[must_use]
pub fn check_sums<S>(series: &Series, added: &[StandardizedFact]) -> Vec<Finding>
where
    S: Summed + TryFrom<CanonicalElementKind>,
{
    Slot::all::<S>(series, added)
        .into_iter()
        .filter_map(|slot| slot.check::<S>())
        .collect()
}

/// Returns the rule ID of the sum of `S`: `core/sum/<SUM_NAME>@1`.
///
/// # Panics
///
/// Panics if [`Summed::SUM_NAME`] is not a kebab-case name without whitespace.
#[must_use]
pub fn sum_rule<S: Summed>() -> RuleId {
    RuleId::from_str(&format!("core/sum/{}@1", S::SUM_NAME))
        .expect("Given a kebab-case sum name without whitespace, the rule ID should always parse")
}

/// The key of a [`Slot`]: the entity, the period end, the length in quarters, the period type, and
/// the unit.
type SlotKey<'a> = (&'a EntityId, NaiveDate, u16, PeriodType, &'a Unit);

/// The values without dimensions of the concepts of one sum for one entity, period, and unit.
struct Slot<'a> {
    entity: EntityId,
    unit: Unit,
    values: BTreeMap<CanonicalElementKind, &'a StandardizedFact>,
}

impl<'a> Slot<'a> {
    /// Returns the slots of the concepts of `S` in `series` and `added`, sorted by their key.
    fn all<S>(series: &'a Series, added: &'a [StandardizedFact]) -> Vec<Self>
    where
        S: Summed + TryFrom<CanonicalElementKind>,
    {
        let mut slots: BTreeMap<SlotKey<'a>, Self> = BTreeMap::new();
        for fact in series
            .facts
            .iter()
            .chain(added)
            .filter(|fact| fact.dimensions.is_empty())
            .filter(|fact| S::try_from(fact.concept).is_ok())
        {
            let key = (
                &fact.entity,
                fact.period.end(),
                fact.period.quarters(),
                fact.period.period_type(),
                &fact.unit,
            );
            slots
                .entry(key)
                .or_insert_with(|| Self {
                    entity: fact.entity.clone(),
                    unit: fact.unit.clone(),
                    values: BTreeMap::new(),
                })
                .values
                .insert(fact.concept, fact);
        }
        slots.into_values().collect()
    }

    /// Checks the sum of `S`, and returns a [`Finding::SumMismatch`] if it fails. Returns `None` if
    /// a concept of `S` has no value.
    fn check<S>(&self) -> Option<Finding>
    where
        S: Summed + TryFrom<CanonicalElementKind>,
    {
        let totals = self.facts_of::<S>(SumRole::Total)?;
        let parts = self.facts_of::<S>(SumRole::Part)?;
        let difference: BigDecimal = parts.iter().map(|fact| &fact.value).sum::<BigDecimal>()
            - totals.iter().map(|fact| &fact.value).sum::<BigDecimal>();
        let facts: Vec<&StandardizedFact> = totals.into_iter().chain(parts).collect();
        exceeds_tolerance(&difference, &facts, S::TOLERANCE_BASIS_POINTS).then(|| {
            Finding::SumMismatch {
                entity: self.entity.clone(),
                period: facts[0].period,
                unit: self.unit.clone(),
                dimensions: Dimensions::default(),
                concepts: facts.iter().map(|fact| fact.concept).collect(),
                reason: S::SUM_NAME,
                confidence: weakest(&facts),
            }
        })
    }

    /// Returns the facts of the concepts of `S` with `role`, in the order of the enum, or `None`
    /// if one of them has no value.
    fn facts_of<S>(&self, role: SumRole) -> Option<Vec<&'a StandardizedFact>>
    where
        S: Summed + TryFrom<CanonicalElementKind>,
    {
        let concepts: Vec<CanonicalElementKind> = self
            .values
            .keys()
            .copied()
            .filter(|concept| S::try_from(*concept).is_ok_and(|summed| summed.role() == role))
            .collect();
        let expected = S::ALL.iter().filter(|summed| summed.role() == role).count();
        (concepts.len() == expected).then(|| {
            concepts
                .iter()
                .map(|concept| self.values[concept])
                .collect()
        })
    }
}

#[cfg(test)]
mod tests {
    use pretty_assertions::assert_eq;

    use super::*;
    use crate::core::confidence::Confidence;
    use crate::core::elements::{Concept, DebtMaturityConcept};
    use crate::core::period::Period;
    use crate::tests::fixtures::sample_fact::{date, in_cloud_segment, standardized};

    /// Returns the fiscal year end of the ladder.
    const fn instant() -> Period {
        Period::Instant {
            end: date(2025, 12, 31),
        }
    }

    /// Returns the fiscal year end before [`instant`].
    const fn earlier_instant() -> Period {
        Period::Instant {
            end: date(2024, 12, 31),
        }
    }

    /// Returns the value `amount` of `concept` at `period`.
    fn value(period: Period, concept: DebtMaturityConcept, amount: &str) -> StandardizedFact {
        let mut fact = standardized(period, amount);
        fact.concept = concept.element();
        fact
    }

    /// Returns a ladder at [`instant`] with the total 1,000, five buckets of 100, and
    /// `after_year_five` after year five. The tolerance of 5% of 1,000 is 50, so the ladder fails
    /// above 550 and below 450.
    fn ladder(after_year_five: &str) -> Vec<StandardizedFact> {
        ladder_at(instant(), after_year_five)
    }

    /// Returns the ladder of [`ladder`] at `period`.
    fn ladder_at(period: Period, after_year_five: &str) -> Vec<StandardizedFact> {
        DebtMaturityConcept::ALL
            .iter()
            .map(|concept| match concept {
                DebtMaturityConcept::LongTermDebtIncludingCurrentMaturities => {
                    value(period, *concept, "1000")
                }
                DebtMaturityConcept::DebtDueAfterYearFive => {
                    value(period, *concept, after_year_five)
                }
                _ => value(period, *concept, "100"),
            })
            .collect()
    }

    fn series(facts: Vec<StandardizedFact>) -> Series {
        Series {
            facts,
            blocked: Vec::new(),
        }
    }

    fn mismatch(confidence: Confidence) -> Finding {
        mismatch_at(instant(), confidence)
    }

    fn mismatch_at(period: Period, confidence: Confidence) -> Finding {
        let fact = standardized(period, "0");
        Finding::SumMismatch {
            entity: fact.entity,
            period,
            unit: fact.unit,
            dimensions: Dimensions::default(),
            concepts: vec![
                DebtMaturityConcept::LongTermDebtIncludingCurrentMaturities.element(),
                DebtMaturityConcept::DebtDueInYearOne.element(),
                DebtMaturityConcept::DebtDueInYearTwo.element(),
                DebtMaturityConcept::DebtDueInYearThree.element(),
                DebtMaturityConcept::DebtDueInYearFour.element(),
                DebtMaturityConcept::DebtDueInYearFive.element(),
                DebtMaturityConcept::DebtDueAfterYearFive.element(),
            ],
            reason: "debt-maturities",
            confidence,
        }
    }

    #[test]
    fn should_return_no_finding_when_the_ladder_misses_by_exactly_the_tolerance() {
        let facts = series(ladder("550"));
        let added = Vec::new();

        let expected_result: Vec<Finding> = Vec::new();

        let result = check_sums::<DebtMaturityConcept>(&facts, &added);

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_report_a_sum_mismatch_when_the_ladder_misses_by_more_than_the_tolerance() {
        let facts = series(ladder("551"));
        let added = Vec::new();

        let expected_result = vec![mismatch(Confidence::Exact)];

        let result = check_sums::<DebtMaturityConcept>(&facts, &added);

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_report_a_sum_mismatch_when_the_parts_sum_below_the_total_by_more_than_the_tolerance()
    {
        let facts = series(ladder("449"));
        let added = Vec::new();

        let expected_result = vec![mismatch(Confidence::Exact)];

        let result = check_sums::<DebtMaturityConcept>(&facts, &added);

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_return_no_finding_when_a_bucket_has_no_value() {
        let mut facts = ladder("0");
        facts.retain(|fact| fact.concept != DebtMaturityConcept::DebtDueInYearThree.element());
        let facts = series(facts);
        let added = Vec::new();

        let expected_result: Vec<Finding> = Vec::new();

        let result = check_sums::<DebtMaturityConcept>(&facts, &added);

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_return_no_finding_when_the_total_has_no_value() {
        let mut facts = ladder("5000");
        facts.remove(0);
        let facts = series(facts);
        let added = Vec::new();

        let expected_result: Vec<Finding> = Vec::new();

        let result = check_sums::<DebtMaturityConcept>(&facts, &added);

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_return_no_finding_when_the_values_have_mixed_units() {
        let mut facts = ladder("5000");
        facts[1].unit = Unit::from_str("EUR")
            .expect("Given a hardcoded unit with one measure, the parse should always succeed");
        let facts = series(facts);
        let added = Vec::new();

        let expected_result: Vec<Finding> = Vec::new();

        let result = check_sums::<DebtMaturityConcept>(&facts, &added);

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_return_no_finding_when_the_values_carry_a_business_segment() {
        let facts = series(in_cloud_segment(ladder("5000")));
        let added = Vec::new();

        let expected_result: Vec<Finding> = Vec::new();

        let result = check_sums::<DebtMaturityConcept>(&facts, &added);

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_report_the_weakest_confidence_when_the_ladder_fails() {
        let mut facts = ladder("551");
        facts[0].confidence = Confidence::Synonym;
        facts[4].confidence = Confidence::Derived;
        let facts = series(facts);
        let added = Vec::new();

        let expected_result = vec![mismatch(Confidence::Derived)];

        let result = check_sums::<DebtMaturityConcept>(&facts, &added);

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_read_the_values_of_added_when_the_series_misses_a_bucket() {
        let mut facts = ladder("551");
        let added = vec![facts.remove(6)];
        let facts = series(facts);

        let expected_result = vec![mismatch(Confidence::Exact)];

        let result = check_sums::<DebtMaturityConcept>(&facts, &added);

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_report_only_the_failed_ladder_when_the_series_holds_two_fiscal_year_ends() {
        let mut facts = ladder_at(earlier_instant(), "551");
        facts.extend(ladder("550"));
        let facts = series(facts);
        let added = Vec::new();

        let expected_result = vec![mismatch_at(earlier_instant(), Confidence::Exact)];

        let result = check_sums::<DebtMaturityConcept>(&facts, &added);

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_give_the_rule_id_core_sum_debt_maturities_for_debt_maturity_concept() {
        let expected_result = "core/sum/debt-maturities@1";

        let result = sum_rule::<DebtMaturityConcept>().to_string();

        assert_eq!(result, expected_result);
    }
}
