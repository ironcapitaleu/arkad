//! # Generator Error
//!
//! Provides [`GeneratorError`], the error of a run of the generator.

use std::path::PathBuf;

/// The error of a run of the generator: a vendored file that it cannot read, a generated file that
/// it cannot write, or content that breaks an XBRL rule it relies on.
///
/// The errors of `std::io` and `roxmltree` are kept as text, so a caller that walks `source()`
/// meets their message once.
#[derive(Debug, thiserror::Error)]
pub enum GeneratorError {
    /// The generator cannot read a vendored file.
    #[error("[FailedFileRead] Failed to read a vendored file, Reason: '{reason}', Path: '{}'", path.display())]
    FailedFileRead {
        /// The path of the file.
        path: PathBuf,
        /// The message of the read error.
        reason: String,
    },

    /// The generator cannot write the generated file.
    #[error("[FailedFileWrite] Failed to write the generated file, Reason: '{reason}', Path: '{}'", path.display())]
    FailedFileWrite {
        /// The path of the file.
        path: PathBuf,
        /// The message of the write error.
        reason: String,
    },

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

    /// A locator points to something that is not a concept of the form `<family>_<name>`.
    #[error("[InvalidConceptReference] Locator does not point to a concept, Href: '{href}', Line: '{line}', Path: '{}'", path.display())]
    InvalidConceptReference {
        /// The path of the file.
        path: PathBuf,
        /// The line of the locator in the file.
        line: u32,
        /// The `xlink:href` of the locator.
        href: String,
    },

    /// An arc names a `from` or `to` label that no locator or resource of its link carries.
    #[error("[UnresolvedArcLabel] No locator or resource carries the label of an arc, Label: '{label}', Line: '{line}', Path: '{}'", path.display())]
    UnresolvedArcLabel {
        /// The path of the file.
        path: PathBuf,
        /// The line of the arc in the file.
        line: u32,
        /// The label that the arc names.
        label: String,
    },

    /// Two locators of one link share a label and point to different concepts.
    #[error("[AmbiguousLocatorLabel] Two locators share a label and differ in concept, Label: '{label}', Line: '{line}', Path: '{}'", path.display())]
    AmbiguousLocatorLabel {
        /// The path of the file.
        path: PathBuf,
        /// The line of the second locator in the file.
        line: u32,
        /// The label that the locators share.
        label: String,
    },

    /// A concept has more than one standard label.
    #[error("[RepeatedLabel] Concept '{concept}' has more than one standard label, Line: '{line}', Path: '{}'", path.display())]
    RepeatedLabel {
        /// The path of the file.
        path: PathBuf,
        /// The line of the arc that gives the second standard label.
        line: u32,
        /// The concept, as `<family>:<name>`.
        concept: String,
    },

    /// A concept of the element schema has no standard label.
    #[error("[MissingLabel] Standard label not found for concept '{concept}'")]
    MissingLabel {
        /// The concept, as `<family>:<name>`.
        concept: String,
    },

    /// Two arcs of one network join the same parent and child with different weights.
    #[error(
        "[ConflictingWeight] Arc '{parent}' to '{child}' has two weights, Weights: '{first}' and '{second}', Role: '{role}'"
    )]
    ConflictingWeight {
        /// The role of the network.
        role: String,
        /// The parent, as `<family>:<name>`.
        parent: String,
        /// The child, as `<family>:<name>`.
        child: String,
        /// The first weight.
        first: String,
        /// The second weight.
        second: String,
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
