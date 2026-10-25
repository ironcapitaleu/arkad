//! # SEC Client
//!
//! Provides the [`SecClient`] that a binary builds at startup, and the rule that picks the
//! `User-Agent` it sends.

use std::env;
use std::error::Error;
use std::ffi::OsString;

use sec::shared::http_client::implementations::reqwest_client::ReqwestClient;
use sec::shared::http_client::implementations::sec_client::SecClient;
use sec::shared::user_agent::{UserAgent, UserAgentError, UserAgentErrorReason};

/// The environment variable that names the `User-Agent` of every SEC request.
pub const USER_AGENT_ENV_VAR: &str = "ARKAD_SEC_USER_AGENT";

/// Builds the [`SecClient`] that sends the `User-Agent` that `ARKAD_SEC_USER_AGENT` names.
///
/// If the variable is unset or empty, the client sends the default [`UserAgent`].
///
/// # Errors
///
/// Returns [`UserAgentError`] if the variable does not have the form
/// `Company Name email@domain.com`. Returns [`reqwest::Error`] if `reqwest` cannot build the HTTP
/// client.
pub fn sec_client() -> Result<SecClient, Box<dyn Error + Send + Sync>> {
    let user_agent = user_agent(env::var_os(USER_AGENT_ENV_VAR))?;
    Ok(SecClient::new(ReqwestClient::new(user_agent)?))
}

/// Returns the [`UserAgent`] that `value` names, or the default if `value` is unset or empty.
///
/// A value that is not valid UTF-8 fails with [`UserAgentErrorReason::InvalidSecFormat`].
fn user_agent(value: Option<OsString>) -> Result<UserAgent, UserAgentError> {
    let Some(value) = value.filter(|agent| !agent.is_empty()) else {
        return Ok(UserAgent::default());
    };
    let agent = value.to_str().ok_or_else(|| {
        UserAgentError::new(
            UserAgentErrorReason::InvalidSecFormat,
            value.to_string_lossy(),
        )
    })?;
    UserAgent::new(agent)
}

#[cfg(test)]
mod tests {
    use pretty_assertions::assert_eq;
    use sec::shared::user_agent::constants::DEFAULT_SEC_USER_AGENT;

    use super::*;

    #[test]
    fn should_return_default_user_agent_when_variable_is_unset() {
        let expected_result = UserAgent::new(DEFAULT_SEC_USER_AGENT);

        let result = user_agent(None);

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_return_default_user_agent_when_variable_is_empty() {
        let expected_result = UserAgent::new(DEFAULT_SEC_USER_AGENT);

        let result = user_agent(Some(OsString::new()));

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_return_given_user_agent_when_variable_is_valid() {
        let expected_result = UserAgent::new("Sample Company contact@example.com");

        let result = user_agent(Some(OsString::from("Sample Company contact@example.com")));

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_return_user_agent_error_when_variable_is_invalid() {
        let expected_result = Err(UserAgentError::new(
            UserAgentErrorReason::InvalidSecFormat,
            "not-a-user-agent",
        ));

        let result = user_agent(Some(OsString::from("not-a-user-agent")));

        assert_eq!(result, expected_result);
    }

    #[cfg(unix)]
    #[test]
    fn should_return_user_agent_error_when_variable_is_not_utf8() {
        use std::os::unix::ffi::OsStringExt;

        let value = OsString::from_vec(b"Sample \xFF contact@example.com".to_vec());
        let expected_result = Err(UserAgentError::new(
            UserAgentErrorReason::InvalidSecFormat,
            "Sample \u{FFFD} contact@example.com",
        ));

        let result = user_agent(Some(value));

        assert_eq!(result, expected_result);
    }
}
