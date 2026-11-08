//! # Concept
//!
//! Provides the name of a taxonomy concept: [`ConceptName`], its [`NamespaceFamily`], and the
//! [`Namespace`] a parser reads.
//!
//! Each release of a taxonomy has its own namespace URI, for example
//! `http://fasb.org/us-gaap/2017-01-31` and `http://fasb.org/us-gaap/2026`. A fact holds the family
//! of the URI, not the URI, so one plug-in resolves the concepts of every release.

use std::fmt;

use super::invalid_value::{InvalidValue, InvalidValueReason, require_token};

/// A namespace as an instance document declares it: a prefix and a URI.
///
/// The URI is the identity, as XBRL defines it. `Namespace` is a parser input only. A parser splits
/// the URI into a [`NamespaceFamily`] and a release, and the fact holds these two.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, serde::Serialize)]
pub struct Namespace {
    prefix: String,
    uri: String,
}

impl Namespace {
    /// Creates a [`Namespace`] from a prefix and a URI.
    ///
    /// The prefix can be empty, for the default namespace of a document.
    ///
    /// # Errors
    ///
    /// Returns [`InvalidValue`] if the URI is empty or holds whitespace, or if the prefix holds
    /// whitespace.
    pub fn new(prefix: impl Into<String>, uri: impl Into<String>) -> Result<Self, InvalidValue> {
        let prefix = prefix.into();
        let uri = uri.into();
        if prefix.chars().any(char::is_whitespace) {
            return Err(InvalidValue::new(
                "Namespace",
                InvalidValueReason::ContainsWhitespace,
                prefix,
            ));
        }
        require_token("Namespace", &uri)?;
        Ok(Self { prefix, uri })
    }

    /// Returns the prefix, for example `us-gaap`.
    #[must_use]
    pub fn prefix(&self) -> &str {
        &self.prefix
    }

    /// Returns the URI, for example `http://fasb.org/us-gaap/2026`.
    #[must_use]
    pub fn uri(&self) -> &str {
        &self.uri
    }
}

/// A namespace URI without its release part, for example `us-gaap` for every
/// `http://fasb.org/us-gaap/<date>` URI.
///
/// The registry routes a fact to the plug-in that owns its family. Three kinds of text are families:
///
/// - The name of a declared family, for example `us-gaap` or `ifrs-full`.
/// - The filing ID, for the extension concepts of a filing, for example `sec:0000320193-26-000001`.
/// - The full URI or Notes `version` that no plug-in declares, for example
///   `http://xbrl.sec.gov/xyz/2027`.
///
/// # Examples
///
/// ```
/// use xbrl::core::concept::NamespaceFamily;
///
/// let text = "us-gaap";
///
/// let expected_result = "us-gaap";
///
/// let family = NamespaceFamily::new(text).expect("Given a hardcoded family without whitespace, the creation should always succeed");
/// let result = family.as_str();
///
/// assert_eq!(result, expected_result);
/// ```
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, serde::Serialize)]
pub struct NamespaceFamily(String);

impl NamespaceFamily {
    /// Creates a [`NamespaceFamily`] from its text.
    ///
    /// # Errors
    ///
    /// Returns [`InvalidValue`] if the text is empty or holds whitespace.
    pub fn new(text: impl Into<String>) -> Result<Self, InvalidValue> {
        let text = text.into();
        require_token("NamespaceFamily", &text)?;
        Ok(Self(text))
    }

    /// Returns the text of the family.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for NamespaceFamily {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// The name of a concept: a [`NamespaceFamily`] and a local name.
///
/// The alias lists and the element reference data of a plug-in match on both parts. The name holds
/// no release, so one name covers the concept in every release of its taxonomy.
///
/// # Examples
///
/// ```
/// use xbrl::core::concept::{ConceptName, NamespaceFamily};
///
/// let family = NamespaceFamily::new("us-gaap").expect("Given a hardcoded family without whitespace, the creation should always succeed");
/// let local_name = "Revenues";
///
/// let expected_result = "us-gaap:Revenues";
///
/// let concept = ConceptName::new(family, local_name).expect("Given a hardcoded local name without a colon, the creation should always succeed");
/// let result = concept.to_string();
///
/// assert_eq!(result, expected_result);
/// ```
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, serde::Serialize)]
pub struct ConceptName {
    family: NamespaceFamily,
    local_name: String,
}

impl ConceptName {
    /// Creates a [`ConceptName`] from a family and a local name.
    ///
    /// # Errors
    ///
    /// Returns [`InvalidValue`] if the local name is empty, holds whitespace, or holds a `:`.
    pub fn new(
        family: NamespaceFamily,
        local_name: impl Into<String>,
    ) -> Result<Self, InvalidValue> {
        let local_name = local_name.into();
        require_token("ConceptName", &local_name)?;
        if local_name.contains(':') {
            return Err(InvalidValue::new(
                "ConceptName",
                InvalidValueReason::ContainsSeparator(':'),
                local_name,
            ));
        }
        Ok(Self { family, local_name })
    }

    /// Returns the family, for example `us-gaap`.
    #[must_use]
    pub const fn family(&self) -> &NamespaceFamily {
        &self.family
    }

    /// Returns the local name, for example `Revenues`.
    #[must_use]
    pub fn local_name(&self) -> &str {
        &self.local_name
    }
}

impl fmt::Display for ConceptName {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}:{}", self.family, self.local_name)
    }
}

#[cfg(test)]
mod tests {
    use pretty_assertions::assert_eq;

    use super::*;

    fn family(text: &str) -> NamespaceFamily {
        NamespaceFamily::new(text).expect(
            "Given a hardcoded family without whitespace, the creation should always succeed",
        )
    }

    const fn implements_auto_traits<T: Sized + Send + Sync + Unpin>() {}
    #[test]
    const fn should_implement_auto_traits_for_namespace() {
        implements_auto_traits::<Namespace>();
    }

    #[test]
    const fn should_implement_auto_traits_for_namespace_family() {
        implements_auto_traits::<NamespaceFamily>();
    }

    #[test]
    const fn should_implement_auto_traits_for_concept_name() {
        implements_auto_traits::<ConceptName>();
    }

    #[test]
    fn should_keep_prefix_when_namespace_is_created() {
        let prefix = "us-gaap";
        let uri = "http://fasb.org/us-gaap/2026";

        let expected_result = "us-gaap";

        let namespace = Namespace::new(prefix, uri)
            .expect("Given a hardcoded URI without whitespace, the creation should always succeed");
        let result = namespace.prefix();

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_accept_empty_prefix_when_namespace_is_the_default() {
        let prefix = "";
        let uri = "http://www.xbrl.org/2003/instance";

        let expected_result = "http://www.xbrl.org/2003/instance";

        let namespace = Namespace::new(prefix, uri)
            .expect("Given a hardcoded URI without whitespace, the creation should always succeed");
        let result = namespace.uri();

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_reject_namespace_when_uri_is_empty() {
        let uri = "";

        let expected_result = Err(InvalidValue::new(
            "Namespace",
            InvalidValueReason::Empty,
            "",
        ));

        let result = Namespace::new("us-gaap", uri);

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_reject_namespace_when_prefix_holds_whitespace() {
        let prefix = "us gaap";

        let expected_result = Err(InvalidValue::new(
            "Namespace",
            InvalidValueReason::ContainsWhitespace,
            "us gaap",
        ));

        let result = Namespace::new(prefix, "http://fasb.org/us-gaap/2026");

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_accept_filing_id_when_it_is_the_family_of_an_extension() {
        let text = "sec:0000320193-26-000001";

        let expected_result = "sec:0000320193-26-000001";

        let result = family(text).to_string();

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_reject_family_when_text_is_empty() {
        let text = "";

        let expected_result = Err(InvalidValue::new(
            "NamespaceFamily",
            InvalidValueReason::Empty,
            "",
        ));

        let result = NamespaceFamily::new(text);

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_return_family_when_concept_name_is_created() {
        let concept = ConceptName::new(family("dei"), "EntityRegistrantName").expect(
            "Given a hardcoded local name without a colon, the creation should always succeed",
        );

        let expected_result = &family("dei");

        let result = concept.family();

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_return_local_name_when_concept_name_is_created() {
        let concept = ConceptName::new(family("us-gaap"), "Assets").expect(
            "Given a hardcoded local name without a colon, the creation should always succeed",
        );

        let expected_result = "Assets";

        let result = concept.local_name();

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_reject_concept_name_when_local_name_holds_a_colon() {
        let local_name = "us-gaap:Assets";

        let expected_result = Err(InvalidValue::new(
            "ConceptName",
            InvalidValueReason::ContainsSeparator(':'),
            "us-gaap:Assets",
        ));

        let result = ConceptName::new(family("us-gaap"), local_name);

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_reject_concept_name_when_local_name_is_empty() {
        let local_name = "";

        let expected_result = Err(InvalidValue::new(
            "ConceptName",
            InvalidValueReason::Empty,
            "",
        ));

        let result = ConceptName::new(family("us-gaap"), local_name);

        assert_eq!(result, expected_result);
    }
}
