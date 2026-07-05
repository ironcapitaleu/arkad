//! # Response Traits
//!
//! The two-layer response contract: a raw [`InnerResponse`] and the domain-level [`SecResponse`].
//!
//! ## Modules
//!
//! - [`inner`]: The [`InnerResponse`] trait abstracting a concrete response type.
//! - [`sec`]: The [`SecResponse`] trait adding validated SEC response semantics on top.

pub mod inner;
pub mod sec;

pub use inner::InnerResponse;
pub use sec::SecResponse;
