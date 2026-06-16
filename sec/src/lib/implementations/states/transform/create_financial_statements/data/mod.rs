//! # Create Financial Statements Data
//!
//! Groups the input and output data types for the
//! [`CreateFinancialStatements`](super::CreateFinancialStatements) state.
//!
//! The state consumes parsed company data and produces financial statements. Input and output
//! live in separate child modules to mirror that transformation.
//!
//! ## Modules
//!
//! - [`input`]: The [`CreateFinancialStatementsInput`] holding the parsed company data.
//! - [`output`]: The [`CreateFinancialStatementsOutput`] placeholder for the produced statements.
//!
//! ## See Also
//!
//! - [`crate::implementations::states::transform::create_financial_statements`]: The parent state implementation.

pub mod input;
pub mod output;

pub use input::CreateFinancialStatementsInput;
pub use output::CreateFinancialStatementsOutput;
