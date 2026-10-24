//! # Document Metadata Constants
//!
//! Provides the version of the shared fields in [`DocumentMetadata`].
//!
//! [`DocumentMetadata`]: crate::implementations::repository::filesystem::DocumentMetadata

/// Version of the shared fields in [`DocumentMetadata`], including the fields of the types they
/// hold.
///
/// Raise it when such a field is removed, renamed, or redefined.
///
/// [`DocumentMetadata`]: crate::implementations::repository::filesystem::DocumentMetadata
pub const METADATA_VERSION: u32 = 1;
