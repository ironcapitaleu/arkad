//! # Cadence Filter
//!
//! Provides [`keep_by_cadence`], the step after tiers 1 and 2 that keeps a candidate of a concept
//! with the cadence [`Cadence::FiscalYearEnd`] only at the fiscal year end of its filing.

use super::alias::FilingCandidates;
use super::candidate::Candidate;
use super::elements::Cadence;
use super::filing::Filing;
use super::period::{Period, is_near};

/// The fiscal period of a filing that reports a fiscal year.
const FISCAL_YEAR: &str = "FY";

/// Keeps the candidates of one filing that the cadence of their concept allows, and keeps every
/// finding.
///
/// `filing` is the filing that gave the candidates. A concept with [`Cadence::EveryPeriod`] keeps
/// every candidate. A concept with [`Cadence::FiscalYearEnd`] keeps a candidate only if the
/// filing has the fiscal period `FY` and the candidate is an instant within 7 days of the period of
/// report of the filing. The Notes data set rounds the period of report to the month end, and a
/// 52-53 week year ends up to 6 days away from it. If the filing has no period of report or no
/// fiscal period, the filter drops every candidate of such a concept. The facts of a dropped
/// candidate stay as-is facts.
///
/// # Examples
///
/// ```
/// use xbrl::core::alias::FilingCandidates;
/// use xbrl::core::cadence_filter::keep_by_cadence;
/// use xbrl::tests::fixtures::sample_fact::{candidate, date, filing, source};
///
/// let quarterly_report = filing("10-Q", date(2024, 6, 29), "0930");
/// let revenue = candidate(&source("0000320193-24-000123", date(2024, 7, 29)), "100");
/// let filing_candidates = FilingCandidates {
///     candidates: vec![revenue],
///     findings: Vec::new(),
/// };
///
/// let expected_result = filing_candidates.clone();
///
/// let result = keep_by_cadence(filing_candidates, &quarterly_report);
///
/// assert_eq!(result, expected_result);
/// ```
#[must_use]
pub fn keep_by_cadence(filing_candidates: FilingCandidates, filing: &Filing) -> FilingCandidates {
    let FilingCandidates {
        candidates,
        findings,
    } = filing_candidates;
    FilingCandidates {
        candidates: candidates
            .into_iter()
            .filter(|candidate| is_kept(candidate, filing))
            .collect(),
        findings,
    }
}

/// Returns `true` if the cadence of the concept of `candidate` allows its period in `filing`.
fn is_kept(candidate: &Candidate, filing: &Filing) -> bool {
    match candidate.concept.cadence() {
        Cadence::EveryPeriod => true,
        Cadence::FiscalYearEnd => is_fiscal_year_end_of(&candidate.period, filing),
    }
}

/// Returns `true` if `filing` reports a fiscal year and `period` is an instant within 7 days of its
/// period of report.
fn is_fiscal_year_end_of(period: &Period, filing: &Filing) -> bool {
    let reports_fiscal_year = filing
        .fiscal_period
        .as_ref()
        .is_some_and(|fiscal_period| fiscal_period.as_str() == FISCAL_YEAR);
    let is_report_instant = filing
        .period_of_report
        .is_some_and(|report| matches!(period, Period::Instant { end } if is_near(*end, report)));
    reports_fiscal_year && is_report_instant
}

#[cfg(test)]
mod tests {
    use chrono::NaiveDate;
    use pretty_assertions::assert_eq;

    use super::*;
    use crate::core::alias::resolve_aliases;
    use crate::core::dedup::more_precise;
    use crate::core::elements::{CanonicalElementKind, DebtMaturityConcept};
    use crate::core::fact::AsIsFact;
    use crate::core::finding::Finding;
    use crate::core::fiscal_period::FiscalPeriod;
    use crate::core::taxonomy::registry::Registry;
    use crate::tests::fixtures::sample_fact::{candidate, date, fact, filing, source};
    use crate::us_gaap::UsGaap;

    /// The fiscal year end of the filer in these tests, 2024-09-28.
    const YEAR_END: NaiveDate = date(2024, 9, 28);

    /// The end of the third quarter of the filer in these tests, 2024-06-29.
    const QUARTER_END: NaiveDate = date(2024, 6, 29);

    /// Returns the filing of `form` with the fiscal period `fiscal_period` for the period that
    /// ends on `period_of_report`.
    fn report(form: &str, period_of_report: NaiveDate, fiscal_period: &str) -> Filing {
        let mut report = filing(form, period_of_report, "0930");
        report.fiscal_period = Some(FiscalPeriod::new(fiscal_period).expect(
            "Given a hardcoded period without whitespace, the creation should always succeed",
        ));
        report
    }

    /// Returns the as-is fact `us-gaap:<local_name>` = `value` of `filing` at the instant `end`.
    fn instant_fact(filing: &Filing, local_name: &str, end: NaiveDate) -> AsIsFact {
        let mut instant = fact(
            &source(filing.id.id(), filing.filed),
            local_name,
            "500000000",
        );
        instant.context.period = Period::Instant { end };
        instant
    }

    /// Runs `core/dedup/more-precise@1`, tiers 1 and 2 of the US GAAP plug-in, and the filter on
    /// `facts` of `filing`, and returns the standard concepts of the candidates.
    fn kept_concepts(facts: &[AsIsFact], filing: &Filing) -> Vec<CanonicalElementKind> {
        let us_gaap = UsGaap::new();
        let registry = Registry::new(vec![&us_gaap])
            .expect("Given one plug-in with distinct families, the registry should always start");
        keep_by_cadence(resolve_aliases(&more_precise(facts), &registry), filing)
            .candidates
            .into_iter()
            .map(|candidate| candidate.concept)
            .collect()
    }

    /// Returns a candidate of `concept` from `filing` at the instant `end`.
    fn instant_candidate(
        concept: DebtMaturityConcept,
        filing: &Filing,
        end: NaiveDate,
    ) -> Candidate {
        let mut instant = candidate(&source(filing.id.id(), filing.filed), "500000000");
        instant.concept = concept.element();
        instant.period = Period::Instant { end };
        instant
    }

    #[test]
    fn should_give_no_bucket_candidate_when_a_10_q_reports_the_bucket() {
        let quarterly_report = report("10-Q", QUARTER_END, "Q3");
        let facts = vec![instant_fact(
            &quarterly_report,
            "LongTermDebtMaturitiesRepaymentsOfPrincipalInYearTwo",
            QUARTER_END,
        )];

        let expected_result: Vec<CanonicalElementKind> = Vec::new();

        let result = kept_concepts(&facts, &quarterly_report);

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_give_a_bucket_candidate_when_a_10_k_reports_the_bucket() {
        let annual_report = report("10-K", YEAR_END, "FY");
        let facts = vec![instant_fact(
            &annual_report,
            "LongTermDebtMaturitiesRepaymentsOfPrincipalInYearTwo",
            YEAR_END,
        )];

        let expected_result = vec![DebtMaturityConcept::DebtDueInYearTwo.element()];

        let result = kept_concepts(&facts, &annual_report);

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_keep_the_total_candidate_when_a_10_q_reports_the_total() {
        let quarterly_report = report("10-Q", QUARTER_END, "Q3");
        let facts = vec![instant_fact(&quarterly_report, "LongTermDebt", QUARTER_END)];

        let expected_result =
            vec![DebtMaturityConcept::LongTermDebtIncludingCurrentMaturities.element()];

        let result = kept_concepts(&facts, &quarterly_report);

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_give_a_bucket_candidate_when_the_instant_lies_within_7_days_of_the_period_of_report()
    {
        let annual_report = report("10-K", date(2024, 9, 30), "FY");
        let facts = vec![instant_fact(
            &annual_report,
            "LongTermDebtMaturitiesRepaymentsOfPrincipalInYearTwo",
            YEAR_END,
        )];

        let expected_result = vec![DebtMaturityConcept::DebtDueInYearTwo.element()];

        let result = kept_concepts(&facts, &annual_report);

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_drop_a_bucket_candidate_when_its_instant_is_a_comparative_of_a_10_k() {
        let annual_report = report("10-K", YEAR_END, "FY");
        let comparative = instant_candidate(
            DebtMaturityConcept::DebtDueInYearOne,
            &annual_report,
            date(2023, 9, 30),
        );
        let filing_candidates = FilingCandidates {
            candidates: vec![comparative],
            findings: Vec::new(),
        };

        let expected_result: Vec<Candidate> = Vec::new();

        let result = keep_by_cadence(filing_candidates, &annual_report).candidates;

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_drop_a_bucket_candidate_when_the_filing_has_no_fiscal_period() {
        let annual_report = filing("10-K", YEAR_END, "0930");
        let bucket = instant_candidate(
            DebtMaturityConcept::DebtDueInYearOne,
            &annual_report,
            YEAR_END,
        );
        let filing_candidates = FilingCandidates {
            candidates: vec![bucket],
            findings: Vec::new(),
        };

        let expected_result: Vec<Candidate> = Vec::new();

        let result = keep_by_cadence(filing_candidates, &annual_report).candidates;

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_drop_a_bucket_candidate_when_the_filing_has_no_period_of_report() {
        let mut annual_report = report("10-K", YEAR_END, "FY");
        let bucket = instant_candidate(
            DebtMaturityConcept::DebtDueInYearOne,
            &annual_report,
            YEAR_END,
        );
        annual_report.period_of_report = None;
        let filing_candidates = FilingCandidates {
            candidates: vec![bucket],
            findings: Vec::new(),
        };

        let expected_result: Vec<Candidate> = Vec::new();

        let result = keep_by_cadence(filing_candidates, &annual_report).candidates;

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_keep_every_finding_when_the_filter_drops_a_bucket_candidate() {
        let quarterly_report = report("10-Q", QUARTER_END, "Q3");
        let bucket = instant_candidate(
            DebtMaturityConcept::DebtDueInYearOne,
            &quarterly_report,
            QUARTER_END,
        );
        let conflict = Finding::Conflict {
            entity: bucket.entity.clone(),
            key: instant_fact(
                &quarterly_report,
                "LongTermDebtMaturitiesRepaymentsOfPrincipalInNextTwelveMonths",
                QUARTER_END,
            )
            .key(),
            ranks: vec![1],
            standard_concept: Some(DebtMaturityConcept::DebtDueInYearOne.element()),
        };
        let filing_candidates = FilingCandidates {
            candidates: vec![bucket],
            findings: vec![conflict.clone()],
        };

        let expected_result = vec![conflict];

        let result = keep_by_cadence(filing_candidates, &quarterly_report).findings;

        assert_eq!(result, expected_result);
    }
}
