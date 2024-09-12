use std::fmt;

const CIK_LENGTH: usize = 10;

#[derive(Debug, Clone, PartialEq, PartialOrd, Hash, Eq, Ord)]
pub struct CIK {
    value: String,
}

impl CIK {
    /// Creates a new `CIK` from a string, trimming whitespace and padding with zeros if less than 10 digits.
    /// Panics if the string contains non-numeric characters or if it's longer than 10 digits.
    pub fn new(cik: &(impl ToString + ?Sized)) -> Self {
        let mut cik_str = cik.to_string().trim().to_string(); // Trim leading and trailing whitespace

        // Check if it contains only digits
        if !cik_str.chars().all(|c| c.is_ascii_digit()) {
            panic!(
                "Invalid CIK: CIK must contain only numeric characters. Got: {}",
                cik.to_string()
            );
        }

        // Prepend zeros if less than 10 digits
        if cik_str.len() < CIK_LENGTH {
            cik_str = format!("{cik_str:0>CIK_LENGTH$}"); // Pads with leading zeros to a length of `CIK_LENGTH`
        }

        // Ensure the length does not exceed `CIK_LENGTH`` digits
        if cik_str.len() > CIK_LENGTH {
            panic!("Invalid CIK: Final CIK cannot exceed the fixed CIK length of {CIK_LENGTH} digits. Final CIK is '{}' and is {}>{CIK_LENGTH} digits long.", cik_str.len(), cik.to_string());
        }

        Self { value: cik_str }
    }

    /// Returns the underlying CIK string.
    #[must_use]
    pub const fn value(&self) -> &String {
        &self.value
    }

    /// Validates if the CIK contains exactly 10 digits.
    #[must_use]
    pub fn is_valid(cik: &str) -> bool {
        cik.len() == CIK_LENGTH && cik.chars().all(|c| c.is_ascii_digit())
    }
}

impl fmt::Display for CIK {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.value)
    }
}

#[cfg(test)]
mod tests {
    use super::CIK;

    #[test]
    fn should_create_valid_cik_struct_if_numeric_string_with_ten_digits_is_passed() {
        let cik_str = "1234567890";

        let expected_result = "1234567890";

        let result = CIK::new(cik_str);

        assert_eq!(result.value(), expected_result);
    }

    #[test]
    #[should_panic(expected = "Invalid CIK")]
    fn should_panic_when_given_cik_str_that_contains_non_numeric_chars() {
        CIK::new("12345abcd!");
    }

    #[test]
    fn should_prepend_cik_with_zeros_when_passed_valid_string_with_less_than_ten_digits() {
        let cik_str = "123456789";

        let expected_result = "0123456789";

        let result = CIK::new(cik_str);

        assert_eq!(result.value(), expected_result);
    }

    #[test]
    fn should_create_cik_with_all_zeros_when_passed_empty_string() {
        let cik_str = "";

        let expected_result = "0000000000";

        let result = CIK::new(cik_str);

        assert_eq!(result.value(), expected_result);
    }
}
