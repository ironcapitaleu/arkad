//! # Company Facts Parser
//!
//! Maps the JSON body of the SEC `/api/xbrl/companyfacts/` endpoint into [`AsIsFact`] values and one
//! [`Filing`] per accession number.
//!
//! ## JSON Structure
//!
//! ```json
//! {
//!   "cik": 320193,
//!   "entityName": "Apple Inc.",
//!   "facts": {
//!     "us-gaap": {
//!       "Revenues": {
//!         "label": "...",
//!         "description": "...",
//!         "units": {
//!           "USD": [
//!             { "end": "2024-09-28", "val": 391035000000, "accn": "...", ... }
//!           ]
//!         }
//!       }
//!     }
//!   }
//! }
//! ```

use std::collections::HashMap;
use std::str::FromStr;

use bigdecimal::BigDecimal;
use chrono::NaiveDate;

use crate::core::concept::{ConceptName, NamespaceFamily};
use crate::core::entity_name::EntityName;
use crate::core::fact::{AsIsFact, Dimensions, FactKey, FactValue};
use crate::core::filing::{EntityId, Filing, FilingId};
use crate::core::fiscal_period::FiscalPeriod;
use crate::core::fiscal_year::FiscalYear;
use crate::core::form::Form;
use crate::core::invalid_value::InvalidValue;
use crate::core::period::{Duration, Period};
use crate::core::unit::Unit;
use crate::error::parsing::ParseErrorKind;

/// The key of the CIK of the company.
pub const KEY_CIK: &str = "cik";

/// The key of the company name.
pub const KEY_ENTITY_NAME: &str = "entityName";

/// The key of the facts, grouped by namespace.
pub const KEY_FACTS: &str = "facts";

/// The key of the data points of a concept, grouped by unit.
pub const KEY_UNITS: &str = "units";

/// The key of the value of a data point.
pub const KEY_VALUE: &str = "val";

/// The key of the end date of a data point.
pub const KEY_END: &str = "end";

/// The key of the start date of a duration.
pub const KEY_START: &str = "start";

/// The key of the accession number of the filing.
pub const KEY_ACCESSION_NUMBER: &str = "accn";

/// The key of the filing date.
pub const KEY_FILED: &str = "filed";

/// The key of the form type of the filing.
pub const KEY_FORM: &str = "form";

/// The key of the fiscal year of the filing.
pub const KEY_FISCAL_YEAR: &str = "fy";

/// The key of the fiscal period of the filing.
pub const KEY_FISCAL_PERIOD: &str = "fp";

/// The format of the dates of the SEC JSON APIs, for example `2024-09-28`.
pub const DATE_FORMAT: &str = "%Y-%m-%d";

/// Required top-level keys in the SEC Company Facts JSON response.
pub const REQUIRED_TOP_LEVEL_KEYS: &[&str] = &[KEY_CIK, KEY_ENTITY_NAME, KEY_FACTS];

/// Namespace for general company information (DEI taxonomy).
pub const COMPANY_INFO_NAMESPACE: &str = "dei";

/// The entity identifier scheme of the SEC: a CIK.
pub const CIK_SCHEME: &str = "http://www.sec.gov/CIK";

/// The source of an SEC filing ID, as in `sec:0000320193-24-000123`.
pub const SEC_FILING_SOURCE: &str = "sec";

/// The as-is facts of one Company Facts response, and one filing per accession number.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CompanyFacts {
    /// Every data point of every namespace, sorted by namespace, concept, and unit, and in the order
    /// of the JSON within one array of data points.
    pub facts: Vec<AsIsFact>,
    /// One filing per accession number, in the order of the first data point that names it.
    pub filings: Vec<Filing>,
}

/// Parses a raw SEC Company Facts JSON body into as-is facts and filings.
///
/// Keeps every data point of every namespace and unit. The JSON gives no taxonomy version, no
/// dimensions, no `decimals`, and no acceptance time, so these fields stay empty. `copy_rank`
/// numbers the data points with one value key in the order of their array. The first data point of
/// an accession number gives its `Filing`. A later data point that names another form, fiscal year,
/// or fiscal period does not change it.
///
/// # Errors
///
/// Returns [`ParseErrorKind`] if the JSON structure is invalid, a required key is missing, a value
/// of the JSON has the wrong type, or a data point lacks a required field.
pub fn parse(json: &serde_json::Value) -> Result<CompanyFacts, ParseErrorKind> {
    validate_top_level_structure(json)?;

    let entity = entity_id(json)?;
    let company_name = extract_entity_name(json)?;

    let namespaces = json
        .get(KEY_FACTS)
        .and_then(serde_json::Value::as_object)
        .ok_or_else(|| ParseErrorKind::UnexpectedJsonSchema {
            reason: format!("Expected '{KEY_FACTS}' to be a JSON object"),
        })?;

    let mut parser = Parser {
        entity,
        company_name: EntityName::new(company_name),
        facts: Vec::new(),
        filings: Vec::new(),
        filing_index: HashMap::new(),
        copies: HashMap::new(),
    };

    for (namespace, concepts) in namespaces {
        let concepts =
            concepts
                .as_object()
                .ok_or_else(|| ParseErrorKind::UnexpectedJsonSchema {
                    reason: format!("Expected namespace '{namespace}' to be a JSON object"),
                })?;
        for (local_name, concept_data) in concepts {
            let concept = concept_name(namespace, local_name)?;
            let units = concept_data
                .get(KEY_UNITS)
                .and_then(serde_json::Value::as_object)
                .ok_or_else(|| {
                    invalid(
                        &concept,
                        &format!("Expected '{KEY_UNITS}' to be a JSON object"),
                    )
                })?;
            for (unit_text, data_points) in units {
                let unit =
                    Unit::from_str(unit_text).map_err(|error| invalid_value(&concept, error))?;
                let data_points = data_points.as_array().ok_or_else(|| {
                    invalid(
                        &concept,
                        &format!("Expected unit '{unit_text}' to hold an array"),
                    )
                })?;
                for data_point in data_points {
                    parser.add(data_point, &concept, &unit)?;
                }
            }
        }
    }

    Ok(CompanyFacts {
        facts: parser.facts,
        filings: parser.filings,
    })
}

/// Returns the entity name from the Company Facts JSON.
///
/// # Errors
///
/// Returns [`ParseErrorKind`] if the structure is invalid.
pub fn extract_entity_name(json: &serde_json::Value) -> Result<String, ParseErrorKind> {
    validate_top_level_structure(json)?;
    json.get(KEY_ENTITY_NAME)
        .and_then(serde_json::Value::as_str)
        .map(ToString::to_string)
        .ok_or_else(|| ParseErrorKind::UnexpectedJsonSchema {
            reason: format!("Expected '{KEY_ENTITY_NAME}' to be a string"),
        })
}

struct Parser {
    entity: EntityId,
    company_name: EntityName,
    facts: Vec<AsIsFact>,
    filings: Vec<Filing>,
    filing_index: HashMap<FilingId, usize>,
    copies: HashMap<FactKey, u16>,
}

impl Parser {
    fn add(
        &mut self,
        data_point: &serde_json::Value,
        concept: &ConceptName,
        unit: &Unit,
    ) -> Result<(), ParseErrorKind> {
        let field = |key: &str| {
            data_point
                .get(key)
                .filter(|value| !value.is_null())
                .ok_or_else(|| invalid(concept, &format!("Missing field '{key}'")))
        };
        let text = |key: &str| {
            field(key)?
                .as_str()
                .ok_or_else(|| invalid(concept, &format!("Field '{key}' is not a string")))
        };
        let date = |key: &str| {
            NaiveDate::parse_from_str(text(key)?, DATE_FORMAT)
                .map_err(|error| invalid(concept, &format!("Field '{key}' is not a date: {error}")))
        };

        let number = field(KEY_VALUE)?
            .as_number()
            .ok_or_else(|| invalid(concept, &format!("Field '{KEY_VALUE}' is not a number")))?;
        let value = BigDecimal::from_str(&number.to_string()).map_err(|error| {
            invalid(
                concept,
                &format!("Field '{KEY_VALUE}' is not a number: {error}"),
            )
        })?;
        let end = date(KEY_END)?;
        let period = if field(KEY_START).is_ok() {
            Period::Duration(
                Duration::from_dates(date(KEY_START)?, end)
                    .map_err(|error| invalid_value(concept, error))?,
            )
        } else {
            Period::Instant { end }
        };
        let filing = FilingId::new(SEC_FILING_SOURCE, text(KEY_ACCESSION_NUMBER)?)
            .map_err(|error| invalid_value(concept, error))?;
        let filed = date(KEY_FILED)?;

        let form = Form::new(text(KEY_FORM)?).map_err(|error| invalid_value(concept, error))?;
        let fiscal_year = match field(KEY_FISCAL_YEAR) {
            Ok(fy) => Some(FiscalYear::new(
                fy.as_u64()
                    .and_then(|year| u16::try_from(year).ok())
                    .ok_or_else(|| {
                        invalid(concept, &format!("Field '{KEY_FISCAL_YEAR}' is not a year"))
                    })?,
            )),
            Err(_) => None,
        };
        let fiscal_period = match text(KEY_FISCAL_PERIOD) {
            Ok(fp) => Some(FiscalPeriod::new(fp).map_err(|error| invalid_value(concept, error))?),
            Err(_) => None,
        };
        if !self.filing_index.contains_key(&filing) {
            self.filing_index.insert(filing.clone(), self.filings.len());
            self.filings.push(Filing {
                id: filing.clone(),
                entity: self.entity.clone(),
                is_amendment: form.is_amendment(),
                form,
                filed,
                accepted: None,
                period_of_report: None,
                fiscal_year,
                fiscal_period,
                company_name: Some(self.company_name.clone()),
                sic: None,
                tickers: Vec::new(),
                fiscal_year_end: None,
            });
        }

        let mut fact = AsIsFact {
            filing,
            entity: self.entity.clone(),
            concept: concept.clone(),
            taxonomy_version: None,
            period,
            unit: Some(unit.clone()),
            dimensions: Dimensions::default(),
            value: FactValue::Numeric(value),
            decimals: None,
            copy_rank: 0,
            footnotes: Vec::new(),
            filed,
            accepted: None,
        };
        let copies = self.copies.entry(fact.key()).or_insert(0);
        fact.copy_rank = *copies;
        *copies = copies
            .checked_add(1)
            .ok_or_else(|| invalid(concept, "More than 65535 copies share one value key"))?;
        self.facts.push(fact);
        Ok(())
    }
}

fn entity_id(json: &serde_json::Value) -> Result<EntityId, ParseErrorKind> {
    let cik = json
        .get(KEY_CIK)
        .and_then(serde_json::Value::as_u64)
        .ok_or_else(|| ParseErrorKind::UnexpectedJsonSchema {
            reason: format!("Expected '{KEY_CIK}' to be a number"),
        })?;
    EntityId::new(CIK_SCHEME, format!("{cik:010}")).map_err(|error| {
        ParseErrorKind::UnexpectedJsonSchema {
            reason: error.to_string(),
        }
    })
}

fn concept_name(namespace: &str, local_name: &str) -> Result<ConceptName, ParseErrorKind> {
    NamespaceFamily::new(namespace)
        .and_then(|family| ConceptName::new(family, local_name))
        .map_err(|source| ParseErrorKind::InvalidDataPointValue {
            concept: format!("{namespace}:{local_name}"),
            source,
        })
}

fn invalid_value(concept: &ConceptName, source: InvalidValue) -> ParseErrorKind {
    ParseErrorKind::InvalidDataPointValue {
        concept: concept.to_string(),
        source,
    }
}

fn invalid(concept: &ConceptName, reason: &str) -> ParseErrorKind {
    ParseErrorKind::InvalidDataPoint {
        concept: concept.to_string(),
        reason: reason.to_string(),
    }
}

fn validate_top_level_structure(json: &serde_json::Value) -> Result<(), ParseErrorKind> {
    let obj = json
        .as_object()
        .ok_or_else(|| ParseErrorKind::InvalidJson {
            reason: "Expected a JSON object at the top level".to_string(),
        })?;

    for &key in REQUIRED_TOP_LEVEL_KEYS {
        if !obj.contains_key(key) {
            return Err(ParseErrorKind::MissingTopLevelKey {
                key: key.to_string(),
            });
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use pretty_assertions::assert_eq;
    use serde_json::json;

    use super::*;

    fn date(year: i32, month: u32, day: u32) -> NaiveDate {
        NaiveDate::from_ymd_opt(year, month, day)
            .expect("Given a hardcoded valid date, the creation should always succeed")
    }

    fn data_point(start: Option<&str>, end: &str, val: &serde_json::Value) -> serde_json::Value {
        let mut point = json!({
            "end": end,
            "val": val,
            "accn": "0000320193-24-000123",
            "fy": 2024,
            "fp": "FY",
            "form": "10-K",
            "filed": "2024-11-01",
            "frame": "CY2024Q4I"
        });
        if let Some(start) = start {
            point["start"] = json!(start);
        }
        point
    }

    fn company_facts_json(us_gaap: &serde_json::Value) -> serde_json::Value {
        json!({
            "cik": 320_193,
            "entityName": "Apple Inc.",
            "facts": {
                "us-gaap": us_gaap,
                "dei": {
                    "EntityCommonStockSharesOutstanding": {
                        "units": { "shares": [data_point(None, "2024-10-18", &json!(15_115_823_000_i64))] }
                    }
                }
            }
        })
    }

    fn sample_company_facts_json() -> serde_json::Value {
        company_facts_json(&json!({
            "Assets": {
                "label": "Assets",
                "units": { "USD": [data_point(None, "2024-09-28", &json!(364_980_000_000_i64))] }
            },
            "Revenues": {
                "label": "Net Sales",
                "units": { "USD": [data_point(Some("2023-10-01"), "2024-09-28", &json!(391_035_000_000_i64))] }
            },
            "EarningsPerShareDiluted": {
                "units": { "USD/shares": [data_point(Some("2023-10-01"), "2024-09-28", &json!(6.08))] }
            }
        }))
    }

    fn fact<'a>(facts: &'a CompanyFacts, local_name: &str) -> &'a AsIsFact {
        facts
            .facts
            .iter()
            .find(|fact| fact.concept.local_name() == local_name)
            .expect("Given the sample body, the concept should have a fact")
    }

    fn concept(family: &str, local_name: &str) -> ConceptName {
        concept_name(family, local_name)
            .expect("Given a hardcoded concept, the creation should always succeed")
    }

    #[test]
    fn should_keep_every_namespace_when_valid_company_facts_json_is_provided() {
        let json = sample_company_facts_json();

        let expected_result = 4;

        let result = parse(&json)
            .expect("Given a valid hardcoded Company Facts body, the parse should always succeed")
            .facts
            .len();

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_use_namespace_key_as_family_when_concept_is_in_dei() {
        let json = sample_company_facts_json();

        let expected_result = &concept("dei", "EntityCommonStockSharesOutstanding");

        let facts = parse(&json)
            .expect("Given a valid hardcoded Company Facts body, the parse should always succeed");
        let result = &fact(&facts, "EntityCommonStockSharesOutstanding").concept;

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_parse_instant_period_when_data_point_has_no_start() {
        let json = sample_company_facts_json();

        let expected_result = Period::Instant {
            end: date(2024, 9, 28),
        };

        let facts = parse(&json)
            .expect("Given a valid hardcoded Company Facts body, the parse should always succeed");
        let result = fact(&facts, "Assets").period;

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_parse_duration_of_four_quarters_when_data_point_spans_a_fiscal_year() {
        let json = sample_company_facts_json();

        let expected_result = Period::Duration(
            Duration::from_dates(date(2023, 10, 1), date(2024, 9, 28))
                .expect("Given an end after the start, the creation should always succeed"),
        );

        let facts = parse(&json)
            .expect("Given a valid hardcoded Company Facts body, the parse should always succeed");
        let result = fact(&facts, "Revenues").period;

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_keep_fraction_when_value_is_a_per_share_amount() {
        let json = sample_company_facts_json();

        let expected_result = FactValue::Numeric(
            BigDecimal::from_str("6.08")
                .expect("Given a hardcoded decimal text, the parse should always succeed"),
        );

        let facts = parse(&json)
            .expect("Given a valid hardcoded Company Facts body, the parse should always succeed");
        let result = fact(&facts, "EarningsPerShareDiluted").value.clone();

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_parse_unit_with_denominator_when_unit_key_divides() {
        let json = sample_company_facts_json();

        let expected_result = Some(
            Unit::from_str("USD/shares")
                .expect("Given a hardcoded unit, the parse should always succeed"),
        );

        let facts = parse(&json)
            .expect("Given a valid hardcoded Company Facts body, the parse should always succeed");
        let result = fact(&facts, "EarningsPerShareDiluted").unit.clone();

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_write_sec_prefix_when_filing_id_comes_from_accession_number() {
        let json = sample_company_facts_json();

        let expected_result = "sec:0000320193-24-000123";

        let facts = parse(&json)
            .expect("Given a valid hardcoded Company Facts body, the parse should always succeed");
        let result = fact(&facts, "Assets").filing.to_string();

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_pad_cik_to_ten_digits_when_entity_id_is_built() {
        let json = sample_company_facts_json();

        let expected_result = "0000320193";

        let facts = parse(&json)
            .expect("Given a valid hardcoded Company Facts body, the parse should always succeed");
        let result = fact(&facts, "Assets").entity.id().to_string();

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_return_one_filing_when_data_points_share_an_accession_number() {
        let json = sample_company_facts_json();

        let expected_result = vec![Filing {
            id: FilingId::new("sec", "0000320193-24-000123")
                .expect("Given a hardcoded source and ID, the creation should always succeed"),
            entity: EntityId::new(CIK_SCHEME, "0000320193")
                .expect("Given a hardcoded scheme and ID, the creation should always succeed"),
            form: Form::new("10-K")
                .expect("Given a hardcoded form type, the creation should always succeed"),
            is_amendment: false,
            filed: date(2024, 11, 1),
            accepted: None,
            period_of_report: None,
            fiscal_year: Some(FiscalYear::new(2024)),
            fiscal_period: Some(
                FiscalPeriod::new("FY")
                    .expect("Given a hardcoded period, the creation should always succeed"),
            ),
            company_name: Some(EntityName::new("Apple Inc.")),
            sic: None,
            tickers: vec![],
            fiscal_year_end: None,
        }];

        let result = parse(&json)
            .expect("Given a valid hardcoded Company Facts body, the parse should always succeed")
            .filings;

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_number_copies_in_json_order_when_two_data_points_share_a_key() {
        let json = company_facts_json(&json!({
            "Assets": {
                "units": { "USD": [
                    data_point(None, "2024-09-28", &json!(364_980_000_000_i64)),
                    data_point(None, "2024-09-28", &json!(364_980_000_000_i64))
                ] }
            }
        }));

        let expected_result = vec![0, 0, 1];

        let result: Vec<u16> = parse(&json)
            .expect("Given a valid hardcoded Company Facts body, the parse should always succeed")
            .facts
            .iter()
            .map(|fact| fact.copy_rank)
            .collect();

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_return_invalid_data_point_when_value_is_missing() {
        let json = company_facts_json(&json!({
            "Assets": { "units": { "USD": [{ "end": "2024-09-28", "accn": "0000320193-24-000123", "form": "10-K", "filed": "2024-11-01" }] } }
        }));

        let expected_result = Some(ParseErrorKind::InvalidDataPoint {
            concept: "us-gaap:Assets".to_string(),
            reason: "Missing field 'val'".to_string(),
        });

        let result = parse(&json).err();

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_return_error_when_top_level_key_is_missing() {
        let json = json!({"cik": 123, "entityName": "Test"});

        let expected_result = Some(ParseErrorKind::MissingTopLevelKey {
            key: "facts".to_string(),
        });

        let result = parse(&json).err();

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_extract_entity_name_when_valid_json_is_provided() {
        let json = sample_company_facts_json();

        let expected_result = "Apple Inc.";

        let result = extract_entity_name(&json).expect("Valid JSON should parse successfully");

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_keep_ifrs_facts_when_body_has_no_us_gaap_namespace() {
        let json = json!({
            "cik": 1_046_179,
            "entityName": "Taiwan Semiconductor Manufacturing Co Ltd",
            "facts": {
                "ifrs-full": {
                    "Revenue": { "units": { "TWD": [data_point(Some("2023-01-01"), "2023-12-31", &json!(2_161_735_841_000_i64))] } }
                }
            }
        });

        let expected_result = vec![concept("ifrs-full", "Revenue")];

        let result: Vec<ConceptName> = parse(&json)
            .expect("Given a valid hardcoded Company Facts body, the parse should always succeed")
            .facts
            .into_iter()
            .map(|fact| fact.concept)
            .collect();

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_keep_every_digit_when_value_has_twenty_four_digits_before_the_point() {
        let val: serde_json::Value = serde_json::from_str("123456789012345678901234.12345678")
            .expect("Given a hardcoded JSON number, the parse should always succeed");
        let json = company_facts_json(&json!({
            "Assets": { "units": { "USD": [data_point(None, "2024-09-28", &val)] } }
        }));

        let expected_result = FactValue::Numeric(
            BigDecimal::from_str("123456789012345678901234.12345678")
                .expect("Given a hardcoded decimal text, the parse should always succeed"),
        );

        let facts = parse(&json)
            .expect("Given a valid hardcoded Company Facts body, the parse should always succeed");
        let result = fact(&facts, "Assets").value.clone();

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_return_invalid_data_point_when_second_data_point_lacks_form() {
        let mut second = data_point(None, "2023-09-30", &json!(352_583_000_000_i64));
        second
            .as_object_mut()
            .expect("Given the data point helper, the point should always be an object")
            .remove("form");
        let json = company_facts_json(&json!({
            "Assets": { "units": { "USD": [data_point(None, "2024-09-28", &json!(364_980_000_000_i64)), second] } }
        }));

        let expected_result = Some(ParseErrorKind::InvalidDataPoint {
            concept: "us-gaap:Assets".to_string(),
            reason: "Missing field 'form'".to_string(),
        });

        let result = parse(&json).err();

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_chain_the_invalid_value_when_start_lies_after_end() {
        let json = company_facts_json(&json!({
            "Revenues": { "units": { "USD": [data_point(Some("2024-10-01"), "2024-09-28", &json!(1))] } }
        }));

        let expected_result = Some(
            "[InvalidDataPointValue] Invalid value in a data point of concept 'us-gaap:Revenues', Caused by: [InvalidValue] Invalid 'Duration', Reason: 'The end lies before the start', Input: '2024-10-01 to 2024-09-28'"
                .to_string(),
        );

        let result = parse(&json).err().map(|error| error.to_string());

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_return_invalid_data_point_when_concept_has_no_units() {
        let json = company_facts_json(&json!({ "Assets": { "label": "Assets" } }));

        let expected_result = Some(ParseErrorKind::InvalidDataPoint {
            concept: "us-gaap:Assets".to_string(),
            reason: "Expected 'units' to be a JSON object".to_string(),
        });

        let result = parse(&json).err();

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_return_invalid_data_point_when_unit_value_is_not_an_array() {
        let json = company_facts_json(&json!({ "Assets": { "units": { "USD": {} } } }));

        let expected_result = Some(ParseErrorKind::InvalidDataPoint {
            concept: "us-gaap:Assets".to_string(),
            reason: "Expected unit 'USD' to hold an array".to_string(),
        });

        let result = parse(&json).err();

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_return_unexpected_schema_when_namespace_is_not_an_object() {
        let json =
            json!({ "cik": 320_193, "entityName": "Apple Inc.", "facts": { "us-gaap": [] } });

        let expected_result = Some(ParseErrorKind::UnexpectedJsonSchema {
            reason: "Expected namespace 'us-gaap' to be a JSON object".to_string(),
        });

        let result = parse(&json).err();

        assert_eq!(result, expected_result);
    }
}
