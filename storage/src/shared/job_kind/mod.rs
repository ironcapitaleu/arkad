//! # Job Kind
//!
//! Provides [`JobKind`], the name of the jobs that write one silver unit key, such as
//! `transform-notes`.
//!
//! ## Modules
//!
//! - [`invalid_job_kind`]: The [`InvalidJobKind`] for a string that is not a job kind.

use std::fmt::{self, Display, Formatter};

pub mod invalid_job_kind;

pub use invalid_job_kind::InvalidJobKind;

/// The name of the jobs that write one silver unit key, such as `transform-notes`: lower-case
/// ASCII letters, digits, and `-`.
///
/// A [`LeaseStore`](crate::LeaseStore) holds one lease per job kind. A backend uses the job kind
/// as a path component, so it holds no `/` and no `.`.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct JobKind(String);

impl JobKind {
    /// Creates a [`JobKind`] from lower-case ASCII letters, digits, and `-`.
    ///
    /// # Errors
    ///
    /// Returns an [`InvalidJobKind`] if the value is empty, or holds another character.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use storage::{InvalidJobKind, JobKind};
    ///
    /// let value = "transform/notes";
    ///
    /// let expected_result = Err(InvalidJobKind::new("transform/notes"));
    ///
    /// let result = JobKind::new(value);
    ///
    /// assert_eq!(result, expected_result);
    /// ```
    pub fn new(value: impl Into<String>) -> Result<Self, InvalidJobKind> {
        let value = value.into();
        let holds_only_allowed_bytes = value
            .bytes()
            .all(|byte| byte.is_ascii_lowercase() || byte.is_ascii_digit() || byte == b'-');
        if value.is_empty() || !holds_only_allowed_bytes {
            return Err(InvalidJobKind { value });
        }
        Ok(Self(value))
    }

    /// Returns the job kind as text.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl Display for JobKind {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

#[cfg(test)]
mod tests {
    use pretty_assertions::assert_eq;

    use super::*;

    const fn implements_auto_traits<T: Sized + Send + Sync + Unpin>() {}
    #[test]
    const fn should_implement_auto_traits_for_job_kind() {
        implements_auto_traits::<JobKind>();
    }

    #[test]
    fn should_return_the_text_when_the_job_kind_is_valid() {
        let value = "transform-notes";

        let expected_result = "transform-notes";

        let job_kind = JobKind::new(value)
            .expect("Given lower-case letters and '-', the creation of a job kind should succeed");
        let result = job_kind.as_str();

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_return_invalid_job_kind_when_the_value_is_empty() {
        let value = "";

        let expected_result = Err(InvalidJobKind::new(""));

        let result = JobKind::new(value);

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_return_invalid_job_kind_when_the_value_holds_a_dot() {
        let value = "..";

        let expected_result = Err(InvalidJobKind::new(".."));

        let result = JobKind::new(value);

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_return_invalid_job_kind_when_the_value_holds_an_upper_case_letter() {
        let value = "Series";

        let expected_result = Err(InvalidJobKind::new("Series"));

        let result = JobKind::new(value);

        assert_eq!(result, expected_result);
    }
}
