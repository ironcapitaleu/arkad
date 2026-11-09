//! # Alias
//!
//! Provides [`resolve_aliases`], tiers 1 and 2 of resolution: the alias lists of the plug-ins
//! turn the as-is facts of one filing into candidates.

use std::collections::{BTreeMap, BTreeSet};

use bigdecimal::BigDecimal;
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
/// If no concept of the list resolves, the first part of [`ConceptAliases::parts`] that the filing
/// reports wins, with the part rule and confidence [`Confidence::Synonym`]. A part wins only if the
/// filing reports none of the other parts with a value other than zero, or with a conflict, in the
/// period. The check covers every dimension of the period, so an other part with a member, such as
/// the noncontrolling interest on the equity components axis, also stops the part. An other part
/// of zero stops nothing, because then the part equals the whole.
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
    let present = present_in_any_dimension(&periods);
    let mut candidates = Vec::new();
    let mut taken = BTreeSet::new();
    for plugin in plugins(copies, registry) {
        for aliases in plugin.aliases() {
            for (period, concepts) in &periods {
                if taken.contains(&(aliases.element, period)) {
                    continue;
                }
                let other_part_reported =
                    |other: &ConceptName| present.contains(&(period.span(), other));
                if let Some(candidate) =
                    pick(plugin, aliases, concepts, &other_part_reported, registry)
                {
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

/// The fields of a [`PeriodKey`] without the dimensions.
type PeriodSpan = (EntityId, NaiveDate, u16, Unit);

impl PeriodKey {
    fn span(&self) -> PeriodSpan {
        (
            self.entity.clone(),
            self.period_end,
            self.quarters,
            self.unit.clone(),
        )
    }

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

/// Returns the period without dimensions and the concept of each fact with a value other than
/// zero, and of each conflict.
fn present_in_any_dimension<'a>(
    periods: &BTreeMap<PeriodKey, BTreeMap<&'a ConceptName, Reported<'_>>>,
) -> BTreeSet<(PeriodSpan, &'a ConceptName)> {
    let zero = BigDecimal::from(0);
    periods
        .iter()
        .flat_map(|(period, concepts)| {
            concepts
                .iter()
                .filter(|(_, reported)| match reported {
                    Reported::Single(fact) => {
                        !matches!(&fact.value, FactValue::Numeric(value) if *value == zero)
                    }
                    Reported::Conflict(_) => true,
                })
                .map(|(concept, _)| (period.span(), *concept))
        })
        .collect()
}

fn pick(
    plugin: &dyn Taxonomy,
    aliases: &ConceptAliases,
    concepts: &BTreeMap<&ConceptName, Reported<'_>>,
    other_part_reported: &dyn Fn(&ConceptName) -> bool,
    registry: &Registry<'_>,
) -> Option<Candidate> {
    let (confidence, reported, rule) =
        first_reported(&aliases.concepts, plugin, concepts, registry)
            .map(|(position, reported)| {
                let confidence = if position == 0 {
                    Confidence::Exact
                } else {
                    Confidence::Synonym
                };
                (confidence, reported, &aliases.rule)
            })
            .or_else(|| {
                let parts = aliases.parts.as_ref()?;
                if parts.other_parts.iter().any(other_part_reported) {
                    return None;
                }
                let (_, reported) = first_reported(&parts.parts, plugin, concepts, registry)?;
                Some((Confidence::Synonym, reported, &parts.rule))
            })?;
    let fact = reported.first();
    let (value, inputs) = match reported {
        Reported::Single(fact) => {
            let FactValue::Numeric(value) = &fact.value else {
                return None;
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
        rule: rule.clone(),
        release: plugin.release().clone(),
        inputs,
    })
}

/// Returns the position and the facts of the first concept of `list` that the filing reports.
fn first_reported<'a>(
    list: &[ConceptName],
    plugin: &dyn Taxonomy,
    concepts: &BTreeMap<&ConceptName, Reported<'a>>,
    registry: &Registry<'_>,
) -> Option<(usize, Reported<'a>)> {
    list.iter()
        .enumerate()
        .filter(|(_, concept)| routes_to(concept, plugin, registry))
        .find_map(|(position, concept)| Some((position, *concepts.get(concept)?)))
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
                .find(|aliases| {
                    aliases.concepts.contains(&first.concept)
                        || aliases
                            .parts
                            .as_ref()
                            .is_some_and(|parts| parts.parts.contains(&first.concept))
                })
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
    use std::str::FromStr;

    use bigdecimal::BigDecimal;
    use pretty_assertions::assert_eq;

    use super::*;
    use crate::core::concept::NamespaceFamily;
    use crate::core::dedup::{more_precise, newest_filing};
    use crate::core::elements::CanonicalElement;
    use crate::core::fact::Decimals;
    use crate::core::release::TaxonomyReleaseId;
    use crate::core::rule::RuleId;
    use crate::core::taxonomy::reference::PartAliases;
    use crate::tests::fixtures::sample_fact::{date, fact, source, us_gaap_plugin};
    use crate::tests::fixtures::sample_taxonomy::{FakeTaxonomy, declaration, plugin_with_aliases};

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
            parts: None,
        }
    }

    fn rule_of(text: &str) -> RuleId {
        RuleId::from_str(text).expect(
            "Given a hardcoded rule ID in the documented form, the parse should always succeed",
        )
    }

    /// Returns a plug-in whose equity list holds the total, with `StockholdersEquity` and then
    /// `ParentEquity` as parts next to `MinorityInterest`.
    fn equity_plugin() -> FakeTaxonomy {
        let equity = ConceptAliases {
            element: CanonicalElement::Equity,
            rule: rule_of("us-gaap/alias/equity@1"),
            concepts: vec![concept_of(
                "us-gaap",
                "StockholdersEquityIncludingPortionAttributableToNoncontrollingInterest",
            )],
            parts: Some(PartAliases {
                rule: rule_of("us-gaap/part/equity@1"),
                parts: vec![
                    concept_of("us-gaap", "StockholdersEquity"),
                    concept_of("us-gaap", "ParentEquity"),
                ],
                other_parts: vec![concept_of("us-gaap", "MinorityInterest")],
            }),
        };
        plugin_with_aliases(
            "us-gaap/2026",
            vec![declaration(
                "us-gaap",
                "http://fasb.org/us-gaap/{release}",
                true,
            )],
            vec![equity],
        )
    }

    /// Returns the rule and the value of each candidate.
    fn rules_and_values(candidates: &FilingCandidates) -> Vec<(RuleId, CandidateValue)> {
        candidates
            .candidates
            .iter()
            .map(|candidate| (candidate.rule.clone(), candidate.value.clone()))
            .collect()
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

    #[test]
    fn should_give_a_synonym_candidate_with_the_part_rule_when_the_filing_reports_a_part_alone() {
        let plugin = equity_plugin();
        let registry = Registry::new(vec![&plugin]).expect(
            "Given one plug-in with one family, the registry creation should always succeed",
        );
        let filing = source("0000320193-24-000123", date(2024, 11, 1));
        let facts = vec![fact(&filing, "StockholdersEquity", "56950000000")];
        let copies = more_precise(&facts);

        let expected_result = vec![(
            rule_of("us-gaap/part/equity@1"),
            CandidateValue::Reported {
                value: BigDecimal::from(56_950_000_000_i64),
                decimals: Some(Decimals::Finite(-6)),
                confidence: Confidence::Synonym,
            },
        )];

        let result = rules_and_values(&resolve_aliases(&copies, &registry));

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_give_no_candidate_when_the_filing_reports_a_part_next_to_another_part() {
        let plugin = equity_plugin();
        let registry = Registry::new(vec![&plugin]).expect(
            "Given one plug-in with one family, the registry creation should always succeed",
        );
        let filing = source("0000320193-24-000123", date(2024, 11, 1));
        let facts = vec![
            fact(&filing, "StockholdersEquity", "56950000000"),
            fact(&filing, "MinorityInterest", "1000000000"),
        ];
        let copies = more_precise(&facts);

        let expected_result: Vec<(RuleId, CandidateValue)> = Vec::new();

        let result = rules_and_values(&resolve_aliases(&copies, &registry));

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_give_no_candidate_when_another_part_has_a_conflict() {
        let plugin = equity_plugin();
        let registry = Registry::new(vec![&plugin]).expect(
            "Given one plug-in with one family, the registry creation should always succeed",
        );
        let filing = source("0000320193-24-000123", date(2024, 11, 1));
        let minority = fact(&filing, "MinorityInterest", "1000000000");
        let mut other_minority = fact(&filing, "MinorityInterest", "2000000000");
        other_minority.copy_rank = 1;
        let facts = vec![
            fact(&filing, "StockholdersEquity", "56950000000"),
            minority,
            other_minority,
        ];
        let copies = more_precise(&facts);

        let expected_result: Vec<(RuleId, CandidateValue)> = Vec::new();

        let result = rules_and_values(&resolve_aliases(&copies, &registry));

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_take_the_list_concept_over_the_part_when_the_filing_reports_both() {
        let plugin = equity_plugin();
        let registry = Registry::new(vec![&plugin]).expect(
            "Given one plug-in with one family, the registry creation should always succeed",
        );
        let filing = source("0000320193-24-000123", date(2024, 11, 1));
        let facts = vec![
            fact(&filing, "StockholdersEquity", "56950000000"),
            fact(
                &filing,
                "StockholdersEquityIncludingPortionAttributableToNoncontrollingInterest",
                "57950000000",
            ),
        ];
        let copies = more_precise(&facts);

        let expected_result = vec![(
            rule_of("us-gaap/alias/equity@1"),
            CandidateValue::Reported {
                value: BigDecimal::from(57_950_000_000_i64),
                decimals: Some(Decimals::Finite(-6)),
                confidence: Confidence::Exact,
            },
        )];

        let result = rules_and_values(&resolve_aliases(&copies, &registry));

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_give_a_blocked_candidate_with_the_part_rule_when_the_winning_part_has_a_conflict() {
        let plugin = equity_plugin();
        let registry = Registry::new(vec![&plugin]).expect(
            "Given one plug-in with one family, the registry creation should always succeed",
        );
        let filing = source("0000320193-24-000123", date(2024, 11, 1));
        let mut other_equity = fact(&filing, "StockholdersEquity", "50000000000");
        other_equity.copy_rank = 1;
        let facts = vec![
            fact(&filing, "StockholdersEquity", "56950000000"),
            other_equity,
        ];
        let copies = more_precise(&facts);

        let expected_result = vec![(rule_of("us-gaap/part/equity@1"), CandidateValue::Blocked)];

        let result = rules_and_values(&resolve_aliases(&copies, &registry));

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_name_the_standard_concept_in_the_conflict_when_the_concept_is_a_part() {
        let plugin = equity_plugin();
        let registry = Registry::new(vec![&plugin]).expect(
            "Given one plug-in with one family, the registry creation should always succeed",
        );
        let filing = source("0000320193-24-000123", date(2024, 11, 1));
        let equity = fact(&filing, "StockholdersEquity", "56950000000");
        let mut other_equity = fact(&filing, "StockholdersEquity", "50000000000");
        other_equity.copy_rank = 1;
        let facts = vec![equity.clone(), other_equity];
        let copies = more_precise(&facts);

        let expected_result = vec![Finding::Conflict {
            entity: equity.context.entity.clone(),
            key: equity.key(),
            ranks: vec![1],
            standard_concept: Some(CanonicalElement::Equity),
        }];

        let result = resolve_aliases(&copies, &registry).findings;

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_give_no_candidate_when_another_part_has_a_member_of_an_axis() {
        let plugin = equity_plugin();
        let registry = Registry::new(vec![&plugin]).expect(
            "Given one plug-in with one family, the registry creation should always succeed",
        );
        let filing = source("0000320193-24-000123", date(2024, 11, 1));
        let mut minority = fact(&filing, "MinorityInterest", "1000000000");
        minority.context.dimensions = Dimensions::new([(
            "StatementEquityComponentsAxis",
            "NoncontrollingInterestMember",
        )])
        .expect("Given one hardcoded pair, the creation should always succeed");
        let facts = vec![fact(&filing, "StockholdersEquity", "56950000000"), minority];
        let copies = more_precise(&facts);

        let expected_result: Vec<(RuleId, CandidateValue)> = Vec::new();

        let result = rules_and_values(&resolve_aliases(&copies, &registry));

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_give_a_part_candidate_when_another_part_is_zero() {
        let plugin = equity_plugin();
        let registry = Registry::new(vec![&plugin]).expect(
            "Given one plug-in with one family, the registry creation should always succeed",
        );
        let filing = source("0000320193-24-000123", date(2024, 11, 1));
        let facts = vec![
            fact(&filing, "StockholdersEquity", "56950000000"),
            fact(&filing, "MinorityInterest", "0"),
        ];
        let copies = more_precise(&facts);

        let expected_result = vec![(
            rule_of("us-gaap/part/equity@1"),
            CandidateValue::Reported {
                value: BigDecimal::from(56_950_000_000_i64),
                decimals: Some(Decimals::Finite(-6)),
                confidence: Confidence::Synonym,
            },
        )];

        let result = rules_and_values(&resolve_aliases(&copies, &registry));

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_take_the_second_part_when_the_filing_reports_only_the_second_part() {
        let plugin = equity_plugin();
        let registry = Registry::new(vec![&plugin]).expect(
            "Given one plug-in with one family, the registry creation should always succeed",
        );
        let filing = source("0000320193-24-000123", date(2024, 11, 1));
        let facts = vec![fact(&filing, "ParentEquity", "56950000000")];
        let copies = more_precise(&facts);

        let expected_result = vec![(
            rule_of("us-gaap/part/equity@1"),
            CandidateValue::Reported {
                value: BigDecimal::from(56_950_000_000_i64),
                decimals: Some(Decimals::Finite(-6)),
                confidence: Confidence::Synonym,
            },
        )];

        let result = rules_and_values(&resolve_aliases(&copies, &registry));

        assert_eq!(result, expected_result);
    }
}
