//! # Row Filter
//!
//! Provides [`RowFilter`], the rows of a unit that [`read_rows()`](super::read_rows()) returns.

use std::collections::BTreeSet;

use arrow::array::{Array, BooleanArray, RecordBatch, UInt8Array};
use arrow::compute::filter_record_batch;
use parquet::arrow::arrow_reader::ArrowReaderMetadata;
use parquet::arrow::arrow_reader::statistics::StatisticsConverter;

use super::constants::ENTITY_BUCKET_COLUMN;

/// The rows of a unit that [`read_rows()`](super::read_rows()) returns.
#[derive(Debug, Clone, Default, PartialEq, Eq, Hash)]
pub enum RowFilter {
    /// Every row.
    #[default]
    All,
    /// The rows whose `entity_bucket` is in the set.
    ///
    /// The reader skips each row group whose minimum and maximum `entity_bucket` hold no bucket of
    /// the set. A part of `candidates` starts a new row group at each bucket, so the reader decodes
    /// only the row groups of those buckets. The part must have the column.
    EntityBuckets(BTreeSet<u8>),
}

impl RowFilter {
    /// Returns the row groups of a part that can hold a row of the filter, or `None` for all.
    ///
    /// A row group without statistics of the column stays in.
    ///
    /// # Errors
    ///
    /// Returns the reason if the part has no `uint8` column `entity_bucket`.
    pub(crate) fn row_groups(
        &self,
        part: &ArrowReaderMetadata,
    ) -> Result<Option<Vec<usize>>, String> {
        let Self::EntityBuckets(buckets) = self else {
            return Ok(None);
        };
        let converter = StatisticsConverter::try_new(
            ENTITY_BUCKET_COLUMN,
            part.schema(),
            part.parquet_schema(),
        )
        .map_err(|error| error.to_string())?;
        let row_groups = part.metadata().row_groups();
        let minimums = converter
            .row_group_mins(row_groups)
            .map_err(|error| error.to_string())?;
        let maximums = converter
            .row_group_maxes(row_groups)
            .map_err(|error| error.to_string())?;
        let (Some(minimums), Some(maximums)) = (
            minimums.as_any().downcast_ref::<UInt8Array>(),
            maximums.as_any().downcast_ref::<UInt8Array>(),
        ) else {
            return Err(not_uint8());
        };
        let can_hold_a_bucket = |group: usize| {
            if minimums.is_null(group) || maximums.is_null(group) {
                return true;
            }
            let (minimum, maximum) = (minimums.value(group), maximums.value(group));
            // A minimum above the maximum is no valid range, so the group stays in.
            minimum > maximum || buckets.range(minimum..=maximum).next().is_some()
        };
        Ok(Some(
            (0..row_groups.len())
                .filter(|&group| can_hold_a_bucket(group))
                .collect(),
        ))
    }

    /// Returns the rows of the batch that the filter keeps.
    ///
    /// # Errors
    ///
    /// Returns the reason if the batch has no `uint8` column `entity_bucket`.
    pub(crate) fn apply(&self, batch: RecordBatch) -> Result<RecordBatch, String> {
        let Self::EntityBuckets(buckets) = self else {
            return Ok(batch);
        };
        let column = batch
            .column_by_name(ENTITY_BUCKET_COLUMN)
            .and_then(|column| column.as_any().downcast_ref::<UInt8Array>())
            .ok_or_else(not_uint8)?;
        let keep: BooleanArray = column
            .iter()
            .map(|bucket| Some(bucket.is_some_and(|bucket| buckets.contains(&bucket))))
            .collect();
        filter_record_batch(&batch, &keep).map_err(|error| error.to_string())
    }
}

fn not_uint8() -> String {
    format!("the part has no uint8 column {ENTITY_BUCKET_COLUMN}")
}

#[cfg(test)]
mod tests {
    use parquet::arrow::arrow_reader::ArrowReaderOptions;
    use pretty_assertions::assert_eq;
    use storage::ReadRepository;
    use storage::implementations::repository::filesystem::DocumentKey;

    use super::*;
    use crate::datasets::ArrowRow;
    use crate::datasets::unit_reader::fixtures::{FakeRow, notes_unit, store, unit_with_part};

    #[tokio::test]
    async fn should_keep_only_the_row_group_of_the_bucket_when_the_filter_names_one_bucket() {
        let (_directory, store) = store();
        let rows = [3, 5, 7]
            .map(|entity_bucket| FakeRow {
                filing_id: "0000320193-24-000006".to_owned(),
                entity_bucket,
                unit: None,
            })
            .to_vec();
        let unit = unit_with_part(&store, notes_unit("2024q1", 1), rows).await;
        let key = DocumentKey::new(&unit.files[0].path)
            .expect("Given a path that the writer built, the key should always form");
        let part = store
            .get(key)
            .await
            .expect("Given a part that the writer wrote, the read should always succeed")
            .expect("Given a part that the writer wrote, the part should always exist");
        let metadata = ArrowReaderMetadata::load(&part.bytes, ArrowReaderOptions::new())
            .expect("Given a part that the writer wrote, the footer should always load");
        let filter = RowFilter::EntityBuckets(BTreeSet::from([5]));

        // The part has one row group per bucket: 3, 5, and 7.
        let expected_result = Ok(Some(vec![1]));

        let result = filter.row_groups(&metadata);

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_drop_the_rows_of_other_buckets_when_a_batch_holds_several_buckets() {
        let rows = [3, 5]
            .map(|entity_bucket| FakeRow {
                filing_id: "0000320193-24-000006".to_owned(),
                entity_bucket,
                unit: None,
            })
            .to_vec();
        let batch =
            FakeRow::to_batch(&rows).expect("Given fake rows, the batch should always build");
        let filter = RowFilter::EntityBuckets(BTreeSet::from([5]));

        let expected_result = vec![FakeRow {
            filing_id: "0000320193-24-000006".to_owned(),
            entity_bucket: 5,
            unit: None,
        }];

        let kept = filter
            .apply(batch)
            .expect("Given a batch with the bucket column, the filter should always apply");
        let result = FakeRow::from_batch(&kept, FakeRow::SCHEMA_VERSION)
            .expect("Given a batch of the current version, the rows should always read");

        assert_eq!(result, expected_result);
    }
}
