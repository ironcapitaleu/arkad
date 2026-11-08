//! # Generator Error
//!
//! Provides [`GeneratorError`], the error of a read of a vendored file.

use std::path::PathBuf;

/// Separates the failures of a read: content that is not XML, and content that does not follow
/// the XBRL rules the generator relies on.
#[derive(Debug, thiserror::Error)]
pub enum GeneratorError {
    /// A vendored file is not well-formed XML.
    #[error("[InvalidXml] Failed to parse XML, Reason: '{source}', Path: '{}'", path.display())]
    InvalidXml {
        /// The path of the file.
        path: PathBuf,
        /// The error of the parser.
        #[source]
        source: roxmltree::Error,
    },

    /// An XML element lacks an attribute that the generator needs.
    #[error("[MissingAttribute] Required attribute '{attribute}' not found on element '{element}', Path: '{}'", path.display())]
    MissingAttribute {
        /// The path of the file.
        path: PathBuf,
        /// The local name of the XML element.
        element: String,
        /// The local name of the missing attribute.
        attribute: &'static str,
    },

    /// An attribute holds a value outside the values that XBRL allows.
    #[error("[InvalidAttributeValue] Attribute '{attribute}' holds an unknown value, Value: '{value}', Path: '{}'", path.display())]
    InvalidAttributeValue {
        /// The path of the file.
        path: PathBuf,
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
