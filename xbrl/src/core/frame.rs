//! # Frame
//!
//! SEC XBRL frame identifiers encoding period and type information.
//! Examples: `CY2024`, `CY2024Q1`, `CY2024Q2I` (instant).

use std::fmt;

use super::quarter::Quarter;

/// An SEC XBRL frame identifier.
///
/// Encodes a calendar year, optional quarter, and whether the data point
/// is an instant (balance sheet) or duration measurement.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, serde::Serialize)]
pub struct Frame {
    year: u16,
    quarter: Option<Quarter>,
    instant: bool,
}

impl Frame {
    /// Parses a [`Frame`] from an SEC frame string (e.g., `"CY2024Q1I"`).
    ///
    /// Returns `None` if the input cannot be parsed.
    #[must_use]
    pub fn parse(s: &str) -> Option<Self> {
        let s = s.strip_prefix("CY")?;

        let (rest, instant) = s
            .strip_suffix('I')
            .map_or((s, false), |stripped| (stripped, true));

        let (year_str, quarter) = if rest.len() > 4 {
            let year_str = &rest[..4];
            let q_str = &rest[4..];
            let quarter = match q_str {
                "Q1" => Some(Quarter::Q1),
                "Q2" => Some(Quarter::Q2),
                "Q3" => Some(Quarter::Q3),
                "Q4" => Some(Quarter::Q4),
                _ => return None,
            };
            (year_str, quarter)
        } else {
            (rest, None)
        };

        let year = year_str.parse::<u16>().ok()?;
        Some(Self {
            year,
            quarter,
            instant,
        })
    }

    /// Returns the calendar year.
    #[must_use]
    pub const fn year(self) -> u16 {
        self.year
    }

    /// Returns the quarter, if present.
    #[must_use]
    pub const fn quarter(self) -> Option<Quarter> {
        self.quarter
    }

    /// Returns `true` if this frame represents an instant (balance sheet) measurement.
    #[must_use]
    pub const fn is_instant(self) -> bool {
        self.instant
    }
}

impl fmt::Display for Frame {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
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
    fn should_parse_annual_frame_when_input_is_cy2024() {
        let expected_result = Some(Frame {
            year: 2024,
            quarter: None,
            instant: false,
        });

        let result = Frame::parse("CY2024");

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_parse_quarterly_frame_when_input_has_quarter() {
        let expected_result = Some(Frame {
            year: 2024,
            quarter: Some(Quarter::Q1),
            instant: false,
        });

        let result = Frame::parse("CY2024Q1");

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_parse_instant_frame_when_input_ends_with_i() {
        let expected_result = Some(Frame {
            year: 2024,
            quarter: Some(Quarter::Q2),
            instant: true,
        });

        let result = Frame::parse("CY2024Q2I");

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_return_none_when_input_has_no_cy_prefix() {
        let expected_result = None;

        let result = Frame::parse("2024Q1");

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_roundtrip_display_when_frame_is_formatted_and_reparsed() {
        let frame = Frame {
            year: 2023,
            quarter: Some(Quarter::Q3),
            instant: true,
        };

        let expected_result = Some(frame);

        let result = Frame::parse(&frame.to_string());

        assert_eq!(result, expected_result);
    }
}
