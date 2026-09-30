//! # Write Repository
//!
//! Implements [`WriteRepository`] for [`FilesystemRepository`].

use async_trait::async_trait;

use super::FilesystemRepository;
use super::raw_document::RawDocument;
use crate::error::WriteError;
use crate::traits::repository::WriteRepository;

#[async_trait]
impl WriteRepository for FilesystemRepository {
    type Record = RawDocument;

    /// Writes a document and its metadata under the root.
    ///
    /// # Panics
    ///
    /// Always panics. This adapter holds no write implementation.
    async fn persist(&self, _record: Self::Record) -> Result<(), WriteError> {
        unimplemented!("FilesystemRepository holds no write implementation")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const fn implements_write_repository<T: WriteRepository>() {}
    #[test]
    const fn should_implement_write_repository_for_filesystem_repository() {
        implements_write_repository::<FilesystemRepository>();
    }
}
