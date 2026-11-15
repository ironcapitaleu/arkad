//! # Resolver
//!
//! Provides the ports that return the internal key of a subject for one of its identifiers.
//!
//! ## Modules
//!
//! - [`entity_resolver`]: [`EntityResolver`] — returns the [`CompanyId`](crate::CompanyId) of a
//!   company identifier at a date.

pub mod entity_resolver;

pub use entity_resolver::EntityResolver;
