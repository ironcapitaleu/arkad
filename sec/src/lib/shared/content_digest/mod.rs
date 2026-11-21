//! # Content Digest
//!
//! Provides [`content_digest()`], the SHA-256 of the rows of a silver unit, and the byte form of
//! the rows that it hashes.
//!
//! Integrity check 4 runs the transform again on the same input and compares the digests of the
//! two units. So the digest depends only on the rows of a unit, not on the split, the parts, or the
//! Arrow release. Each row type implements [`DigestRow`] and prints its values into a
//! [`ByteForm`]. The writer calls [`group_hashes`] on one whole digest group, keeps only its
//! [`GroupHash`], and calls [`content_digest()`] on the group hashes at the end of the unit.
//!
//! ## Modules
//!
//! - [`byte_form`]: The [`ByteForm`] of the values of a row.
//! - [`digest_row`]: The [`DigestRow`] trait of a row that enters the digest.
//! - [`duplicate_digest_group`]: The [`DuplicateDigestGroup`] for two group hashes with one ID.
//!
//! ## See Also
//!
//! - [ADR 0033](https://github.com/ironcapitaleu/arkad/blob/main/docs/adr/0033-byte-form-of-content-digest.md):
//!   the byte form and the sort order.
//! - `sec/design/pipeline/silver_writer_and_jobs.md`, "Byte Form of `content_digest`": how the
//!   writer uses the digest.

use std::collections::BTreeMap;

use sha2::{Digest, Sha256};
use storage::shared::Sha256Digest;

pub mod byte_form;
pub mod digest_row;
pub mod duplicate_digest_group;

pub use byte_form::ByteForm;
pub use digest_row::DigestRow;
pub use duplicate_digest_group::DuplicateDigestGroup;

/// The ID and the hash of one digest group of a silver unit.
///
/// The hash is the SHA-256 over the byte form of the rows of the group, sorted by the bytes of
/// their row key, and then by the bytes of the whole row. [`group_hashes`] computes it.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct GroupHash {
    /// The byte form of the key columns of the group.
    group_id: Vec<u8>,
    /// The SHA-256 over the sorted rows of the group.
    hash: [u8; 32],
}

impl GroupHash {
    /// Returns the byte form of the key columns of the group.
    #[must_use]
    pub fn group_id(&self) -> &[u8] {
        &self.group_id
    }

    /// Returns the SHA-256 over the sorted rows of the group.
    #[must_use]
    pub const fn hash(&self) -> &[u8; 32] {
        &self.hash
    }
}

/// The byte form of the row key of a row, then the byte form of the whole row.
///
/// The tuple order sorts by the row key first, and then by the whole row.
type KeyedRow = (Vec<u8>, Vec<u8>);

/// Returns the hash of each digest group of the rows, sorted by group ID.
///
/// Groups the rows by the bytes of their group ID. Sorts the rows of each group by the bytes of
/// their row key, and then by the bytes of the whole row. Hashes the sorted rows of each group, one
/// after the other. So the order of the rows does not change the result.
///
/// Each group must be whole in `rows`. If the rows of one group come in two calls, the two results
/// hold one group ID twice, and [`content_digest`] rejects them.
///
/// # Examples
///
/// ```rust
/// use sec::shared::content_digest::{ByteForm, DigestRow, group_hashes};
///
/// struct Fact {
///     filing_id: &'static str,
///     concept: &'static str,
/// }
///
/// impl DigestRow for Fact {
///     fn write_group_id(&self, byte_form: &mut ByteForm) {
///         byte_form.utf8(self.filing_id);
///     }
///
///     fn write_row_key(&self, byte_form: &mut ByteForm) {
///         byte_form.utf8(self.concept);
///     }
///
///     fn write_row(&self, byte_form: &mut ByteForm) {
///         byte_form.utf8(self.filing_id).utf8(self.concept);
///     }
/// }
///
/// let rows = [
///     Fact { filing_id: "0000320193-24-000006", concept: "Assets" },
///     Fact { filing_id: "0000320193-24-000006", concept: "Liabilities" },
/// ];
///
/// let expected_result = 1;
///
/// let result = group_hashes(&rows).len();
///
/// assert_eq!(result, expected_result);
/// ```
pub fn group_hashes<R: DigestRow>(rows: &[R]) -> Vec<GroupHash> {
    let mut groups: BTreeMap<Vec<u8>, Vec<KeyedRow>> = BTreeMap::new();
    for row in rows {
        let mut group_id = ByteForm::new();
        row.write_group_id(&mut group_id);
        let mut row_key = ByteForm::new();
        row.write_row_key(&mut row_key);
        let mut row_bytes = ByteForm::new();
        row.write_row(&mut row_bytes);
        groups
            .entry(group_id.into_bytes())
            .or_default()
            .push((row_key.into_bytes(), row_bytes.into_bytes()));
    }
    groups
        .into_iter()
        .map(|(group_id, mut keyed_rows)| {
            keyed_rows.sort_unstable();
            let mut hasher = Sha256::new();
            for (_, row_bytes) in &keyed_rows {
                hasher.update(row_bytes);
            }
            GroupHash {
                group_id,
                hash: hasher.finalize().into(),
            }
        })
        .collect()
}

/// Returns the `content_digest` of a silver unit from the hashes of its digest groups.
///
/// Sorts the groups by the bytes of their ID. The digest is the SHA-256 over each group ID followed
/// by its 32 hash bytes. A unit with no rows has no groups, so its digest is the SHA-256 of no
/// bytes. The groups can come in any order, for example one list per part of the unit.
///
/// # Errors
///
/// Returns a [`DuplicateDigestGroup`] if two group hashes have the same group ID. Then the rows of
/// one group came in two calls of [`group_hashes`], and the digest would depend on the split.
///
/// # Examples
///
/// ```rust
/// use sec::shared::content_digest::content_digest;
///
/// let groups = Vec::new();
///
/// let expected_result = "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855";
///
/// let digest = content_digest(groups)
///     .expect("Given no groups, no group ID can repeat");
/// let result = digest.as_str();
///
/// assert_eq!(result, expected_result);
/// ```
pub fn content_digest(
    groups: impl IntoIterator<Item = GroupHash>,
) -> Result<Sha256Digest, DuplicateDigestGroup> {
    let mut groups: Vec<GroupHash> = groups.into_iter().collect();
    groups.sort_unstable();
    if let Some(pair) = groups
        .windows(2)
        .find(|pair| pair[0].group_id == pair[1].group_id)
    {
        return Err(DuplicateDigestGroup::new(&pair[0].group_id));
    }
    let mut bytes = Vec::new();
    for group in &groups {
        bytes.extend_from_slice(&group.group_id);
        bytes.extend_from_slice(&group.hash);
    }
    Ok(Sha256Digest::of(bytes))
}

#[cfg(test)]
mod tests {
    use pretty_assertions::{assert_eq, assert_ne};

    use super::*;

    /// A row with a group ID, a row key, and one more column.
    #[derive(Debug, Clone)]
    struct FakeRow {
        filing_id: &'static str,
        concept: &'static str,
        value: i64,
    }

    impl DigestRow for FakeRow {
        fn write_group_id(&self, byte_form: &mut ByteForm) {
            byte_form.utf8(self.filing_id);
        }

        fn write_row_key(&self, byte_form: &mut ByteForm) {
            byte_form.utf8(self.concept);
        }

        fn write_row(&self, byte_form: &mut ByteForm) {
            byte_form
                .utf8(self.filing_id)
                .utf8(self.concept)
                .integer(self.value);
        }
    }

    const fn row(filing_id: &'static str, concept: &'static str, value: i64) -> FakeRow {
        FakeRow {
            filing_id,
            concept,
            value,
        }
    }

    /// The rows of one unit: two filings, and two rows with one row key in the second filing.
    fn unit_rows() -> Vec<FakeRow> {
        vec![
            row("0000320193-24-000006", "Assets", 100),
            row("0000320193-24-000006", "Liabilities", 40),
            row("0000789019-24-000010", "Assets", 7),
            row("0000789019-24-000010", "Assets", 5),
        ]
    }

    fn sha256(bytes: &[u8]) -> [u8; 32] {
        Sha256::digest(bytes).into()
    }

    /// A row whose whole-row bytes sort in another order than its row key.
    struct ValueFirstRow {
        concept: &'static str,
        value: i64,
    }

    impl DigestRow for ValueFirstRow {
        fn write_group_id(&self, byte_form: &mut ByteForm) {
            byte_form.utf8("0000320193-24-000006");
        }

        fn write_row_key(&self, byte_form: &mut ByteForm) {
            byte_form.utf8(self.concept);
        }

        fn write_row(&self, byte_form: &mut ByteForm) {
            byte_form.integer(self.value).utf8(self.concept);
        }
    }

    const fn implements_auto_traits<T: Sized + Send + Sync + Unpin>() {}
    #[test]
    const fn should_implement_auto_traits_for_group_hash() {
        implements_auto_traits::<GroupHash>();
    }

    #[test]
    fn should_give_the_sha256_of_no_bytes_when_the_unit_has_no_rows() {
        let rows: Vec<FakeRow> = Vec::new();

        let expected_result =
            Ok("e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855".to_string());

        let result = content_digest(group_hashes(&rows)).map(|digest| digest.to_string());

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_hash_the_rows_sorted_by_row_key_when_a_group_is_hashed() {
        let rows = vec![
            row("0000320193-24-000006", "Liabilities", 40),
            row("0000320193-24-000006", "Assets", 100),
        ];
        let assets_row = [
            &[0x01, 0, 0, 0, 20][..],
            b"0000320193-24-000006",
            &[0x01, 0, 0, 0, 6],
            b"Assets",
            &[0x01, 0, 0, 0, 3],
            b"100",
        ]
        .concat();
        let liabilities_row = [
            &[0x01, 0, 0, 0, 20][..],
            b"0000320193-24-000006",
            &[0x01, 0, 0, 0, 11],
            b"Liabilities",
            &[0x01, 0, 0, 0, 2],
            b"40",
        ]
        .concat();
        let group_id = [&[0x01, 0, 0, 0, 20][..], b"0000320193-24-000006"].concat();
        let sorted_rows = [assets_row, liabilities_row].concat();

        let expected_result = vec![GroupHash {
            group_id,
            hash: sha256(&sorted_rows),
        }];

        let result = group_hashes(&rows);

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_sort_by_the_row_key_before_the_whole_row_when_the_two_orders_differ() {
        let rows = vec![
            ValueFirstRow {
                concept: "Liabilities",
                value: 40,
            },
            ValueFirstRow {
                concept: "Assets",
                value: 100,
            },
        ];
        // By row key, `Assets` comes first. By whole row, the row of `40` would come first.
        let sorted_rows = [
            &[0x01, 0, 0, 0, 3][..],
            b"100",
            &[0x01, 0, 0, 0, 6],
            b"Assets",
            &[0x01, 0, 0, 0, 2],
            b"40",
            &[0x01, 0, 0, 0, 11],
            b"Liabilities",
        ]
        .concat();
        let group_id = [&[0x01, 0, 0, 0, 20][..], b"0000320193-24-000006"].concat();

        let expected_result = vec![GroupHash {
            group_id,
            hash: sha256(&sorted_rows),
        }];

        let result = group_hashes(&rows);

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_sort_by_the_whole_row_when_two_rows_have_one_row_key() {
        let rows = vec![
            row("0000789019-24-000010", "Assets", 7),
            row("0000789019-24-000010", "Assets", 5),
        ];
        let reversed_rows = vec![
            row("0000789019-24-000010", "Assets", 5),
            row("0000789019-24-000010", "Assets", 7),
        ];

        let expected_result = group_hashes(&reversed_rows);

        let result = group_hashes(&rows);

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_hash_each_group_id_followed_by_its_hash_when_the_digest_is_computed() {
        let first_group = GroupHash {
            group_id: vec![0x01, 0, 0, 0, 1, b'b'],
            hash: [0xBB; 32],
        };
        let second_group = GroupHash {
            group_id: vec![0x01, 0, 0, 0, 1, b'a'],
            hash: [0xAA; 32],
        };
        let groups = vec![first_group, second_group];
        let sorted_bytes = [
            &[0x01, 0, 0, 0, 1, b'a'][..],
            &[0xAA; 32],
            &[0x01, 0, 0, 0, 1, b'b'],
            &[0xBB; 32],
        ]
        .concat();

        let expected_result = Ok(Sha256Digest::of(sorted_bytes));

        let result = content_digest(groups);

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_give_one_digest_when_two_splits_hold_the_same_rows() {
        let rows = unit_rows();
        let first_split = [
            vec![rows[2].clone(), rows[3].clone()],
            vec![rows[1].clone(), rows[0].clone()],
        ];
        let second_split = [
            vec![rows[0].clone(), rows[1].clone()],
            vec![rows[3].clone(), rows[2].clone()],
        ];

        let expected_result =
            content_digest(first_split.iter().flat_map(|part| group_hashes(part)));

        let result = content_digest(second_split.iter().flat_map(|part| group_hashes(part)));

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_give_another_digest_when_a_value_changes() {
        let rows = unit_rows();
        let mut changed_rows = unit_rows();
        changed_rows[0].value = 101;

        let expected_result = content_digest(group_hashes(&rows));

        let result = content_digest(group_hashes(&changed_rows));

        assert_ne!(result, expected_result);
    }

    #[test]
    fn should_return_duplicate_digest_group_when_a_group_comes_in_two_parts() {
        let first_part = vec![row("0000320193-24-000006", "Assets", 100)];
        let second_part = vec![row("0000320193-24-000006", "Liabilities", 40)];
        let groups = [group_hashes(&first_part), group_hashes(&second_part)].concat();
        let group_id = [&[0x01, 0, 0, 0, 20][..], b"0000320193-24-000006"].concat();

        let expected_result = Err(DuplicateDigestGroup::new(&group_id));

        let result = content_digest(groups);

        assert_eq!(result, expected_result);
    }
}
