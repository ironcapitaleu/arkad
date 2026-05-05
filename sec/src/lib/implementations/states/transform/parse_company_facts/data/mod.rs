//! # Parse Company Facts Data Module
//!
//! This module provides the input and output data structures for the `ParseCompanyFacts` state
//! within the SEC transformation state machine. It defines types and builders for handling the
//! company facts parsing process, encapsulating both the raw JSON input and the parsed output.
//!
//! ## Modules
//! - [`input`]: Contains [`ParseCompanyFactsInput`] and related types for representing and updating the JSON input data.
//! - [`output`]: Contains [`ParseCompanyFactsOutput`] and related types for representing and updating the parsed output data.
//!
//! ## Usage
//! These types are used by the `ParseCompanyFacts` state to receive, parse, and output company data
//! as part of the SEC transformation workflow. They implement the necessary traits for integration
//! with the state machine framework, including `StateData` and custom updater patterns.
//!
//! ## See Also
//! - [`crate::implementations::states::transform::parse_company_facts`]: The parent state implementation.
//! - [`state_maschine::state_machine::state::StateData`]: Trait defining the methods and logic of how to interact with state-internal data.

pub mod input;
pub mod output;

pub use input::ParseCompanyFactsInput;
pub use output::ParseCompanyFactsOutput;
