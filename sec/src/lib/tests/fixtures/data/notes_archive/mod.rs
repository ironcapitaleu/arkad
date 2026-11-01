use std::io::{Cursor, Write};

use zip::write::SimpleFileOptions;
use zip::{CompressionMethod, ZipWriter};

/// A 150-byte ZIP with one stored file, `readme.htm`.
///
/// Built with Python's `zipfile` on 29.10.2026. It is not an SEC download.
pub const SAMPLE_NOTES_ZIP: &[u8] = include_bytes!("sample_notes.zip");

/// Returns a ZIP with one deflated entry, `sub.tsv`, that holds `sub_tsv`.
///
/// A Notes file of the SEC compresses its entries with deflate too.
///
/// # Panics
///
/// Never panics: a ZIP writer over a `Vec` cannot fail to write.
#[must_use]
pub fn notes_zip(sub_tsv: &str) -> Vec<u8> {
    let mut writer = ZipWriter::new(Cursor::new(Vec::new()));
    writer
        .start_file(
            "sub.tsv",
            SimpleFileOptions::default().compression_method(CompressionMethod::Deflated),
        )
        .expect("Given a writer over a Vec, starting an entry should always succeed");
    writer
        .write_all(sub_tsv.as_bytes())
        .expect("Given a writer over a Vec, writing an entry should always succeed");
    writer
        .finish()
        .expect("Given a writer over a Vec, finishing the ZIP should always succeed")
        .into_inner()
}
