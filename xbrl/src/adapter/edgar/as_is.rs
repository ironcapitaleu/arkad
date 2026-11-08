//! # As-Is Rules
//!
//! Provides the rules that the Notes parser and the instance parser both apply to the as-is facts
//! of one filing: the family of a namespace that no plug-in declares, the copy ranks, and the
//! findings of undeclared families. One implementation keeps
//! the value keys of both sources equal.

use std::collections::{BTreeSet, HashMap};

use crate::core::concept::NamespaceFamily;
use crate::core::fact::{AsIsFact, FactKey, TaxonomyVersion};
use crate::core::filing::FilingId;
use crate::core::finding::Finding;
use crate::core::invalid_value::{InvalidValue, InvalidValueReason};
use crate::core::taxonomy::registry::Registry;

/// The type name that [`InvalidValue`] names when a key has more copies than `copy_rank` holds.
const COPY_RANK_TYPE_NAME: &str = "copy_rank";

/// Returns the family and the version of a namespace that the registry does not split.
///
/// An extension namespace gives the filing ID as family and version. Any other namespace keeps its
/// full text, the URI or the Notes `version`, as family and version.
///
/// # Errors
///
/// Returns [`InvalidValue`] if the namespace is empty or holds whitespace.
pub fn unsplit_namespace(
    namespace: &str,
    is_extension: bool,
    filing: &FilingId,
) -> Result<(NamespaceFamily, TaxonomyVersion), InvalidValue> {
    let text = if is_extension {
        filing.to_string()
    } else {
        namespace.to_owned()
    };
    Ok((
        NamespaceFamily::new(text.clone())?,
        TaxonomyVersion::new(text)?,
    ))
}

/// A fact before the parser numbers its copies, with the row it comes from and its place in the
/// copy order.
pub struct Candidate<O, R> {
    /// The place of the fact in the copy order. A lower value gets a lower `copy_rank`.
    pub order: O,
    /// The row or element that gave the fact, for a finding.
    pub row: R,
    /// The fact, with `copy_rank` 0.
    pub fact: AsIsFact,
}

/// Drops the facts that repeat a fact, and numbers the copies of each key in copy order.
///
/// A fact repeats an earlier fact if the two are equal in key, value, and `decimals`. The other
/// facts of one key get the ranks 0, 1, 2, and so on, in copy order, then in the order of the
/// candidates. The facts keep the order of the candidates.
///
/// Returns the kept facts, and each row whose rank does not fit `copy_rank`, with the reason.
pub fn number_copies<O: Ord, R>(
    candidates: Vec<Candidate<O, R>>,
) -> (Vec<AsIsFact>, Vec<(R, InvalidValue)>) {
    let mut order: Vec<usize> = (0..candidates.len()).collect();
    order.sort_by(|&left, &right| candidates[left].order.cmp(&candidates[right].order));
    let mut copies: HashMap<FactKey, Vec<usize>> = HashMap::new();
    let mut outcomes: Vec<Option<Result<u16, InvalidValue>>> = vec![None; candidates.len()];
    for index in order {
        let fact = &candidates[index].fact;
        let earlier = copies.entry(fact.key()).or_default();
        if earlier.iter().any(|&other| {
            candidates[other].fact.value == fact.value
                && candidates[other].fact.decimals == fact.decimals
        }) {
            continue;
        }
        let rank = u16::try_from(earlier.len()).map_err(|_| {
            InvalidValue::new(
                COPY_RANK_TYPE_NAME,
                InvalidValueReason::OutOfRange,
                earlier.len().to_string(),
            )
        });
        if rank.is_ok() {
            earlier.push(index);
        }
        outcomes[index] = Some(rank);
    }
    let mut facts = Vec::new();
    let mut overflow = Vec::new();
    for (candidate, outcome) in candidates.into_iter().zip(outcomes) {
        match outcome {
            Some(Ok(rank)) => facts.push(AsIsFact {
                copy_rank: rank,
                ..candidate.fact
            }),
            Some(Err(reason)) => overflow.push((candidate.row, reason)),
            None => {}
        }
    }
    (facts, overflow)
}

/// Returns one [`Finding::UndeclaredFamily`] per family of the facts that no plug-in declares,
/// sorted by family. The extension family, which is the filing ID, gives no finding.
pub fn undeclared_families(
    filing: &FilingId,
    registry: &Registry<'_>,
    facts: &[AsIsFact],
) -> Vec<Finding> {
    let extension = filing.to_string();
    let families: BTreeSet<&NamespaceFamily> = facts
        .iter()
        .map(|fact| fact.concept.family())
        .filter(|family| family.as_str() != extension && registry.declaration(family).is_none())
        .collect();
    families
        .into_iter()
        .map(|family| Finding::UndeclaredFamily {
            filing: filing.clone(),
            family: family.clone(),
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use std::str::FromStr;

    use bigdecimal::BigDecimal;
    use chrono::NaiveDate;
    use pretty_assertions::assert_eq;

    use super::*;
    use crate::core::concept::ConceptName;
    use crate::core::fact::{Context, Dimensions, FactValue, FilingSource};
    use crate::core::filing::EntityId;
    use crate::core::period::Period;

    fn candidate(order: u32, row: usize, value: &str) -> Candidate<u32, usize> {
        let filing = FilingId::new("sec", "0000000001-26-000001")
            .expect("Given a hardcoded source and ID, the creation should always succeed");
        let date = NaiveDate::from_ymd_opt(2026, 9, 30)
            .expect("Given a hardcoded valid date, the creation should always succeed");
        let family = NamespaceFamily::new("us-gaap").expect(
            "Given a hardcoded family without whitespace, the creation should always succeed",
        );
        Candidate {
            order,
            row,
            fact: AsIsFact {
                source: FilingSource {
                    filing,
                    filed: date,
                    accepted: None,
                },
                context: Context {
                    entity: EntityId::new("http://www.sec.gov/CIK", "0000000001").expect(
                        "Given a hardcoded scheme and ID, the creation should always succeed",
                    ),
                    period: Period::Instant { end: date },
                    dimensions: Dimensions::default(),
                },
                concept: ConceptName::new(family, "Assets").expect(
                    "Given a hardcoded local name without a colon, the creation should always succeed",
                ),
                taxonomy_version: None,
                unit: None,
                value: FactValue::Numeric(
                    BigDecimal::from_str(value)
                        .expect("Given a hardcoded decimal text, the parse should always succeed"),
                ),
                decimals: None,
                copy_rank: 0,
                footnotes: vec![format!("row {row}")],
            },
        }
    }

    #[test]
    fn should_rank_in_candidate_order_when_two_copies_tie_in_copy_order() {
        let candidates = vec![candidate(0, 0, "100"), candidate(0, 1, "200")];

        let expected_result = vec![0, 1];

        let (facts, _) = number_copies(candidates);
        let result: Vec<u16> = facts.iter().map(|fact| fact.copy_rank).collect();

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_keep_the_earlier_copy_in_copy_order_when_two_copies_repeat_a_fact() {
        let candidates = vec![
            candidate(1, 0, "100"),
            candidate(0, 1, "100"),
            candidate(2, 2, "200"),
        ];

        let expected_result = vec![
            ("100".to_owned(), 0, vec!["row 1".to_owned()]),
            ("200".to_owned(), 1, vec!["row 2".to_owned()]),
        ];

        let (facts, _) = number_copies(candidates);
        let result: Vec<(String, u16, Vec<String>)> = facts
            .into_iter()
            .map(|fact| match fact.value {
                FactValue::Numeric(number) => (number.to_string(), fact.copy_rank, fact.footnotes),
                _ => (String::new(), fact.copy_rank, fact.footnotes),
            })
            .collect();

        assert_eq!(result, expected_result);
    }
}
