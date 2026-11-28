//! # US GAAP Removed Elements Generator
//!
//! Writes `xbrl/src/us_gaap/removed.rs` from the FASB files that
//! `xbrl/taxonomy/us-gaap-removed/MANIFEST` names.
//!
//! The files are not vendored. Download each one into a folder, at the path of its URL after
//! `https://xbrl.fasb.org/`, and pass the folder as the only argument:
//!
//! ```text
//! while read -r digest url; do
//!   curl --create-dirs -o "$SOURCES/${url#https://xbrl.fasb.org/}" "$url"
//! done < xbrl/taxonomy/us-gaap-removed/MANIFEST
//! cargo run -p xbrl --features generator --bin generate_us_gaap_removed -- "$SOURCES"
//! ```

use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::ExitCode;

use xbrl::us_gaap::generator::error::GeneratorError;
use xbrl::us_gaap::generator::removed::{REMOVED_OUTPUT_FILE, generate_removed};

// Deviation from the logging rules of the project: this is a developer tool that runs once per
// release on a laptop, never in a container. It prints the error chain to stderr and needs no
// structured log.
fn main() -> ExitCode {
    let xbrl_crate = Path::new(env!("CARGO_MANIFEST_DIR"));
    let Some(sources) = env::args_os().nth(1).map(PathBuf::from) else {
        eprintln!("Usage: generate_us_gaap_removed <folder of the downloaded FASB files>");
        return ExitCode::FAILURE;
    };
    match run(xbrl_crate, &sources) {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("{error}");
            ExitCode::FAILURE
        }
    }
}

/// Generates the table and writes it to the output file of the `xbrl` crate.
fn run(xbrl_crate: &Path, sources: &Path) -> Result<(), GeneratorError> {
    let source = generate_removed(xbrl_crate, sources)?;
    let output = xbrl_crate.join(REMOVED_OUTPUT_FILE);
    fs::write(&output, source).map_err(|error| GeneratorError::FailedFileWrite {
        path: output,
        reason: error.to_string(),
    })
}
