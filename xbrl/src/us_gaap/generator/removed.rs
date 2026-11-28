//! # Removed Elements
//!
//! Reads the US GAAP elements that earlier releases define and the release in use does not.
//!
//! The source files are the element schemas and label linkbases of earlier releases, as the FASB
//! serves them. They are not vendored. `taxonomy/us-gaap-removed/MANIFEST` names the URL and the
//! SHA-256 of each one. A developer downloads them into a folder, and [`generate_removed`] reads
//! them from there. It refuses a file whose SHA-256 differs from the `MANIFEST`.
//!
//! An element counts if it is a concept of an earlier schema and not of the US GAAP schema of the
//! release in use. Its row takes the data of the newest release that defines it.

use std::collections::{BTreeMap, BTreeSet, HashMap};
use std::fs;
use std::path::{Path, PathBuf};

use sha2::{Digest, Sha256};

use super::error::GeneratorError;
use super::linkbase::{self, ConceptRef};
use super::render::{self, RemovedElementLine};
use super::schema::{self, Concept};
use super::{RELEASE_ID, US_GAAP_SCHEMA, hex, read};

/// The `MANIFEST` of the source files, relative to the `xbrl` crate.
pub const REMOVED_MANIFEST_FILE: &str = "taxonomy/us-gaap-removed/MANIFEST";

/// The generated file, relative to the `xbrl` crate.
///
/// `.gitattributes` marks this path as generated. Change both together.
pub const REMOVED_OUTPUT_FILE: &str = "src/us_gaap/removed.rs";

/// The start of each source URL. The rest of the URL is the path of the file in the download
/// folder.
pub const SOURCE_HOST: &str = "https://xbrl.fasb.org/";

/// The family of the removed elements, which is the first segment of each source path.
const FAMILY: &str = "us-gaap";

/// The folder of the element files of a release, the third segment of each source path.
const ELEMENT_FOLDER: &str = "elts";

/// The part of a file name that marks a label linkbase.
const LABEL_MARKER: &str = "-lab-";

/// The text before the deprecation date at the end of a standard label.
const DEPRECATION_PREFIX: &str = "(Deprecated ";

/// The month and day of each release up to 2021. A full deprecation date ends with it.
const RELEASE_MONTH_AND_DAY: &str = "-01-31";

/// The number of digits of a year.
const YEAR_LENGTH: usize = 4;

/// The number of lowercase hex digits of a SHA-256.
const SHA256_HEX_LENGTH: usize = 64;

/// The element schema and the label linkbase of one release, as the `MANIFEST` names them.
#[derive(Debug, Default)]
struct SourceRelease {
    /// The element schema: its path below the download folder and its SHA-256.
    schema: Option<(PathBuf, String)>,
    /// The label linkbase: its path below the download folder and its SHA-256.
    labels: Option<(PathBuf, String)>,
}

/// Returns the Rust source of `us_gaap/removed.rs`, read from the source files in `sources`.
///
/// `sources` holds each file at the path of its URL after [`SOURCE_HOST`], for example
/// `us-gaap/2020/elts/us-gaap-2020-01-31.xsd`. The US GAAP schema of the release in use comes from
/// the vendored folder below `xbrl_crate`.
///
/// # Errors
///
/// - [`GeneratorError::UnexpectedChecksum`] if a source file differs from the `MANIFEST`.
/// - [`GeneratorError::InvalidManifestLine`], [`GeneratorError::RepeatedSourceFile`], or
///   [`GeneratorError::MissingSourceFile`] if the `MANIFEST` breaks its form.
/// - [`GeneratorError::InvalidDeprecationDate`] if a label ends with a date of an unknown form.
/// - The errors of reading and parsing a file.
pub fn generate_removed(xbrl_crate: &Path, sources: &Path) -> Result<String, GeneratorError> {
    let manifest_path = xbrl_crate.join(REMOVED_MANIFEST_FILE);
    let releases = source_releases(&manifest_path, &read(&manifest_path)?)?;
    let in_use_path = xbrl_crate
        .join(US_GAAP_SCHEMA.folder)
        .join(US_GAAP_SCHEMA.schema_file);
    let known = schema::concepts(&in_use_path, &read(&in_use_path)?)?
        .into_iter()
        .map(|concept| concept.name)
        .collect();
    let mut read_releases = Vec::new();
    for (year, release) in &releases {
        read_releases.push(read_release(sources, year, release)?);
    }
    let elements = removed_lines(known, read_releases)?;
    Ok(render::render_removed(RELEASE_ID, &elements))
}

/// One earlier release, read and checked: its ID, its concepts, and their standard labels.
struct ReadRelease {
    /// The release ID, for example `us-gaap/2020`.
    id: String,
    /// The concepts of the element schema.
    concepts: Vec<Concept>,
    /// The standard labels of the label linkbase.
    labels: HashMap<ConceptRef, String>,
}

/// Reads the source files of the release of `year` from `sources`, after a check of their SHA-256.
fn read_release(
    sources: &Path,
    year: &str,
    release: &SourceRelease,
) -> Result<ReadRelease, GeneratorError> {
    let id = format!("{FAMILY}/{year}");
    let missing = |file| GeneratorError::MissingSourceFile {
        release: id.clone(),
        file,
    };
    let (schema_path, schema_digest) = release
        .schema
        .as_ref()
        .ok_or_else(|| missing("element schema"))?;
    let (label_path, label_digest) = release
        .labels
        .as_ref()
        .ok_or_else(|| missing("label linkbase"))?;
    let schema_path = sources.join(schema_path);
    let label_path = sources.join(label_path);
    Ok(ReadRelease {
        concepts: schema::concepts(&schema_path, &verified_read(&schema_path, schema_digest)?)?,
        labels: linkbase::standard_labels(&label_path, &verified_read(&label_path, label_digest)?)?,
        id,
    })
}

/// Returns one line per concept of `releases` that `known` lacks, sorted by name.
///
/// Each line takes the data of the newest release that defines its concept. `known` holds the
/// concepts of the release in use.
///
/// # Errors
///
/// Returns [`GeneratorError::MissingLabel`] if a concept has no standard label in its release, and
/// [`GeneratorError::InvalidDeprecationDate`] if a label ends with a date of an unknown form.
fn removed_lines(
    mut known: BTreeSet<String>,
    mut releases: Vec<ReadRelease>,
) -> Result<Vec<RemovedElementLine>, GeneratorError> {
    releases.sort_by(|first, second| second.id.cmp(&first.id));
    let mut elements = Vec::new();
    for mut release in releases {
        for concept in release.concepts {
            if !known.insert(concept.name.clone()) {
                continue;
            }
            let key = ConceptRef {
                family: FAMILY.to_string(),
                name: concept.name,
            };
            let label =
                release
                    .labels
                    .remove(&key)
                    .ok_or_else(|| GeneratorError::MissingLabel {
                        concept: key.to_string(),
                    })?;
            elements.push(RemovedElementLine {
                deprecated_in: deprecated_in(&key, &label)?,
                name: key.name,
                label,
                data_type: concept.data_type,
                balance: concept.balance,
                period_type: concept.period_type,
                last_release: release.id.clone(),
            });
        }
    }
    elements.sort_by(|first, second| first.name.cmp(&second.name));
    Ok(elements)
}

/// Returns the releases of `manifest` by year, with the path and SHA-256 of each source file.
///
/// Each line holds a SHA-256 and a URL, separated by two spaces, as `sha256sum` writes them. The
/// URL has the form `https://xbrl.fasb.org/us-gaap/<year>/elts/<file>`.
fn source_releases(
    path: &Path,
    manifest: &str,
) -> Result<BTreeMap<String, SourceRelease>, GeneratorError> {
    let mut releases: BTreeMap<String, SourceRelease> = BTreeMap::new();
    for (index, text) in manifest.lines().enumerate() {
        let invalid = || GeneratorError::InvalidManifestLine {
            path: path.to_path_buf(),
            line: index + 1,
            text: text.to_string(),
        };
        let (digest, url) = text.split_once("  ").ok_or_else(invalid)?;
        let is_sha256 = digest.len() == SHA256_HEX_LENGTH
            && digest
                .bytes()
                .all(|byte| matches!(byte, b'0'..=b'9' | b'a'..=b'f'));
        if !is_sha256 {
            return Err(invalid());
        }
        let relative = url.strip_prefix(SOURCE_HOST).ok_or_else(invalid)?;
        let segments: Vec<&str> = relative.split('/').collect();
        let [FAMILY, year, ELEMENT_FOLDER, file] = segments[..] else {
            return Err(invalid());
        };
        if year.len() != YEAR_LENGTH || !year.bytes().all(|byte| byte.is_ascii_digit()) {
            return Err(invalid());
        }
        let release = releases.entry(year.to_string()).or_default();
        let slot = if file.contains(LABEL_MARKER) {
            &mut release.labels
        } else {
            &mut release.schema
        };
        if slot.is_some() {
            return Err(GeneratorError::RepeatedSourceFile {
                path: path.to_path_buf(),
                line: index + 1,
                text: text.to_string(),
            });
        }
        *slot = Some((PathBuf::from(relative), digest.to_string()));
    }
    Ok(releases)
}

/// Returns the text of the file at `path`, if its SHA-256 is `expected`.
fn verified_read(path: &Path, expected: &str) -> Result<String, GeneratorError> {
    let failed_read = |reason: String| GeneratorError::FailedFileRead {
        path: path.to_path_buf(),
        reason,
    };
    let bytes = fs::read(path).map_err(|error| failed_read(error.to_string()))?;
    let actual = hex(&Sha256::digest(&bytes));
    if actual != expected {
        return Err(GeneratorError::UnexpectedChecksum {
            path: path.to_path_buf(),
            expected: expected.to_string(),
            actual,
        });
    }
    String::from_utf8(bytes).map_err(|error| failed_read(error.to_string()))
}

/// Returns the ID of the release that `label` names as the deprecation date, or `None` if the
/// label names none.
///
/// Up to 2021 the date is the date of the release, as in `(Deprecated 2018-01-31)`. From 2022 it
/// is the year, as in `(Deprecated 2020)`.
fn deprecated_in(concept: &ConceptRef, label: &str) -> Result<Option<String>, GeneratorError> {
    let Some(start) = label.rfind(DEPRECATION_PREFIX) else {
        return Ok(None);
    };
    let date = &label[start + DEPRECATION_PREFIX.len()..];
    let year = date
        .strip_suffix(')')
        .map(|date| date.strip_suffix(RELEASE_MONTH_AND_DAY).unwrap_or(date))
        .filter(|year| year.len() == YEAR_LENGTH && year.bytes().all(|byte| byte.is_ascii_digit()))
        .ok_or_else(|| GeneratorError::InvalidDeprecationDate {
            concept: concept.to_string(),
            label: label.to_string(),
        })?;
    Ok(Some(format!("{FAMILY}/{year}")))
}

#[cfg(test)]
mod tests {
    use pretty_assertions::assert_eq;

    use super::*;
    use crate::core::taxonomy::reference::PeriodType;

    fn concept(name: &str) -> Concept {
        Concept {
            name: name.to_string(),
            data_type: "xbrli:monetaryItemType".to_string(),
            balance: None,
            period_type: PeriodType::Duration,
        }
    }

    fn release(id: &str, names: &[&str], label_suffix: &str) -> ReadRelease {
        ReadRelease {
            id: id.to_string(),
            concepts: names.iter().map(|name| concept(name)).collect(),
            labels: names
                .iter()
                .map(|name| {
                    let key = ConceptRef {
                        family: FAMILY.to_string(),
                        name: (*name).to_string(),
                    };
                    (key, format!("{name} {label_suffix}"))
                })
                .collect(),
        }
    }

    fn sales_revenue_net() -> ConceptRef {
        ConceptRef {
            family: FAMILY.to_string(),
            name: "SalesRevenueNet".to_string(),
        }
    }

    #[test]
    fn should_return_release_of_the_year_when_label_ends_with_a_release_date() {
        let label = "Revenue, Net (Deprecated 2018-01-31)";

        let expected_result = Some("us-gaap/2018".to_string());

        let result = deprecated_in(&sales_revenue_net(), label)
            .expect("Given a label with a release date, the parse should always succeed");

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_return_release_of_the_year_when_label_ends_with_a_year() {
        let label = "Revenue, Net (Deprecated 2020)";

        let expected_result = Some("us-gaap/2020".to_string());

        let result = deprecated_in(&sales_revenue_net(), label)
            .expect("Given a label with a year, the parse should always succeed");

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_return_none_when_label_names_no_deprecation() {
        let label = "Revenue, Net";

        let expected_result = None;

        let result = deprecated_in(&sales_revenue_net(), label)
            .expect("Given a label without a deprecation, the parse should always succeed");

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_return_invalid_deprecation_date_when_date_is_not_a_release_date() {
        let label = "Revenue, Net (Deprecated 2018-03-31)";

        let expected_result = Some(
            "[InvalidDeprecationDate] Label of concept 'us-gaap:SalesRevenueNet' ends with an \
             unknown deprecation date, Label: 'Revenue, Net (Deprecated 2018-03-31)'"
                .to_string(),
        );

        let result = deprecated_in(&sales_revenue_net(), label)
            .err()
            .map(|error| error.to_string());

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_return_invalid_manifest_line_when_url_is_not_an_element_file() {
        let digest = "0".repeat(64);
        let manifest = format!(
            "{digest}  https://xbrl.fasb.org/us-gaap/2020/stm/us-gaap-stm-soi-2020-01-31.xsd"
        );

        let expected_result = Some(format!(
            "[InvalidManifestLine] Line does not name a SHA-256 and a FASB element file, Text: \
             '{digest}  https://xbrl.fasb.org/us-gaap/2020/stm/us-gaap-stm-soi-2020-01-31.xsd', \
             Line: '1', Path: 'MANIFEST'"
        ));

        let result = source_releases(Path::new("MANIFEST"), &manifest)
            .err()
            .map(|error| error.to_string());

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_return_invalid_manifest_line_when_year_is_not_four_digits() {
        let digest = "0".repeat(64);
        let manifest =
            format!("{digest}  https://xbrl.fasb.org/us-gaap/20x0/elts/us-gaap-2020-01-31.xsd");

        let expected_result = Some(format!(
            "[InvalidManifestLine] Line does not name a SHA-256 and a FASB element file, Text: \
             '{digest}  https://xbrl.fasb.org/us-gaap/20x0/elts/us-gaap-2020-01-31.xsd', Line: \
             '1', Path: 'MANIFEST'"
        ));

        let result = source_releases(Path::new("MANIFEST"), &manifest)
            .err()
            .map(|error| error.to_string());

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_return_invalid_manifest_line_when_digest_is_not_a_lowercase_sha256() {
        let digest = "A".repeat(64);
        let manifest =
            format!("{digest}  https://xbrl.fasb.org/us-gaap/2020/elts/us-gaap-2020-01-31.xsd");

        let expected_result = Some(format!(
            "[InvalidManifestLine] Line does not name a SHA-256 and a FASB element file, Text: \
             '{digest}  https://xbrl.fasb.org/us-gaap/2020/elts/us-gaap-2020-01-31.xsd', Line: \
             '1', Path: 'MANIFEST'"
        ));

        let result = source_releases(Path::new("MANIFEST"), &manifest)
            .err()
            .map(|error| error.to_string());

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_return_unexpected_checksum_when_file_differs_from_the_manifest() {
        let path = std::env::temp_dir().join(format!(
            "xbrl-removed-unexpected-checksum-{}.xsd",
            std::process::id()
        ));
        fs::write(&path, "abc").expect("Given the temp folder, a write should always succeed");
        let expected = "0".repeat(64);

        let expected_result = Some(format!(
            "[UnexpectedChecksum] Source file differs from the MANIFEST, Expected: '{expected}', \
             Actual: 'ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad', Path: '{}'",
            path.display()
        ));

        let result = verified_read(&path, &expected)
            .err()
            .map(|error| error.to_string());
        fs::remove_file(&path).expect("Given the file this test wrote, the removal should succeed");

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_take_data_of_the_newest_release_when_two_releases_define_the_concept() {
        let releases = vec![
            release("us-gaap/2019", &["SalesRevenueNet"], "old"),
            release(
                "us-gaap/2020",
                &["SalesRevenueNet"],
                "(Deprecated 2018-01-31)",
            ),
        ];

        let expected_result = vec![RemovedElementLine {
            name: "SalesRevenueNet".to_string(),
            label: "SalesRevenueNet (Deprecated 2018-01-31)".to_string(),
            data_type: "xbrli:monetaryItemType".to_string(),
            balance: None,
            period_type: PeriodType::Duration,
            last_release: "us-gaap/2020".to_string(),
            deprecated_in: Some("us-gaap/2018".to_string()),
        }];

        let result = removed_lines(BTreeSet::new(), releases)
            .expect("Given a label for each concept, the lines should always build");

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_skip_concept_when_the_release_in_use_defines_it() {
        let known = BTreeSet::from(["Revenues".to_string()]);
        let releases = vec![release("us-gaap/2020", &["Revenues"], "label")];

        let expected_result: Vec<RemovedElementLine> = Vec::new();

        let result = removed_lines(known, releases)
            .expect("Given a label for each concept, the lines should always build");

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_return_missing_label_when_release_has_no_label_for_a_concept() {
        let mut releases = vec![release("us-gaap/2020", &["SalesRevenueNet"], "label")];
        releases[0].labels.clear();

        let expected_result = Some(
            "[MissingLabel] Standard label not found for concept 'us-gaap:SalesRevenueNet'"
                .to_string(),
        );

        let result = removed_lines(BTreeSet::new(), releases)
            .err()
            .map(|error| error.to_string());

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_return_repeated_source_file_when_manifest_names_two_schemas_of_one_release() {
        let digest = "0".repeat(64);
        let manifest = format!(
            "{digest}  https://xbrl.fasb.org/us-gaap/2020/elts/us-gaap-2020-01-31.xsd\n\
             {digest}  https://xbrl.fasb.org/us-gaap/2020/elts/us-gaap-2020.xsd"
        );

        let expected_result = Some(format!(
            "[RepeatedSourceFile] The MANIFEST names a second file of the same kind for one \
             release, Text: '{digest}  https://xbrl.fasb.org/us-gaap/2020/elts/us-gaap-2020.xsd', \
             Line: '2', Path: 'MANIFEST'"
        ));

        let result = source_releases(Path::new("MANIFEST"), &manifest)
            .err()
            .map(|error| error.to_string());

        assert_eq!(result, expected_result);
    }
}
