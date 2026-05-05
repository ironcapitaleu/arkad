//! # Create Financial Statements Data Module
//!
//! This module provides the input and output data structures for the `CreateFinancialStatements` state
//! within the SEC transform state machine. It defines types and builders for handling financial statement
//! creation, encapsulating both the company data input and the resulting output.
//!
//! ## Modules
//! - [`input`]: Contains [`CreateFinancialStatementsInput`] and related types for representing and updating the input data.
//! - [`output`]: Contains [`CreateFinancialStatementsOutput`] and related types for representing the output data.
//!
//! ## Usage
//! These types are used by the `CreateFinancialStatements` state to receive and produce financial statement data
//! as part of the SEC document transform workflow. They implement the necessary traits for integration
//! with the state machine framework, including `StateData` and custom updater patterns.
//!
//! ## See Also
//! - [`crate::implementations::states::transform::create_financial_statements`]: The parent state implementation.
//! - [`state_maschine::state_machine::state::StateData`]: Trait defining the methods and logic of how to interact with state-internal data.
//!
//! ## Examples
//! See the documentation and tests in the submodules for usage patterns.

pub mod input;
pub mod output;

pub use input::CreateFinancialStatementsInput;
pub use output::CreateFinancialStatementsOutput;
