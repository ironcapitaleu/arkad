//! # From Domain
//!
//! Provides the [`FromDomain`] trait of a contract type that converts from a domain type.

use serde::Serialize;
use utoipa::ToSchema;

/// Marks a type of the `/v1` contract that converts from the domain type `Domain`.
///
/// An implementor writes `From<Domain>` or `TryFrom<Domain>`, and then an empty impl of this
/// trait.
pub trait FromDomain<Domain>: Serialize + ToSchema + TryFrom<Domain> {}
