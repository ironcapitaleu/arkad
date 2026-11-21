//! # Silver
//!
//! Provides the code that every silver writer shares: the byte form of a row and the
//! `content_digest` of a unit.
//!
//! Integrity check 4 runs the transform again on the same input and compares the `content_digest`
//! of the two units. So the digest depends only on the rows of a unit, not on the split, the parts,
//! or the Arrow release. Each row type implements [`DigestRow`] and prints its values into a
//! [`ByteForm`]. [`group_hashes`] and [`content_digest()`] then sort and hash the rows.
//!
//! ## Modules
//!
//! - [`byte_form`]: The [`ByteForm`] of the values of a row.
//! - [`content_digest`](mod@content_digest): The digest of a unit from the hashes of its digest
//!   groups.
//! - [`digest_row`]: The [`DigestRow`] trait of a row that enters the digest.
//!
//! ## See Also
//!
//! - `sec/design/pipeline/silver_writer_and_jobs.md`, "Byte Form of `content_digest`": how the
//!   writer uses the digest.
//! - [ADR 0033](https://github.com/ironcapitaleu/arkad/blob/main/docs/adr/0033-byte-form-of-content-digest.md):
//!   the byte form.

pub mod byte_form;
pub mod content_digest;
pub mod digest_row;

pub use byte_form::ByteForm;
pub use content_digest::{DuplicateDigestGroup, GroupHash, content_digest, group_hashes};
pub use digest_row::DigestRow;

/// The version of the silver output that this build writes.
///
/// A change of the byte form or of the rows of a fixed input needs a new version. The golden test
/// of `content_digest` holds the expected digest of its fixture unit for each version.
pub const TRANSFORM_VERSION: &str = "1";
