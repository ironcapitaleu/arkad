//! # Common Test Fixtures
//!
//! Reusable test doubles for building `api` tests.
//!
//! ## Modules
//!
//! - [`fake_token_verifier`]: A fake [`TokenVerifier`](crate::traits::token_verifier::TokenVerifier)
//!   that accepts the tokens a test sets up.

pub mod fake_token_verifier;
