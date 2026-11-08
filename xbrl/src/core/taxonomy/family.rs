//! # Family Declaration
//!
//! Provides [`FamilyDeclaration`], how a plug-in declares a namespace family, and [`UriPattern`],
//! where the release sits in a namespace URI.

use std::fmt;

use crate::core::concept::NamespaceFamily;
use crate::core::fact::TaxonomyVersion;
use crate::core::invalid_value::{InvalidValue, InvalidValueReason, require_token};

/// The placeholder for the release part in a [`UriPattern`].
pub const RELEASE_PLACEHOLDER: &str = "{release}";

/// A namespace URI with the placeholder `{release}` where the release sits, for example
/// `http://fasb.org/us-gaap/{release}` or `https://xbrl.ifrs.org/taxonomy/{release}/ifrs-full`.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, serde::Serialize)]
pub struct UriPattern {
    before: String,
    after: String,
}

impl UriPattern {
    /// Creates a [`UriPattern`] from its text.
    ///
    /// # Errors
    ///
    /// Returns [`InvalidValue`] if the text is empty, holds whitespace, or does not hold
    /// `{release}` exactly once.
    pub fn new(text: &str) -> Result<Self, InvalidValue> {
        require_token("UriPattern", text)?;
        let (before, after) = text.split_once(RELEASE_PLACEHOLDER).ok_or_else(|| {
            InvalidValue::new(
                "UriPattern",
                InvalidValueReason::MissingPlaceholder(RELEASE_PLACEHOLDER),
                text,
            )
        })?;
        if after.contains(RELEASE_PLACEHOLDER) {
            return Err(InvalidValue::new(
                "UriPattern",
                InvalidValueReason::MissingPlaceholder(RELEASE_PLACEHOLDER),
                text,
            ));
        }
        Ok(Self {
            before: before.to_string(),
            after: after.to_string(),
        })
    }

    /// Returns the year of the release if the URI matches the pattern.
    ///
    /// The release part must start with four digits and hold no `/`. So `2017-01-31` and `2017`
    /// both give `2017`.
    #[must_use]
    pub fn year_of<'a>(&self, uri: &'a str) -> Option<&'a str> {
        let release = uri.strip_prefix(&self.before)?.strip_suffix(&self.after)?;
        if release.contains('/') {
            return None;
        }
        leading_year(release)
    }

    /// Returns `true` if one URI can match both patterns.
    ///
    /// The check is conservative: it compares the text before and after the placeholder, and does
    /// not check the release part.
    #[must_use]
    pub fn can_match_one_uri(&self, other: &Self) -> bool {
        (self.before.starts_with(&other.before) || other.before.starts_with(&self.before))
            && (self.after.ends_with(&other.after) || other.after.ends_with(&self.after))
    }
}

impl fmt::Display for UriPattern {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}{RELEASE_PLACEHOLDER}{}", self.before, self.after)
    }
}

/// How a plug-in declares one namespace family.
///
/// The declaration names the URI patterns of the family, the prefix of its Notes `version`, and
/// whether the plug-in resolves the facts of the family. The registry uses it to split a URI or a
/// Notes `version` into the family and a [`TaxonomyVersion`], and to route facts.
///
/// A plug-in can declare a family it does not resolve. The split then works, and the registry
/// routes no fact of the family to that plug-in. `us_gaap` declares the other SEC taxonomies this
/// way, for example `ecd`, `country`, and `currency`.
///
/// # Examples
///
/// ```
/// use xbrl::core::concept::NamespaceFamily;
/// use xbrl::core::taxonomy::family::{FamilyDeclaration, UriPattern};
///
/// let family = NamespaceFamily::new("us-gaap").expect("Given a hardcoded family without whitespace, the creation should always succeed");
/// let pattern = UriPattern::new("http://fasb.org/us-gaap/{release}").expect("Given a hardcoded pattern with one placeholder, the creation should always succeed");
/// let declaration = FamilyDeclaration::new(family, vec![pattern], "us-gaap", true).expect("Given a hardcoded declaration, the creation should always succeed");
/// let uri = "http://fasb.org/us-gaap/2017-01-31";
///
/// let expected_result = Some("us-gaap/2017");
///
/// let version = declaration.version_of_uri(uri);
/// let result = version.as_ref().map(|version| version.as_str());
///
/// assert_eq!(result, expected_result);
/// ```
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, serde::Serialize)]
pub struct FamilyDeclaration {
    family: NamespaceFamily,
    uri_patterns: Vec<UriPattern>,
    notes_version_prefix: String,
    resolves: bool,
}

impl FamilyDeclaration {
    /// Creates a [`FamilyDeclaration`] and sorts its URI patterns.
    ///
    /// # Errors
    ///
    /// Returns [`InvalidValue`] if the list of patterns is empty, or if the Notes `version` prefix
    /// is empty, holds whitespace, or holds a `/`.
    pub fn new(
        family: NamespaceFamily,
        mut uri_patterns: Vec<UriPattern>,
        notes_version_prefix: impl Into<String>,
        resolves: bool,
    ) -> Result<Self, InvalidValue> {
        let notes_version_prefix = notes_version_prefix.into();
        if uri_patterns.is_empty() {
            return Err(InvalidValue::new(
                "FamilyDeclaration",
                InvalidValueReason::MissingPart("URI pattern"),
                family.as_str(),
            ));
        }
        require_token("FamilyDeclaration", &notes_version_prefix)?;
        if notes_version_prefix.contains('/') {
            return Err(InvalidValue::new(
                "FamilyDeclaration",
                InvalidValueReason::ContainsSeparator('/'),
                notes_version_prefix,
            ));
        }
        uri_patterns.sort();
        uri_patterns.dedup();
        Ok(Self {
            family,
            uri_patterns,
            notes_version_prefix,
            resolves,
        })
    }

    /// Returns the declared family.
    #[must_use]
    pub const fn family(&self) -> &NamespaceFamily {
        &self.family
    }

    /// Returns the URI patterns, sorted.
    #[must_use]
    pub fn uri_patterns(&self) -> &[UriPattern] {
        &self.uri_patterns
    }

    /// Returns the prefix of the Notes `version`, for example `us-gaap` in `us-gaap/2017`.
    #[must_use]
    pub fn notes_version_prefix(&self) -> &str {
        &self.notes_version_prefix
    }

    /// Returns `true` if the plug-in resolves the facts of the family.
    #[must_use]
    pub const fn resolves(&self) -> bool {
        self.resolves
    }

    /// Returns `true` if both declarations name the same family, the same URI patterns, and the
    /// same Notes `version` prefix. Whether a plug-in resolves the family does not count.
    #[must_use]
    pub fn agrees_with(&self, other: &Self) -> bool {
        self.family == other.family
            && self.uri_patterns == other.uri_patterns
            && self.notes_version_prefix == other.notes_version_prefix
    }

    /// Returns `true` if two declarations of different families share a Notes `version` prefix, or
    /// have URI patterns that can match one URI. Then the split of a URI or a Notes `version` would
    /// depend on the order of the families.
    #[must_use]
    pub fn overlaps(&self, other: &Self) -> bool {
        self.family != other.family
            && (self.notes_version_prefix == other.notes_version_prefix
                || self.uri_patterns.iter().any(|pattern| {
                    other
                        .uri_patterns
                        .iter()
                        .any(|other_pattern| pattern.can_match_one_uri(other_pattern))
                }))
    }

    /// Returns the taxonomy version, for example `us-gaap/2017`, if a pattern matches the URI.
    #[must_use]
    pub fn version_of_uri(&self, uri: &str) -> Option<TaxonomyVersion> {
        let year = self
            .uri_patterns
            .iter()
            .find_map(|pattern| pattern.year_of(uri))?;
        self.version(year)
    }

    /// Returns the taxonomy version, for example `us-gaap/2017`, if the Notes `version` has the
    /// declared prefix and a year, as in `us-gaap/2017`.
    #[must_use]
    pub fn version_of_notes(&self, notes_version: &str) -> Option<TaxonomyVersion> {
        let (prefix, release) = notes_version.split_once('/')?;
        if prefix != self.notes_version_prefix || release.contains('/') {
            return None;
        }
        self.version(leading_year(release)?)
    }

    fn version(&self, year: &str) -> Option<TaxonomyVersion> {
        TaxonomyVersion::new(format!("{}/{year}", self.family)).ok()
    }
}

fn leading_year(release: &str) -> Option<&str> {
    let year = release.get(..4)?;
    year.bytes()
        .all(|byte| byte.is_ascii_digit())
        .then_some(year)
}

#[cfg(test)]
mod tests {
    use pretty_assertions::assert_eq;

    use super::*;

    fn pattern(text: &str) -> UriPattern {
        UriPattern::new(text).expect(
            "Given a hardcoded pattern with one placeholder, the creation should always succeed",
        )
    }

    fn declaration(
        family: &str,
        patterns: &[&str],
        prefix: &str,
        resolves: bool,
    ) -> FamilyDeclaration {
        let family = NamespaceFamily::new(family).expect(
            "Given a hardcoded family without whitespace, the creation should always succeed",
        );
        let patterns = patterns.iter().map(|text| pattern(text)).collect();
        FamilyDeclaration::new(family, patterns, prefix, resolves)
            .expect("Given a hardcoded declaration, the creation should always succeed")
    }

    fn version(text: &str) -> TaxonomyVersion {
        TaxonomyVersion::new(text).expect(
            "Given a hardcoded version without whitespace, the creation should always succeed",
        )
    }

    const fn implements_auto_traits<T: Sized + Send + Sync + Unpin>() {}
    #[test]
    const fn should_implement_auto_traits_for_family_declaration() {
        implements_auto_traits::<FamilyDeclaration>();
    }

    #[test]
    const fn should_implement_auto_traits_for_uri_pattern() {
        implements_auto_traits::<UriPattern>();
    }

    #[test]
    fn should_read_year_when_release_sits_in_the_middle_of_the_uri() {
        let ifrs = declaration(
            "ifrs-full",
            &["https://xbrl.ifrs.org/taxonomy/{release}/ifrs-full"],
            "ifrs-full",
            true,
        );
        let uri = "https://xbrl.ifrs.org/taxonomy/2023-03-23/ifrs-full";

        let expected_result = Some(version("ifrs-full/2023"));

        let result = ifrs.version_of_uri(uri);

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_give_one_version_when_uri_and_notes_name_one_release() {
        let us_gaap = declaration(
            "us-gaap",
            &["http://fasb.org/us-gaap/{release}"],
            "us-gaap",
            true,
        );
        let notes_version = "us-gaap/2017";

        let expected_result = Some(version("us-gaap/2017"));

        let result = us_gaap.version_of_notes(notes_version);

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_read_year_when_uri_holds_a_full_release_date() {
        let us_gaap = declaration(
            "us-gaap",
            &["http://fasb.org/us-gaap/{release}"],
            "us-gaap",
            true,
        );
        let uri = "http://fasb.org/us-gaap/2017-01-31";

        let expected_result = Some(version("us-gaap/2017"));

        let result = us_gaap.version_of_uri(uri);

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_overlap_when_two_families_share_a_notes_prefix() {
        let ifrs_full = declaration(
            "ifrs-full",
            &["https://xbrl.ifrs.org/taxonomy/{release}/ifrs-full"],
            "ifrs",
            true,
        );
        let ifrs = declaration(
            "ifrs",
            &["https://xbrl.ifrs.org/taxonomy/{release}/ifrs"],
            "ifrs",
            false,
        );

        let expected_result = true;

        let result = ifrs_full.overlaps(&ifrs);

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_overlap_when_patterns_of_two_families_can_match_one_uri() {
        let narrow = declaration("dei", &["http://xbrl.sec.gov/dei/{release}"], "dei", true);
        let wide = declaration("sec", &["http://xbrl.sec.gov/{release}"], "sec", false);

        let expected_result = true;

        let result = narrow.overlaps(&wide);

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_not_overlap_when_patterns_differ_only_after_the_release() {
        let ifrs_full = declaration(
            "ifrs-full",
            &["https://xbrl.ifrs.org/taxonomy/{release}/ifrs-full"],
            "ifrs-full",
            true,
        );
        let ifrs_smes = declaration(
            "ifrs-smes",
            &["https://xbrl.ifrs.org/taxonomy/{release}/ifrs-smes"],
            "ifrs-smes",
            true,
        );

        let expected_result = false;

        let result = ifrs_full.overlaps(&ifrs_smes);

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_return_none_when_uri_matches_no_pattern() {
        let us_gaap = declaration(
            "us-gaap",
            &["http://fasb.org/us-gaap/{release}"],
            "us-gaap",
            true,
        );
        let uri = "http://xbrl.sec.gov/dei/2024";

        let expected_result = None;

        let result = us_gaap.version_of_uri(uri);

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_return_none_when_release_part_has_no_year() {
        let us_gaap = declaration(
            "us-gaap",
            &["http://fasb.org/us-gaap/{release}"],
            "us-gaap",
            true,
        );
        let uri = "http://fasb.org/us-gaap/latest";

        let expected_result = None;

        let result = us_gaap.version_of_uri(uri);

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_return_none_when_notes_prefix_differs() {
        let us_gaap = declaration(
            "us-gaap",
            &["http://fasb.org/us-gaap/{release}"],
            "us-gaap",
            true,
        );
        let notes_version = "ecd/2023";

        let expected_result = None;

        let result = us_gaap.version_of_notes(notes_version);

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_agree_when_only_the_resolve_flag_differs() {
        let resolving = declaration(
            "country",
            &["http://xbrl.sec.gov/country/{release}"],
            "country",
            true,
        );
        let declaring = declaration(
            "country",
            &["http://xbrl.sec.gov/country/{release}"],
            "country",
            false,
        );

        let expected_result = true;

        let result = resolving.agrees_with(&declaring);

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_disagree_when_notes_prefix_differs() {
        let first = declaration(
            "country",
            &["http://xbrl.sec.gov/country/{release}"],
            "country",
            false,
        );
        let second = declaration(
            "country",
            &["http://xbrl.sec.gov/country/{release}"],
            "ctry",
            false,
        );

        let expected_result = false;

        let result = first.agrees_with(&second);

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_disagree_when_uri_patterns_differ() {
        let first = declaration(
            "country",
            &["http://xbrl.sec.gov/country/{release}"],
            "country",
            false,
        );
        let second = declaration(
            "country",
            &["https://xbrl.sec.gov/country/{release}"],
            "country",
            false,
        );

        let expected_result = false;

        let result = first.agrees_with(&second);

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_reject_pattern_when_placeholder_is_missing() {
        let text = "http://fasb.org/us-gaap/2026";

        let expected_result = Err(InvalidValue::new(
            "UriPattern",
            InvalidValueReason::MissingPlaceholder(RELEASE_PLACEHOLDER),
            "http://fasb.org/us-gaap/2026",
        ));

        let result = UriPattern::new(text);

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_reject_declaration_when_it_has_no_pattern() {
        let family = NamespaceFamily::new("dei").expect(
            "Given a hardcoded family without whitespace, the creation should always succeed",
        );

        let expected_result = Err(InvalidValue::new(
            "FamilyDeclaration",
            InvalidValueReason::MissingPart("URI pattern"),
            "dei",
        ));

        let result = FamilyDeclaration::new(family, vec![], "dei", true);

        assert_eq!(result, expected_result);
    }
}
