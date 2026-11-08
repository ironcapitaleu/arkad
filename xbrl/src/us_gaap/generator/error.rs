//! # Generator Error
//!
//! Provides [`GeneratorError`], the error of a read of a vendored file.

use std::path::PathBuf;

/// The error of a read of a vendored FASB file.
///
/// The parser error of `roxmltree` is kept as text, so a caller that walks `source()` meets its
/// message once.
#[derive(Debug, thiserror::Error)]
pub enum GeneratorError {
    /// A vendored file is not well-formed XML.
    #[error("[InvalidXml] Failed to parse XML, Reason: '{reason}', Path: '{}'", path.display())]
    InvalidXml {
        /// The path of the file.
        path: PathBuf,
        /// The message of the parser.
        reason: String,
    },

    /// An XML element lacks an attribute that the generator needs.
    #[error("[MissingAttribute] Required attribute '{attribute}' not found on element '{element}', Line: '{line}', Path: '{}'", path.display())]
    MissingAttribute {
        /// The path of the file.
        path: PathBuf,
        /// The line of the XML element in the file.
        line: u32,
        /// The local name of the XML element.
        element: String,
        /// The local name of the missing attribute.
        attribute: &'static str,
    },

    /// An attribute holds a value outside the values that XBRL allows.
    #[error("[InvalidAttributeValue] Attribute '{attribute}' holds an unknown value, Value: '{value}', Line: '{line}', Path: '{}'", path.display())]
    InvalidAttributeValue {
        /// The path of the file.
        path: PathBuf,
        /// The line of the XML element in the file.
        line: u32,
        /// The local name of the attribute.
        attribute: &'static str,
        /// The value of the attribute.
        value: String,
    },
}

#[cfg(test)]
mod tests {
    use super::*;

    const fn implements_auto_traits<T: Sized + Send + Sync + Unpin>() {}
    #[test]
    const fn should_implement_auto_traits_for_generator_error() {
        implements_auto_traits::<GeneratorError>();
    }
}
