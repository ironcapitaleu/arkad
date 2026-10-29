//! # Document Metadata Constants
//!
//! Provides the version of the shared fields in [`DocumentMetadata`].
//!
//! [`DocumentMetadata`]: crate::implementations::repository::filesystem::DocumentMetadata

/// Version of the shared fields in [`DocumentMetadata`], including the fields of the types they
/// hold.
///
/// Raise it when such a field is added, removed, renamed, or redefined.
///
/// Version 2 adds the optional run field `ingestion_mode`. A version-1 file reads with no
/// ingestion mode.
///
/// [`DocumentMetadata`]: crate::implementations::repository::filesystem::DocumentMetadata
pub const METADATA_VERSION: u32 = 2;
