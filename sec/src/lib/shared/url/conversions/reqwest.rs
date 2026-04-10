use super::super::Url;

impl From<reqwest::Url> for Url {
    fn from(url: reqwest::Url) -> Self {
        Self {
            value: url.to_string(),
        }
    }
}

#[cfg(test)]
mod tests {
    use pretty_assertions::assert_eq;

    use super::*;

    #[test]
    fn should_convert_from_reqwest_url_when_using_from() {
        let reqwest_url =
            reqwest::Url::parse("https://data.sec.gov/submissions/CIK0001067983.json")
                .expect("A hardcoded valid URL should always parse successfully");

        let expected_result = "https://data.sec.gov/submissions/CIK0001067983.json";

        let result = Url::from(reqwest_url);

        assert_eq!(result.as_str(), expected_result);
    }
}
