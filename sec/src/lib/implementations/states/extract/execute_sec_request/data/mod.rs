//! # Execute SEC Request Data Module
//!
//! This module contains the input and output data structures for the [`ExecuteSecRequest`](../mod.rs) state
//! in the SEC filings extraction workflow.
//!
//! ## Components
//! - [`input_data`]: Input data structures and updaters for SEC request execution parameters.
//! - [`output_data`]: Output data structures and updaters for SEC request execution results.
//!
//! ## Usage
//! These data structures are used by the [`ExecuteSecRequest`](../mod.rs) state to manage the flow of
//! information through the state machine. They support builder-based updates and integrate with the
//! state machine framework's data management system.
//!
//! ## See Also
//! - [`crate::implementations::states::extract::execute_sec_request`]: Parent module for the SEC request execution state.
//! - [`crate::shared`]: Shared domain types used by these data structures.

pub mod input_data;
pub mod output_data;

pub use input_data::ExecuteSecRequestInputData;
pub use output_data::ExecuteSecRequestOutputData;
