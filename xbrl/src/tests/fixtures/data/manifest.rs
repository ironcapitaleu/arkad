//! # Fixture Manifest
//!
//! The list of the fixture files in `manifest.tsv`: one row per file, with the company, the
//! filing, the source, and two SHA-256 digests. `sha256` covers the file as stored. For a `.gz`
//! file, `sha256_uncompressed` covers its content, the bytes as the SEC wrote them. For a plain
//! file, both digests are equal.

/// The rows of `manifest.tsv`, with a header.
const MANIFEST_TSV: &str = include_str!("manifest.tsv");

/// The character between the fields of a row.
const FIELD_SEPARATOR: char = '\t';

/// One fixture file of the manifest.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ManifestEntry {
    /// The company of the fixture set, for example `JPMorgan Chase`.
    pub company: String,
    /// The CIK of the filer, without leading zeros, as in `sub.tsv`.
    pub cik: String,
    /// The accession number of the filing.
    pub accession: String,
    /// The form, for example `10-K` or `10-QT`, as in `sub.tsv`.
    pub form: String,
    /// The fiscal year, as in the `fy` column of `sub.tsv`.
    pub fiscal_year: String,
    /// The fiscal period, for example `FY` or `Q2`, as in the `fp` column of `sub.tsv`.
    pub fiscal_period: String,
    /// The Notes file the rows come from, for example `2014q1_notes.zip`, or `EDGAR filing
    /// folder` for an instance document.
    pub source: String,
    /// The path of the file, relative to the fixture data directory.
    pub file: String,
    /// The SHA-256 of the file as stored, in lowercase hex.
    pub sha256: String,
    /// The SHA-256 of the content of the file, in lowercase hex.
    pub sha256_uncompressed: String,
}

/// Returns the rows of the manifest, in the order of the file.
///
/// # Panics
///
/// Never panics: each row of the manifest has ten fields.
#[must_use]
pub fn manifest() -> Vec<ManifestEntry> {
    MANIFEST_TSV
        .lines()
        .skip(1)
        .map(|line| {
            let fields: Vec<String> = line.split(FIELD_SEPARATOR).map(str::to_owned).collect();
            let [
                company,
                cik,
                accession,
                form,
                fiscal_year,
                fiscal_period,
                source,
                file,
                sha256,
                sha256_uncompressed,
            ]: [String; 10] = fields.try_into().expect(
                "Given a manifest row with ten fields, the conversion should always succeed",
            );
            ManifestEntry {
                company,
                cik,
                accession,
                form,
                fiscal_year,
                fiscal_period,
                source,
                file,
                sha256,
                sha256_uncompressed,
            }
        })
        .collect()
}
