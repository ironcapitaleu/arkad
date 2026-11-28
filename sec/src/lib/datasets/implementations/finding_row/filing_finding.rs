//! # Filing Finding
//!
//! Provides [`FindingRow::from_filing_finding`], the row of a finding of one filing.

use xbrl::core::filing::EntityId;
use xbrl::core::finding::Finding;

use super::{FindingKind, FindingPeriod, FindingRow, FindingStep};
use crate::datasets::RunColumns;

impl FindingRow {
    /// Returns the row of a finding of the parse or the standardize filing step, with
    /// `step = filing`.
    ///
    /// - `UndeclaredFamily`: `subject` holds the family.
    /// - `UnmappedRow`: `subject` holds the row as the parser prints it, and `reason` the name of
    ///   the rule the row breaks, for example `NotANumber`.
    /// - `Conflict`: `concepts` holds the as-is concept, then the standard concept if a plug-in
    ///   maps it. `periods` holds the period of the value key.
    ///
    /// A parser finding names no entity, so its row takes `entity`, the entity of the filing.
    /// Returns `None` for a kind of the series step, such as `Gap`.
    #[must_use]
    pub fn from_filing_finding(
        finding: &Finding,
        entity: &EntityId,
        run: RunColumns,
    ) -> Option<Self> {
        let row = |kind, entity: &EntityId| Self {
            kind,
            entity: entity.clone(),
            concepts: Vec::new(),
            periods: Vec::new(),
            subject: None,
            reason: None,
            filing: None,
            confirmed_by: None,
            confidence: None,
            step: FindingStep::Filing,
            run,
        };
        match finding {
            Finding::UndeclaredFamily { filing, family } => Some(Self {
                subject: Some(family.to_string()),
                filing: Some(filing.clone()),
                ..row(FindingKind::UndeclaredFamily, entity)
            }),
            Finding::UnmappedRow {
                filing,
                row: source_row,
                reason,
            } => Some(Self {
                subject: Some(source_row.clone()),
                reason: Some(reason.reason.name().to_owned()),
                filing: Some(filing.clone()),
                ..row(FindingKind::UnmappedRow, entity)
            }),
            Finding::Conflict {
                entity,
                key,
                standard_concept,
                ..
            } => Some(Self {
                concepts: std::iter::once(key.concept.to_string())
                    .chain(standard_concept.map(|concept| concept.name().to_owned()))
                    .collect(),
                periods: vec![FindingPeriod {
                    period_end: key.period_end,
                    quarters: key.quarters,
                }],
                filing: Some(key.filing.clone()),
                ..row(FindingKind::Conflict, entity)
            }),
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use std::str::FromStr;

    use jiff::civil::date;
    use pretty_assertions::assert_eq;
    use xbrl::core::concept::{ConceptName, NamespaceFamily};
    use xbrl::core::elements::IncomeStatementConcept;
    use xbrl::core::fact::{Dimensions, FactKey};
    use xbrl::core::filing::FilingId;
    use xbrl::core::invalid_value::{InvalidValue, InvalidValueReason};
    use xbrl::core::unit::Unit;

    use super::*;
    use crate::datasets::implementations::run_columns::sample_run_columns;

    const PARSE: &str = "Given hardcoded valid parts, the creation should always succeed";

    fn filing() -> FilingId {
        FilingId::from_str("sec:0000320193-26-000001").expect(PARSE)
    }

    fn entity() -> EntityId {
        EntityId::new("sec", "320193").expect(PARSE)
    }

    /// The row of the filing of [`filing`] about [`entity`], with no optional column set.
    fn filing_row(kind: FindingKind) -> FindingRow {
        FindingRow {
            kind,
            entity: entity(),
            concepts: Vec::new(),
            periods: Vec::new(),
            subject: None,
            reason: None,
            filing: Some(filing()),
            confirmed_by: None,
            confidence: None,
            step: FindingStep::Filing,
            run: sample_run_columns(),
        }
    }

    fn conflict(standard_concept: bool) -> Finding {
        Finding::Conflict {
            entity: entity(),
            key: FactKey {
                filing: filing(),
                concept: ConceptName::new(
                    NamespaceFamily::new("us-gaap").expect(PARSE),
                    "Revenues",
                )
                .expect(PARSE),
                taxonomy_version: None,
                period_end: date(2026, 9, 26),
                quarters: 4,
                unit: Some(Unit::from_str("USD").expect(PARSE)),
                dimensions: Dimensions::default(),
                copy_rank: 0,
            },
            ranks: vec![1],
            standard_concept: standard_concept.then(|| IncomeStatementConcept::Revenue.element()),
        }
    }

    #[test]
    fn should_hold_the_family_as_subject_for_an_undeclared_family() {
        let finding = Finding::UndeclaredFamily {
            filing: filing(),
            family: NamespaceFamily::new("xyz/2027").expect(PARSE),
        };

        let expected_result = Some(FindingRow {
            subject: Some("xyz/2027".to_owned()),
            ..filing_row(FindingKind::UndeclaredFamily)
        });

        let result = FindingRow::from_filing_finding(&finding, &entity(), sample_run_columns());

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_hold_the_row_and_the_rule_name_for_an_unmapped_row() {
        let finding = Finding::UnmappedRow {
            filing: filing(),
            row: "num.tsv Revenues us-gaap/2026 2026-09-30 1 USD 0x00000000 0".to_owned(),
            reason: InvalidValue::new(
                "NumRow",
                InvalidValueReason::NoMatchingRow("dim.tsv"),
                "0xff",
            ),
        };

        let expected_result = Some(FindingRow {
            subject: Some("num.tsv Revenues us-gaap/2026 2026-09-30 1 USD 0x00000000 0".to_owned()),
            reason: Some("NoMatchingRow".to_owned()),
            ..filing_row(FindingKind::UnmappedRow)
        });

        let result = FindingRow::from_filing_finding(&finding, &entity(), sample_run_columns());

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_hold_both_concepts_and_the_period_for_a_conflict_with_a_standard_concept() {
        let finding = conflict(true);

        let expected_result = Some(FindingRow {
            concepts: vec!["us-gaap:Revenues".to_owned(), "Revenue".to_owned()],
            periods: vec![FindingPeriod {
                period_end: date(2026, 9, 26),
                quarters: 4,
            }],
            ..filing_row(FindingKind::Conflict)
        });

        let result = FindingRow::from_filing_finding(&finding, &entity(), sample_run_columns());

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_hold_only_the_as_is_concept_for_a_conflict_without_a_standard_concept() {
        let finding = conflict(false);

        let expected_result = Some(vec!["us-gaap:Revenues".to_owned()]);

        let result = FindingRow::from_filing_finding(&finding, &entity(), sample_run_columns())
            .map(|row| row.concepts);

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_return_none_for_a_finding_of_the_series_step() {
        let finding = Finding::MissingRequired {
            entity: entity(),
            filing: filing(),
            concept: IncomeStatementConcept::Revenue.element(),
            period_end: date(2026, 9, 26),
            reason: None,
        };

        let expected_result = None;

        let result = FindingRow::from_filing_finding(&finding, &entity(), sample_run_columns());

        assert_eq!(result, expected_result);
    }
}
