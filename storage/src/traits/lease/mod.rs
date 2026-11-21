//! # Lease
//!
//! Provides the port through which the runs of a job kind take turns to write.
//!
//! ## Modules
//!
//! - [`lease_store`]: [`LeaseStore`] — takes and reads the lease of a job kind.

pub mod lease_store;

pub use lease_store::LeaseStore;
