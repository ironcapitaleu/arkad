//! # Digest Row
//!
//! Provides [`DigestRow`], the trait of a silver row that enters `content_digest`.

use super::byte_form::ByteForm;

/// A silver row that writes its digest group ID, its row key, and itself in the byte form.
///
/// A trait, because each silver dataset has its own row type, and [`content_digest`] and
/// [`group_hashes`] take the rows of any dataset. Each row type of a dataset implements it. Each
/// method appends values to the [`ByteForm`] with the methods of [`ByteForm`], one per column.
///
/// The digest group and the row key of each dataset are in `sec/design/pipeline/silver_layer.md`,
/// `silver_files`, column `content_digest`.
///
/// [`content_digest`]: super::content_digest()
/// [`group_hashes`]: super::group_hashes
pub trait DigestRow {
    /// Appends the key columns of the digest group of the row.
    ///
    /// A digest group is one filing, so the ID is `filing_id`. For a row without a filing, the ID
    /// is the key of its entity or of its taxonomy release, for example `entity_scheme` then
    /// `entity_id`. Rows with the same ID bytes form one digest group.
    fn write_group_id(&self, byte_form: &mut ByteForm);

    /// Appends the columns of the row key, in the order of the key.
    ///
    /// The rows of a digest group sort by the bytes of their row key, and then by the bytes of the
    /// whole row.
    fn write_row_key(&self, byte_form: &mut ByteForm);

    /// Appends every column of the row in the order of the schema of its dataset.
    ///
    /// Leaves out `execution_id`, `loaded_at`, and `computed_at`, because a rebuild of the same
    /// rows changes them.
    fn write_row(&self, byte_form: &mut ByteForm);
}
