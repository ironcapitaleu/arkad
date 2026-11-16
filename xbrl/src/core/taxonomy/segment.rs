//! # Segment
//!
//! Provides the [`SegmentAxes`] trait, which a plug-in implements next to
//! [`Taxonomy`] to name the axes whose members split a company.
//!
//! `core` names no axis and no member of a taxonomy. It reads only the [`SegmentAxisKind`] of an
//! axis and the [`ReconciliationRole`] of a member.

use super::Taxonomy;

/// The axes whose members split a company, and the role of each member in the reconciliation.
///
/// A plug-in without segment data implements only [`Taxonomy`]. Each [`MemberRole`] names an axis
/// that [`SegmentAxes::segment_axes`] returns with kind [`SegmentAxisKind::ConsolidationItems`].
///
/// # Required Traits
///
/// [`Taxonomy`]: the registry reads the axes of the plug-in that resolves the concept of a fact.
pub trait SegmentAxes: Taxonomy {
    /// Returns each segment axis, as written in `dimensions`, with its kind.
    fn segment_axes(&self) -> &[SegmentAxis];

    /// Returns each member of an axis of kind [`SegmentAxisKind::ConsolidationItems`] that has a
    /// role.
    fn member_roles(&self) -> &[MemberRole];
}

/// What the members of a segment axis are.
#[non_exhaustive]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SegmentAxisKind {
    /// Business segments.
    BusinessSegment,
    /// Countries and regions.
    Geography,
    /// Products and services.
    Product,
    /// The role of a value in the reconciliation.
    ConsolidationItems,
    /// Parts of one segment.
    Subsegment,
    /// Eliminations below one segment.
    SubsegmentConsolidationItems,
}

/// The role of a member in the reconciliation of the business segments to the consolidated value.
#[non_exhaustive]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ReconciliationRole {
    /// The value of a business segment.
    Segment,
    /// An amount that removes transactions between segments.
    Elimination,
    /// An amount outside the segments, such as corporate costs.
    Corporate,
}

/// One segment axis of a taxonomy, as written in `dimensions`, without prefix.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct SegmentAxis {
    name: &'static str,
    kind: SegmentAxisKind,
}

impl SegmentAxis {
    /// Creates a [`SegmentAxis`] from the axis name without prefix, for example
    /// `BusinessSegments`, and its kind.
    #[must_use]
    pub const fn new(name: &'static str, kind: SegmentAxisKind) -> Self {
        Self { name, kind }
    }

    /// Returns the axis name without prefix.
    #[must_use]
    pub const fn name(&self) -> &'static str {
        self.name
    }

    /// Returns the kind of the axis.
    #[must_use]
    pub const fn kind(&self) -> SegmentAxisKind {
        self.kind
    }
}

/// The role of one member of an axis of kind [`SegmentAxisKind::ConsolidationItems`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct MemberRole {
    axis: &'static str,
    member: &'static str,
    role: ReconciliationRole,
}

impl MemberRole {
    /// Creates a [`MemberRole`] from the axis and member names without prefix, for example
    /// `ConsolidationItems` and `IntersegmentElimination`, and the role of the member.
    #[must_use]
    pub const fn new(axis: &'static str, member: &'static str, role: ReconciliationRole) -> Self {
        Self { axis, member, role }
    }

    /// Returns the axis name without prefix.
    #[must_use]
    pub const fn axis(&self) -> &'static str {
        self.axis
    }

    /// Returns the member name without prefix.
    #[must_use]
    pub const fn member(&self) -> &'static str {
        self.member
    }

    /// Returns the role of the member.
    #[must_use]
    pub const fn role(&self) -> ReconciliationRole {
        self.role
    }
}

#[cfg(test)]
mod tests {
    use pretty_assertions::assert_eq;

    use super::*;

    const fn implements_auto_traits<T: Sized + Send + Sync + Unpin>() {}
    #[test]
    const fn should_implement_auto_traits_for_segment_axis() {
        implements_auto_traits::<SegmentAxis>();
    }

    #[test]
    const fn should_implement_auto_traits_for_member_role() {
        implements_auto_traits::<MemberRole>();
    }

    #[test]
    const fn should_implement_auto_traits_for_segment_axis_kind() {
        implements_auto_traits::<SegmentAxisKind>();
    }

    #[test]
    const fn should_implement_auto_traits_for_reconciliation_role() {
        implements_auto_traits::<ReconciliationRole>();
    }

    #[test]
    fn should_return_name_and_kind_when_segment_axis_is_created() {
        let axis = SegmentAxis::new("Geographical", SegmentAxisKind::Geography);

        let expected_result = ("Geographical", SegmentAxisKind::Geography);

        let result = (axis.name(), axis.kind());

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_return_axis_member_and_role_when_member_role_is_created() {
        let member_role = MemberRole::new(
            "ConsolidationItems",
            "CorporateNonSegment",
            ReconciliationRole::Corporate,
        );

        let expected_result = (
            "ConsolidationItems",
            "CorporateNonSegment",
            ReconciliationRole::Corporate,
        );

        let result = (member_role.axis(), member_role.member(), member_role.role());

        assert_eq!(result, expected_result);
    }
}
