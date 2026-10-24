//! # Document Metadata Constants
//!
//! Provides the version of the shared fields in [`DocumentMetadata`].
//!
//! [`DocumentMetadata`]: crate::implementations::repository::filesystem::DocumentMetadata

/// Version of the shared fields in [`DocumentMetadata`].
///
/// [`Producer`] and [`InputDocument`] serialize inside the shared fields, so this version covers
/// their fields too. Raise it when a shared field is removed, renamed, or redefined.
///
/// [`DocumentMetadata`]: crate::implementations::repository::filesystem::DocumentMetadata
/// [`Producer`]: crate::implementations::repository::filesystem::Producer
/// [`InputDocument`]: crate::implementations::repository::filesystem::InputDocument
pub const METADATA_VERSION: u32 = 1;
