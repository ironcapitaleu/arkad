//! # Transitions from `ParseCompanyFacts`
//!
//! Holds the transitions leaving the
//! [`ParseCompanyFacts`](crate::implementations::states::transform::parse_company_facts::ParseCompanyFacts)
//! state.
//!
//! ## Modules
//!
//! - [`create_financial_statements`]: Transition into the financial-statements state.

pub mod create_financial_statements;
