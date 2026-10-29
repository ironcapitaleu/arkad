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
/// It stays at 1 until the MVP, and a field change before the MVP does not raise it. Files that
/// an earlier build wrote before the MVP do not need to stay readable.
///
/// [`DocumentMetadata`]: crate::implementations::repository::filesystem::DocumentMetadata
pub const METADATA_VERSION: u32 = 1;
