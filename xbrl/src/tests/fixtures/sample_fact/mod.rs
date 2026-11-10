//! # Sample Fact
//!
//! Provides builders of as-is facts, candidates, and standardized facts for the tests of
//! resolution, dedup, and derivation.
//!
//! ## Modules
//!
//! - [`builders`]: The helpers that build a filing source, an as-is fact, a candidate, a
//!   standardized fact, and a filing.

pub mod builders;

pub use builders::{
    candidate, date, fact, filing, revenue_aliases, source, standardized, us_gaap_plugin,
};
