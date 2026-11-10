//! # Document Key Constants
//!
//! Provides the characters that no component of a [`DocumentKey`] can hold.
//!
//! [`DocumentKey`]: crate::implementations::repository::filesystem::DocumentKey

/// Characters that split or prefix a path on Windows, so no component of a [`DocumentKey`] can
/// hold them.
///
/// [`DocumentKey`]: crate::implementations::repository::filesystem::DocumentKey
pub const NON_PORTABLE_CHARACTERS: [char; 2] = ['\\', ':'];
