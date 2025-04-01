use reqwest::{self, Error, blocking::Client};

const USER_AGENT: &str = "Demir Catovic d.catovic9@gmail.com";
pub const DEFAULT_CIK: &str = "1067983"; // BRK
//pub const DEFAULT_CIK: &str = "00001067983"; // invalid CIK (used for testing purposes)

/// Creates a new synchronous(!) SEC user client with a custom user agent.
///
/// # Errors
///
/// This function will return an error if the client cannot be built,
/// which can happen if there's an issue with the underlying configuration
/// of the `reqwest::Client` (e.g., invalid user agent).
///
/// The `reqwest::Error` is returned when the builder fails.
pub fn get_sec_user_client() -> Result<Client, Error> {
    let client = Client::builder()
        .user_agent(get_sec_user_agent().to_string())
        .build()?;
    Ok(client)
}

#[must_use]
pub fn get_sec_user_agent() -> impl ToString {
    USER_AGENT
}

#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::assert_eq;

    #[test]
    fn should_return_default_email_when_calling_get_sec_user_agent() {
        let expected_user_agent = "Demir Catovic d.catovic9@gmail.com";

        let result = get_sec_user_agent().to_string();

        assert_eq!(result, expected_user_agent);
    }
}
