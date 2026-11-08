//! # Generator Error
//!
//! Provides [`GeneratorError`], the error of a run of the generator.

use std::io;
use std::path::PathBuf;

/// Separates the failures of a run: a file that the generator cannot read or write, and content
/// that does not follow the XBRL rules the generator relies on.
#[derive(Debug, thiserror::Error)]
pub enum GeneratorError {
    /// The generator cannot read a vendored file.
    #[error("[FailedFileRead] Failed to read a vendored file, Reason: '{source}', Path: '{}'", path.display())]
    FailedFileRead {
        /// The path of the file.
        path: PathBuf,
        /// The error of the read.
        #[source]
        source: io::Error,
    },

    /// The generator cannot write the generated file.
    #[error("[FailedFileWrite] Failed to write the generated file, Reason: '{source}', Path: '{}'", path.display())]
    FailedFileWrite {
        /// The path of the file.
        path: PathBuf,
        /// The error of the write.
        #[source]
        source: io::Error,
    },

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

    /// A locator points to something that is not a concept of the form `<family>_<name>`.
    #[error("[InvalidConceptReference] Locator does not point to a concept, Href: '{href}', Path: '{}'", path.display())]
    InvalidConceptReference {
        /// The path of the file.
        path: PathBuf,
        /// The `xlink:href` of the locator.
        href: String,
    },

    /// An arc names a `from` or `to` label that no locator or resource of its link carries.
    #[error("[UnresolvedArcLabel] No locator or resource carries the label of an arc, Label: '{label}', Path: '{}'", path.display())]
    UnresolvedArcLabel {
        /// The path of the file.
        path: PathBuf,
        /// The label that the arc names.
        label: String,
    },

    /// A concept of the element schema has no standard label.
    #[error("[MissingLabel] Standard label not found for concept '{concept}'")]
    MissingLabel {
        /// The concept, as `<family>:<name>`.
        concept: String,
    },

    /// A concept has more than one standard label.
    #[error("[RepeatedLabel] Concept '{concept}' has more than one standard label")]
    RepeatedLabel {
        /// The concept, as `<family>:<name>`.
        concept: String,
    },
}

#[cfg(test)]
mod tests {
    use pretty_assertions::assert_eq;

    use super::*;

    const fn implements_auto_traits<T: Sized + Send + Sync + Unpin>() {}
    #[test]
    const fn should_implement_auto_traits_for_generator_error() {
        implements_auto_traits::<GeneratorError>();
    }

    #[test]
    fn should_name_concept_when_label_is_missing() {
        let error = GeneratorError::MissingLabel {
            concept: "us-gaap:Assets".to_string(),
        };

        let expected_result =
            "[MissingLabel] Standard label not found for concept 'us-gaap:Assets'";

        let result = error.to_string();

        assert_eq!(result, expected_result);
    }
}
