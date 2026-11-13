//! # `backfill` Tests
//!
//! Runs the tests of the shared modules of the `backfill` binary. The binary is its own crate
//! root, so this test compiles its modules as private modules.
//!
//! ## External Dependencies
//!
//! None. A fake SEC client answers every request.

#[path = "../src/bin/shared/files_in_flight.rs"]
mod files_in_flight;
#[path = "../src/bin/shared/notes_periods.rs"]
mod notes_periods;
