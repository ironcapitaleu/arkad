//! # Shared
//!
//! Provides the domain types of the API, which depend on no route.
//!
//! Most types here are types of the `/v1` contract. Each one converts from its `xbrl` domain
//! type through [`FromDomain`](crate::traits::from_domain::FromDomain).
//!
//! ## Modules
//!
//! - [`accession_number`]: The accession number of a filing.
//! - [`concept_id`]: The `snake_case` identifier of a canonical concept.
//! - [`confidence`]: The resolution tier that produced a fact.
//! - [`exact_decimal`]: An exact decimal number as a string.
//! - [`fiscal_period`]: The fiscal period of a fact or a filing.
//! - [`pascal_case`]: The [`PascalCase`](pascal_case::PascalCase) name and its conversion to
//!   `snake_case`.
//! - [`period`]: The period that a fact covers.
//! - [`mod@unit`]: The unit of a fact as one text.

pub mod accession_number;
pub mod concept_id;
pub mod confidence;
pub mod exact_decimal;
pub mod fiscal_period;
pub mod pascal_case;
pub mod period;
pub mod unit;
