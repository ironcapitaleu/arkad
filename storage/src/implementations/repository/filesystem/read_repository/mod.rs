//! # Read Repository
//!
//! Implements [`ReadRepository`] for [`FilesystemRepository`].

use async_trait::async_trait;

use super::FilesystemRepository;
use super::document_key::DocumentKey;
use super::raw_document::RawDocument;
use crate::error::ReadError;
use crate::traits::repository::ReadRepository;

#[async_trait]
impl ReadRepository for FilesystemRepository {
    type Record = RawDocument;
    type Key = DocumentKey;

    /// Reads the document a key names, together with its metadata.
    ///
    /// # Panics
    ///
    /// Always panics. This adapter holds no read implementation.
    async fn get(&self, _key: Self::Key) -> Result<Option<Self::Record>, ReadError> {
        unimplemented!("FilesystemRepository holds no read implementation")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const fn implements_read_repository<T: ReadRepository>() {}
    #[test]
    const fn should_implement_read_repository_for_filesystem_repository() {
        implements_read_repository::<FilesystemRepository>();
    }
}
