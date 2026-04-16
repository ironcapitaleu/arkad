/// Raw JSON fixture for CIK0001067983 (Berkshire Hathaway) company facts.
/// Retrieved from SEC API on 03.03.2026.
pub const CIK0001067983: &str = include_str!("CIK0001067983.json");

#[cfg(test)]
mod tests {
    use serde_json::Value;

    use super::*;

    #[test]
    fn should_load_fixture_correctly_when_accessing_constant() {
        let expected_result = false;

        let result = CIK0001067983.is_empty();

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_correctly_extract_berkshire_cik_when_fixture_is_parsed() {
        let json: Value = serde_json::from_str(CIK0001067983)
            .expect("CIK0001067983.json is a well-formed fixture and should always parse");

        let expected_result = 1067983;

        let result = json["cik"]
            .as_u64()
            .expect("'cik' field must be present and numeric in the fixture");

        assert_eq!(result, expected_result);
    }
}
