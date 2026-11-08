//! # Sample Fact
//!
//! Provides builders of as-is facts and candidates for the tests of resolution and dedup.
//!
//! ## Modules
//!
//! - [`builders`]: The helpers that build a filing source, an as-is fact, and a candidate.

pub mod builders;

pub use builders::{candidate, date, fact, revenue_aliases, source, us_gaap_plugin};
