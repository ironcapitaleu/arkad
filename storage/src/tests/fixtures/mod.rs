//! # Fixtures
//!
//! Reusable test doubles for the `storage` crate's persistence ports.
//!
//! Per the house convention, fakes live in each crate's own `#[cfg(test)]` `tests/fixtures/` and are
//! not exported; a second consumer needing the same fake is the trigger to promote it to a shared
//! testkit crate — not before.
//!
//! ## Modules
//!
//! - [`fake_repository`]: The [`FakeRepository`](fake_repository::FakeRepository) test double for
//!   the [`Repository`](crate::Repository) port.

pub mod fake_repository;
