//! # SEC EDGAR API Parsers
//!
//! JSON deserializers for the three SEC XBRL JSON API endpoints.
//! All parsers produce the same [`RawObservation`](crate::core::observation::RawObservation)
//! output type regardless of which API the data came from.

pub mod company_concept;
pub mod company_facts;
pub mod frames;
