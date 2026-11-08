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
