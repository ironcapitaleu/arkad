//! # Implementations Module
//!
//! This module contains concrete implementations of state machine components for the SEC state machine library.
//! It provides real-world state, context, and data types that leverage the core traits and error types defined in the [`crate::traits`] and [`crate::error`] modules.
//!
//! ## Structure
//! - [`states`]: Contains implementations for the Extract, Transform, and Load (ETL) states used in SEC data processing pipelines. Each submodule provides concrete state logic, input/output/context data, and validation routines.
//!
//! ## Usage
//! These implementations are intended to be used as building blocks for constructing SEC-specific state machines. They demonstrate how to apply the framework's extensible traits and error handling in practice.
//!
//! ## Related Modules
//! - [`crate::traits`]: Defines the core traits for state machines, states, transitions, context, and data.
//! - [`crate::error`]: Provides strongly-typed error handling for all state machine operations.
//! - [`crate::shared`]: Shared utilities and domain types (e.g., CIK parsing) used by implementations.
//! - [`crate::prelude`]: Re-exports core traits for easy downstream use.
//!
//! ## Example
//! ```rust
//! use tokio;
//!
//!
//! #[tokio::main]
//! async fn main() {
//!     use state_maschine::prelude::State as SMState;
//!
//!     use sec::implementations::states::extract::validate_cik_format::{ValidateCikFormatInputData, ValidateCikFormatContext};
//!     use sec::implementations::states::extract::validate_cik_format::ValidateCikFormat;
//!     use sec::prelude::State;
//!
//!     let input = ValidateCikFormatInputData { raw_cik: "1234".into() };
//!     let context = ValidateCikFormatContext::default();
//!
//!     let expected_result = "0000001234";
//!
//!     let mut state = ValidateCikFormat::new(input, context);
//!     state.compute_output_data_async().await.unwrap();
//!     let result = state.get_output_data().expect("Output data should always be present in default `ValidateCikFormat` state.").validated_cik.value();
//!
//!     assert_eq!(result, expected_result);
//! }
//! ```
//!
//! See the [`states`] module for details on each concrete state implementation.

pub mod states;
