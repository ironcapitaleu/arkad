//! # Listing Peak Memory Integration Test
//!
//! Lists two stores through [`ListRepository::list`] and compares the peak heap of each. The
//! second store holds twice the filings for the same companies. The peak of a listing grows with
//! the widest directory, not with the count of files, so the two peaks stay within 10%.
//!
//! A global allocator applies to a whole test binary, so this file holds one test.
//!
//! ## External Dependencies
//!
//! - A writable system temp directory. The test creates its stores there and deletes them at the
//!   end.

use std::fmt;
use std::future::poll_fn;
use std::path::Path;
use std::pin::Pin;

use bytes::Bytes;
use chrono::{DateTime, Utc};
use futures_core::stream::Stream;
use pretty_assertions::assert_eq;
use tempfile::TempDir;
use uuid::Uuid;

use storage::ListRepository;
use storage::implementations::repository::filesystem::{
    Document, DocumentKey, DocumentMetadata, FilesystemRepository, IngestionMode, KeyPrefix,
    METADATA_SUFFIX, MediaType, Producer,
};

#[global_allocator]
static ALLOCATOR: dhat::Alloc = dhat::Alloc;

/// The companies of each store. The directory of the companies is the widest directory.
const COMPANIES: usize = 300;

/// The documents of one filing: the folder index, the instance, and the label file.
const FILING_DOCUMENTS: [&str; 3] = ["index.json", "instance.xml", "label.xml"];

/// The document types of this test.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum SampleDocumentType {
    SampleDocument,
}

impl fmt::Display for SampleDocumentType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "sample_document")
    }
}

/// A document type that exists only for this test, with an empty context.
struct SampleDocument {
    key: DocumentKey,
    bytes: Bytes,
    metadata: DocumentMetadata<()>,
}

impl Document for SampleDocument {
    type DocumentType = SampleDocumentType;
    const DOCUMENT_TYPE: SampleDocumentType = SampleDocumentType::SampleDocument;
    const DOCUMENT_TYPE_VERSION: u32 = 1;
    type Context = ();

    fn key(&self) -> &DocumentKey {
        &self.key
    }

    fn bytes(&self) -> &[u8] {
        &self.bytes
    }

    fn metadata(&self) -> &DocumentMetadata<()> {
        &self.metadata
    }

    fn from_parts(
        key: DocumentKey,
        bytes: impl Into<Bytes>,
        metadata: DocumentMetadata<()>,
    ) -> Self {
        Self {
            key,
            bytes: bytes.into(),
            metadata,
        }
    }
}

/// Writes a store with the given count of filings per company, each filing with its documents and
/// their metadata files.
fn write_store(root: &Path, filings_per_company: usize) {
    let bytes = b"{}";
    let metadata = DocumentMetadata::builder()
        .document_type::<SampleDocument>()
        .media_type(
            MediaType::new("application/json")
                .expect("Given a hardcoded valid media type, creation should always succeed"),
        )
        .content(bytes)
        .produced_at(DateTime::<Utc>::UNIX_EPOCH)
        .run(
            Uuid::nil(),
            Producer::new("Sample State", "0.1.0", "63c1f59"),
        )
        .ingestion_mode(IngestionMode::Backfill)
        .context(())
        .build();
    let metadata_json = serde_json::to_vec(&metadata)
        .expect("Given metadata of owned fields, serialization should always succeed");
    for company in 0..COMPANIES {
        for filing in 0..filings_per_company {
            let directory = root.join(format!("sec/filings/CIK{company:010}/{filing:018}"));
            std::fs::create_dir_all(&directory).expect(
                "Given a writable temp directory, creating subdirectories should always succeed",
            );
            for document in FILING_DOCUMENTS {
                std::fs::write(directory.join(document), bytes)
                    .expect("Given an existing directory, writing a file should always succeed");
                std::fs::write(
                    directory.join(format!("{document}{METADATA_SUFFIX}")),
                    &metadata_json,
                )
                .expect("Given an existing directory, writing a file should always succeed");
            }
        }
    }
}

/// Lists every document of the store and returns the peak heap in bytes during the listing.
///
/// Drops each entry as it arrives, so the count of entries does not add to the peak.
async fn peak_bytes_of_listing(root: &Path) -> usize {
    let repository = FilesystemRepository::builder()
        .reader()
        .document_type::<SampleDocument>()
        .root(root)
        .build();
    let profiler = dhat::Profiler::builder().testing().build();
    let mut listing = repository.list(KeyPrefix::root());
    while let Some(item) = poll_fn(|cx| Pin::new(&mut listing).poll_next(cx)).await {
        item.expect("Given a store this test wrote, every entry should always read");
    }
    drop(listing);
    let peak_bytes = dhat::HeapStats::get().max_bytes;
    drop(profiler);
    peak_bytes
}

#[tokio::test]
async fn should_peak_within_ten_percent_when_the_store_holds_twice_the_filings_per_company() {
    let store = TempDir::new().expect(
        "Given a writable system temp directory, creating a directory in it should always succeed",
    );
    let store_with_twice_the_filings = TempDir::new().expect(
        "Given a writable system temp directory, creating a directory in it should always succeed",
    );
    write_store(store.path(), 1);
    write_store(store_with_twice_the_filings.path(), 2);
    // Starts the blocking threads of the runtime, so neither measured listing pays for them.
    peak_bytes_of_listing(store.path()).await;

    let expected_result = true;

    let peak_bytes = peak_bytes_of_listing(store.path()).await;
    let peak_bytes_with_twice_the_filings =
        peak_bytes_of_listing(store_with_twice_the_filings.path()).await;
    let result = peak_bytes_with_twice_the_filings * 10 <= peak_bytes * 11
        && peak_bytes * 10 <= peak_bytes_with_twice_the_filings * 11;

    assert_eq!(
        result, expected_result,
        "peak of the store: {peak_bytes} bytes, peak with twice the filings: \
         {peak_bytes_with_twice_the_filings} bytes"
    );
}
