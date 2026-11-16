//! # Disclosure Concepts
//!
//! Provides the concept enums of the disclosures. A disclosure is a set of concepts outside the
//! trees of the statements, so its concepts have no placement.
//!
//! ## Modules
//!
//! - [`debt_maturities`]: The maturity ladder of long-term debt and its total.
//! - [`per_share`]: The per-share amounts.

pub mod debt_maturities;
pub mod per_share;

pub use debt_maturities::DebtMaturityConcept;
pub use per_share::PerShareConcept;
