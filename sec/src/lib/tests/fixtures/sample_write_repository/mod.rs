//! # Sample Write Repository
//!
//! Provides fake [`WriteRepository`](storage::WriteRepository) implementations for raw documents.
//!
//! ## Modules
//!
//! - [`fake_write_repository`]: The [`FakeRawRepository`] that records or rejects each write, and
//!   its aliases [`FakeWriteRepository`], [`FakeNotesRepository`], [`FakeDailyIndexRepository`], and
//!   [`FakeSubmissionsRepository`].

pub mod fake_write_repository;

pub use fake_write_repository::{
    FakeDailyIndexRepository, FakeNotesRepository, FakeRawRepository, FakeSubmissionsRepository,
    FakeWriteRepository,
};
