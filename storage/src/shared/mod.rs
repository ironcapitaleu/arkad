//! # Shared
//!
//! Provides the types that the backends and the callers of the crate both use.
//!
//! ## Modules
//!
//! - [`account_id`]: The [`AccountId`], the key of an account, and the [`InvalidAccountId`] for a
//!   string that is not a UUID.
//! - [`company_id`]: The [`CompanyId`], the internal key of a company.
//! - [`company_identifier`]: The [`CompanyIdentifier`], a `(scheme, value)` pair that names a
//!   company, and its [`CompanyScheme`].
//! - [`listing_id`]: The [`ListingId`], the internal key of a listing.
//! - [`listing_identifier`]: The [`ListingIdentifier`], a `(scheme, value)` pair that names a
//!   listing, and its [`ListingScheme`].
//! - [`mic`]: The [`Mic`] of ISO 10383 that names an exchange, and the [`InvalidMic`] for a string
//!   that is not a MIC.
//! - [`sha256`]: The [`Sha256Digest`] of a byte slice, and the [`InvalidSha256Digest`] for a
//!   string that is not a digest.

pub mod account_id;
pub mod company_id;
pub mod company_identifier;
pub mod listing_id;
pub mod listing_identifier;
pub mod mic;
pub mod sha256;

pub use account_id::{AccountId, InvalidAccountId};
pub use company_id::CompanyId;
pub use company_identifier::{CompanyIdentifier, CompanyScheme};
pub use listing_id::ListingId;
pub use listing_identifier::{ListingIdentifier, ListingScheme};
pub use mic::{InvalidMic, Mic};
pub use sha256::{InvalidSha256Digest, Sha256Digest};
