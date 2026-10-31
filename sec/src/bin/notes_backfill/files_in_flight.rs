//! # Files in Flight
//!
//! Provides [`files_in_flight`], which reads the number of Notes files that the run downloads at
//! once from the value of [`FILES_IN_FLIGHT_ENV_VAR`].

use std::ffi::OsString;

use thiserror::Error;

use crate::constants::{DEFAULT_FILES_IN_FLIGHT, FILES_IN_FLIGHT_ENV_VAR};

/// The error of a value of [`FILES_IN_FLIGHT_ENV_VAR`] that is not a positive integer.
#[derive(Debug, Error, PartialEq, Eq)]
#[error(
    "[InvalidFilesInFlight] {FILES_IN_FLIGHT_ENV_VAR} value '{value}' is not a positive integer"
)]
pub struct InvalidFilesInFlight {
    /// The value as the environment gave it, with invalid Unicode replaced.
    pub value: String,
}

/// Returns the number of Notes files in flight for `value`, the value of [`FILES_IN_FLIGHT_ENV_VAR`].
///
/// An unset or empty value gives [`DEFAULT_FILES_IN_FLIGHT`].
///
/// # Errors
///
/// Returns [`InvalidFilesInFlight`] if the value is not a positive integer.
pub fn files_in_flight(value: Option<OsString>) -> Result<usize, InvalidFilesInFlight> {
    let Some(value) = value.filter(|value| !value.is_empty()) else {
        return Ok(DEFAULT_FILES_IN_FLIGHT);
    };
    let value = value.to_string_lossy();
    value
        .parse::<usize>()
        .ok()
        .filter(|count| *count > 0)
        .ok_or_else(|| InvalidFilesInFlight {
            value: value.into_owned(),
        })
}

#[cfg(test)]
mod tests {
    use pretty_assertions::assert_eq;

    use super::*;

    #[test]
    fn should_return_the_default_when_the_variable_is_unset() {
        let value = None;

        let expected_result = Ok(2);

        let result = files_in_flight(value);

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_return_the_default_when_the_variable_is_empty() {
        let value = Some(OsString::new());

        let expected_result = Ok(2);

        let result = files_in_flight(value);

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_return_the_value_when_the_variable_holds_a_positive_integer() {
        let value = Some(OsString::from("4"));

        let expected_result = Ok(4);

        let result = files_in_flight(value);

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_return_invalid_files_in_flight_when_the_variable_is_zero() {
        let value = Some(OsString::from("0"));

        let expected_result = Err(InvalidFilesInFlight {
            value: "0".to_string(),
        });

        let result = files_in_flight(value);

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_return_invalid_files_in_flight_when_the_variable_is_not_a_number() {
        let value = Some(OsString::from("many"));

        let expected_result = Err(InvalidFilesInFlight {
            value: "many".to_string(),
        });

        let result = files_in_flight(value);

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_return_invalid_files_in_flight_when_the_variable_has_spaces() {
        let value = Some(OsString::from(" 4 "));

        let expected_result = Err(InvalidFilesInFlight {
            value: " 4 ".to_string(),
        });

        let result = files_in_flight(value);

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_name_the_variable_and_the_value_when_displayed() {
        let error = InvalidFilesInFlight {
            value: "many".to_string(),
        };

        let expected_result =
            "[InvalidFilesInFlight] ARKAD_FILES_IN_FLIGHT value 'many' is not a positive integer";

        let result = error.to_string();

        assert_eq!(result, expected_result);
    }
}
