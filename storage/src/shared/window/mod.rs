//! # Window
//!
//! Provides [`Window`], the span of time that one quota limit counts over.
//!
//! ## Modules
//!
//! - [`invalid_window`]: The [`InvalidWindow`] for a string that is not a window.

use std::fmt::{self, Display, Formatter};
use std::str::FromStr;

pub mod invalid_window;

pub use invalid_window::InvalidWindow;

/// The span of time that one quota limit counts over: a UTC day or a minute.
///
/// A rejected call names the window whose limit rejected it. The name also labels the window in
/// the `RateLimit` headers.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Window {
    /// The UTC day, from 00:00 to 24:00, that the daily quota counts over.
    Day,
    /// The clock minute that the per-minute limit counts over.
    Minute,
}

impl Window {
    /// Returns the name of the window: `day` or `minute`.
    ///
    /// `docs/adr/0045-stored-names-of-access-enums.md` records the names.
    #[must_use]
    pub const fn as_str(&self) -> &'static str {
        match self {
            Self::Day => "day",
            Self::Minute => "minute",
        }
    }
}

impl Display for Window {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

impl FromStr for Window {
    type Err = InvalidWindow;

    /// Parses `day` or `minute` into a [`Window`].
    ///
    /// # Errors
    ///
    /// Returns an [`InvalidWindow`] for any other string, including `Day`.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use storage::Window;
    ///
    /// let value = "minute";
    ///
    /// let expected_result = Ok(Window::Minute);
    ///
    /// let result = value.parse::<Window>();
    ///
    /// assert_eq!(result, expected_result);
    /// ```
    fn from_str(value: &str) -> Result<Self, Self::Err> {
        match value {
            "day" => Ok(Self::Day),
            "minute" => Ok(Self::Minute),
            _ => Err(InvalidWindow::new(value)),
        }
    }
}

#[cfg(test)]
mod tests {
    use std::fmt::Debug;
    use std::hash::Hash;

    use pretty_assertions::assert_eq;

    use super::*;

    const fn implements_auto_traits<T: Sized + Send + Sync + Unpin>() {}
    #[test]
    const fn should_implement_auto_traits_for_window() {
        implements_auto_traits::<Window>();
    }

    const fn implements_value_traits<
        T: Debug + Clone + Copy + PartialEq + Eq + PartialOrd + Ord + Hash,
    >() {
    }
    #[test]
    const fn should_implement_value_traits_for_window() {
        implements_value_traits::<Window>();
    }

    #[test]
    fn should_display_day_for_the_day_window() {
        let window = Window::Day;

        let expected_result = "day";

        let result = window.to_string();

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_display_minute_for_the_minute_window() {
        let window = Window::Minute;

        let expected_result = "minute";

        let result = window.to_string();

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_parse_the_displayed_text_back_to_the_same_window_for_every_window() {
        let windows = [Window::Day, Window::Minute];

        let expected_result = vec![Ok(Window::Day), Ok(Window::Minute)];

        let result: Vec<_> = windows
            .iter()
            .map(|window| window.to_string().parse::<Window>())
            .collect();

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_return_invalid_window_when_the_string_is_capitalized() {
        let value = "Day";

        let expected_result = Err(InvalidWindow::new("Day"));

        let result = value.parse::<Window>();

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_return_invalid_window_when_the_string_is_empty() {
        let value = "";

        let expected_result = Err(InvalidWindow::new(""));

        let result = value.parse::<Window>();

        assert_eq!(result, expected_result);
    }
}
