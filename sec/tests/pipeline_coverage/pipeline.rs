use std::pin::pin;

use futures_util::StreamExt;
use sec::prelude::*;
use sec::shared::raw_document::RawDocument;
use sec::shared::raw_source::RawCompanyFacts;
use storage::implementations::repository::filesystem::{FilesystemRepository, Write};
use uuid::Uuid;

// The test builds its pipeline the same way the binaries do. `#[path]` compiles their shared file
// into this module.
#[path = "../../src/bin/shared/pipeline.rs"]
mod shared;

pub use shared::Pipeline;

/// The storage repository that stores the raw Company Facts of each run.
pub type RawCompanyFactsWriter = FilesystemRepository<Write, RawDocument<RawCompanyFacts>>;

/// Builds a [`RawCompanyFactsWriter`] rooted in the temporary directory. Building touches no disk.
pub fn raw_company_facts_writer() -> RawCompanyFactsWriter {
    FilesystemRepository::builder()
        .writer()
        .document_type::<RawDocument<RawCompanyFacts>>()
        .root(std::env::temp_dir().join("arkad-pipeline-coverage"))
        .build()
}

impl Pipeline<RawCompanyFactsWriter> {
    /// Runs the pipeline and returns the first error of its stream.
    pub async fn run(self) -> Result<(), StreamError> {
        let mut stream = pin!(self.into_stream(Uuid::new_v4()));

        while let Some(result) = stream.next().await {
            result?;
        }

        Ok(())
    }
}
