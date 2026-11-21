//! # Implementations
//!
//! Provides the concrete backends behind the crate's [`traits`](crate::traits).
//!
//! ## Modules
//!
//! - [`lease`]: Backends implementing [`LeaseStore`](crate::LeaseStore).
//! - [`repository`]: Backends implementing the repository traits.

pub mod lease;
pub mod repository;
