//! # State Implementations Module
//!
//! This module organizes and exposes concrete state implementations used within the SEC state machine library.
//! Each submodule represents a logical phase or operation in the state machine's workflow for SEC filings processing.
//!
//! ## Submodules
//! - [`extract`]: Contains states responsible for extracting and validating data from raw SEC filings (e.g., CIK format validation).
//! - [`transform`]: Contains states that transform extracted data into normalized or enriched forms for downstream processing.
//! - [`load`]: Contains states that handle loading or persisting processed data into target systems or storage.
//!
//! These modules are designed to be composed within state machines, supporting robust, testable, and extensible workflows.
//!
//! See the documentation for each submodule for details on their specific states and responsibilities.
pub mod extract;
pub mod load;
pub mod transform;
