//! # Shared Utilities and Domain Types
//!
//! This module contains shared utilities and domain-specific types used throughout the SEC state machine library.
//! These components provide reusable logic and strongly-typed representations for common SEC concepts, such as the Central Index Key (CIK).
//!
//! ## Modules
//! - [`cik`]: Provides parsing, validation, and formatting utilities for SEC Central Index Keys (CIKs).
//! - [`sec_client`]: Provides utilities for creating and managing SEC API clients, including user agent handling.
//! - [`sec_request`]: Provides utilities for constructing SEC API requests, ensuring proper URL formatting.
//! - [`sec_response`]: Provides utilities for handling HTTP responses from SEC endpoints.
//! - [`json_response`]: Provides validated wrappers around SEC responses to ensure valid JSON data.
//! - [`user_agent`]: Provides utilities for creating and validating SEC-compliant user agent strings.
//!
//! ## Usage
//! The types and functions in this module are intended to be used by state implementations, error handling, and data validation routines across the library.
//! For example, CIK parsing and validation is leveraged by the `extract/validate_cik_format` state and its input/output data modules.
//!
//! ## See Also
//! - [`crate::implementations`]: Concrete state and state machine implementations that use these shared utilities.
//! - [`crate::error`]: Error types that may reference shared domain types for detailed error reporting.

pub mod cik;
pub mod json_response;
pub mod sec_client;
pub mod sec_request;
pub mod sec_response;
pub mod user_agent;
