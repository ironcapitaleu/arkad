//! # Filing Fields
//!
//! Provides [`FilingFields`], the metadata fields that name the filing a raw document belongs to.

use chrono::NaiveDate;
use serde::{Deserialize, Serialize};

use crate::shared::filing_ref::FilingRef;

/// The filing a raw document belongs to: its filed date and its form.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct FilingFields {
    /// The filed date of the filing in the daily index.
    pub filed_date: NaiveDate,
    /// The form of the filing, for example `10-Q`, without the `/A` of an amendment.
    pub form: String,
    /// `true` if the filing amends an earlier one, for example a `10-Q/A`.
    pub is_amendment: bool,
}

impl From<&FilingRef> for FilingFields {
    fn from(filing: &FilingRef) -> Self {
        Self {
            filed_date: filing.filed_date(),
            form: filing.form().to_string(),
            is_amendment: filing.is_amendment(),
        }
    }
}

#[cfg(test)]
mod tests {
    use pretty_assertions::assert_eq;

    use super::*;
    use crate::shared::cik::Cik;
    use crate::shared::financial::accession_number::AccessionNumber;
    use crate::shared::financial::form::{Form, ParsedForm};

    fn date(value: &str) -> NaiveDate {
        NaiveDate::parse_from_str(value, "%Y-%m-%d")
            .expect("Given a hardcoded valid date, parsing should always succeed")
    }

    #[test]
    fn should_take_the_filed_date_and_the_form_from_the_filing() {
        let filing = FilingRef::new(
            Cik::new("3545").expect("Given a hardcoded valid CIK, parsing should always succeed"),
            AccessionNumber::new("0000003545-26-000047").expect(
                "Given a hardcoded accession number of the right shape, creation should always succeed",
            ),
            ParsedForm::new(Form::TenQ, true),
            date("2026-10-30"),
        );

        let expected_result = FilingFields {
            filed_date: date("2026-10-30"),
            form: "10-Q".to_string(),
            is_amendment: true,
        };

        let result = FilingFields::from(&filing);

        assert_eq!(result, expected_result);
    }
}
