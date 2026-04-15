//! # Frame Module
//!
//! Provides the [`Frame`] struct representing an SEC XBRL frame identifier.
//! Frames like `CY2023`, `CY2023Q1`, or `CY2023Q3I` tag data points with their
//! calendar year, optional quarter, and whether the measurement is an instant or duration.

use std::fmt;

use crate::shared::financial::quarter::Quarter;

/// An SEC XBRL frame identifier.
///
/// Parsed from strings like `"CY2023"` (annual duration), `"CY2023Q1"` (quarterly duration),
/// or `"CY2023Q3I"` (quarterly instant). The frame encodes the calendar year, optional quarter,
/// and whether the observation is an instant (point-in-time) or duration measurement.
///
/// # Example
/// ```
/// use sec::shared::financial::frame::Frame;
///
/// let frame = Frame::parse("CY2023Q3I");
/// assert!(frame.is_some());
///
/// let frame = frame.unwrap();
/// assert_eq!(frame.year(), 2023);
/// assert!(frame.instant());
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, serde::Serialize)]
pub struct Frame {
    year: u16,
    quarter: Option<Quarter>,
    instant: bool,
}

impl Frame {
    /// Creates a new [`Frame`] with the given components.
    #[must_use]
    pub const fn new(year: u16, quarter: Option<Quarter>, instant: bool) -> Self {
        Self {
            year,
            quarter,
            instant,
        }
    }

    /// Returns the calendar year.
    #[must_use]
    pub const fn year(&self) -> u16 {
        self.year
    }

    /// Returns the optional calendar quarter.
    #[must_use]
    pub const fn quarter(&self) -> Option<Quarter> {
        self.quarter
    }

    /// Returns whether this frame represents an instant (point-in-time) measurement.
    #[must_use]
    pub const fn instant(&self) -> bool {
        self.instant
    }

    /// Attempts to parse a [`Frame`] from an SEC frame string (e.g., `"CY2023Q3I"`).
    ///
    /// Expected format: `CY{year}[Q{1-4}][I]`
    ///
    /// # Errors
    ///
    /// Returns `None` if the input string does not match the expected frame format.
    #[must_use]
    pub fn parse(s: &str) -> Option<Self> {
        let s = s.strip_prefix("CY")?;

        let (instant, s) = s
            .strip_suffix('I')
            .map_or((false, s), |stripped| (true, stripped));

        let (year_str, quarter_str) = s
            .find('Q')
            .map_or((s, None), |q_pos| (&s[..q_pos], Some(&s[q_pos + 1..])));

        let year: u16 = year_str.parse().ok()?;
        let quarter = match quarter_str {
            Some(q) => Some(Quarter::from_number_str(q)?),
            None => None,
        };

        Some(Self {
            year,
            quarter,
            instant,
        })
    }
}

impl fmt::Display for Frame {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "CY{}", self.year)?;
        if let Some(q) = self.quarter {
            write!(f, "{q}")?;
        }
        if self.instant {
            write!(f, "I")?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use pretty_assertions::assert_eq;

    use super::*;

    #[test]
    fn should_parse_annual_duration_frame_when_input_is_cy2023() {
        let expected_result = Some(Frame::new(2023, None, false));

        let result = Frame::parse("CY2023");

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_parse_quarterly_instant_frame_when_input_is_cy2023q3i() {
        let expected_result = Some(Frame::new(2023, Some(Quarter::Q3), true));

        let result = Frame::parse("CY2023Q3I");

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_parse_quarterly_duration_frame_when_input_is_cy2023q1() {
        let expected_result = Some(Frame::new(2023, Some(Quarter::Q1), false));

        let result = Frame::parse("CY2023Q1");

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_return_none_when_input_is_invalid_frame_string() {
        let expected_result = None;

        let result = Frame::parse("INVALID");

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_display_as_original_format_when_frame_is_annual_duration() {
        let frame = Frame::new(2023, None, false);

        let expected_result = "CY2023";

        let result = frame.to_string();

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_display_as_original_format_when_frame_is_quarterly_instant() {
        let frame = Frame::new(2023, Some(Quarter::Q3), true);

        let expected_result = "CY2023Q3I";

        let result = frame.to_string();

        assert_eq!(result, expected_result);
    }
}
