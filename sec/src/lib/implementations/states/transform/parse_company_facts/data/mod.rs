//! # Parse Company Facts Data
//!
//! Groups the input and output data types for the
//! [`ParseCompanyFacts`](super::ParseCompanyFacts) state.
//!
//! The state consumes the raw Company Facts JSON and produces structured
//! [`CompanyData`](crate::shared::financial::company_data::CompanyData). The two roles live
//! in separate child modules to mirror that transformation.
//!
//! ## Modules
//!
//! - [`input`]: The [`ParseCompanyFactsInput`] holding the raw JSON response.
//! - [`output`]: The [`ParseCompanyFactsOutput`] holding the parsed company data.
//!
//! ## See Also
//!
//! - [`crate::implementations::states::transform::parse_company_facts`]: The parent state implementation.

pub mod input;
pub mod output;

pub use input::ParseCompanyFactsInput;
pub use output::ParseCompanyFactsOutput;
