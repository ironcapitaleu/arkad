//! # Company Facts Parser
//!
//! Deserializes the SEC `/api/xbrl/companyfacts/` JSON endpoint
//! into a collection of `RawObservation`s.
//!
//! ## JSON Structure
//!
//! ```json
//! {
//!   "cik": 320193,
//!   "entityName": "Apple Inc.",
//!   "facts": {
//!     "us-gaap": {
//!       "Revenue": {
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

use chrono::NaiveDate;

use crate::core::accession_number::AccessionNumber;
use crate::core::fiscal_period::FiscalPeriod;
use crate::core::fiscal_year::FiscalYear;
use crate::core::form::Form;
use crate::core::frame::Frame;
use crate::core::namespace::Namespace;
use crate::core::observation::RawObservation;
use crate::core::period::Period;
use crate::core::provenance::Provenance;
use crate::core::unit::Unit;
use crate::error::ErrorKind;
use crate::error::parsing::ParseErrorKind;
use crate::us_gaap::mappings::{REQUIRED_FACTS_NAMESPACE, REQUIRED_TOP_LEVEL_KEYS};

/// Parses a raw SEC Company Facts JSON body into raw observations.
///
/// Extracts all data points from all namespaces and concepts present in the response.
/// Does not perform concept resolution — returns flat observations for downstream processing.
///
/// # Errors
///
/// Returns [`ErrorKind`] if the JSON structure is invalid or required keys are missing.
pub fn parse(json: &serde_json::Value) -> Result<Vec<RawObservation>, ErrorKind> {
    validate_top_level_structure(json)?;

    let facts = json
        .get("facts")
        .and_then(serde_json::Value::as_object)
        .ok_or_else(|| {
            ParseErrorKind::MissingTopLevelKey {
                key: "facts".to_string(),
            }
        })?;

    if !facts.contains_key(REQUIRED_FACTS_NAMESPACE) {
        return Err(ParseErrorKind::MissingNamespace {
            namespace: REQUIRED_FACTS_NAMESPACE.to_string(),
        }
        .into());
    }

    let mut observations = Vec::new();

    for (ns_key, ns_value) in facts {
        let Some(namespace) = Namespace::from_sec_str(ns_key) else {
            continue;
        };

        let Some(concepts) = ns_value.as_object() else {
            continue;
        };

        for (concept_name, concept_data) in concepts {
            let Some(units_obj) = concept_data.get("units").and_then(serde_json::Value::as_object)
            else {
                continue;
            };

            for (unit_str, data_points) in units_obj {
                let Some(unit) = Unit::from_sec_str(unit_str) else {
                    continue;
                };

                let Some(data_points) = data_points.as_array() else {
                    continue;
                };

                for dp in data_points {
                    if let Some(obs) =
                        parse_data_point(dp, namespace, concept_name, unit)
                    {
                        observations.push(obs);
                    }
                }
            }
        }
    }

    Ok(observations)
}

/// Returns the entity name from the Company Facts JSON.
///
/// # Errors
///
/// Returns [`ErrorKind`] if the structure is invalid.
pub fn extract_entity_name(json: &serde_json::Value) -> Result<String, ErrorKind> {
    validate_top_level_structure(json)?;
    json.get("entityName")
        .and_then(serde_json::Value::as_str)
        .map(ToString::to_string)
        .ok_or_else(|| {
            ParseErrorKind::MissingTopLevelKey {
                key: "entityName".to_string(),
            }
            .into()
        })
}

fn validate_top_level_structure(json: &serde_json::Value) -> Result<(), ErrorKind> {
    let obj = json.as_object().ok_or_else(|| ParseErrorKind::InvalidJson {
        reason: "Expected a JSON object at the top level".to_string(),
    })?;

    for &key in REQUIRED_TOP_LEVEL_KEYS {
        if !obj.contains_key(key) {
            return Err(ParseErrorKind::MissingTopLevelKey {
                key: key.to_string(),
            }
            .into());
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use pretty_assertions::assert_eq;
    use serde_json::json;

    use super::*;

    fn sample_company_facts_json() -> serde_json::Value {
        json!({
            "cik": 320193,
            "entityName": "Apple Inc.",
            "facts": {
                "us-gaap": {
                    "Assets": {
                        "label": "Assets",
                        "description": "Total assets",
                        "units": {
                            "USD": [
                                {
                                    "end": "2024-09-28",
                                    "val": 364980000000_i64,
                                    "accn": "0000320193-24-000123",
                                    "fy": 2024,
                                    "fp": "FY",
                                    "form": "10-K",
                                    "filed": "2024-11-01",
                                    "frame": "CY2024Q4I"
                                }
                            ]
                        }
                    },
                    "Revenues": {
                        "label": "Net Sales",
                        "description": "Revenue",
                        "units": {
                            "USD": [
                                {
                                    "start": "2023-10-01",
                                    "end": "2024-09-28",
                                    "val": 391035000000_i64,
                                    "accn": "0000320193-24-000123",
                                    "fy": 2024,
                                    "fp": "FY",
                                    "form": "10-K",
                                    "filed": "2024-11-01",
                                    "frame": "CY2024"
                                }
                            ]
                        }
                    }
                },
                "dei": {
                    "EntityCommonStockSharesOutstanding": {
                        "label": "Entity Common Stock, Shares Outstanding",
                        "description": "Shares outstanding",
                        "units": {
                            "shares": [
                                {
                                    "end": "2024-10-18",
                                    "val": 15115823000_i64,
                                    "accn": "0000320193-24-000123",
                                    "fy": 2024,
                                    "fp": "FY",
                                    "form": "10-K",
                                    "filed": "2024-11-01"
                                }
                            ]
                        }
                    }
                }
            }
        })
    }

    #[test]
    fn should_parse_observations_when_valid_company_facts_json_is_provided() {
        let json = sample_company_facts_json();

        let expected_result = 3;

        let result = parse(&json).expect("Valid JSON should parse successfully");

        assert_eq!(result.len(), expected_result);
    }

    #[test]
    fn should_parse_instant_period_when_data_point_has_no_start() {
        let json = sample_company_facts_json();

        let observations = parse(&json).expect("Valid JSON should parse successfully");
        let assets_obs = observations
            .iter()
            .find(|o| o.concept_name() == "Assets")
            .expect("Assets observation should exist");

        let expected_result = Period::Instant {
            date: NaiveDate::from_ymd_opt(2024, 9, 28)
                .expect("Hardcoded date should always be valid"),
        };

        let result = assets_obs.period();

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_parse_duration_period_when_data_point_has_start_and_end() {
        let json = sample_company_facts_json();

        let observations = parse(&json).expect("Valid JSON should parse successfully");
        let revenue_obs = observations
            .iter()
            .find(|o| o.concept_name() == "Revenues")
            .expect("Revenues observation should exist");

        let expected_result = Period::Duration {
            start: NaiveDate::from_ymd_opt(2023, 10, 1)
                .expect("Hardcoded date should always be valid"),
            end: NaiveDate::from_ymd_opt(2024, 9, 28)
                .expect("Hardcoded date should always be valid"),
        };

        let result = revenue_obs.period();

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_return_error_when_top_level_key_is_missing() {
        let json = json!({"cik": 123, "entityName": "Test"});

        let expected_result = true;

        let result = parse(&json).is_err();

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_extract_entity_name_when_valid_json_is_provided() {
        let json = sample_company_facts_json();

        let expected_result = "Apple Inc.";

        let result = extract_entity_name(&json).expect("Valid JSON should parse successfully");

        assert_eq!(result, expected_result);
    }
}

fn parse_data_point(
    dp: &serde_json::Value,
    namespace: Namespace,
    concept_name: &str,
    unit: Unit,
) -> Option<RawObservation> {
    let val = dp.get("val")?.as_i64()?;
    let end_str = dp.get("end")?.as_str()?;
    let end_date = NaiveDate::parse_from_str(end_str, "%Y-%m-%d").ok()?;

    let period = if let Some(start_str) = dp.get("start").and_then(serde_json::Value::as_str) {
        let start_date = NaiveDate::parse_from_str(start_str, "%Y-%m-%d").ok()?;
        Period::Duration {
            start: start_date,
            end: end_date,
        }
    } else {
        Period::Instant { date: end_date }
    };

    let frame = dp
        .get("frame")
        .and_then(serde_json::Value::as_str)
        .and_then(Frame::parse);

    let accn = dp.get("accn")?.as_str()?;
    let form_str = dp.get("form")?.as_str()?;
    let form = Form::from_sec_str(form_str)?;
    let fy = u16::try_from(dp.get("fy")?.as_u64()?).ok()?;
    let fp_str = dp.get("fp")?.as_str()?;
    let fiscal_period = FiscalPeriod::from_sec_str(fp_str)?;
    let filed_str = dp.get("filed")?.as_str()?;
    let filed_date = NaiveDate::parse_from_str(filed_str, "%Y-%m-%d").ok()?;

    let provenance = Provenance::new(
        AccessionNumber::new(accn),
        form,
        FiscalYear::new(fy),
        fiscal_period,
        filed_date,
        end_date,
    );

    Some(RawObservation::new(
        namespace,
        concept_name,
        val,
        unit,
        period,
        frame,
        provenance,
    ))
}
