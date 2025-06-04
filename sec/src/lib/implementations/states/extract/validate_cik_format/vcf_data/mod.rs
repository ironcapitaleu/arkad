//! # VCF Data Module
//!
//! This module provides the input and output data structures for the `ValidateCikFormat` state
//! within the SEC extraction state machine. It defines types and builders for handling the
//! Central Index Key (CIK) validation process, encapsulating both the raw input and the validated output.
//!
//! ## Modules
//! - [`vcf_input_data`]: Contains [`ValidateCikFormatInputData`] and related types for representing and updating the CIK input data.
//! - [`vcf_output_data`]: Contains [`ValidateCikFormatOutputData`] and related types for representing and updating the validated CIK output data.
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

pub mod vcf_input_data;
pub mod vcf_output_data;

pub use vcf_input_data::ValidateCikFormatInputData;
pub use vcf_output_data::ValidateCikFormatOutputData;
