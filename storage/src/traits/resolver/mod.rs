//! # Resolver
//!
//! Provides the ports that return the internal key of a subject for one of its identifiers.
//!
//! ## Modules
//!
//! - [`entity_resolver`]: [`EntityResolver`] — returns the [`CompanyId`](crate::CompanyId) of a
//!   company identifier at a date.
//! - [`listing_resolver`]: [`ListingResolver`] — returns the [`ListingId`](crate::ListingId) of a
//!   listing identifier at a date.

pub mod entity_resolver;
pub mod listing_resolver;

pub use entity_resolver::EntityResolver;
pub use listing_resolver::ListingResolver;
