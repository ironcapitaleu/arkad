//! # Debt Maturity Concepts
//!
//! Provides [`DebtMaturityConcept`], the maturity ladder and its total of the disclosure
//! `DebtMaturities`.

use crate::core::elements::{Cadence, SumRole, Summed, Timed};

innermost_concepts! {
    /// A concept of the maturity ladder of long-term debt, or the total that the ladder adds up to.
    ///
    /// Each value is an instant. A bucket holds the principal that falls due in its span, counted
    /// from the fiscal year end. The total holds the carrying amount, current maturities included.
    /// Neither holds finance leases.
    pub enum DebtMaturityConcept in Disclosure(DisclosureConcept::DebtMaturities) {
        /// Long-term debt with its current maturities: the total of the ladder.
        LongTermDebtIncludingCurrentMaturities => "Long-Term Debt Including Current Maturities",
        /// The principal due in the first fiscal year after the fiscal year end.
        DebtDueInYearOne => "Debt Due in Year One",
        /// The principal due in the second fiscal year after the fiscal year end.
        DebtDueInYearTwo => "Debt Due in Year Two",
        /// The principal due in the third fiscal year after the fiscal year end.
        DebtDueInYearThree => "Debt Due in Year Three",
        /// The principal due in the fourth fiscal year after the fiscal year end.
        DebtDueInYearFour => "Debt Due in Year Four",
        /// The principal due in the fifth fiscal year after the fiscal year end.
        DebtDueInYearFive => "Debt Due in Year Five",
        /// The principal due after the fifth fiscal year after the fiscal year end.
        DebtDueAfterYearFive => "Debt Due After Year Five",
    }
}

impl Summed for DebtMaturityConcept {
    const SUM_NAME: &'static str = "debt-maturities";
    // ADR 0028 records the tolerance: the buckets hold principal, the total the carrying amount.
    const TOLERANCE_BASIS_POINTS: u32 = 500;

    fn role(self) -> SumRole {
        match self {
            Self::LongTermDebtIncludingCurrentMaturities => SumRole::Total,
            Self::DebtDueInYearOne
            | Self::DebtDueInYearTwo
            | Self::DebtDueInYearThree
            | Self::DebtDueInYearFour
            | Self::DebtDueInYearFive
            | Self::DebtDueAfterYearFive => SumRole::Part,
        }
    }
}

impl Timed for DebtMaturityConcept {
    /// Gives [`Cadence::FiscalYearEnd`] for a bucket, because only an annual report holds the
    /// ladder. Gives [`Cadence::EveryPeriod`] for the total, a balance sheet value.
    fn cadence(self) -> Cadence {
        match self {
            Self::LongTermDebtIncludingCurrentMaturities => Cadence::EveryPeriod,
            Self::DebtDueInYearOne
            | Self::DebtDueInYearTwo
            | Self::DebtDueInYearThree
            | Self::DebtDueInYearFour
            | Self::DebtDueInYearFive
            | Self::DebtDueAfterYearFive => Cadence::FiscalYearEnd,
        }
    }
}

#[cfg(test)]
mod tests {
    use pretty_assertions::assert_eq;

    use super::DebtMaturityConcept;
    use crate::core::elements::{Cadence, Concept, SumRole, Summed, Timed};

    const fn implements_auto_traits<T: Sized + Send + Sync + Unpin>() {}

    #[test]
    const fn should_implement_auto_traits_for_debt_maturity_concept() {
        implements_auto_traits::<DebtMaturityConcept>();
    }

    #[test]
    fn should_give_the_cadence_of_the_concept_when_asking_its_element_for_every_concept() {
        let concepts = DebtMaturityConcept::ALL;

        // The expected result is the cadence of `Timed`, the reference path. The test fails if the
        // middle macro misses `Timed` in the entry of the disclosure.
        let expected_result: Vec<Cadence> =
            concepts.iter().map(|concept| concept.cadence()).collect();

        let result: Vec<Cadence> = concepts
            .iter()
            .map(|concept| concept.element().cadence())
            .collect();

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_give_fiscal_year_end_when_the_concept_is_a_bucket() {
        let element = DebtMaturityConcept::DebtDueInYearTwo.element();

        let expected_result = Cadence::FiscalYearEnd;

        let result = element.cadence();

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_give_every_period_when_the_concept_is_the_total() {
        let element = DebtMaturityConcept::LongTermDebtIncludingCurrentMaturities.element();

        let expected_result = Cadence::EveryPeriod;

        let result = element.cadence();

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_give_the_total_first_then_six_parts_for_the_roles_of_all_concepts() {
        let concepts = DebtMaturityConcept::ALL;

        let expected_result = vec![
            SumRole::Total,
            SumRole::Part,
            SumRole::Part,
            SumRole::Part,
            SumRole::Part,
            SumRole::Part,
            SumRole::Part,
        ];

        let result: Vec<SumRole> = concepts.iter().map(|concept| concept.role()).collect();

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_name_the_sum_debt_maturities_with_a_tolerance_of_500_basis_points_for_debt_maturity_concept()
     {
        let expected_result = ("debt-maturities", 500);

        let result = (
            DebtMaturityConcept::SUM_NAME,
            DebtMaturityConcept::TOLERANCE_BASIS_POINTS,
        );

        assert_eq!(result, expected_result);
    }
}
