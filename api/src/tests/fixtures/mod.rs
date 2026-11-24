//! # Common Test Fixtures
//!
//! Reusable test doubles for building `api` tests.
//!
//! ## Modules
//!
//! - [`fake_token_verifier`]: A fake [`TokenVerifier`](crate::traits::token_verifier::TokenVerifier)
//!   that accepts the tokens a test sets up.
//! - [`test_signing_key`]: A key pair that a test generates to sign login tokens.

pub mod fake_token_verifier;
pub mod test_signing_key;
