//! # Prepare SEC Request Data
//!
//! Groups the input and output data types for the
//! [`PrepareSecRequest`](super::PrepareSecRequest) state.
//!
//! The state consumes a validated CIK plus HTTP client and produces a ready-to-send request.
//! Input and output live in separate child modules to mirror that transformation.
//!
//! ## Modules
//!
//! - [`input`]: The [`PrepareSecRequestInput`] holding the validated CIK and shared HTTP client.
//! - [`output`]: The [`PrepareSecRequestOutput`] holding the prepared client and request.
//!
//! ## See Also
//!
//! - [`crate::implementations::states::extract::prepare_sec_request`]: The parent state implementation.

pub mod input;
pub mod output;

pub use input::PrepareSecRequestInput;
pub use output::PrepareSecRequestOutput;
