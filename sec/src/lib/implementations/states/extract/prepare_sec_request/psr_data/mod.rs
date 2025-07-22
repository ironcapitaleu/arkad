//! # PSR Data Module
//!
//! This module provides the input and output data structures for the `PrepareSecRequest` state
//! within the SEC extraction state machine. It defines types and builders for handling the
//! SEC API client and request preparation process, encapsulating both the configuration input
//! and the prepared client/request output.
//!
//! ## Modules
//! - [`psr_input_data`]: Contains [`PrepareSecRequestInputData`] and related types for representing and updating the CIK and user agent input data.
//! - [`psr_output_data`]: Contains [`PrepareSecRequestOutputData`] and related types for representing and updating the prepared SEC client and request output data.
//!
//! ## Usage
//! These types are used by the `PrepareSecRequest` state to receive configuration data,
//! prepare SEC API client and request objects, and output them as part of the SEC document
//! extraction workflow. They implement the necessary traits for integration with the state
//! machine framework, including `StateData` and custom updater patterns.
//!
//! ## See Also
//! - [`crate::implementations::states::extract::prepare_sec_request`]: The parent state implementation.
//! - [`state_maschine::state_machine::state::StateData`]: Trait defining the methods and logic of how to interact with state-internal data.
//!
//! ## Examples
//! See the documentation and tests in the submodules for usage patterns.

pub mod psr_input_data;
pub mod psr_output_data;

pub use psr_input_data::PrepareSecRequestInputData;
pub use psr_output_data::PrepareSecRequestOutputData;
