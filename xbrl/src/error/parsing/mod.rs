//! # Parsing Errors
//!
//! Error types for failures during SEC JSON parsing.

use thiserror::Error;

pub mod invalid_data_point;
pub mod invalid_json;
pub mod missing_namespace;
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

    /// A required taxonomy namespace is missing from the response.
    #[error("[MissingNamespace] Required namespace '{namespace}' not found under 'facts'")]
    MissingNamespace {
        /// The name of the missing namespace.
        namespace: String,
    },

    /// A data point within a concept could not be parsed.
    #[error("[InvalidDataPoint] Invalid data point for concept '{concept}', Reason: '{reason}'")]
    InvalidDataPoint {
        /// The concept name the data point belongs to.
        concept: String,
        /// Description of what went wrong.
        reason: String,
    },
}
