//! # Traits
//!
//! Provides the traits of the API.
//!
//! ## Modules
//!
//! - [`from_domain`]: The [`FromDomain`](from_domain::FromDomain) trait of a contract type that
//!   converts from a domain type.
//! - [`token_verifier`]: The [`TokenVerifier`](token_verifier::TokenVerifier) port that checks a
//!   login token.

pub mod from_domain;
pub mod token_verifier;
