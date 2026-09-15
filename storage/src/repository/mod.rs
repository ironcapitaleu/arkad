//! # Repository
//!
//! Provides the persistence ports, split by access capability.
//!
//! Each port names one capability. A caller depends on the port for the access it needs, so a
//! read-only caller has no write method in scope.
//!
//! Each port module holds that claim with a paired doctest: an example that uses the method the
//! port has, and a `compile_fail` example that calls the method it does not. A guard of that shape
//! is only as good as the errors it rules out, so each one is written to leave the missing method
//! as the single reachable failure. Should the method ever arrive, the block compiles and the
//! guard turns red.
//!
//! That construction rests on a leak compiling with warnings rather than errors. Neither the
//! `-D warnings` that `.cargo/config.toml` passes to rustdoc nor a `warnings` entry in the
//! manifest's `[lints.rust]` table reaches a doctest body, so the warnings stay warnings.
//! `#![doc(test(attr(deny(warnings))))]` on the crate root is the one setting that would promote
//! them, and it would send both guards green through the leak they exist to catch.
//!
//! ## Modules
//!
//! - [`read_repository`]: [`ReadRepository`] — reads a record by key.
//! - [`read_write_repository`]: [`ReadWriteRepository`] — names a store that reads and writes.
//! - [`write_repository`]: [`WriteRepository`] — persists a record.
//!
//! ## Usage
//!
//! ```rust
//! use storage::{ReadRepository, WriteRepository};
//!
//! fn takes_a_store<S>(_store: S)
//! where
//!     S: ReadRepository<Record = String, Key = String> + WriteRepository<Record = String>,
//! {
//! }
//! ```

pub mod read_repository;
pub mod read_write_repository;
pub mod write_repository;

pub use read_repository::ReadRepository;
pub use read_write_repository::ReadWriteRepository;
pub use write_repository::WriteRepository;
