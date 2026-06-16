//! # Execute SEC Request Data
//!
//! Groups the input and output data types for the
//! [`ExecuteSecRequest`](super::ExecuteSecRequest) state.
//!
//! The state consumes a prepared client and request and produces the SEC response.
//! Input and output live in separate child modules to mirror that transformation.
//!
//! ## Modules
//!
//! - [`input`]: The [`ExecuteSecRequestInput`] holding the prepared client and request.
//! - [`output`]: The [`ExecuteSecRequestOutput`] holding the received SEC response.
//!
//! ## See Also
//!
//! - [`crate::implementations::states::extract::execute_sec_request`]: The parent state implementation.

pub mod input;
pub mod output;

pub use input::ExecuteSecRequestInput;
pub use output::ExecuteSecRequestOutput;
