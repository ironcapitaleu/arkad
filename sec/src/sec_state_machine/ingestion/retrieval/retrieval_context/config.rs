use reqwest::{self, Client, Error};

const USER_AGENT: &str = "Demir Catovic d.catovic9@gmail.com";
pub const DEFAULT_CIK: &str = "0001067983"; // BRK

pub fn get_sec_user_client() -> Result<Client, Error> {
    let client = reqwest::Client::builder()
        .user_agent(get_sec_user_agent().to_string())
        .build()?;
    Ok(client)
}

pub fn get_sec_user_agent() -> impl ToString {
    USER_AGENT.to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn should_return_default_email_when_calling_get_sec_user_agent() {
        let expected_user_agent = "Demir Catovic d.catovic9@gmail.com";
        let result = get_sec_user_agent().to_string();

        assert_eq!(result, expected_user_agent);
    }
}
