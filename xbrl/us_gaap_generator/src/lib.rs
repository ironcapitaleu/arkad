//! # US GAAP Generator
//!
//! Writes the tables of the `us_gaap` plug-in of the `xbrl` crate from the vendored FASB files.
//!
//! The build of `xbrl` parses no XML. This crate reads the element schema, the label linkbase, and
//! the statement calculation linkbases of the vendored release with `roxmltree`. It renders the tables as
//! Rust source into `xbrl/src/us_gaap/taxonomy.rs`, which is checked in. A developer runs it once
//! per release.
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

use crate::error::GeneratorError;
use crate::linkbase::ConceptRef;
use crate::render::{ElementLine, Tables};

pub mod error;
pub mod linkbase;
pub mod render;
pub mod schema;

/// The ID of the release that the generator reads.
pub const RELEASE_ID: &str = "us-gaap/2026";

/// The folder of the vendored release, relative to the `xbrl` crate.
pub const RELEASE_FOLDER: &str = "taxonomy/us-gaap/2026";

/// The generated file, relative to the `xbrl` crate.
pub const OUTPUT_FILE: &str = "src/us_gaap/taxonomy.rs";

/// The namespace family of the concepts of the element schema.
const FAMILY: &str = "us-gaap";

/// The element schema, relative to the release folder.
const SCHEMA_FILE: &str = "elts/us-gaap-2026.xsd";

/// The label linkbase, relative to the release folder.
const LABEL_FILE: &str = "elts/us-gaap-lab-2026.xml";

/// The folder of the statement groups, relative to the release folder.
const STATEMENT_FOLDER: &str = "stm";

/// The part of a file name that marks a calculation linkbase.
const CALCULATION_MARKER: &str = "-cal-";

/// The manifest of the vendored files, relative to the release folder.
const MANIFEST_FILE: &str = "MANIFEST";

/// Returns the Rust source of `us_gaap/taxonomy.rs`, read from the release below `xbrl_crate`.
///
/// The calculation arcs come from the statement groups in `stm/`. The disclosure groups of the
/// release are not vendored.
///
/// # Errors
///
/// Returns a [`GeneratorError`] if a vendored file cannot be read, is not well-formed XML, or
/// breaks an XBRL rule that the generator relies on, such as a concept without a standard label.
pub fn generate(xbrl_crate: &Path) -> Result<String, GeneratorError> {
    let release = xbrl_crate.join(RELEASE_FOLDER);
    let tables = Tables {
        release_id: RELEASE_ID.to_string(),
        release_folder: RELEASE_FOLDER.to_string(),
        checksum: manifest_checksum(&release.join(MANIFEST_FILE))?,
        arcs: calculation_arcs(&release.join(STATEMENT_FOLDER))?,
        elements: elements(&release)?,
    };
    Ok(render::render(&tables))
}

/// Returns the concepts of the element schema with their standard labels, sorted by name.
fn elements(release: &Path) -> Result<Vec<ElementLine>, GeneratorError> {
    let schema_path = release.join(SCHEMA_FILE);
    let label_path = release.join(LABEL_FILE);
    let concepts = schema::concepts(&schema_path, &read(&schema_path)?)?;
    let mut labels = linkbase::standard_labels(&label_path, &read(&label_path)?)?;
    let mut elements = concepts
        .into_iter()
        .map(|concept| {
            let key = ConceptRef {
                family: FAMILY.to_string(),
                name: concept.name,
            };
            let label = labels
                .remove(&key)
                .ok_or_else(|| GeneratorError::MissingLabel {
                    concept: key.to_string(),
                })?;
            Ok(ElementLine {
                name: key.name,
                label,
                data_type: concept.data_type,
                balance: concept.balance,
                period_type: concept.period_type,
            })
        })
        .collect::<Result<Vec<_>, GeneratorError>>()?;
    elements.sort();
    Ok(elements)
}

/// Returns the arcs of every calculation linkbase in `statements`, sorted, without repeats.
///
/// Two calculation files can hold the same arc in the same role, so the tables keep it once.
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

/// Returns the SHA-256 of the file at `manifest`, as 64 lowercase hex digits.
fn manifest_checksum(manifest: &Path) -> Result<String, GeneratorError> {
    let bytes = fs::read(manifest).map_err(|error| GeneratorError::FailedFileRead {
        path: manifest.to_path_buf(),
        reason: error.to_string(),
    })?;
    let mut checksum = String::new();
    for byte in Sha256::digest(&bytes) {
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

    use super::*;
    use crate::linkbase::CalculationArc;

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
}
