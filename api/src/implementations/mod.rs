//! # Implementations
//!
//! Provides the adapters of the ports in [`traits`](crate::traits).
//!
//! ## Modules
//!
//! - [`supabase_token_verifier`]: The [`SupabaseTokenVerifier`] that checks Supabase login tokens.

pub mod supabase_token_verifier;

pub use supabase_token_verifier::SupabaseTokenVerifier;
