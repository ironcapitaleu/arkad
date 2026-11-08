//! # Parsing Errors
//!
//! Error types for failures during the parse of SEC JSON and of Notes rows.

use thiserror::Error;

use crate::core::invalid_value::InvalidValue;

pub mod invalid_data_point;
pub mod invalid_json;
pub mod missing_top_level_key;

/// Specific parsing failure variants.
#[derive(Debug, PartialEq, Eq, Error)]
pub enum ParseErrorKind {
    /// The input cannot be parsed as valid JSON (malformed bytes).
    #[error("[InvalidJson] Failed to parse JSON body, Reason: '{reason}'")]
    InvalidJson {
        /// Description of the deserialization failure.
        reason: String,
    },

    /// The JSON is valid but does not match the expected schema. Keys may be missing or their values may have the wrong type.
    #[error("[UnexpectedJsonSchema] Unexpected JSON schema, Reason: '{reason}'")]
    UnexpectedJsonSchema {
        /// Description of what was expected vs what was found.
        reason: String,
    },

    /// A required top-level key is missing from the response.
    #[error("[MissingTopLevelKey] Required key '{key}' not found in response")]
    MissingTopLevelKey {
        /// The name of the missing key.
        key: String,
    },

    /// A data point within a concept could not be parsed.
    #[error("[InvalidDataPoint] Invalid data point for concept '{concept}', Reason: '{reason}'")]
    InvalidDataPoint {
        /// The concept name the data point belongs to.
        concept: String,
        /// Description of what went wrong.
        reason: String,
    },

    /// A key or a value of a data point breaks a rule of its core type, for example an end before
    /// the start.
    #[error(
        "[InvalidDataPointValue] Invalid value in a data point of concept '{concept}', Caused by: {source}"
    )]
    InvalidDataPointValue {
        /// The concept name the data point belongs to.
        concept: String,
        /// The rule the value breaks.
        #[source]
        source: InvalidValue,
    },

    /// The `sub` row of a Notes filing breaks a rule of a core type, so the filing has no
    /// [`Filing`](crate::core::filing::Filing).
    #[error(
        "[InvalidSubmissionRow] Invalid sub row of the Notes filing '{adsh}', Caused by: {source}"
    )]
    InvalidSubmissionRow {
        /// The accession number of the filing.
        adsh: String,
        /// The rule the row breaks.
        #[source]
        source: InvalidValue,
    },
}
