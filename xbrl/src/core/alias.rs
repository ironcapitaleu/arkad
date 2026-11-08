//! # Alias
//!
//! Provides [`resolve_aliases`], tiers 1 and 2 of resolution: the alias lists of the plug-ins
//! turn the as-is facts of one filing into candidates.

use std::collections::{BTreeMap, BTreeSet};

use chrono::NaiveDate;

use super::candidate::{Candidate, CandidateValue};
use super::concept::ConceptName;
use super::confidence::Confidence;
use super::dedup::{Copies, facts_agree};
use super::fact::{AsIsFact, Dimensions, FactValue};
use super::filing::EntityId;
use super::finding::Finding;
use super::taxonomy::Taxonomy;
use super::taxonomy::reference::ConceptAliases;
use super::taxonomy::registry::Registry;
use super::unit::Unit;

/// The candidates and findings of one filing after tiers 1 and 2.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FilingCandidates {
    /// At most one candidate per standard concept and period.
    pub candidates: Vec<Candidate>,
    /// One [`Finding::Conflict`] per value key whose copy ranks disagree.
    pub findings: Vec<Finding>,
}

/// Applies tiers 1 and 2 to the as-is facts of one filing after `core/dedup/more-precise@1`.
///
/// For each period and each alias list, the first concept of the list that the filing reports
/// wins. The first concept gives confidence [`Confidence::Exact`], a later one
/// [`Confidence::Synonym`]. A fact counts only if the registry routes its family to the plug-in of
/// the list, and only if it has a unit and a numeric value.
///
/// If the winning concept has a conflict, the filing gives a blocked candidate. A conflict on a
/// lower alias, or on a concept no plug-in maps, blocks nothing. Every conflict gives a
/// [`Finding::Conflict`], with the standard concept if an alias list of its plug-in names the
/// concept. If two plug-ins map one standard concept, the plug-in of the first family in name
/// order wins.
#[must_use]
pub fn resolve_aliases(copies: &Copies<'_>, registry: &Registry<'_>) -> FilingCandidates {
    let mut periods: BTreeMap<PeriodKey, BTreeMap<&ConceptName, Reported<'_>>> = BTreeMap::new();
    for fact in &copies.facts {
        if let (Some(key), FactValue::Numeric(_)) = (PeriodKey::of(fact), &fact.value) {
            let concepts = periods.entry(key).or_default();
            concepts.insert(&fact.concept, Reported::Single(fact));
        }
    }
    for ranks in &copies.conflicts {
        if let Some(key) = PeriodKey::of(ranks[0]) {
            let concepts = periods.entry(key).or_default();
            concepts.insert(&ranks[0].concept, Reported::Conflict(ranks));
        }
    }
    let mut candidates = Vec::new();
    let mut taken = BTreeSet::new();
    for plugin in plugins(copies, registry) {
        for aliases in plugin.aliases() {
            for (period, concepts) in &periods {
                if taken.contains(&(aliases.element, period)) {
                    continue;
                }
                if let Some(candidate) = pick(plugin, aliases, concepts, registry) {
                    taken.insert((aliases.element, period));
                    candidates.push(candidate);
                }
            }
        }
    }
    let findings = copies
        .conflicts
        .iter()
        .map(|ranks| conflict(ranks, registry))
        .collect();
    FilingCandidates {
        candidates,
        findings,
    }
}

/// The facts of one concept in one period: one agreeing fact, or the ranks of a conflict.
#[derive(Debug, Clone, Copy)]
enum Reported<'a> {
    Single(&'a AsIsFact),
    Conflict(&'a [&'a AsIsFact]),
}

impl<'a> Reported<'a> {
    const fn first(self) -> &'a AsIsFact {
        match self {
            Self::Single(fact) => fact,
            Self::Conflict(ranks) => ranks[0],
        }
    }
}

/// The fields of a value key that a candidate shares with its as-is facts, without the concept.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
struct PeriodKey {
    entity: EntityId,
    period_end: NaiveDate,
    quarters: u16,
    unit: Unit,
    dimensions: Dimensions,
}

impl PeriodKey {
    fn of(fact: &AsIsFact) -> Option<Self> {
        Some(Self {
            entity: fact.context.entity.clone(),
            period_end: fact.context.period.end(),
            quarters: fact.context.period.quarters(),
            unit: fact.unit.clone()?,
            dimensions: fact.context.dimensions.clone(),
        })
    }
}

fn plugins<'a>(copies: &Copies<'_>, registry: &Registry<'a>) -> Vec<&'a dyn Taxonomy> {
    let families: BTreeMap<_, _> = copies
        .facts
        .iter()
        .copied()
        .chain(copies.conflicts.iter().map(|ranks| ranks[0]))
        .filter_map(|fact| {
            let family = fact.concept.family();
            registry.resolver(family).map(|plugin| (family, plugin))
        })
        .collect();
    let mut plugins: Vec<&'a dyn Taxonomy> = Vec::new();
    for plugin in families.into_values() {
        if !plugins
            .iter()
            .any(|known| std::ptr::addr_eq(*known, plugin))
        {
            plugins.push(plugin);
        }
    }
    plugins
}

fn pick(
    plugin: &dyn Taxonomy,
    aliases: &ConceptAliases,
    concepts: &BTreeMap<&ConceptName, Reported<'_>>,
    registry: &Registry<'_>,
) -> Option<Candidate> {
    let (position, reported) = aliases
        .concepts
        .iter()
        .enumerate()
        .filter(|(_, concept)| routes_to(concept, plugin, registry))
        .find_map(|(position, concept)| Some((position, *concepts.get(concept)?)))?;
    let fact = reported.first();
    let (value, inputs) = match reported {
        Reported::Single(fact) => {
            let FactValue::Numeric(value) = &fact.value else {
                return None;
            };
            let confidence = if position == 0 {
                Confidence::Exact
            } else {
                Confidence::Synonym
            };
            let value = CandidateValue::Reported {
                value: value.clone(),
                decimals: fact.decimals,
                confidence,
            };
            (value, vec![fact.key()])
        }
        Reported::Conflict(ranks) => (
            CandidateValue::Blocked,
            ranks.iter().map(|rank| rank.key()).collect(),
        ),
    };
    Some(Candidate {
        source: fact.source.clone(),
        entity: fact.context.entity.clone(),
        concept: aliases.element,
        period: fact.context.period,
        unit: fact.unit.clone()?,
        dimensions: fact.context.dimensions.clone(),
        value,
        rule: aliases.rule.clone(),
        release: plugin.release().clone(),
        inputs,
    })
}

fn routes_to(concept: &ConceptName, plugin: &dyn Taxonomy, registry: &Registry<'_>) -> bool {
    registry
        .resolver(concept.family())
        .is_some_and(|resolver| std::ptr::addr_eq(resolver, plugin))
}

fn conflict(ranks: &[&AsIsFact], registry: &Registry<'_>) -> Finding {
    let first = ranks[0];
    let disagreeing = ranks[1..]
        .iter()
        .filter(|rank| !facts_agree(first, rank))
        .map(|rank| rank.copy_rank)
        .collect();
    let standard_concept = registry
        .resolver(first.concept.family())
        .and_then(|plugin| {
            plugin
                .aliases()
                .iter()
                .find(|aliases| aliases.concepts.contains(&first.concept))
                .map(|aliases| aliases.element)
        });
    Finding::Conflict {
        entity: first.context.entity.clone(),
        key: first.key(),
        ranks: disagreeing,
        standard_concept,
    }
}

#[cfg(test)]
mod tests {
    use bigdecimal::BigDecimal;
    use pretty_assertions::assert_eq;

    use super::*;
    use crate::core::dedup::{more_precise, newest_filing};
    use crate::core::elements::CanonicalElement;
    use crate::core::fact::Decimals;
    use std::str::FromStr;

    use crate::core::concept::NamespaceFamily;
    use crate::core::release::TaxonomyReleaseId;
    use crate::core::rule::RuleId;
    use crate::core::taxonomy::reference::ConceptAliases;
    use crate::tests::fixtures::sample_fact::{date, fact, source, us_gaap_plugin};
    use crate::tests::fixtures::sample_taxonomy::{declaration, plugin_with_aliases};

    fn concept_of(family: &str, local_name: &str) -> ConceptName {
        let family = NamespaceFamily::new(family).expect(
            "Given a hardcoded family without whitespace, the creation should always succeed",
        );
        ConceptName::new(family, local_name).expect(
            "Given a hardcoded local name without a colon, the creation should always succeed",
        )
    }

    fn revenue_list(concepts: Vec<ConceptName>) -> ConceptAliases {
        ConceptAliases {
            element: CanonicalElement::Revenue,
            rule: RuleId::from_str("aaa/alias/revenue@1").expect(
                "Given a hardcoded rule ID in the documented form, the parse should always succeed",
            ),
            concepts,
        }
    }

    fn values(candidates: &FilingCandidates) -> Vec<CandidateValue> {
        candidates
            .candidates
            .iter()
            .map(|candidate| candidate.value.clone())
            .collect()
    }

    const fn implements_auto_traits<T: Sized + Send + Sync + Unpin>() {}
    #[test]
    const fn should_implement_auto_traits_for_filing_candidates() {
        implements_auto_traits::<FilingCandidates>();
    }

    #[test]
    fn should_give_an_exact_candidate_when_the_filing_reports_the_first_alias() {
        let plugin = us_gaap_plugin();
        let registry = Registry::new(vec![&plugin]).expect(
            "Given one plug-in with one family, the registry creation should always succeed",
        );
        let filing = source("0000320193-24-000123", date(2024, 11, 1));
        let facts = vec![
            fact(&filing, "SalesRevenueNet", "390000000000"),
            fact(&filing, "Revenues", "391035000000"),
        ];
        let copies = more_precise(&facts);

        let expected_result = vec![CandidateValue::Reported {
            value: BigDecimal::from(391_035_000_000_i64),
            decimals: Some(Decimals::Finite(-6)),
            confidence: Confidence::Exact,
        }];

        let result = values(&resolve_aliases(&copies, &registry));

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_give_a_synonym_candidate_when_the_filing_reports_a_later_alias_only() {
        let plugin = us_gaap_plugin();
        let registry = Registry::new(vec![&plugin]).expect(
            "Given one plug-in with one family, the registry creation should always succeed",
        );
        let filing = source("0000320193-24-000123", date(2024, 11, 1));
        let facts = vec![fact(&filing, "SalesRevenueNet", "390000000000")];
        let copies = more_precise(&facts);

        let expected_result = vec![CandidateValue::Reported {
            value: BigDecimal::from(390_000_000_000_i64),
            decimals: Some(Decimals::Finite(-6)),
            confidence: Confidence::Synonym,
        }];

        let result = values(&resolve_aliases(&copies, &registry));

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_give_a_blocked_candidate_when_the_winning_alias_has_a_conflict() {
        let plugin = us_gaap_plugin();
        let registry = Registry::new(vec![&plugin]).expect(
            "Given one plug-in with one family, the registry creation should always succeed",
        );
        let filing = source("0000320193-24-000123", date(2024, 11, 1));
        let mut stub = fact(&filing, "Revenues", "61000000000");
        stub.copy_rank = 1;
        let facts = vec![
            fact(&filing, "Revenues", "94930000000"),
            stub,
            fact(&filing, "SalesRevenueNet", "94930000000"),
        ];
        let copies = more_precise(&facts);

        let expected_result = vec![CandidateValue::Blocked];

        let result = values(&resolve_aliases(&copies, &registry));

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_block_nothing_when_the_conflict_is_on_a_lower_alias() {
        let plugin = us_gaap_plugin();
        let registry = Registry::new(vec![&plugin]).expect(
            "Given one plug-in with one family, the registry creation should always succeed",
        );
        let filing = source("0000320193-24-000123", date(2024, 11, 1));
        let mut stub = fact(&filing, "SalesRevenueNet", "61000000000");
        stub.copy_rank = 1;
        let facts = vec![
            fact(&filing, "Revenues", "94930000000"),
            fact(&filing, "SalesRevenueNet", "94930000000"),
            stub,
        ];
        let copies = more_precise(&facts);

        let expected_result = vec![CandidateValue::Reported {
            value: BigDecimal::from(94_930_000_000_i64),
            decimals: Some(Decimals::Finite(-6)),
            confidence: Confidence::Exact,
        }];

        let result = values(&resolve_aliases(&copies, &registry));

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_name_the_standard_concept_in_the_conflict_when_a_plugin_maps_the_concept() {
        let plugin = us_gaap_plugin();
        let registry = Registry::new(vec![&plugin]).expect(
            "Given one plug-in with one family, the registry creation should always succeed",
        );
        let filing = source("0000320193-24-000123", date(2024, 11, 1));
        let quarter = fact(&filing, "SalesRevenueNet", "94930000000");
        let mut stub = fact(&filing, "SalesRevenueNet", "61000000000");
        stub.copy_rank = 1;
        let facts = vec![quarter.clone(), stub];
        let copies = more_precise(&facts);

        let expected_result = vec![Finding::Conflict {
            entity: quarter.context.entity.clone(),
            key: quarter.key(),
            ranks: vec![1],
            standard_concept: Some(CanonicalElement::Revenue),
        }];

        let result = resolve_aliases(&copies, &registry).findings;

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_name_no_standard_concept_in_the_conflict_when_no_plugin_maps_the_concept() {
        let plugin = us_gaap_plugin();
        let registry = Registry::new(vec![&plugin]).expect(
            "Given one plug-in with one family, the registry creation should always succeed",
        );
        let filing = source("0000320193-24-000123", date(2024, 11, 1));
        let first = fact(&filing, "OtherIncome", "5000000");
        let mut second = fact(&filing, "OtherIncome", "7000000");
        second.copy_rank = 1;
        let facts = vec![first.clone(), second];
        let copies = more_precise(&facts);

        let expected_result = vec![Finding::Conflict {
            entity: first.context.entity.clone(),
            key: first.key(),
            ranks: vec![1],
            standard_concept: None,
        }];

        let result = resolve_aliases(&copies, &registry).findings;

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_name_only_the_disagreeing_ranks_in_the_conflict_when_one_rank_agrees() {
        let plugin = us_gaap_plugin();
        let registry = Registry::new(vec![&plugin]).expect(
            "Given one plug-in with one family, the registry creation should always succeed",
        );
        let filing = source("0000320193-24-000123", date(2024, 11, 1));
        let quarter = fact(&filing, "Revenues", "94930000000");
        let mut copy = fact(&filing, "Revenues", "94930000000");
        copy.copy_rank = 1;
        let mut stub = fact(&filing, "Revenues", "61000000000");
        stub.copy_rank = 2;
        let facts = vec![quarter.clone(), copy, stub];
        let copies = more_precise(&facts);

        let expected_result = vec![Finding::Conflict {
            entity: quarter.context.entity.clone(),
            key: quarter.key(),
            ranks: vec![2],
            standard_concept: Some(CanonicalElement::Revenue),
        }];

        let result = resolve_aliases(&copies, &registry).findings;

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_take_the_restatement_when_a_newer_filing_restates_under_a_renamed_concept() {
        let plugin = us_gaap_plugin();
        let registry = Registry::new(vec![&plugin]).expect(
            "Given one plug-in with one family, the registry creation should always succeed",
        );
        let older = source("0000320193-24-000123", date(2024, 11, 1));
        let newer = source("0000320193-25-000010", date(2025, 10, 31));
        let older_facts = vec![fact(&older, "Revenues", "391000000000")];
        let newer_facts = vec![fact(
            &newer,
            "RevenueFromContractWithCustomerExcludingAssessedTax",
            "395000000000",
        )];
        let mut candidates = resolve_aliases(&more_precise(&older_facts), &registry).candidates;
        candidates.extend(resolve_aliases(&more_precise(&newer_facts), &registry).candidates);

        let expected_result = vec![(
            BigDecimal::from(395_000_000_000_i64),
            Confidence::Synonym,
            true,
        )];

        let result: Vec<(BigDecimal, Confidence, bool)> = newest_filing(&candidates)
            .facts
            .into_iter()
            .map(|fact| (fact.value, fact.confidence, fact.restated))
            .collect();

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_take_the_plugin_of_the_first_family_in_name_order_when_two_plugins_map_a_concept() {
        let aaa = plugin_with_aliases(
            "aaa/2026",
            vec![declaration("aaa", "http://aaa.example/{release}", true)],
            vec![revenue_list(vec![concept_of("aaa", "Revenues")])],
        );
        let us_gaap = us_gaap_plugin();
        let registry = Registry::new(vec![&us_gaap, &aaa])
            .expect("Given two plug-ins with distinct families, the registry creation should always succeed");
        let filing = source("0000320193-24-000123", date(2024, 11, 1));
        let mut aaa_fact = fact(&filing, "Revenues", "100000000");
        aaa_fact.concept = concept_of("aaa", "Revenues");
        let facts = vec![fact(&filing, "Revenues", "200000000"), aaa_fact];
        let copies = more_precise(&facts);

        let expected_result = vec![TaxonomyReleaseId::new("aaa/2026").expect(
            "Given a hardcoded release ID with a year, the creation should always succeed",
        )];

        let result: Vec<TaxonomyReleaseId> = resolve_aliases(&copies, &registry)
            .candidates
            .into_iter()
            .map(|candidate| candidate.release.id)
            .collect();

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_give_no_candidate_when_the_fact_has_no_unit() {
        let plugin = us_gaap_plugin();
        let registry = Registry::new(vec![&plugin]).expect(
            "Given one plug-in with one family, the registry creation should always succeed",
        );
        let filing = source("0000320193-24-000123", date(2024, 11, 1));
        let mut no_unit = fact(&filing, "Revenues", "391035000000");
        no_unit.unit = None;
        let facts = vec![no_unit];
        let copies = more_precise(&facts);

        let expected_result: Vec<CandidateValue> = Vec::new();

        let result = values(&resolve_aliases(&copies, &registry));

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_take_the_next_alias_when_the_first_alias_is_nil() {
        let plugin = us_gaap_plugin();
        let registry = Registry::new(vec![&plugin]).expect(
            "Given one plug-in with one family, the registry creation should always succeed",
        );
        let filing = source("0000320193-24-000123", date(2024, 11, 1));
        let mut nil = fact(&filing, "Revenues", "0");
        nil.value = FactValue::Nil;
        let facts = vec![nil, fact(&filing, "SalesRevenueNet", "390000000000")];
        let copies = more_precise(&facts);

        let expected_result = vec![CandidateValue::Reported {
            value: BigDecimal::from(390_000_000_000_i64),
            decimals: Some(Decimals::Finite(-6)),
            confidence: Confidence::Synonym,
        }];

        let result = values(&resolve_aliases(&copies, &registry));

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_give_no_candidate_when_the_alias_family_routes_to_another_plugin() {
        let srt = plugin_with_aliases(
            "srt/2026",
            vec![declaration("srt", "http://fasb.org/srt/{release}", true)],
            Vec::new(),
        );
        let us_gaap = plugin_with_aliases(
            "us-gaap/2026",
            vec![declaration(
                "us-gaap",
                "http://fasb.org/us-gaap/{release}",
                true,
            )],
            vec![revenue_list(vec![concept_of("srt", "Revenues")])],
        );
        let registry = Registry::new(vec![&us_gaap, &srt])
            .expect("Given two plug-ins with distinct families, the registry creation should always succeed");
        let filing = source("0000320193-24-000123", date(2024, 11, 1));
        let mut srt_fact = fact(&filing, "Revenues", "391035000000");
        srt_fact.concept = concept_of("srt", "Revenues");
        let facts = vec![srt_fact];
        let copies = more_precise(&facts);

        let expected_result: Vec<CandidateValue> = Vec::new();

        let result = values(&resolve_aliases(&copies, &registry));

        assert_eq!(result, expected_result);
    }
}
