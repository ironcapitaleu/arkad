//! # Per-Share Concepts
//!
//! Provides [`PerShareConcept`], the per-share amounts of the disclosure `PerShare`.

innermost_concepts! {
    /// A per-share amount. It adds to no line of a statement and does not add over periods.
    pub enum PerShareConcept in Disclosure(DisclosureConcept::PerShare) {
        /// Earnings per share (basic).
        EarningsPerShareBasic => "EPS (Basic)",
        /// Earnings per share (diluted).
        EarningsPerShareDiluted => "EPS (Diluted)",
    }
}

#[cfg(test)]
mod tests {
    const fn implements_auto_traits<T: Sized + Send + Sync + Unpin>() {}

    #[test]
    const fn should_implement_auto_traits_for_per_share_concept() {
        implements_auto_traits::<super::PerShareConcept>();
    }
}
