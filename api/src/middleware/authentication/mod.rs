//! # Authentication
//!
//! Provides the extractors that check the credential of a request before a handler runs.
//!
//! ## Modules
//!
//! - [`authenticated_account`]: The [`AuthenticatedAccount`] extractor for a login token.

pub mod authenticated_account;

pub use authenticated_account::AuthenticatedAccount;
