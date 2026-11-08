//! # US GAAP Generator
//!
//! Writes `xbrl/src/us_gaap/taxonomy.rs` from the vendored US GAAP release.

use std::fs;
use std::path::Path;
use std::process::ExitCode;

use us_gaap_generator::error::GeneratorError;
use us_gaap_generator::{OUTPUT_FILE, generate};

/// The `xbrl` crate, relative to this crate.
const XBRL_CRATE: &str = "..";

// Deviation from the logging rules of AGENTS.md: this is a developer tool that runs once per
// release on a laptop, never in a container. It prints the error chain to stderr and needs no
// structured log.
fn main() -> ExitCode {
    let xbrl_crate = Path::new(env!("CARGO_MANIFEST_DIR")).join(XBRL_CRATE);
    match run(&xbrl_crate) {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("{error}");
            ExitCode::FAILURE
        }
    }
}

/// Generates the tables and writes them to the output file of the `xbrl` crate.
fn run(xbrl_crate: &Path) -> Result<(), GeneratorError> {
    let source = generate(xbrl_crate)?;
    let output = xbrl_crate.join(OUTPUT_FILE);
    fs::write(&output, source).map_err(|source| GeneratorError::FailedFileWrite {
        path: output,
        source,
    })
}
