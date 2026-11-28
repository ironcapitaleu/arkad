//! # Filing Row
//!
//! Provides [`FilingRow`], one row of the dataset `filings`: a [`Filing`], the [`SicSource`] of
//! its industry code, and its [`Provenance`].

use std::str::FromStr;
use std::sync::Arc;

use arrow::array::{
    ArrayRef, BooleanArray, Date32Array, Int16Array, ListArray, RecordBatch, StringArray,
    TimestampMillisecondArray, UInt8Array,
};
use arrow::datatypes::{DataType, Field, FieldRef, Fields, Schema, SchemaRef};
use jiff::Timestamp;
use storage::shared::Sha256Digest;
use xbrl::core::entity_name::EntityName;
use xbrl::core::filing::{CompanyAttributes, EntityId, Filing, FilingId, IndustryCode, Ticker};
use xbrl::core::fiscal_period::FiscalPeriod;
use xbrl::core::fiscal_year::FiscalYear;
use xbrl::core::fiscal_year_end::FiscalYearEnd;
use xbrl::core::form::Form;
use xbrl::core::invalid_value::InvalidValue;

use crate::datasets::columns::{
    BatchColumns, date_from_days, date32, failed_batch, list_array, list_item, nullable,
    string_struct_array, string_struct_list, timestamp_array, timestamp_from_milliseconds,
    timestamp_type, unexpected_value, utf8_or_null,
};
use crate::datasets::{ArrowRow, ArrowRowError, Provenance};
use crate::shared::row_digest::{ByteForm, DigestRow};

/// One row of `filings`: a filing, the document of its industry code, and the raw document it
/// comes from.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct FilingRow {
    /// The filing as the source reports it.
    pub filing: Filing,
    /// The `submissions` document that gives the industry code of a new filing. `None` for a
    /// Notes filing, whose raw document gives the code.
    pub sic_source: Option<SicSource>,
    /// The raw document of the filing, and the run that wrote the row.
    pub provenance: Provenance,
}

/// The `submissions` document that gives the industry code of a filing: the columns `sic_*`.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct SicSource {
    /// The key of the document in the document store.
    pub source_key: String,
    /// The SHA-256 of the document.
    pub source_sha256: Sha256Digest,
    /// The time the pipeline fetched the document.
    pub retrieved_at: Timestamp,
}

impl ArrowRow for FilingRow {
    const DATASET: &'static str = "filings";
    const SCHEMA_VERSION: u16 = 1;

    fn schema(version: u16) -> Option<SchemaRef> {
        (version == 1).then(schema_v1)
    }

    fn to_batch(rows: &[Self]) -> Result<RecordBatch, ArrowRowError> {
        let schema = schema_v1();
        let filings = || rows.iter().map(|row| &row.filing);
        let text = |value: fn(&Filing) -> Option<String>| -> ArrayRef {
            Arc::new(StringArray::from_iter(filings().map(value)))
        };
        let sic_text = |value: fn(&SicSource) -> String| -> ArrayRef {
            Arc::new(StringArray::from_iter(
                rows.iter().map(|row| row.sic_source.as_ref().map(value)),
            ))
        };
        let mut columns: Vec<ArrayRef> = vec![
            text(|filing| Some(filing.id.to_string())),
            text(|filing| Some(filing.entity.scheme().to_owned())),
            text(|filing| Some(filing.entity.id().to_owned())),
            text(|filing| Some(filing.form.to_string())),
            Arc::new(BooleanArray::from_iter(
                filings().map(|filing| Some(filing.is_amendment)),
            )),
            Arc::new(Date32Array::from_iter_values(
                filings().map(|filing| date32(filing.filed)),
            )),
            timestamp_array(filings().map(|filing| filing.accepted)),
            Arc::new(Date32Array::from_iter(
                filings().map(|filing| filing.period_of_report.map(date32)),
            )),
            fiscal_year_array(rows)?,
            text(|filing| filing.fiscal_period.as_ref().map(ToString::to_string)),
            text(|filing| filing.company.name.as_ref().map(ToString::to_string)),
            text(|filing| {
                let industry = filing.company.industry.as_ref();
                industry.map(|code| code.scheme().to_owned())
            }),
            text(|filing| {
                let industry = filing.company.industry.as_ref();
                industry.map(|code| code.code().to_owned())
            }),
            tickers_array(&schema, rows),
            text(|filing| {
                let year_end = filing.company.fiscal_year_end.as_ref();
                year_end.map(ToString::to_string)
            }),
            Arc::new(UInt8Array::from_iter(
                filings().map(|filing| filing.company.fiscal_year_end_month()),
            )),
            sic_text(|sic| sic.source_key.clone()),
            sic_text(|sic| sic.source_sha256.to_string()),
            timestamp_array(rows.iter().map(|row| {
                let sic = row.sic_source.as_ref();
                sic.map(|sic| sic.retrieved_at)
            })),
        ];
        let provenances: Vec<&Provenance> = rows.iter().map(|row| &row.provenance).collect();
        columns.extend(Provenance::arrays(&provenances));
        RecordBatch::try_new(schema, columns).map_err(|error| failed_batch(Self::DATASET, &error))
    }

    fn from_batch(batch: &RecordBatch, version: u16) -> Result<Vec<Self>, ArrowRowError> {
        let columns = BatchColumns::check::<Self>(batch, version)?;
        let provenances = Provenance::read(&columns)?;
        let reader = FilingColumns::new(&columns)?;
        provenances
            .into_iter()
            .enumerate()
            .map(|(row, provenance)| {
                Ok(Self {
                    filing: reader.filing(row)?,
                    sic_source: reader.sic_source(row)?,
                    provenance,
                })
            })
            .collect()
    }
}

impl DigestRow for FilingRow {
    fn write_group_id(&self, byte_form: &mut ByteForm) {
        byte_form.utf8(&self.filing.id.to_string());
    }

    fn write_row_key(&self, byte_form: &mut ByteForm) {
        byte_form.utf8(&self.filing.id.to_string());
    }

    fn write_row(&self, byte_form: &mut ByteForm) {
        let filing = &self.filing;
        let company = &filing.company;
        byte_form
            .utf8(&filing.id.to_string())
            .utf8(filing.entity.scheme())
            .utf8(filing.entity.id())
            .utf8(filing.form.as_str())
            .bool(filing.is_amendment)
            .date(filing.filed);
        match filing.accepted {
            Some(accepted) => byte_form.timestamp(accepted),
            None => byte_form.null(),
        };
        match filing.period_of_report {
            Some(period_of_report) => byte_form.date(period_of_report),
            None => byte_form.null(),
        };
        match filing.fiscal_year {
            Some(year) => byte_form.integer(year.value()),
            None => byte_form.null(),
        };
        utf8_or_null(
            byte_form,
            filing.fiscal_period.as_ref().map(FiscalPeriod::as_str),
        );
        utf8_or_null(byte_form, company.name.as_ref().map(EntityName::as_str));
        utf8_or_null(
            byte_form,
            company.industry.as_ref().map(IndustryCode::scheme),
        );
        utf8_or_null(byte_form, company.industry.as_ref().map(IndustryCode::code));
        byte_form.list(company.tickers.len());
        for ticker in &company.tickers {
            byte_form.struct_(2).utf8(&ticker.symbol);
            utf8_or_null(byte_form, ticker.exchange.as_deref());
        }
        utf8_or_null(
            byte_form,
            company.fiscal_year_end.as_ref().map(FiscalYearEnd::as_str),
        );
        match company.fiscal_year_end_month() {
            Some(month) => byte_form.integer(month),
            None => byte_form.null(),
        };
        match &self.sic_source {
            Some(sic) => byte_form
                .utf8(&sic.source_key)
                .utf8(sic.source_sha256.as_str())
                .timestamp(sic.retrieved_at),
            None => byte_form.null().null().null(),
        };
        self.provenance.write_digest(byte_form);
    }
}

/// Returns the schema of version 1.
fn schema_v1() -> SchemaRef {
    let text = |name: &str, nullable: bool| Field::new(name, DataType::Utf8, nullable);
    let date = |name: &str, nullable: bool| Field::new(name, DataType::Date32, nullable);
    let mut fields = vec![
        text("filing_id", false),
        text("entity_scheme", false),
        text("entity_id", false),
        text("form", false),
        Field::new("is_amendment", DataType::Boolean, false),
        date("filed", false),
        Field::new("accepted", timestamp_type(), true),
        date("period_of_report", true),
        Field::new("fiscal_year", DataType::Int16, true),
        text("fiscal_period", true),
        text("company_name", true),
        text("industry_scheme", true),
        text("industry_code", true),
        Field::new("tickers", DataType::List(ticker_item()), false),
        text("fiscal_year_end_raw", true),
        Field::new("fiscal_year_end_month", DataType::UInt8, true),
        text("sic_source_key", true),
        text("sic_source_sha256", true),
        Field::new("sic_retrieved_at", timestamp_type(), true),
    ];
    fields.extend(Provenance::fields());
    Arc::new(Schema::new(fields))
}

/// Returns the item field of the `tickers` column: a struct of `symbol` and a nullable
/// `exchange`.
fn ticker_item() -> FieldRef {
    let ticker = Fields::from(vec![
        Field::new("symbol", DataType::Utf8, false),
        Field::new("exchange", DataType::Utf8, true),
    ]);
    Arc::new(Field::new_list_field(DataType::Struct(ticker), false))
}

/// Returns the `fiscal_year` array of the rows.
///
/// # Errors
///
/// Returns [`ArrowRowError::UnexpectedColumnValue`] if a fiscal year lies above 32767, the
/// largest `int16`.
fn fiscal_year_array(rows: &[FilingRow]) -> Result<ArrayRef, ArrowRowError> {
    let years = rows
        .iter()
        .enumerate()
        .map(|(row, filing_row)| {
            filing_row
                .filing
                .fiscal_year
                .map(|year| {
                    i16::try_from(year.value()).map_err(|_| {
                        unexpected_value(
                            FilingRow::DATASET,
                            "fiscal_year",
                            row,
                            format!("'{year}' lies above {}", i16::MAX),
                        )
                    })
                })
                .transpose()
        })
        .collect::<Result<Vec<_>, _>>()?;
    Ok(Arc::new(Int16Array::from(years)))
}

/// Returns the `tickers` array of the rows.
fn tickers_array(schema: &Schema, rows: &[FilingRow]) -> ArrayRef {
    let item = list_item(schema, "tickers");
    let tickers = || rows.iter().flat_map(|row| &row.filing.company.tickers);
    let lengths = rows
        .iter()
        .map(|row| row.filing.company.tickers.len())
        .collect();
    let symbols = StringArray::from_iter_values(tickers().map(|ticker| &ticker.symbol));
    let exchanges = StringArray::from_iter(tickers().map(|ticker| ticker.exchange.as_deref()));
    let structs = string_struct_array(&item, vec![symbols, exchanges]);
    list_array(item, lengths, structs)
}

/// The arrays of the filing and `sic_*` columns of one batch.
struct FilingColumns<'a> {
    dataset: &'static str,
    filing_ids: &'a StringArray,
    entity_schemes: &'a StringArray,
    entity_ids: &'a StringArray,
    forms: &'a StringArray,
    amendments: &'a BooleanArray,
    filed_dates: &'a Date32Array,
    accepted_times: &'a TimestampMillisecondArray,
    periods_of_report: &'a Date32Array,
    fiscal_years: &'a Int16Array,
    fiscal_periods: &'a StringArray,
    company_names: &'a StringArray,
    industry_schemes: &'a StringArray,
    industry_codes: &'a StringArray,
    tickers: &'a ListArray,
    fiscal_year_ends: &'a StringArray,
    sic_source_keys: &'a StringArray,
    sic_source_digests: &'a StringArray,
    sic_retrieved_times: &'a TimestampMillisecondArray,
}

impl<'a> FilingColumns<'a> {
    /// Takes the arrays of the filing and `sic_*` columns from a batch that passed the schema
    /// check.
    ///
    /// The reader takes no `fiscal_year_end_month`, because the month follows from
    /// `fiscal_year_end_raw`.
    fn new(columns: &BatchColumns<'a>) -> Result<Self, ArrowRowError> {
        Ok(Self {
            dataset: columns.dataset(),
            filing_ids: columns.get("filing_id")?,
            entity_schemes: columns.get("entity_scheme")?,
            entity_ids: columns.get("entity_id")?,
            forms: columns.get("form")?,
            amendments: columns.get("is_amendment")?,
            filed_dates: columns.get("filed")?,
            accepted_times: columns.get("accepted")?,
            periods_of_report: columns.get("period_of_report")?,
            fiscal_years: columns.get("fiscal_year")?,
            fiscal_periods: columns.get("fiscal_period")?,
            company_names: columns.get("company_name")?,
            industry_schemes: columns.get("industry_scheme")?,
            industry_codes: columns.get("industry_code")?,
            tickers: columns.get("tickers")?,
            fiscal_year_ends: columns.get("fiscal_year_end_raw")?,
            sic_source_keys: columns.get("sic_source_key")?,
            sic_source_digests: columns.get("sic_source_sha256")?,
            sic_retrieved_times: columns.get("sic_retrieved_at")?,
        })
    }

    /// Reads the filing of one row.
    fn filing(&self, row: usize) -> Result<Filing, ArrowRowError> {
        let invalid = |column: &'static str| {
            move |source: InvalidValue| ArrowRowError::InvalidColumnValue {
                dataset: self.dataset,
                column,
                row,
                source,
            }
        };
        let unexpected = |column: &'static str| {
            move |reason| unexpected_value(self.dataset, column, row, reason)
        };
        let fiscal_year = nullable(self.fiscal_years, row)
            .map(|year| {
                u16::try_from(year)
                    .map(FiscalYear::new)
                    .map_err(|_| unexpected("fiscal_year")(format!("'{year}' lies below 0")))
            })
            .transpose()?;
        Ok(Filing {
            id: FilingId::from_str(self.filing_ids.value(row)).map_err(invalid("filing_id"))?,
            entity: EntityId::new(self.entity_schemes.value(row), self.entity_ids.value(row))
                .map_err(invalid("entity_id"))?,
            form: Form::new(self.forms.value(row)).map_err(invalid("form"))?,
            is_amendment: self.amendments.value(row),
            filed: date_from_days(self.filed_dates.value(row)).map_err(unexpected("filed"))?,
            accepted: nullable(self.accepted_times, row)
                .map(timestamp_from_milliseconds)
                .transpose()
                .map_err(unexpected("accepted"))?,
            period_of_report: nullable(self.periods_of_report, row)
                .map(date_from_days)
                .transpose()
                .map_err(unexpected("period_of_report"))?,
            fiscal_year,
            fiscal_period: nullable(self.fiscal_periods, row)
                .map(FiscalPeriod::new)
                .transpose()
                .map_err(invalid("fiscal_period"))?,
            company: CompanyAttributes {
                name: nullable(self.company_names, row).map(EntityName::new),
                industry: self.industry(row)?,
                tickers: self.tickers(row),
                fiscal_year_end: nullable(self.fiscal_year_ends, row)
                    .map(FiscalYearEnd::from_str)
                    .transpose()
                    .map_err(invalid("fiscal_year_end_raw"))?,
            },
        })
    }

    /// Reads the industry code of one row from `industry_scheme` and `industry_code`.
    ///
    /// Fails if one of the two columns is set and the other is null. The error names the null
    /// column.
    fn industry(&self, row: usize) -> Result<Option<IndustryCode>, ArrowRowError> {
        let mix = |column| {
            Err(unexpected_value(
                self.dataset,
                column,
                row,
                "'industry_scheme' and 'industry_code' are not both set or both null".to_owned(),
            ))
        };
        match (
            nullable(self.industry_schemes, row),
            nullable(self.industry_codes, row),
        ) {
            (Some(scheme), Some(code)) => {
                IndustryCode::new(scheme, code).map(Some).map_err(|source| {
                    ArrowRowError::InvalidColumnValue {
                        dataset: self.dataset,
                        column: "industry_code",
                        row,
                        source,
                    }
                })
            }
            (None, None) => Ok(None),
            (None, Some(_)) => mix("industry_scheme"),
            (Some(_), None) => mix("industry_code"),
        }
    }

    /// Reads the tickers of one row.
    fn tickers(&self, row: usize) -> Vec<Ticker> {
        string_struct_list(self.tickers, row)
            .into_iter()
            .map(|ticker| {
                let mut fields = ticker.into_iter();
                Ticker {
                    // `symbol` is not null in the schema, so the default never applies.
                    symbol: fields.next().flatten().unwrap_or_default(),
                    exchange: fields.next().flatten(),
                }
            })
            .collect()
    }

    /// Reads the `sic_*` columns of one row.
    ///
    /// Fails if some of the three columns are set and others are null. The error names the first
    /// null column.
    fn sic_source(&self, row: usize) -> Result<Option<SicSource>, ArrowRowError> {
        let unexpected = |column, reason| unexpected_value(self.dataset, column, row, reason);
        let values = (
            nullable(self.sic_source_keys, row),
            nullable(self.sic_source_digests, row),
            nullable(self.sic_retrieved_times, row),
        );
        match values {
            (Some(key), Some(digest), Some(milliseconds)) => Ok(Some(SicSource {
                source_key: key.to_owned(),
                source_sha256: Sha256Digest::from_str(digest)
                    .map_err(|error| unexpected("sic_source_sha256", error.to_string()))?,
                retrieved_at: timestamp_from_milliseconds(milliseconds)
                    .map_err(|reason| unexpected("sic_retrieved_at", reason))?,
            })),
            (None, None, None) => Ok(None),
            (key, digest, milliseconds) => {
                let nulls = [
                    ("sic_source_key", key.is_none()),
                    ("sic_source_sha256", digest.is_none()),
                    ("sic_retrieved_at", milliseconds.is_none()),
                ];
                // This arm holds at least one null column, so the default never applies.
                let column = nulls
                    .into_iter()
                    .find_map(|(name, is_null)| is_null.then_some(name))
                    .unwrap_or("sic_source_key");
                Err(unexpected(
                    column,
                    "the 'sic_*' columns are not all set or all null".to_owned(),
                ))
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use std::fmt::Debug;
    use std::hash::Hash;

    use jiff::civil::date;
    use pretty_assertions::assert_eq;
    use uuid::Uuid;

    use super::*;
    use crate::datasets::implementations::provenance::sample_provenance;
    use crate::datasets::{FOLDER_LABELS, UTC_TIME_ZONE, schema_text};

    const PARSE: &str = "Given hardcoded valid parts, the creation should always succeed";

    /// A filing of `sec:0000320193-26-000001` with every nullable field empty.
    fn empty_filing() -> Filing {
        Filing {
            id: FilingId::from_str("sec:0000320193-26-000001").expect(PARSE),
            entity: EntityId::new("http://www.sec.gov/CIK", "0000320193").expect(PARSE),
            form: Form::new("10-Q").expect(PARSE),
            is_amendment: false,
            filed: date(2026, 11, 6),
            accepted: None,
            period_of_report: None,
            fiscal_year: None,
            fiscal_period: None,
            company: CompanyAttributes::default(),
        }
    }

    /// The fixture rows: an amended 10-K with every field set, a non-ASCII company name, and two
    /// tickers with and without exchange, then a row with every nullable field empty.
    fn sample_rows() -> Vec<FilingRow> {
        let full = Filing {
            form: Form::new("10-K/A").expect(PARSE),
            is_amendment: true,
            accepted: Some(Timestamp::from_millisecond(1_793_966_700_000).expect(PARSE)),
            period_of_report: Some(date(2026, 9, 26)),
            fiscal_year: Some(FiscalYear::new(2026)),
            fiscal_period: Some(FiscalPeriod::new("FY").expect(PARSE)),
            company: CompanyAttributes {
                name: Some(EntityName::new("Société Générale — 株式会社 ✓")),
                industry: Some(IndustryCode::new(IndustryCode::SIC_SCHEME, "3571").expect(PARSE)),
                tickers: vec![
                    Ticker {
                        symbol: "AAPL".to_owned(),
                        exchange: Some("NASDAQ".to_owned()),
                    },
                    Ticker {
                        symbol: "BRK-B".to_owned(),
                        exchange: None,
                    },
                ],
                fiscal_year_end: Some(FiscalYearEnd::from_str("--09-27").expect(PARSE)),
            },
            ..empty_filing()
        };
        vec![
            FilingRow {
                filing: full,
                sic_source: Some(SicSource {
                    source_key: "sec/submissions/CIK0000320193.json".to_owned(),
                    source_sha256: Sha256Digest::of(b"submissions"),
                    retrieved_at: Timestamp::from_millisecond(1_793_966_400_000).expect(PARSE),
                }),
                provenance: sample_provenance(),
            },
            FilingRow {
                filing: empty_filing(),
                sic_source: None,
                provenance: sample_provenance(),
            },
        ]
    }

    /// Returns the batch of the first fixture row, with the column `name` replaced by `column`.
    fn batch_with(name: &str, column: ArrayRef) -> RecordBatch {
        let batch = FilingRow::to_batch(&sample_rows()[..1])
            .expect("Given rows whose values fit their columns, the batch should always build");
        let index = batch
            .schema()
            .index_of(name)
            .expect("Given a column of the schema of version 1, the lookup should always succeed");
        let mut columns = batch.columns().to_vec();
        columns[index] = column;
        RecordBatch::try_new(batch.schema(), columns)
            .expect("Given a column of the same type, the batch should always build")
    }

    /// Returns the rows of the first fixture row with the fiscal year `year`.
    fn rows_with_fiscal_year(year: u16) -> Vec<FilingRow> {
        let mut rows = sample_rows();
        rows.truncate(1);
        rows[0].filing.fiscal_year = Some(FiscalYear::new(year));
        rows
    }

    /// Returns the column that the error of the read of `batch` names, or `None` if the read
    /// passes or fails with another error.
    fn failed_column(batch: &RecordBatch) -> Option<&'static str> {
        match FilingRow::from_batch(batch, 1) {
            Err(
                ArrowRowError::UnexpectedColumnValue { column, .. }
                | ArrowRowError::InvalidColumnValue { column, .. },
            ) => Some(column),
            _ => None,
        }
    }

    const fn implements_auto_traits<T: Sized + Send + Sync + Unpin>() {}
    #[test]
    const fn should_implement_auto_traits_for_filing_row() {
        implements_auto_traits::<FilingRow>();
    }

    const fn implements_value_traits<T: Debug + Clone + PartialEq + Eq + Hash>() {}
    #[test]
    const fn should_implement_value_traits_for_filing_row() {
        implements_value_traits::<FilingRow>();
    }

    #[test]
    const fn should_implement_value_traits_for_sic_source() {
        implements_value_traits::<SicSource>();
    }

    #[test]
    fn should_return_equal_rows_when_rows_pass_through_a_batch() {
        let rows = sample_rows();

        let expected_result = sample_rows();

        let batch = FilingRow::to_batch(&rows)
            .expect("Given rows whose values fit their columns, the batch should always build");
        let result = FilingRow::from_batch(&batch, FilingRow::SCHEMA_VERSION)
            .expect("Given a batch that `to_batch` built, the read should always succeed");

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_match_the_checked_in_snapshot_for_the_schema_of_version_1() {
        let schema = FilingRow::schema(1).expect("Given version 1, the schema should exist");

        let expected_result = include_str!("schema_v1.txt");

        let result = schema_text(&schema);

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_name_no_field_after_a_folder_label_for_filings() {
        let schema = FilingRow::schema(FilingRow::SCHEMA_VERSION)
            .expect("Given the version that `to_batch` writes, the schema should exist");

        let expected_result: Vec<&str> = Vec::new();

        let result: Vec<&str> = schema
            .fields()
            .iter()
            .map(|field| field.name().as_str())
            .filter(|name| FOLDER_LABELS.contains(name))
            .collect();

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_fail_when_the_schema_version_is_unknown() {
        let batch = FilingRow::to_batch(&sample_rows())
            .expect("Given rows whose values fit their columns, the batch should always build");

        let expected_result = Err(ArrowRowError::UnknownSchemaVersion {
            dataset: "filings",
            version: 2,
        });

        let result = FilingRow::from_batch(&batch, 2);

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_fail_when_a_field_of_the_batch_differs_from_the_schema() {
        let schema = Arc::new(Schema::new(vec![
            Field::new("filing_id", DataType::Utf8, false),
            Field::new("entity_scheme", DataType::Utf8, true),
        ]));
        let columns: Vec<ArrayRef> = vec![
            Arc::new(StringArray::from(vec!["sec:0000320193-26-000001"])),
            Arc::new(StringArray::from(vec!["http://www.sec.gov/CIK"])),
        ];
        let batch = RecordBatch::try_new(schema, columns)
            .expect("Given two utf8 columns for two utf8 fields, the batch should always build");

        let expected_result = Err(ArrowRowError::UnexpectedSchema {
            dataset: "filings",
            version: 1,
            reason: "expected field 'entity_scheme: utf8', found 'entity_scheme: utf8 nullable'"
                .to_owned(),
        });

        let result = FilingRow::from_batch(&batch, 1);

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_fail_when_the_sic_columns_mix_set_and_null_values() {
        let batch = batch_with("sic_retrieved_at", timestamp_array([None::<Timestamp>]));

        let expected_result = Err(ArrowRowError::UnexpectedColumnValue {
            dataset: "filings",
            column: "sic_retrieved_at",
            row: 0,
            reason: "the 'sic_*' columns are not all set or all null".to_owned(),
        });

        let result = FilingRow::from_batch(&batch, 1);

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_fail_when_the_industry_columns_mix_set_and_null_values() {
        let batch = batch_with(
            "industry_scheme",
            Arc::new(StringArray::from(vec![None::<&str>])),
        );

        let expected_result = Err(ArrowRowError::UnexpectedColumnValue {
            dataset: "filings",
            column: "industry_scheme",
            row: 0,
            reason: "'industry_scheme' and 'industry_code' are not both set or both null"
                .to_owned(),
        });

        let result = FilingRow::from_batch(&batch, 1);

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_fail_when_a_fiscal_year_lies_above_the_largest_int16() {
        let rows = rows_with_fiscal_year(32_768);

        let expected_result = Err(ArrowRowError::UnexpectedColumnValue {
            dataset: "filings",
            column: "fiscal_year",
            row: 0,
            reason: "'32768' lies above 32767".to_owned(),
        });

        let result = FilingRow::to_batch(&rows);

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_return_equal_rows_when_the_fiscal_year_is_the_largest_int16() {
        let rows = rows_with_fiscal_year(32_767);

        let expected_result = rows_with_fiscal_year(32_767);

        let batch = FilingRow::to_batch(&rows)
            .expect("Given a fiscal year that fits int16, the batch should always build");
        let result = FilingRow::from_batch(&batch, FilingRow::SCHEMA_VERSION)
            .expect("Given a batch that `to_batch` built, the read should always succeed");

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_fail_when_a_stored_fiscal_year_is_negative() {
        let batch = batch_with("fiscal_year", Arc::new(Int16Array::from(vec![-1])));

        let expected_result = Err(ArrowRowError::UnexpectedColumnValue {
            dataset: "filings",
            column: "fiscal_year",
            row: 0,
            reason: "'-1' lies below 0".to_owned(),
        });

        let result = FilingRow::from_batch(&batch, 1);

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_write_the_month_of_the_fiscal_year_end_when_the_raw_text_is_set() {
        let batch = FilingRow::to_batch(&sample_rows())
            .expect("Given rows whose values fit their columns, the batch should always build");

        let expected_result = Some(UInt8Array::from(vec![Some(9), None]));

        let result = batch
            .column_by_name("fiscal_year_end_month")
            .and_then(|column| column.as_any().downcast_ref::<UInt8Array>())
            .cloned();

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_read_the_fiscal_year_end_from_the_raw_text_when_the_month_column_differs() {
        let batch = batch_with("fiscal_year_end_month", Arc::new(UInt8Array::from(vec![1])));

        let expected_result = sample_rows()[..1].to_vec();

        let result = FilingRow::from_batch(&batch, 1).expect(
            "Given a batch whose other columns `to_batch` built, the read should always succeed",
        );

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_return_no_rows_when_the_batch_holds_no_rows() {
        let rows: Vec<FilingRow> = Vec::new();

        let expected_result: Vec<FilingRow> = Vec::new();

        let batch =
            FilingRow::to_batch(&rows).expect("Given no rows, the batch should always build");
        let result = FilingRow::from_batch(&batch, FilingRow::SCHEMA_VERSION)
            .expect("Given a batch that `to_batch` built, the read should always succeed");

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_write_the_same_digest_bytes_when_only_the_run_columns_differ() {
        let first = sample_rows().remove(0);
        let mut second = first.clone();
        second.provenance.run.execution_id = Uuid::from_u128(2);
        second.provenance.run.loaded_at = Timestamp::UNIX_EPOCH;
        let mut first_bytes = ByteForm::new();
        first.write_row(&mut first_bytes);

        let expected_result = first_bytes;

        let mut result = ByteForm::new();
        second.write_row(&mut result);

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_write_the_filing_id_as_the_digest_group() {
        let row = sample_rows().remove(0);

        // marker 01 | length 00000018 | text sec:0000320193-26-000001
        let expected_result: &[u8] = b"\x01\x00\x00\x00\x18sec:0000320193-26-000001";

        let mut byte_form = ByteForm::new();
        row.write_group_id(&mut byte_form);
        let result = byte_form.as_bytes();

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_write_the_filing_id_as_the_row_key() {
        let row = sample_rows().remove(0);

        // marker 01 | length 00000018 | text sec:0000320193-26-000001
        let expected_result: &[u8] = b"\x01\x00\x00\x00\x18sec:0000320193-26-000001";

        let mut byte_form = ByteForm::new();
        row.write_row_key(&mut byte_form);
        let result = byte_form.as_bytes();

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_write_every_column_but_the_run_ids_in_schema_order_as_the_row() {
        let row = sample_rows().remove(0);

        // filing_id | entity_scheme | entity_id | form | is_amendment | filed | accepted |
        // period_of_report | fiscal_year | fiscal_period | company_name | industry_scheme |
        // industry_code | tickers: list of 2, struct of 2 (AAPL, NASDAQ), struct of 2 (BRK-B,
        // null) | fiscal_year_end_raw | fiscal_year_end_month | sic_source_key |
        // sic_source_sha256 | sic_retrieved_at | source_kind | source_key | source_sha256 |
        // ingestion_mode | transform_version
        let expected_result: &[u8] = b"\x01\x00\x00\x00\x18sec:0000320193-26-000001\
\x01\x00\x00\x00\x16http://www.sec.gov/CIK\
\x01\x00\x00\x00\x0a0000320193\
\x01\x00\x00\x00\x0610-K/A\
\x01\x00\x00\x00\x04true\
\x01\x00\x00\x00\x0a2026-11-06\
\x01\x00\x00\x00\x182026-11-06T12:05:00.000Z\
\x01\x00\x00\x00\x0a2026-09-26\
\x01\x00\x00\x00\x042026\
\x01\x00\x00\x00\x02FY\
\x01\x00\x00\x00\x29Soci\xc3\xa9t\xc3\xa9 G\xc3\xa9n\xc3\xa9rale \xe2\x80\x94 \xe6\xa0\xaa\xe5\xbc\x8f\xe4\xbc\x9a\xe7\xa4\xbe \xe2\x9c\x93\
\x01\x00\x00\x00\x03sic\
\x01\x00\x00\x00\x043571\
\x01\x00\x00\x00\x02\x01\x00\x00\x00\x02\x01\x00\x00\x00\x04AAPL\x01\x00\x00\x00\x06NASDAQ\x01\x00\x00\x00\x02\x01\x00\x00\x00\x05BRK-B\x00\
\x01\x00\x00\x00\x07--09-27\
\x01\x00\x00\x00\x019\
\x01\x00\x00\x00\x22sec/submissions/CIK0000320193.json\
\x01\x00\x00\x00\x4021e945ad5e3b81adf2ac82114f7e701ca858a81c9b678ea7d23f95927b71d424\
\x01\x00\x00\x00\x182026-11-06T12:00:00.000Z\
\x01\x00\x00\x00\x08instance\
\x01\x00\x00\x00\x26sec/instances/0000320193-26-000001.xml\
\x01\x00\x00\x00\x405e8c03a94c7835b87dd82c690f9f85c08592ef7433aa141ceaacb36a9f4da434\
\x01\x00\x00\x00\x0bincremental\
\x01\x00\x00\x00\x011";

        let mut byte_form = ByteForm::new();
        row.write_row(&mut byte_form);
        let result = byte_form.as_bytes();

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_write_a_null_for_each_empty_column_when_every_nullable_field_is_empty() {
        let row = sample_rows().remove(1);

        // filing_id | entity_scheme | entity_id | form 10-Q | is_amendment false | filed |
        // accepted to industry_code: 7 nulls | tickers: list of 0 | fiscal_year_end_raw to
        // sic_retrieved_at: 5 nulls | the provenance as in the full row
        let expected_result: &[u8] = b"\x01\x00\x00\x00\x18sec:0000320193-26-000001\
\x01\x00\x00\x00\x16http://www.sec.gov/CIK\
\x01\x00\x00\x00\x0a0000320193\
\x01\x00\x00\x00\x0410-Q\
\x01\x00\x00\x00\x05false\
\x01\x00\x00\x00\x0a2026-11-06\
\x00\x00\x00\x00\x00\x00\x00\
\x01\x00\x00\x00\x00\
\x00\x00\x00\x00\x00\
\x01\x00\x00\x00\x08instance\
\x01\x00\x00\x00\x26sec/instances/0000320193-26-000001.xml\
\x01\x00\x00\x00\x405e8c03a94c7835b87dd82c690f9f85c08592ef7433aa141ceaacb36a9f4da434\
\x01\x00\x00\x00\x0bincremental\
\x01\x00\x00\x00\x011";

        let mut byte_form = ByteForm::new();
        row.write_row(&mut byte_form);
        let result = byte_form.as_bytes();

        assert_eq!(result, expected_result);
    }
    #[test]
    fn should_name_the_column_when_the_filing_id_has_no_source() {
        let batch = batch_with(
            "filing_id",
            Arc::new(StringArray::from(vec!["0000320193-26-000001"])),
        );

        let expected_result = Some("filing_id");

        let result = failed_column(&batch);

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_name_the_column_when_the_form_is_empty() {
        let batch = batch_with("form", Arc::new(StringArray::from(vec![""])));

        let expected_result = Some("form");

        let result = failed_column(&batch);

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_name_the_column_when_the_fiscal_period_is_empty() {
        let batch = batch_with("fiscal_period", Arc::new(StringArray::from(vec![""])));

        let expected_result = Some("fiscal_period");

        let result = failed_column(&batch);

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_name_the_column_when_the_fiscal_year_end_is_not_a_day_of_the_year() {
        let batch = batch_with(
            "fiscal_year_end_raw",
            Arc::new(StringArray::from(vec!["1332"])),
        );

        let expected_result = Some("fiscal_year_end_raw");

        let result = failed_column(&batch);

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_name_the_column_when_the_industry_code_holds_whitespace() {
        let batch = batch_with("industry_code", Arc::new(StringArray::from(vec!["35 71"])));

        let expected_result = Some("industry_code");

        let result = failed_column(&batch);

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_name_the_column_when_the_sic_source_sha256_is_not_a_digest() {
        let batch = batch_with(
            "sic_source_sha256",
            Arc::new(StringArray::from(vec!["abc"])),
        );

        let expected_result = Some("sic_source_sha256");

        let result = failed_column(&batch);

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_name_the_column_when_sic_retrieved_at_lies_past_the_times_of_jiff() {
        let batch = batch_with(
            "sic_retrieved_at",
            Arc::new(TimestampMillisecondArray::from(vec![i64::MAX]).with_timezone(UTC_TIME_ZONE)),
        );

        let expected_result = Some("sic_retrieved_at");

        let result = failed_column(&batch);

        assert_eq!(result, expected_result);
    }
}
