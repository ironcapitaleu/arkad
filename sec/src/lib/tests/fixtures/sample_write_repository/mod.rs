//! # Sample Write Repository
//!
//! Provides fake [`WriteRepository`](storage::WriteRepository) implementations for raw documents.
//!
//! ## Modules
//!
//! - [`fake_write_repository`]: The [`FakeWriteRepository`] that records or rejects each write.

pub mod fake_write_repository;

pub use fake_write_repository::FakeWriteRepository;
