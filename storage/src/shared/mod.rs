//! # Shared
//!
//! Provides the types that the backends and the callers of the crate both use.
//!
//! ## Modules
//!
//! - [`company_id`]: The [`CompanyId`], the internal key of a company.
//! - [`company_identifier`]: The [`CompanyIdentifier`], a `(scheme, value)` pair that names a
//!   company, and its [`CompanyScheme`].
//! - [`sha256`]: The [`Sha256Digest`] of a byte slice, and the [`InvalidSha256Digest`] for a
//!   string that is not a digest.

pub mod company_id;
pub mod company_identifier;
pub mod sha256;

pub use company_id::CompanyId;
pub use company_identifier::{CompanyIdentifier, CompanyScheme};
pub use sha256::{InvalidSha256Digest, Sha256Digest};
