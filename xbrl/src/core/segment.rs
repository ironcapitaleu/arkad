//! # Segment
//!
//! Provides [`reconcile`], which builds the segment reconciliation of each filing from its
//! candidates, and its read types [`Reconciliation`] and [`ReconciliationLine`].
//!
//! The reconciliation shows how the business segments add up to the consolidated value. It runs
//! only on the axes of kind [`SegmentAxisKind::BusinessSegment`]. The roles come from
//! [`SegmentAxes::member_roles`] of the plug-in that resolved each candidate, so this module names no
//! axis and no member of a taxonomy. The datasets store no line of a reconciliation, and
//! [`reconcile`] gives no finding.
//!
//! `docs/adr/0030-segment-reconciliation-per-report.md` records why the reconciliation runs per
//! report.

use std::collections::BTreeMap;
use std::str::FromStr;

use bigdecimal::BigDecimal;
use jiff::civil::Date;

use super::candidate::{Candidate, CandidateValue};
use super::elements::CanonicalElementKind;
use super::fact::Dimensions;
use super::filing::{EntityId, FilingId};
use super::rule::RuleId;
use super::taxonomy::registry::Registry;
use super::taxonomy::segment::{
    MemberRole, ReconciliationRole, SegmentAxes, SegmentAxis, SegmentAxisKind,
};
use super::unit::Unit;

/// The segment reconciliation of one filing, entity, standard concept, period, and unit.
///
/// unreconciled = consolidated − Σ segment − Σ elimination − Σ corporate. Each value keeps the sign
/// of the filing.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Reconciliation {
    /// The filing.
    pub filing: FilingId,
    /// The entity the values are about.
    pub entity: EntityId,
    /// The standard concept.
    pub concept: CanonicalElementKind,
    /// The exact end of the period.
    pub period_end: Date,
    /// The length in quarters.
    pub quarters: u16,
    /// The unit.
    pub unit: Unit,
    /// The candidate without dimensions, if the filing gives one.
    pub consolidated: Option<Candidate>,
    /// The segment lines sorted by dimensions, then the elimination lines and the corporate lines
    /// in the order of the input, then the unreconciled line if it exists.
    pub lines: Vec<ReconciliationLine>,
}

/// One line of a [`Reconciliation`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ReconciliationLine {
    /// A business segment: one business-segment axis, alone or with a member of role
    /// [`ReconciliationRole::Segment`].
    Segment(Candidate),
    /// A member of role [`ReconciliationRole::Elimination`], alone.
    Elimination(Candidate),
    /// A member of role [`ReconciliationRole::Corporate`], alone.
    Corporate(Candidate),
    /// The rest that the filer did not tag with a role.
    Unreconciled(BigDecimal),
}

impl ReconciliationLine {
    /// Returns the role rule ID of the line, or `None` for a segment line.
    ///
    /// # Panics
    ///
    /// Panics if a hardcoded rule ID breaks the form of [`RuleId`], which a unit test rules out.
    #[must_use]
    pub fn role_rule(&self) -> Option<RuleId> {
        let text = match self {
            Self::Segment(_) => return None,
            Self::Elimination(_) => "core/segment/elimination@1",
            Self::Corporate(_) => "core/segment/corporate@1",
            Self::Unreconciled(_) => "core/segment/unreconciled@1",
        };
        Some(RuleId::from_str(text).expect(
            "Given a hardcoded rule ID in the documented form, the parse should always succeed",
        ))
    }
}

/// Builds one [`Reconciliation`] per filing, entity, standard concept, period, and unit from
/// `candidates`.
///
/// A period is its exact end and its length in quarters, as in the value key.
///
/// A reconciliation exists only if a candidate has a role. The role of a candidate with dimensions
/// comes from the [`SegmentAxes`] of the family of its first input. A candidate with an axis of
/// another kind, such as [`SegmentAxisKind::Geography`], has no role. If segment *m* has a value
/// alone and a value with a member of role [`ReconciliationRole::Segment`], the value alone counts,
/// even if it is blocked.
///
/// The unreconciled line exists only if the consolidated candidate and each candidate with a role
/// are reported, and a segment line exists.
#[must_use]
pub fn reconcile(candidates: &[Candidate], registry: &Registry<'_>) -> Vec<Reconciliation> {
    let mut groups: BTreeMap<GroupKey, Group> = BTreeMap::new();
    for candidate in candidates {
        let key = GroupKey {
            filing: candidate.source.filing.clone(),
            entity: candidate.entity.clone(),
            concept: candidate.concept,
            period_end: candidate.period.end(),
            quarters: candidate.period.quarters(),
            unit: candidate.unit.clone(),
        };
        if candidate.dimensions.is_empty() {
            groups.entry(key).or_default().consolidated = Some(candidate.clone());
        } else if let Some(role) = role_of(candidate, registry) {
            groups.entry(key).or_default().add(role, candidate.clone());
        }
    }
    groups
        .into_iter()
        .filter(|(_, group)| group.has_role())
        .map(|(key, group)| group.into_reconciliation(key))
        .collect()
}

/// The filing, entity, standard concept, period, and unit of one [`Reconciliation`].
#[derive(PartialEq, Eq, PartialOrd, Ord)]
struct GroupKey {
    filing: FilingId,
    entity: EntityId,
    concept: CanonicalElementKind,
    period_end: Date,
    quarters: u16,
    unit: Unit,
}

/// The role of a candidate with dimensions.
enum Role {
    /// Segment *m* on a business-segment axis. `paired` is `true` if the dimensions also hold a
    /// member of role [`ReconciliationRole::Segment`].
    Segment {
        segment: Dimensions,
        paired: bool,
    },
    Elimination,
    Corporate,
}

/// Returns the role of `candidate`, or `None` if its dimensions give it no role.
fn role_of(candidate: &Candidate, registry: &Registry<'_>) -> Option<Role> {
    let plugin = registry.segment_axes(candidate.inputs.first()?.concept.family())?;
    let mut segment = None;
    let mut consolidation_role = None;
    for (axis, member) in candidate.dimensions.iter() {
        match kind_of(plugin, axis)? {
            SegmentAxisKind::BusinessSegment if segment.is_none() => segment = Some((axis, member)),
            SegmentAxisKind::ConsolidationItems if consolidation_role.is_none() => {
                consolidation_role = Some(role_of_member(plugin, axis, member)?);
            }
            _ => return None,
        }
    }
    match (segment, consolidation_role) {
        (Some(pair), paired @ (None | Some(ReconciliationRole::Segment))) => Some(Role::Segment {
            segment: Dimensions::new([pair]).ok()?,
            paired: paired.is_some(),
        }),
        (None, Some(ReconciliationRole::Elimination)) => Some(Role::Elimination),
        (None, Some(ReconciliationRole::Corporate)) => Some(Role::Corporate),
        _ => None,
    }
}

/// Returns the kind of `axis` in `plugin`, or `None` if it names no such segment axis.
fn kind_of(plugin: &dyn SegmentAxes, axis: &str) -> Option<SegmentAxisKind> {
    plugin
        .segment_axes()
        .iter()
        .find(|segment_axis| segment_axis.name() == axis)
        .map(SegmentAxis::kind)
}

/// Returns the role of `member` on `axis` in `plugin`, or `None` if it has no role.
fn role_of_member(
    plugin: &dyn SegmentAxes,
    axis: &str,
    member: &str,
) -> Option<ReconciliationRole> {
    plugin
        .member_roles()
        .iter()
        .find(|role| role.axis() == axis && role.member() == member)
        .map(MemberRole::role)
}

/// The candidates of one filing, entity, standard concept, period, and unit, sorted by role.
#[derive(Default)]
struct Group {
    consolidated: Option<Candidate>,
    /// Per segment *m*, whether the value has a pair of role segment, and the candidate.
    segments: BTreeMap<Dimensions, (bool, Candidate)>,
    eliminations: Vec<Candidate>,
    corporate: Vec<Candidate>,
}

impl Group {
    /// Adds `candidate` under `role`. A segment value alone replaces a value with a pair.
    fn add(&mut self, role: Role, candidate: Candidate) {
        match role {
            Role::Segment { segment, paired } => {
                let kept_alone = self.segments.get(&segment).is_some_and(|(kept, _)| !*kept);
                if !(paired && kept_alone) {
                    self.segments.insert(segment, (paired, candidate));
                }
            }
            Role::Elimination => self.eliminations.push(candidate),
            Role::Corporate => self.corporate.push(candidate),
        }
    }

    fn has_role(&self) -> bool {
        !(self.segments.is_empty() && self.eliminations.is_empty() && self.corporate.is_empty())
    }

    fn into_reconciliation(self, key: GroupKey) -> Reconciliation {
        let mut lines: Vec<ReconciliationLine> = self
            .segments
            .into_values()
            .map(|(_, candidate)| ReconciliationLine::Segment(candidate))
            .chain(
                self.eliminations
                    .into_iter()
                    .map(ReconciliationLine::Elimination),
            )
            .chain(
                self.corporate
                    .into_iter()
                    .map(ReconciliationLine::Corporate),
            )
            .collect();
        if let Some(unreconciled) = unreconciled(self.consolidated.as_ref(), &lines) {
            lines.push(ReconciliationLine::Unreconciled(unreconciled));
        }
        Reconciliation {
            filing: key.filing,
            entity: key.entity,
            concept: key.concept,
            period_end: key.period_end,
            quarters: key.quarters,
            unit: key.unit,
            consolidated: self.consolidated,
            lines,
        }
    }
}

/// Returns consolidated − Σ lines, or `None` if the consolidated candidate is absent or blocked, a
/// line is blocked, or no segment line exists.
fn unreconciled(
    consolidated: Option<&Candidate>,
    lines: &[ReconciliationLine],
) -> Option<BigDecimal> {
    if !lines
        .iter()
        .any(|line| matches!(line, ReconciliationLine::Segment(_)))
    {
        return None;
    }
    lines
        .iter()
        .try_fold(reported(consolidated?)?.clone(), |rest, line| {
            let (ReconciliationLine::Segment(candidate)
            | ReconciliationLine::Elimination(candidate)
            | ReconciliationLine::Corporate(candidate)) = line
            else {
                return None;
            };
            Some(rest - reported(candidate)?)
        })
}

/// Returns the value of `candidate`, or `None` if it is blocked.
const fn reported(candidate: &Candidate) -> Option<&BigDecimal> {
    match &candidate.value {
        CandidateValue::Reported { value, .. } => Some(value),
        CandidateValue::Blocked => None,
    }
}

#[cfg(test)]
mod tests {
    use pretty_assertions::assert_eq;

    use super::*;
    use crate::core::period::{Duration, Period};
    use crate::core::taxonomy::registry::Plugin;
    use crate::tests::fixtures::sample_fact::{candidate, date, source};
    use crate::us_gaap::UsGaap;

    /// Runs [`reconcile`] on `candidates` with [`UsGaap`] and its segment axes.
    fn reconcile_with_us_gaap(candidates: &[Candidate]) -> Vec<Reconciliation> {
        let us_gaap = UsGaap::new();
        let registry = Registry::with_plugins(vec![Plugin::WithSegmentAxes(&us_gaap)])
            .expect("Given one plug-in with distinct families, the registry should always start");
        reconcile(candidates, &registry)
    }

    /// Returns the revenue candidate `value` with the axis and member pairs `pairs`.
    fn revenue(value: &str, pairs: &[(&str, &str)]) -> Candidate {
        let mut revenue = candidate(&source("0000320193-24-000123", date(2024, 11, 1)), value);
        revenue.dimensions = Dimensions::new(pairs.iter().copied())
            .expect("Given hardcoded axis and member pairs, the creation should always succeed");
        revenue
    }

    /// Returns `candidate` for the year that starts on `start` and ends on 2024-09-28.
    fn with_start(candidate: Candidate, start: Date) -> Candidate {
        let year = Duration::from_dates(start, date(2024, 9, 28))
            .expect("Given a start before the end, the creation should always succeed");
        Candidate {
            period: Period::Duration(year),
            ..candidate
        }
    }

    /// Returns `candidate` with its value blocked.
    fn blocked(candidate: Candidate) -> Candidate {
        Candidate {
            value: CandidateValue::Blocked,
            ..candidate
        }
    }

    /// Returns the last line of the first reconciliation.
    fn last_line(reconciliations: &[Reconciliation]) -> Option<ReconciliationLine> {
        reconciliations
            .first()
            .and_then(|reconciliation| reconciliation.lines.last().cloned())
    }

    /// Returns the lines of the first reconciliation.
    fn lines(reconciliations: &[Reconciliation]) -> Vec<ReconciliationLine> {
        reconciliations
            .first()
            .map(|reconciliation| reconciliation.lines.clone())
            .unwrap_or_default()
    }

    /// Returns the unreconciled line `value`.
    fn unreconciled_line(value: i64) -> ReconciliationLine {
        ReconciliationLine::Unreconciled(BigDecimal::from(value))
    }

    #[test]
    fn should_give_unreconciled_zero_when_two_segments_and_an_elimination_add_up() {
        let candidates = vec![
            revenue("190", &[]),
            revenue("100", &[("BusinessSegments", "Cloud")]),
            revenue("100", &[("BusinessSegments", "Devices")]),
            revenue("-10", &[("ConsolidationItems", "IntersegmentElimination")]),
        ];

        let expected_result = Some(unreconciled_line(0));

        let result = last_line(&reconcile_with_us_gaap(&candidates));

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_put_the_amount_in_the_unreconciled_line_when_the_member_is_of_the_company() {
        let candidates = vec![
            revenue("185", &[]),
            revenue("100", &[("BusinessSegments", "Cloud")]),
            revenue("100", &[("BusinessSegments", "Devices")]),
            revenue("-15", &[("ConsolidationItems", "CorporateAndEliminations")]),
        ];

        let expected_result = Some(unreconciled_line(-15));

        let result = last_line(&reconcile_with_us_gaap(&candidates));

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_give_one_segment_line_when_the_filing_reports_a_single_segment() {
        let cloud = revenue("100", &[("BusinessSegments", "Cloud")]);
        let candidates = vec![revenue("100", &[]), cloud.clone()];

        let expected_result = vec![ReconciliationLine::Segment(cloud), unreconciled_line(0)];

        let result = lines(&reconcile_with_us_gaap(&candidates));

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_give_no_unreconciled_line_when_a_segment_is_blocked() {
        let cloud = revenue("100", &[("BusinessSegments", "Cloud")]);
        let devices = blocked(revenue("90", &[("BusinessSegments", "Devices")]));
        let candidates = vec![revenue("190", &[]), cloud.clone(), devices.clone()];

        let expected_result = vec![
            ReconciliationLine::Segment(cloud),
            ReconciliationLine::Segment(devices),
        ];

        let result = lines(&reconcile_with_us_gaap(&candidates));

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_give_no_unreconciled_line_when_the_consolidated_candidate_is_blocked() {
        let cloud = revenue("100", &[("BusinessSegments", "Cloud")]);
        let candidates = vec![blocked(revenue("100", &[])), cloud.clone()];

        let expected_result = vec![ReconciliationLine::Segment(cloud)];

        let result = lines(&reconcile_with_us_gaap(&candidates));

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_give_no_unreconciled_line_when_an_elimination_is_blocked() {
        let cloud = revenue("100", &[("BusinessSegments", "Cloud")]);
        let elimination = blocked(revenue(
            "-10",
            &[("ConsolidationItems", "IntersegmentElimination")],
        ));
        let candidates = vec![revenue("90", &[]), cloud.clone(), elimination.clone()];

        let expected_result = vec![
            ReconciliationLine::Segment(cloud),
            ReconciliationLine::Elimination(elimination),
        ];

        let result = lines(&reconcile_with_us_gaap(&candidates));

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_give_no_role_when_the_value_is_the_segment_total_or_intersegment_revenue() {
        let candidates = vec![
            revenue("100", &[]),
            revenue("100", &[("BusinessSegments", "Cloud")]),
            revenue("100", &[("ConsolidationItems", "OperatingSegments")]),
            revenue(
                "5",
                &[
                    ("BusinessSegments", "Cloud"),
                    ("ConsolidationItems", "IntersegmentElimination"),
                ],
            ),
        ];

        let expected_result = Some(unreconciled_line(0));

        let result = last_line(&reconcile_with_us_gaap(&candidates));

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_put_the_segment_with_the_consolidated_value_when_only_the_start_differs() {
        let consolidated = with_start(revenue("100", &[]), date(2023, 9, 30));
        let cloud = with_start(
            revenue("100", &[("BusinessSegments", "Cloud")]),
            date(2023, 10, 1),
        );
        let candidates = vec![consolidated, cloud];

        let expected_result = Some(unreconciled_line(0));

        let result = last_line(&reconcile_with_us_gaap(&candidates));

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_give_one_reconciliation_per_entity_when_two_entities_report_the_period() {
        let parent = revenue("100", &[("BusinessSegments", "Cloud")]);
        let mut subsidiary = revenue("50", &[("BusinessSegments", "Cloud")]);
        subsidiary.entity = EntityId::new("cik", "0000000002")
            .expect("Given a hardcoded scheme and ID, the creation should always succeed");
        let candidates = vec![parent, subsidiary];

        let expected_result = 2;

        let result = reconcile_with_us_gaap(&candidates).len();

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_give_one_reconciliation_per_filing_when_two_filings_report_the_period() {
        let original = revenue("100", &[("BusinessSegments", "Cloud")]);
        let mut amendment = revenue("100", &[("BusinessSegments", "Cloud")]);
        amendment.source = source("0000320193-24-000200", date(2024, 12, 1));
        let candidates = vec![original, amendment];

        let expected_result = 2;

        let result = reconcile_with_us_gaap(&candidates).len();

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_give_no_unreconciled_line_when_the_filing_has_no_segment() {
        let corporate = revenue("-5", &[("ConsolidationItems", "CorporateNonSegment")]);
        let candidates = vec![revenue("100", &[]), corporate.clone()];

        let expected_result = vec![ReconciliationLine::Corporate(corporate)];

        let result = lines(&reconcile_with_us_gaap(&candidates));

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_give_no_reconciliation_when_the_values_carry_only_geography() {
        let candidates = vec![
            revenue("100", &[]),
            revenue("60", &[("Geographical", "US")]),
            revenue("40", &[("Geographical", "NonUs")]),
        ];

        let expected_result: Vec<Reconciliation> = Vec::new();

        let result = reconcile_with_us_gaap(&candidates);

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_count_the_blocked_value_alone_when_the_segment_with_the_pair_follows_it() {
        let alone = blocked(revenue("100", &[("BusinessSegments", "Cloud")]));
        let paired = revenue(
            "100",
            &[
                ("BusinessSegments", "Cloud"),
                ("ConsolidationItems", "OperatingSegments"),
            ],
        );
        let candidates = vec![revenue("100", &[]), alone.clone(), paired];

        let expected_result = vec![ReconciliationLine::Segment(alone)];

        let result = lines(&reconcile_with_us_gaap(&candidates));

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_count_the_value_alone_when_the_segment_with_the_pair_comes_first() {
        let paired = revenue(
            "90",
            &[
                ("BusinessSegments", "Cloud"),
                ("ConsolidationItems", "OperatingSegments"),
            ],
        );
        let candidates = vec![
            revenue("100", &[]),
            paired,
            revenue("100", &[("BusinessSegments", "Cloud")]),
        ];

        let expected_result = Some(unreconciled_line(0));

        let result = last_line(&reconcile_with_us_gaap(&candidates));

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_count_the_segment_with_the_pair_when_the_filing_reports_no_value_alone() {
        let candidates = vec![
            revenue("100", &[]),
            revenue(
                "100",
                &[
                    ("BusinessSegments", "Cloud"),
                    ("ConsolidationItems", "OperatingSegments"),
                ],
            ),
        ];

        let expected_result = Some(unreconciled_line(0));

        let result = last_line(&reconcile_with_us_gaap(&candidates));

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_give_the_role_rules_for_each_kind_of_line() {
        let lines = [
            ReconciliationLine::Segment(revenue("1", &[])),
            ReconciliationLine::Elimination(revenue("1", &[])),
            ReconciliationLine::Corporate(revenue("1", &[])),
            unreconciled_line(0),
        ];

        let expected_result = vec![
            None,
            Some("core/segment/elimination@1".to_string()),
            Some("core/segment/corporate@1".to_string()),
            Some("core/segment/unreconciled@1".to_string()),
        ];

        let result: Vec<Option<String>> = lines
            .iter()
            .map(|line| line.role_rule().map(|rule| rule.as_str().to_string()))
            .collect();

        assert_eq!(result, expected_result);
    }
}
