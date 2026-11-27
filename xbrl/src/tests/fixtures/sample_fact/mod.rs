//! # Sample Fact
//!
//! Provides builders of as-is facts, candidates, and standardized facts for the tests of
//! resolution, dedup, and derivation.
//!
//! ## Modules
//!
//! - [`builders`]: The helpers that build a filing source, an entity ID, an as-is fact, a candidate, a
//!   standardized fact, a value of the `Cloud` segment, and a filing.

pub mod builders;

pub use builders::{
    apple, candidate, date, fact, filing, in_cloud_segment, revenue_aliases, source, standardized,
    us_gaap_plugin,
};
