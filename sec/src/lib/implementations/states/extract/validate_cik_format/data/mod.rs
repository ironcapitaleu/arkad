//! # VCF Data Module
//!
//! This module provides the input and output data structures for the `ValidateCikFormat` state
//! within the SEC extraction state machine. It defines types and builders for handling the
//! Central Index Key (CIK) validation process, encapsulating both the raw input and the validated output.
//!
//! ## Modules
//! - [`input`]: Contains [`ValidateCikFormatInput`] and related types for representing and updating the CIK input data.
//! - [`output`]: Contains [`ValidateCikFormatOutput`] and related types for representing and updating the validated CIK output data.
//!
//! ## Usage
//! These types are used by the `ValidateCikFormat` state to receive, validate, and output CIK data
//! as part of the SEC document extraction workflow. They implement the necessary traits for integration
//! with the state machine framework, including `StateData` and custom updater patterns.
//!
//! ## See Also
//! - [`crate::implementations::states::extract::validate_cik_format`]: The parent state implementation.
//! - [`state_maschine::state_machine::state::StateData`]: Trait defining the methods and logic of how to interact with state-internal data.
//!
//! ## Examples
//! See the documentation and tests in the submodules for usage patterns.

pub mod input;
pub mod output;

pub use input::ValidateCikFormatInput;
pub use output::ValidateCikFormatOutput;
