//! # Execute SEC Request Data Module
//!
//! This module contains the input and output data structures for the [`ExecuteSecRequest`](../mod.rs) state
//! in the SEC filings extraction workflow.
//!
//! ## Components
//! - [`input`]: Input data structures and updaters for SEC request execution parameters.
//! - [`output`]: Output data structures and updaters for SEC request execution results.
//!
//! ## Usage
//! These data structures are used by the [`ExecuteSecRequest`](../mod.rs) state to manage the flow of
//! information through the state machine. They support builder-based updates and integrate with the
//! state machine framework's data management system.
//!
//! ## See Also
//! - [`crate::implementations::states::extract::execute_sec_request`]: Parent module for the SEC request execution state.
//! - [`crate::shared`]: Shared domain types used by these data structures.

pub mod input;
pub mod output;

pub use input::ExecuteSecRequestInput;
pub use output::ExecuteSecRequestOutput;
