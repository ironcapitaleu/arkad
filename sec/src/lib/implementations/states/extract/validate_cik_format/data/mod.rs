//! # Validate CIK Format Data
//!
//! Groups the input and output data types for the
//! [`ValidateCikFormat`](super::ValidateCikFormat) state.
//!
//! The state consumes a raw CIK string and produces a validated [`Cik`](crate::shared::cik::Cik).
//! These two roles are kept in separate child modules so the unvalidated and validated forms
//! remain distinct types and cannot be confused at a transition boundary.
//!
//! ## Modules
//!
//! - [`input`]: The [`ValidateCikFormatInput`] holding the raw, unvalidated CIK string.
//! - [`output`]: The [`ValidateCikFormatOutput`] holding the validated, normalized CIK.
//!
//! ## See Also
//!
//! - [`crate::implementations::states::extract::validate_cik_format`]: The parent state implementation.
//! - [`crate::shared::cik`]: The CIK parsing and validation utilities these types build on.

pub mod input;
pub mod output;

pub use input::ValidateCikFormatInput;
pub use output::ValidateCikFormatOutput;
