//! Golden test of `content_digest`: pins the byte form to `TRANSFORM_VERSION`.
//!
//! The fixture unit holds every column type of the byte form. If a print rule, the sort order, or
//! the frame changes, the digest of the fixture changes. Then the test fails until
//! `TRANSFORM_VERSION` changes and the table holds the new digest for the new version. Keep the
//! rows of the old versions in the table.

use std::str::FromStr;

use bigdecimal::BigDecimal;
use chrono::{DateTime, NaiveDate, TimeZone, Utc};
use pretty_assertions::assert_eq;

use sec::shared::content_digest::{ByteForm, DigestRow, content_digest, group_hashes};
use sec::shared::transform_version::TRANSFORM_VERSION;

/// The expected `content_digest` of the fixture unit, by `transform_version`.
const GOLDEN_DIGESTS: [(&str, &str); 1] = [(
    "1",
    "512cb2e35253c41117e31412379e1b7013e0933b15d7c0fb5810e11f03d548b2",
)];

/// One dimension of a fixture row: an axis and a member, which can be null.
struct Dimension {
    axis: &'static str,
    member: Option<&'static str>,
}

/// A fixture row with one column of each type of the byte form.
struct FixtureRow {
    filing_id: &'static str,
    concept: &'static str,
    dimensions: Vec<Dimension>,
    period_end: NaiveDate,
    quarters: u16,
    value: Option<BigDecimal>,
    is_custom: bool,
    accepted_at: DateTime<Utc>,
}

impl DigestRow for FixtureRow {
    fn write_group_id(&self, byte_form: &mut ByteForm) {
        byte_form.utf8(self.filing_id);
    }

    fn write_row_key(&self, byte_form: &mut ByteForm) {
        byte_form.utf8(self.concept);
        write_dimensions(byte_form, &self.dimensions);
    }

    fn write_row(&self, byte_form: &mut ByteForm) {
        byte_form.utf8(self.filing_id).utf8(self.concept);
        write_dimensions(byte_form, &self.dimensions);
        byte_form.date(self.period_end).integer(self.quarters);
        match &self.value {
            Some(value) => byte_form.decimal(value),
            None => byte_form.null(),
        };
        byte_form.bool(self.is_custom).timestamp(self.accepted_at);
    }
}

fn write_dimensions(byte_form: &mut ByteForm, dimensions: &[Dimension]) {
    byte_form.list(dimensions.len());
    for dimension in dimensions {
        byte_form.struct_(2).utf8(dimension.axis);
        match dimension.member {
            Some(member) => byte_form.utf8(member),
            None => byte_form.null(),
        };
    }
}

fn decimal(text: &str) -> BigDecimal {
    BigDecimal::from_str(text)
        .expect("Given a hardcoded decimal string, the parse should always succeed")
}

const fn date(year: i32, month: u32, day: u32) -> NaiveDate {
    NaiveDate::from_ymd_opt(year, month, day)
        .expect("Given a hardcoded valid date, the creation should always succeed")
}

fn accepted_at(hour: u32, minute: u32) -> DateTime<Utc> {
    Utc.with_ymd_and_hms(2024, 2, 2, hour, minute, 0)
        .single()
        .expect("Given a hardcoded valid UTC time, the creation should always succeed")
}

/// The fixture unit: two filings, a null value, a null member, a negative decimal, and a decimal
/// with trailing zeros.
fn fixture_unit() -> Vec<FixtureRow> {
    vec![
        FixtureRow {
            filing_id: "0000789019-24-000010",
            concept: "Revenues",
            dimensions: vec![Dimension {
                axis: "srt:ProductOrServiceAxis",
                member: Some("msft:CloudMember"),
            }],
            period_end: date(2023, 12, 31),
            quarters: 1,
            value: Some(decimal("25880000000.00")),
            is_custom: false,
            accepted_at: accepted_at(21, 30),
        },
        FixtureRow {
            filing_id: "0000320193-24-000006",
            concept: "Assets",
            dimensions: Vec::new(),
            period_end: date(2023, 12, 30),
            quarters: 0,
            value: Some(decimal("353514000000")),
            is_custom: false,
            accepted_at: accepted_at(18, 3),
        },
        FixtureRow {
            filing_id: "0000320193-24-000006",
            concept: "EarningsPerShareDiluted",
            dimensions: Vec::new(),
            period_end: date(2023, 12, 30),
            quarters: 1,
            value: Some(decimal("2.180")),
            is_custom: false,
            accepted_at: accepted_at(18, 3),
        },
        FixtureRow {
            filing_id: "0000320193-24-000006",
            concept: "aapl:OtherIncomeExpenseNet",
            dimensions: vec![Dimension {
                axis: "us-gaap:StatementBusinessSegmentsAxis",
                member: None,
            }],
            period_end: date(2023, 12, 30),
            quarters: 4007,
            value: Some(decimal("-0.25")),
            is_custom: true,
            accepted_at: accepted_at(18, 3),
        },
        FixtureRow {
            filing_id: "0000320193-24-000006",
            concept: "Liabilities",
            dimensions: Vec::new(),
            period_end: date(2023, 12, 30),
            quarters: 0,
            value: None,
            is_custom: false,
            accepted_at: accepted_at(18, 3),
        },
    ]
}

#[test]
fn should_match_the_golden_digest_for_the_current_transform_version() {
    let rows = fixture_unit();

    let expected_result = GOLDEN_DIGESTS
        .iter()
        .find(|(version, _)| *version == TRANSFORM_VERSION)
        .map(|(_, digest)| (*digest).to_string());

    let result = content_digest(group_hashes(&rows))
        .ok()
        .map(|digest| digest.to_string());

    assert_eq!(result, expected_result);
}
