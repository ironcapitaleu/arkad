//! # Part Order
//!
//! Provides [`PartOrder`], the order of the rows of one dataset in a part, and the rows that
//! share a row group.

use crate::shared::row_digest::ByteForm;

/// The order of the rows of one dataset in a part, and where a part starts a new row group.
///
/// A part sorts its rows by the bytes of their sort key. A [`ByteForm`] puts the length of each
/// value before its text. So two values of the same length sort in the order of their text, and a
/// shorter value sorts first. A filing ID always has 20 characters, so the rows of one filing sort
/// together, in the order of the filing IDs. Rows with equal keys keep their order.
///
/// A part also starts a new row group where the row group key of two neighbor rows differs.
///
/// # Examples
///
/// ```rust
/// use sec::datasets::PartOrder;
/// use sec::shared::row_digest::ByteForm;
///
/// struct Fact {
///     filing_id: &'static str,
///     concept: &'static str,
/// }
///
/// impl PartOrder for Fact {
///     fn write_sort_key(&self, byte_form: &mut ByteForm) {
///         byte_form.utf8(self.filing_id).utf8(self.concept);
///     }
/// }
///
/// let fact = Fact { filing_id: "0000320193-24-000006", concept: "Assets" };
///
/// let expected_result: &[u8] = &[];
///
/// let mut byte_form = ByteForm::new();
/// fact.write_row_group_key(&mut byte_form);
/// let result = byte_form.as_bytes();
///
/// assert_eq!(result, expected_result);
/// ```
pub trait PartOrder {
    /// Appends the sort key of the row.
    fn write_sort_key(&self, byte_form: &mut ByteForm);

    /// Appends the row group key of the row.
    ///
    /// The default appends nothing. Then only the row limit of a row group splits the rows.
    fn write_row_group_key(&self, _byte_form: &mut ByteForm) {}
}
