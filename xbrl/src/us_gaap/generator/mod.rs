//! # US GAAP Generator
//!
//! Writes the tables of the `us_gaap` plug-in from the vendored FASB and SEC files.
//!
//! The build of `xbrl` parses no XML for the tables. This module reads the vendored files with
//! `roxmltree`: the statement calculation linkbases of the US GAAP release, and the element schema
//! and label linkbase of `us-gaap` and of the `srt` and `dei` releases that it imports. It renders
//! the tables as Rust source into `src/us_gaap/taxonomy.rs`, which is checked in. A developer runs
//! it once per release, with `cargo run -p xbrl --features generator --bin generate_us_gaap`.
//!
//! The module is compiled only with the `generator` feature, so a normal build of `xbrl` or of a
//! crate that depends on it leaves it out.
//!
//! ## Modules
//!
//! - [`error`]: The error of a run.
//! - [`linkbase`]: Reads labels and calculation arcs.
//! - [`render`]: Writes the Rust source of the tables.
//! - [`schema`]: Reads the concepts of the element schema.
//!
//! ## See Also
//!
//! - `xbrl/DESIGN.md`, "Taxonomy Data in Code".

use std::collections::BTreeSet;
use std::fmt::Write;
use std::fs;
use std::path::{Path, PathBuf};

use sha2::{Digest, Sha256};

use self::error::GeneratorError;
use self::linkbase::ConceptRef;
use self::render::{ElementLine, Tables};

pub mod error;
pub mod linkbase;
pub mod render;
pub mod schema;

/// The ID of the release that the generator reads.
pub const RELEASE_ID: &str = "us-gaap/2026";

/// The folder of the vendored US GAAP release, relative to the `xbrl` crate.
pub const RELEASE_FOLDER: &str = "taxonomy/us-gaap/2026";

/// The vendored folders that the release checksum covers, relative to the `xbrl` crate.
///
/// The checksum joins their `MANIFEST` files in this order. The first folder is the US GAAP
/// release. The others hold the `srt` and `dei` releases that it imports.
pub const CHECKSUM_FOLDERS: &[&str] = &[RELEASE_FOLDER, SRT_FOLDER, DEI_FOLDER];

/// The generated file, relative to the `xbrl` crate.
///
/// `.gitattributes` marks this path as generated. Change both together.
pub const OUTPUT_FILE: &str = "src/us_gaap/taxonomy.rs";

/// The folder of the vendored `srt` release, relative to the `xbrl` crate.
const SRT_FOLDER: &str = "taxonomy/srt/2026";

/// The folder of the vendored `dei` release, relative to the `xbrl` crate.
const DEI_FOLDER: &str = "taxonomy/dei/2026";

/// The element schemas that the generator reads, with the family of their concepts.
const ELEMENT_SCHEMAS: &[ElementSchema] = &[
    ElementSchema {
        family: "us-gaap",
        folder: RELEASE_FOLDER,
        schema_file: "elts/us-gaap-2026.xsd",
        label_file: "elts/us-gaap-lab-2026.xml",
    },
    ElementSchema {
        family: "srt",
        folder: SRT_FOLDER,
        schema_file: "elts/srt-2026.xsd",
        label_file: "elts/srt-lab-2026.xml",
    },
    ElementSchema {
        family: "dei",
        folder: DEI_FOLDER,
        schema_file: "dei-2026.xsd",
        label_file: "dei-2026_lab.xsd",
    },
];

/// The folder of the statement groups, relative to the release folder.
const STATEMENT_FOLDER: &str = "stm";

/// The part of a file name that marks a calculation linkbase.
const CALCULATION_MARKER: &str = "-cal-";

/// The manifest of the vendored files, relative to a vendored folder.
const MANIFEST_FILE: &str = "MANIFEST";

/// One vendored element schema and its label linkbase.
struct ElementSchema {
    /// The namespace family of the concepts of the schema.
    family: &'static str,
    /// The vendored folder, relative to the `xbrl` crate.
    folder: &'static str,
    /// The element schema, relative to `folder`.
    schema_file: &'static str,
    /// The label linkbase, relative to `folder`.
    label_file: &'static str,
}

/// Returns the Rust source of `us_gaap/taxonomy.rs`, read from the release below `xbrl_crate`.
///
/// The calculation arcs come from the statement groups in `stm/` of the US GAAP release. The
/// disclosure groups of the release are not vendored. The elements come from the element schemas
/// of `us-gaap`, `srt`, and `dei`.
///
/// # Errors
///
/// Returns a [`GeneratorError`] if a vendored file cannot be read, is not well-formed XML, or
/// breaks an XBRL rule that the generator relies on, such as a concept without a standard label.
pub fn generate(xbrl_crate: &Path) -> Result<String, GeneratorError> {
    let release = xbrl_crate.join(RELEASE_FOLDER);
    let mut elements = Vec::new();
    for schema in ELEMENT_SCHEMAS {
        elements.extend(schema_elements(xbrl_crate, schema)?);
    }
    elements.sort();
    let tables = Tables {
        release_id: RELEASE_ID.to_string(),
        folders: CHECKSUM_FOLDERS.iter().map(ToString::to_string).collect(),
        checksum: release_checksum(xbrl_crate)?,
        arcs: calculation_arcs(&release.join(STATEMENT_FOLDER))?,
        elements,
    };
    Ok(render::render(&tables))
}

/// Returns the concepts of one element schema with their family and standard labels.
fn schema_elements(
    xbrl_crate: &Path,
    schema: &ElementSchema,
) -> Result<Vec<ElementLine>, GeneratorError> {
    let folder = xbrl_crate.join(schema.folder);
    let schema_path = folder.join(schema.schema_file);
    let label_path = folder.join(schema.label_file);
    let concepts = schema::concepts(&schema_path, &read(&schema_path)?)?;
    let mut labels = linkbase::standard_labels(&label_path, &read(&label_path)?)?;
    concepts
        .into_iter()
        .map(|concept| {
            let key = ConceptRef {
                family: schema.family.to_string(),
                name: concept.name,
            };
            let label = labels
                .remove(&key)
                .ok_or_else(|| GeneratorError::MissingLabel {
                    concept: key.to_string(),
                })?;
            Ok(ElementLine {
                family: key.family,
                name: key.name,
                label,
                data_type: concept.data_type,
                balance: concept.balance,
                period_type: concept.period_type,
            })
        })
        .collect()
}

/// Returns the arcs of every calculation linkbase in `statements`, sorted by role, parent, and
/// child. Two files can hold the same arc in the same role, so an arc appears once per role.
///
/// # Errors
///
/// Returns [`GeneratorError::ConflictingWeight`] if two arcs of one role join the same concepts
/// with different weights, and the errors of [`linkbase::calculation_arcs`].
fn calculation_arcs(statements: &Path) -> Result<Vec<linkbase::CalculationArc>, GeneratorError> {
    let entries = fs::read_dir(statements).map_err(|error| GeneratorError::FailedFileRead {
        path: statements.to_path_buf(),
        reason: error.to_string(),
    })?;
    let mut arcs = BTreeSet::new();
    for entry in entries {
        let path = entry
            .map_err(|error| GeneratorError::FailedFileRead {
                path: statements.to_path_buf(),
                reason: error.to_string(),
            })?
            .path();
        let is_calculation = path
            .file_name()
            .is_some_and(|name| name.to_string_lossy().contains(CALCULATION_MARKER));
        if is_calculation {
            arcs.extend(linkbase::calculation_arcs(&path, &read(&path)?)?);
        }
    }
    let arcs: Vec<_> = arcs.into_iter().collect();
    check_weights(&arcs)?;
    Ok(arcs)
}

/// Checks that no two arcs of `arcs` join the same concepts in one role with different weights.
///
/// `arcs` must be sorted, so two such arcs sit next to each other.
fn check_weights(arcs: &[linkbase::CalculationArc]) -> Result<(), GeneratorError> {
    for pair in arcs.windows(2) {
        let (first, second) = (&pair[0], &pair[1]);
        if (&first.role, &first.parent, &first.child)
            == (&second.role, &second.parent, &second.child)
        {
            return Err(GeneratorError::ConflictingWeight {
                role: first.role.clone(),
                parent: first.parent.to_string(),
                child: first.child.to_string(),
                first: first.weight.clone(),
                second: second.weight.clone(),
            });
        }
    }
    Ok(())
}

/// Returns the SHA-256 of the `MANIFEST` files of [`CHECKSUM_FOLDERS`], joined in that order, as
/// 64 lowercase hex digits.
fn release_checksum(xbrl_crate: &Path) -> Result<String, GeneratorError> {
    let mut hasher = Sha256::new();
    for folder in CHECKSUM_FOLDERS {
        let manifest = xbrl_crate.join(folder).join(MANIFEST_FILE);
        let bytes = fs::read(&manifest).map_err(|error| GeneratorError::FailedFileRead {
            path: manifest.clone(),
            reason: error.to_string(),
        })?;
        hasher.update(&bytes);
    }
    let mut checksum = String::new();
    for byte in hasher.finalize() {
        write!(checksum, "{byte:02x}").expect("Given a String, a write should never fail");
    }
    Ok(checksum)
}

/// Returns the text of the file at `path`.
fn read(path: &Path) -> Result<String, GeneratorError> {
    fs::read_to_string(path).map_err(|error| GeneratorError::FailedFileRead {
        path: PathBuf::from(path),
        reason: error.to_string(),
    })
}

#[cfg(test)]
mod tests {
    use pretty_assertions::assert_eq;

    use super::linkbase::CalculationArc;
    use super::*;

    fn arc(weight: &str) -> CalculationArc {
        CalculationArc {
            role: "http://fasb.org/us-gaap/role/statement/A".to_string(),
            parent: ConceptRef {
                family: "us-gaap".to_string(),
                name: "Assets".to_string(),
            },
            child: ConceptRef {
                family: "us-gaap".to_string(),
                name: "AssetsCurrent".to_string(),
            },
            weight: weight.to_string(),
        }
    }

    #[test]
    fn should_return_conflicting_weight_when_one_role_holds_an_arc_with_two_weights() {
        let arcs = vec![arc("-1.0"), arc("1.0")];

        let expected_result = Some(
            "[ConflictingWeight] Arc 'us-gaap:Assets' to 'us-gaap:AssetsCurrent' has two weights, \
             Weights: '-1.0' and '1.0', Role: 'http://fasb.org/us-gaap/role/statement/A'"
                .to_string(),
        );

        let result = check_weights(&arcs).err().map(|error| error.to_string());

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_accept_arcs_when_two_roles_join_the_same_concepts_with_different_weights() {
        let mut other_role = arc("-1.0");
        other_role.role = "http://fasb.org/us-gaap/role/statement/B".to_string();
        let arcs = vec![arc("1.0"), other_role];

        let expected_result = None;

        let result = check_weights(&arcs).err().map(|error| error.to_string());

        assert_eq!(result, expected_result);
    }
}
