//! # Alias
//!
//! Provides [`resolve_aliases`], tiers 1 and 2 of resolution: the alias lists of the plug-ins
//! turn the as-is facts of one filing into candidates.

use std::collections::{BTreeMap, BTreeSet};

use bigdecimal::BigDecimal;
use jiff::civil::Date;

use super::candidate::{Candidate, CandidateValue};
use super::concept::ConceptName;
use super::confidence::Confidence;
use super::dedup::{Copies, conflicts_with};
use super::elements::CanonicalElementKind;
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
    /// At most one candidate per standard concept, period, and set of dimensions.
    pub candidates: Vec<Candidate>,
    /// One [`Finding::Conflict`] per value key whose copy ranks disagree.
    pub findings: Vec<Finding>,
}

/// Applies tiers 1 and 2 to the as-is facts of one filing after `core/dedup/more-precise@1`.
///
/// A fact without dimensions and a segment fact give candidates. A fact is a segment fact if each
/// of its axes is a segment axis. [`Registry::segment_axes`] returns the segment axes for the
/// family of the concept. A segment fact gives a candidate under its own dimensions, with the alias
/// lists of the value without dimensions. It gives no candidate of a
/// [`CanonicalElementKind::Disclosure`] concept. For example, a per-share value counts the shares
/// of the whole company, so it has no segment value. Any other fact with dimensions gives no
/// candidate, but it still counts in the part rule and still gives its [`Finding::Conflict`].
///
/// For each period and each alias list, the first concept of the list that the filing reports
/// wins. The first concept gives confidence [`Confidence::Exact`], a later one
/// [`Confidence::Synonym`]. A fact counts only if the registry routes its family to the plug-in of
/// the list, and only if it has a unit and a numeric value.
///
/// If no concept of the list resolves, the part rule runs on [`ConceptAliases::parts`]. Of the
/// parts that the filing reports, the first that no other part blocks wins, with the part rule and
/// confidence [`Confidence::Synonym`]. If the filing reports an other part in the period, the other
/// part blocks the part. A part never blocks itself, so two parts can each be an other part of the
/// other. A value of zero does not block. A conflict does. The check covers every dimension of the
/// period, so an other part with a member, such as the noncontrolling interest on the equity
/// components axis, also stops the part.
///
/// The candidate takes the scope that [`ConceptAliases::scopes`] names for the winning concept or
/// part.
///
/// If the winning concept has a conflict, the filing gives a blocked candidate under the dimensions
/// of the conflict. A conflict on a segment fact leaves the candidate without dimensions as it is.
/// A conflict on a lower alias, or on a concept no plug-in maps, blocks nothing. Every conflict
/// gives a [`Finding::Conflict`], with the standard concept if an alias list of its plug-in names
/// the concept. If two plug-ins map one standard concept, the plug-in of the first family in name
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
            for (period, concepts) in periods.iter().filter(|(period, _)| {
                period.dimensions.is_empty()
                    || !matches!(aliases.element, CanonicalElementKind::Disclosure(_))
            }) {
                if taken.contains(&(aliases.element, period)) {
                    continue;
                }
                let other_part_reported =
                    |other: &ConceptName| present.contains(&(period.span(), other));
                let resolves = |concept: &ConceptName| {
                    routes_to(concept, plugin, registry)
                        && names_each_axis(concept, &period.dimensions, registry)
                };
                if let Some(candidate) =
                    pick(plugin, aliases, concepts, &resolves, &other_part_reported)
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
    period_end: Date,
    quarters: u16,
    unit: Unit,
    dimensions: Dimensions,
}

/// The fields of a [`PeriodKey`] without the dimensions.
type PeriodSpan = (EntityId, Date, u16, Unit);

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

/// Picks the candidate of one alias list in one period. `resolves` accepts a concept that routes to
/// `plugin` and whose plug-in names each axis of the period.
fn pick(
    plugin: &dyn Taxonomy,
    aliases: &ConceptAliases,
    concepts: &BTreeMap<&ConceptName, Reported<'_>>,
    resolves: &dyn Fn(&ConceptName) -> bool,
    other_part_reported: &dyn Fn(&ConceptName) -> bool,
) -> Option<Candidate> {
    let (confidence, reported, rule) = first_reported(&aliases.concepts, concepts, resolves)
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
            parts
                .parts
                .iter()
                .filter(|part| resolves(part))
                .filter_map(|part| Some((part, *concepts.get(part)?)))
                .find(|(part, _)| {
                    !parts
                        .other_parts
                        .iter()
                        .filter(|other| other != part)
                        .any(other_part_reported)
                })
                .map(|(_, reported)| (Confidence::Synonym, reported, &parts.rule))
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
        scope: aliases.scope_of(&fact.concept),
    })
}

/// Returns the position and the facts of the first concept of `list` that the filing reports.
fn first_reported<'a>(
    list: &[ConceptName],
    concepts: &BTreeMap<&ConceptName, Reported<'a>>,
    resolves: &dyn Fn(&ConceptName) -> bool,
) -> Option<(usize, Reported<'a>)> {
    list.iter()
        .enumerate()
        .filter(|(_, concept)| resolves(concept))
        .find_map(|(position, concept)| Some((position, *concepts.get(concept)?)))
}

fn routes_to(concept: &ConceptName, plugin: &dyn Taxonomy, registry: &Registry<'_>) -> bool {
    registry
        .resolver(concept.family())
        .is_some_and(|resolver| std::ptr::addr_eq(resolver, plugin))
}

/// Returns `true` if `dimensions` is empty, or if the plug-in that resolves `concept` names each
/// axis of `dimensions` as a segment axis.
fn names_each_axis(
    concept: &ConceptName,
    dimensions: &Dimensions,
    registry: &Registry<'_>,
) -> bool {
    if dimensions.is_empty() {
        return true;
    }
    registry
        .segment_axes(concept.family())
        .is_some_and(|plugin| {
            dimensions.iter().all(|(axis, _)| {
                plugin
                    .segment_axes()
                    .iter()
                    .any(|segment_axis| segment_axis.name() == axis)
            })
        })
}

fn conflict(ranks: &[&AsIsFact], registry: &Registry<'_>) -> Finding {
    let first = ranks[0];
    let disagreeing = ranks[1..]
        .iter()
        .filter(|rank| conflicts_with(first, rank))
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
    use crate::core::elements::{BalanceSheetConcept, CashFlowConcept, IncomeStatementConcept};
    use crate::core::fact::{Decimals, FilingSource};
    use crate::core::release::TaxonomyReleaseId;
    use crate::core::rule::RuleId;
    use crate::core::scope::Scope;
    use crate::core::taxonomy::reference::PartAliases;
    use crate::tests::fixtures::sample_fact::{apple, date, fact, source, us_gaap_plugin};
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
            element: IncomeStatementConcept::Revenue.element(),
            rule: RuleId::from_str("aaa/alias/revenue@1").expect(
                "Given a hardcoded rule ID in the documented form, the parse should always succeed",
            ),
            concepts,
            parts: None,
            scopes: Vec::new(),
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
            element: BalanceSheetConcept::Equity.element(),
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
            scopes: Vec::new(),
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

    /// Returns a plug-in whose short-term debt list holds the total, with two parts that are each
    /// an other part of the other.
    fn short_term_debt_plugin() -> FakeTaxonomy {
        let parts = vec![
            concept_of("us-gaap", "LongTermDebtAndCapitalLeaseObligationsCurrent"),
            concept_of("us-gaap", "ShortTermBorrowings"),
        ];
        let short_term_debt = ConceptAliases {
            element: BalanceSheetConcept::ShortTermDebt.element(),
            rule: rule_of("us-gaap/alias/short-term-debt@1"),
            concepts: vec![concept_of("us-gaap", "DebtCurrent")],
            parts: Some(PartAliases {
                rule: rule_of("us-gaap/part/short-term-debt@1"),
                parts: parts.clone(),
                other_parts: parts,
            }),
            scopes: Vec::new(),
        };
        plugin_with_aliases(
            "us-gaap/2026",
            vec![declaration(
                "us-gaap",
                "http://fasb.org/us-gaap/{release}",
                true,
            )],
            vec![short_term_debt],
        )
    }

    /// Returns a plug-in whose exchange rate effect list holds the effect on cash with restricted
    /// cash, then the effect on cash without it, with the effect of continuing operations as a part.
    /// The concepts without restricted cash carry the scope `ExcludesRestrictedCash`.
    fn exchange_rate_effect_plugin() -> FakeTaxonomy {
        let narrow = concept_of("us-gaap", "EffectOfExchangeRateOnCashAndCashEquivalents");
        let continuing = concept_of(
            "us-gaap",
            "EffectOfExchangeRateOnCashAndCashEquivalentsContinuingOperations",
        );
        let exchange_rate_effect = ConceptAliases {
            element: CashFlowConcept::ExchangeRateEffect.element(),
            rule: rule_of("us-gaap/alias/exchange-rate-effect@1"),
            concepts: vec![
                concept_of(
                    "us-gaap",
                    "EffectOfExchangeRateOnCashCashEquivalentsRestrictedCashAndRestrictedCashEquivalents",
                ),
                narrow.clone(),
            ],
            parts: Some(PartAliases {
                rule: rule_of("us-gaap/part/exchange-rate-effect@1"),
                parts: vec![continuing.clone()],
                other_parts: vec![concept_of(
                    "us-gaap",
                    "EffectOfExchangeRateOnCashAndCashEquivalentsDiscontinuedOperations",
                )],
            }),
            scopes: vec![
                (narrow, Scope::ExcludesRestrictedCash),
                (continuing, Scope::ExcludesRestrictedCash),
            ],
        };
        plugin_with_aliases(
            "us-gaap/2026",
            vec![declaration(
                "us-gaap",
                "http://fasb.org/us-gaap/{release}",
                true,
            )],
            vec![exchange_rate_effect],
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

    /// Returns the dimensions of the segment member `member` on the business segments axis.
    fn segment(member: &str) -> Dimensions {
        Dimensions::new([("BusinessSegments", member)])
            .expect("Given one hardcoded pair, the creation should always succeed")
    }

    /// Returns two ranks of revenue on the services segment that disagree.
    fn segment_conflict(filing: &FilingSource) -> Vec<AsIsFact> {
        let mut quarter = fact(filing, "Revenues", "24000000000");
        quarter.context.dimensions = segment("ServicesMember");
        let mut stub = quarter.clone();
        stub.value = FactValue::Numeric(BigDecimal::from(16_000_000_000_i64));
        stub.copy_rank = 1;
        vec![quarter, stub]
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
            entity: apple(),
            key: quarter.key(),
            ranks: vec![1],
            standard_concept: Some(IncomeStatementConcept::Revenue.element()),
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
            entity: apple(),
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
            entity: apple(),
            key: quarter.key(),
            ranks: vec![2],
            standard_concept: Some(IncomeStatementConcept::Revenue.element()),
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
    fn should_give_the_scope_of_the_winning_concept_to_the_candidate_and_the_value_for_each_concept()
     {
        let plugin = exchange_rate_effect_plugin();
        let registry = Registry::new(vec![&plugin]).expect(
            "Given one plug-in with one family, the registry creation should always succeed",
        );
        let filing = source("0000320193-24-000123", date(2024, 11, 1));
        let reported = [
            "EffectOfExchangeRateOnCashCashEquivalentsRestrictedCashAndRestrictedCashEquivalents",
            "EffectOfExchangeRateOnCashAndCashEquivalents",
            "EffectOfExchangeRateOnCashAndCashEquivalentsContinuingOperations",
        ];

        let expected_result = vec![
            (rule_of("us-gaap/alias/exchange-rate-effect@1"), None, None),
            (
                rule_of("us-gaap/alias/exchange-rate-effect@1"),
                Some(Scope::ExcludesRestrictedCash),
                Some(Scope::ExcludesRestrictedCash),
            ),
            (
                rule_of("us-gaap/part/exchange-rate-effect@1"),
                Some(Scope::ExcludesRestrictedCash),
                Some(Scope::ExcludesRestrictedCash),
            ),
        ];

        let result: Vec<(RuleId, Option<Scope>, Option<Scope>)> = reported
            .into_iter()
            .map(|local_name| {
                let facts = vec![fact(&filing, local_name, "-42000000")];
                let candidates = resolve_aliases(&more_precise(&facts), &registry).candidates;
                let value = newest_filing(&candidates).facts.remove(0);
                (candidates[0].rule.clone(), candidates[0].scope, value.scope)
            })
            .collect();

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
            entity: apple(),
            key: equity.key(),
            ranks: vec![1],
            standard_concept: Some(BalanceSheetConcept::Equity.element()),
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

    #[test]
    fn should_give_the_first_part_when_the_filing_reports_it_alone_and_it_is_an_other_part_of_the_second()
     {
        let plugin = short_term_debt_plugin();
        let registry = Registry::new(vec![&plugin]).expect(
            "Given one plug-in with one family, the registry creation should always succeed",
        );
        let filing = source("0000320193-24-000123", date(2024, 11, 1));
        let facts = vec![fact(
            &filing,
            "LongTermDebtAndCapitalLeaseObligationsCurrent",
            "45000000",
        )];
        let copies = more_precise(&facts);

        let expected_result = vec![(
            rule_of("us-gaap/part/short-term-debt@1"),
            CandidateValue::Reported {
                value: BigDecimal::from(45_000_000_i64),
                decimals: Some(Decimals::Finite(-6)),
                confidence: Confidence::Synonym,
            },
        )];

        let result = rules_and_values(&resolve_aliases(&copies, &registry));

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_give_the_second_part_when_the_filing_reports_it_alone_and_it_is_an_other_part_of_the_first()
     {
        let plugin = short_term_debt_plugin();
        let registry = Registry::new(vec![&plugin]).expect(
            "Given one plug-in with one family, the registry creation should always succeed",
        );
        let filing = source("0000320193-24-000123", date(2024, 11, 1));
        let facts = vec![fact(&filing, "ShortTermBorrowings", "45000000")];
        let copies = more_precise(&facts);

        let expected_result = vec![(
            rule_of("us-gaap/part/short-term-debt@1"),
            CandidateValue::Reported {
                value: BigDecimal::from(45_000_000_i64),
                decimals: Some(Decimals::Finite(-6)),
                confidence: Confidence::Synonym,
            },
        )];

        let result = rules_and_values(&resolve_aliases(&copies, &registry));

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_give_the_second_part_when_the_first_part_is_zero_and_each_is_an_other_part_of_the_other()
     {
        let plugin = short_term_debt_plugin();
        let registry = Registry::new(vec![&plugin]).expect(
            "Given one plug-in with one family, the registry creation should always succeed",
        );
        let filing = source("0000320193-24-000123", date(2024, 11, 1));
        let facts = vec![
            fact(
                &filing,
                "LongTermDebtAndCapitalLeaseObligationsCurrent",
                "0",
            ),
            fact(&filing, "ShortTermBorrowings", "500000000"),
        ];
        let copies = more_precise(&facts);

        let expected_result = vec![(
            rule_of("us-gaap/part/short-term-debt@1"),
            CandidateValue::Reported {
                value: BigDecimal::from(500_000_000_i64),
                decimals: Some(Decimals::Finite(-6)),
                confidence: Confidence::Synonym,
            },
        )];

        let result = rules_and_values(&resolve_aliases(&copies, &registry));

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_give_no_candidate_when_the_filing_reports_two_parts_that_are_each_an_other_part_of_the_other()
     {
        let plugin = short_term_debt_plugin();
        let registry = Registry::new(vec![&plugin]).expect(
            "Given one plug-in with one family, the registry creation should always succeed",
        );
        let filing = source("0000320193-24-000123", date(2024, 11, 1));
        let facts = vec![
            fact(
                &filing,
                "LongTermDebtAndCapitalLeaseObligationsCurrent",
                "45000000",
            ),
            fact(&filing, "ShortTermBorrowings", "756000000"),
        ];
        let copies = more_precise(&facts);

        let expected_result: Vec<(RuleId, CandidateValue)> = Vec::new();

        let result = rules_and_values(&resolve_aliases(&copies, &registry));

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_give_only_the_candidate_without_dimensions_when_the_plugin_has_no_segment_axes() {
        let plugin = us_gaap_plugin();
        let registry = Registry::new(vec![&plugin]).expect(
            "Given one plug-in with one family, the registry creation should always succeed",
        );
        let filing = source("0000320193-24-000123", date(2024, 11, 1));
        let mut segment_revenue = fact(&filing, "Revenues", "100000000000");
        segment_revenue.context.dimensions = segment("ServicesMember");
        let facts = vec![fact(&filing, "Revenues", "391035000000"), segment_revenue];
        let copies = more_precise(&facts);

        let expected_result = vec![(
            Dimensions::default(),
            CandidateValue::Reported {
                value: BigDecimal::from(391_035_000_000_i64),
                decimals: Some(Decimals::Finite(-6)),
                confidence: Confidence::Exact,
            },
        )];

        let result: Vec<(Dimensions, CandidateValue)> = resolve_aliases(&copies, &registry)
            .candidates
            .into_iter()
            .map(|candidate| (candidate.dimensions, candidate.value))
            .collect();

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_give_no_blocked_candidate_when_the_plugin_of_the_conflict_has_no_segment_axes() {
        let plugin = us_gaap_plugin();
        let registry = Registry::new(vec![&plugin]).expect(
            "Given one plug-in with one family, the registry creation should always succeed",
        );
        let filing = source("0000320193-24-000123", date(2024, 11, 1));
        let facts = segment_conflict(&filing);
        let copies = more_precise(&facts);

        let expected_result: Vec<CandidateValue> = Vec::new();

        let result = values(&resolve_aliases(&copies, &registry));

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_give_a_conflict_finding_when_the_conflict_is_on_a_fact_with_dimensions() {
        let plugin = us_gaap_plugin();
        let registry = Registry::new(vec![&plugin]).expect(
            "Given one plug-in with one family, the registry creation should always succeed",
        );
        let filing = source("0000320193-24-000123", date(2024, 11, 1));
        let facts = segment_conflict(&filing);
        let quarter = &facts[0];
        let copies = more_precise(&facts);

        let expected_result = vec![Finding::Conflict {
            entity: apple(),
            key: quarter.key(),
            ranks: vec![1],
            standard_concept: Some(IncomeStatementConcept::Revenue.element()),
        }];

        let result = resolve_aliases(&copies, &registry).findings;

        assert_eq!(result, expected_result);
    }
}
