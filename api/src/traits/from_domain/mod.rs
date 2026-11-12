//! # From Domain
//!
//! Provides the [`FromDomain`] trait of a contract type that converts from a domain type.

use serde::Serialize;
use utoipa::ToSchema;

/// Marks a type of the `/v1` contract that converts from the domain type `Domain`.
///
/// A domain type serializes with its Rust names and shapes. The contract fixes its own format.
/// So a response holds contract types, and a handler converts each domain value with `.into()`
/// or `.try_into()`. The bounds give every contract type a JSON form, a schema in the spec, and
/// a conversion from its domain type.
///
/// An implementor writes `From<Domain>` or `TryFrom<Domain>`, and then an empty impl of this
/// trait.
pub trait FromDomain<Domain>: Serialize + ToSchema + TryFrom<Domain> {}
