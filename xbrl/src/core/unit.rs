//! # Unit
//!
//! Provides [`Unit`], the unit of a numeric fact, and [`Measure`], one part of a unit.

use std::fmt;
use std::str::FromStr;

use super::currency_codes::iso_4217::{CURRENCY_CODES, HISTORIC_CURRENCY_CODES};
use super::invalid_value::{InvalidValue, InvalidValueReason, require_token};

/// One measure of a unit, for example `USD`, `EUR`, `shares`, or `pure`.
///
/// A measure holds the local name as the source writes it. The Notes data set and the SEC JSON APIs
/// write no prefix, so the instance parser drops the prefix too, and both sources give one text.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, serde::Serialize)]
pub struct Measure(String);

impl Measure {
    /// The type name that [`InvalidValue`] names when an input breaks a rule.
    const TYPE_NAME: &'static str = "Measure";

    /// Creates a [`Measure`] from its text.
    ///
    /// # Errors
    ///
    /// Returns [`InvalidValue`] if the text is empty, holds whitespace, or holds `/` or `*`.
    pub fn new(text: impl Into<String>) -> Result<Self, InvalidValue> {
        let text = text.into();
        require_token(Self::TYPE_NAME, &text)?;
        if let Some(separator) = text
            .chars()
            .find(|c| [Unit::DIVISION, Unit::MULTIPLICATION].contains(c))
        {
            return Err(InvalidValue::new(
                Self::TYPE_NAME,
                InvalidValueReason::ContainsSeparator(separator),
                text,
            ));
        }
        Ok(Self(text))
    }

    /// Returns the text of the measure.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for Measure {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// The unit of a numeric fact: numerator measures and denominator measures.
///
/// The set of units is open. A filer can report in any currency, for example `EUR`, and an XBRL
/// unit can divide or multiply measures, for example `USD/shares`. The text form is the canonical
/// form of silver: the numerator measures joined by `*`, then `/` and the denominator measures if
/// there are any. The measures of each side are sorted, so one unit has one text.
///
/// # Examples
///
/// ```
/// use std::str::FromStr;
///
/// use xbrl::core::unit::Unit;
///
/// let text = "USD/shares";
///
/// let expected_result = "USD/shares";
///
/// let unit = Unit::from_str(text).expect("Given a hardcoded unit with one measure per side, the parse should always succeed");
/// let result = unit.to_string();
///
/// assert_eq!(result, expected_result);
/// ```
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, serde::Serialize)]
pub struct Unit {
    numerator: Vec<Measure>,
    denominator: Vec<Measure>,
}

impl Unit {
    /// The type name that [`InvalidValue`] names when an input breaks a rule.
    const TYPE_NAME: &'static str = "Unit";

    /// The character between the numerator and the denominator in the text form.
    pub const DIVISION: char = '/';

    /// The character between two measures of one side in the text form.
    pub const MULTIPLICATION: char = '*';

    /// Creates a [`Unit`] from its numerator and denominator measures, and sorts each side.
    ///
    /// # Errors
    ///
    /// Returns [`InvalidValue`] with [`InvalidValueReason::Empty`] if the numerator has no measure.
    pub fn new(
        mut numerator: Vec<Measure>,
        mut denominator: Vec<Measure>,
    ) -> Result<Self, InvalidValue> {
        if numerator.is_empty() {
            return Err(InvalidValue::new(
                Self::TYPE_NAME,
                InvalidValueReason::Empty,
                "",
            ));
        }
        numerator.sort();
        denominator.sort();
        Ok(Self {
            numerator,
            denominator,
        })
    }

    /// Returns the numerator measures, sorted.
    #[must_use]
    pub fn numerator(&self) -> &[Measure] {
        &self.numerator
    }

    /// Returns the denominator measures, sorted. The slice is empty if the unit divides by nothing.
    #[must_use]
    pub fn denominator(&self) -> &[Measure] {
        &self.denominator
    }

    /// Returns `true` if the unit is one currency, for example `USD` or `EUR`, and divides by
    /// nothing.
    ///
    /// A currency is a code in [`CURRENCY_CODES`] or [`HISTORIC_CURRENCY_CODES`]. The measures carry
    /// no prefix, so the tables tell a currency from a custom measure such as `BOE`. A withdrawn
    /// code such as `BGN` counts, because an old filing reports in it. The tables leave out codes
    /// that are not money, such as `XAU` (gold) and `XDR`. `shares`, `pure`, and `USD/shares` are
    /// not a currency either.
    ///
    /// # Examples
    ///
    /// ```
    /// use std::str::FromStr;
    ///
    /// use xbrl::core::unit::Unit;
    ///
    /// let unit = Unit::from_str("BOE").expect("Given a hardcoded unit with one measure, the parse should always succeed");
    ///
    /// let expected_result = false;
    ///
    /// let result = unit.is_currency();
    ///
    /// assert_eq!(result, expected_result);
    /// ```
    #[must_use]
    pub fn is_currency(&self) -> bool {
        match (self.numerator.as_slice(), self.denominator.as_slice()) {
            ([measure], []) => {
                let code = measure.as_str();
                CURRENCY_CODES.binary_search(&code).is_ok()
                    || HISTORIC_CURRENCY_CODES.binary_search(&code).is_ok()
            }
            _ => false,
        }
    }
}

impl FromStr for Unit {
    type Err = InvalidValue;

    /// Parses a unit from its text form, for example `USD`, `USD/shares`, or `USD*shares/pure`.
    ///
    /// # Errors
    ///
    /// Returns [`InvalidValue`] if a side is empty, a measure is invalid, or the text holds more than
    /// one `/`.
    fn from_str(text: &str) -> Result<Self, Self::Err> {
        let mut sides = text.splitn(2, Self::DIVISION);
        let numerator = parse_side(sides.next().unwrap_or_default(), text)?;
        let denominator = match sides.next() {
            Some(side) => parse_side(side, text)?,
            None => Vec::new(),
        };
        Self::new(numerator, denominator)
    }
}

fn parse_side(side: &str, text: &str) -> Result<Vec<Measure>, InvalidValue> {
    side.split(Unit::MULTIPLICATION)
        .map(|measure| {
            if measure.is_empty() {
                Err(InvalidValue::new(
                    Unit::TYPE_NAME,
                    InvalidValueReason::Empty,
                    text,
                ))
            } else {
                Measure::new(measure)
                    .map_err(|error| InvalidValue::new(Unit::TYPE_NAME, error.reason, text))
            }
        })
        .collect()
}

impl fmt::Display for Unit {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write_measures(f, &self.numerator)?;
        if !self.denominator.is_empty() {
            write!(f, "{}", Self::DIVISION)?;
            write_measures(f, &self.denominator)?;
        }
        Ok(())
    }
}

fn write_measures(f: &mut fmt::Formatter<'_>, measures: &[Measure]) -> fmt::Result {
    for (index, measure) in measures.iter().enumerate() {
        if index > 0 {
            write!(f, "{}", Unit::MULTIPLICATION)?;
        }
        write!(f, "{measure}")?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use pretty_assertions::assert_eq;

    use super::*;

    fn measure(text: &str) -> Measure {
        Measure::new(text).expect(
            "Given a hardcoded measure without separators, the creation should always succeed",
        )
    }

    const fn implements_auto_traits<T: Sized + Send + Sync + Unpin>() {}
    #[test]
    const fn should_implement_auto_traits_for_measure() {
        implements_auto_traits::<Measure>();
    }

    #[test]
    const fn should_implement_auto_traits_for_unit() {
        implements_auto_traits::<Unit>();
    }

    #[test]
    fn should_parse_currency_when_unit_has_one_measure() {
        let text = "EUR";

        let expected_result = Unit::new(vec![measure("EUR")], vec![]);

        let result = Unit::from_str(text);

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_report_a_currency_when_the_unit_is_usd() {
        let unit = Unit::from_str("USD")
            .expect("Given a hardcoded unit with valid measures, the parse should always succeed");

        let expected_result = true;

        let result = unit.is_currency();

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_report_a_currency_when_the_unit_is_eur() {
        let unit = Unit::from_str("EUR")
            .expect("Given a hardcoded unit with valid measures, the parse should always succeed");

        let expected_result = true;

        let result = unit.is_currency();

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_report_a_currency_when_the_unit_is_a_withdrawn_currency() {
        let unit = Unit::from_str("BGN")
            .expect("Given a hardcoded unit with valid measures, the parse should always succeed");

        let expected_result = true;

        let result = unit.is_currency();

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_report_no_currency_when_the_unit_is_a_custom_measure_of_three_capitals() {
        let unit = Unit::from_str("BOE")
            .expect("Given a hardcoded unit with valid measures, the parse should always succeed");

        let expected_result = false;

        let result = unit.is_currency();

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_report_no_currency_when_the_unit_is_gold() {
        let unit = Unit::from_str("XAU")
            .expect("Given a hardcoded unit with valid measures, the parse should always succeed");

        let expected_result = false;

        let result = unit.is_currency();

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_report_no_currency_when_the_unit_is_a_unit_of_account() {
        let unit = Unit::from_str("XDR")
            .expect("Given a hardcoded unit with valid measures, the parse should always succeed");

        let expected_result = false;

        let result = unit.is_currency();

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_report_no_currency_when_the_unit_is_a_fund_code() {
        let unit = Unit::from_str("USN")
            .expect("Given a hardcoded unit with valid measures, the parse should always succeed");

        let expected_result = false;

        let result = unit.is_currency();

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_report_no_currency_when_the_unit_is_shares() {
        let unit = Unit::from_str("shares")
            .expect("Given a hardcoded unit with valid measures, the parse should always succeed");

        let expected_result = false;

        let result = unit.is_currency();

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_report_no_currency_when_a_currency_divides_by_shares() {
        let unit = Unit::from_str("USD/shares")
            .expect("Given a hardcoded unit with valid measures, the parse should always succeed");

        let expected_result = false;

        let result = unit.is_currency();

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_hold_sorted_codes_for_the_current_currency_table() {
        let codes = CURRENCY_CODES;

        let expected_result = true;

        let result = codes.is_sorted_by(|left, right| left < right);

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_hold_sorted_codes_for_the_historic_currency_table() {
        let codes = HISTORIC_CURRENCY_CODES;

        let expected_result = true;

        let result = codes.is_sorted_by(|left, right| left < right);

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_hold_only_three_capital_letters_for_the_current_currency_table() {
        let codes = CURRENCY_CODES;

        let expected_result: Vec<&str> = vec![];

        let result: Vec<&str> = codes
            .iter()
            .copied()
            .filter(|code| code.len() != 3 || !code.bytes().all(|byte| byte.is_ascii_uppercase()))
            .collect();

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_hold_only_three_capital_letters_for_the_historic_currency_table() {
        let codes = HISTORIC_CURRENCY_CODES;

        let expected_result: Vec<&str> = vec![];

        let result: Vec<&str> = codes
            .iter()
            .copied()
            .filter(|code| code.len() != 3 || !code.bytes().all(|byte| byte.is_ascii_uppercase()))
            .collect();

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_hold_no_shared_code_for_the_current_and_historic_tables() {
        let historic = HISTORIC_CURRENCY_CODES;

        let expected_result: Vec<&str> = vec![];

        let result: Vec<&str> = historic
            .iter()
            .copied()
            .filter(|code| CURRENCY_CODES.binary_search(code).is_ok())
            .collect();

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_parse_denominator_when_unit_divides() {
        let text = "USD/shares";

        let expected_result = Unit::new(vec![measure("USD")], vec![measure("shares")]);

        let result = Unit::from_str(text);

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_print_sorted_measures_when_numerator_has_two_measures() {
        let unit = Unit::new(
            vec![measure("shares"), measure("USD")],
            vec![measure("pure")],
        )
        .expect("Given a hardcoded numerator with measures, the creation should always succeed");

        let expected_result = "USD*shares/pure";

        let result = unit.to_string();

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_reject_unit_when_numerator_is_empty() {
        let numerator = vec![];

        let expected_result = Err(InvalidValue::new("Unit", InvalidValueReason::Empty, ""));

        let result = Unit::new(numerator, vec![measure("shares")]);

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_reject_text_when_denominator_is_empty() {
        let text = "USD/";

        let expected_result = Err(InvalidValue::new("Unit", InvalidValueReason::Empty, "USD/"));

        let result = Unit::from_str(text);

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_reject_text_when_it_holds_two_slashes() {
        let text = "USD/shares/day";

        let expected_result = Err(InvalidValue::new(
            "Unit",
            InvalidValueReason::ContainsSeparator('/'),
            "USD/shares/day",
        ));

        let result = Unit::from_str(text);

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_reject_text_when_a_measure_holds_whitespace() {
        let text = "US D";

        let expected_result = Err(InvalidValue::new(
            "Unit",
            InvalidValueReason::ContainsWhitespace,
            "US D",
        ));

        let result = Unit::from_str(text);

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_reject_measure_when_it_holds_a_slash() {
        let text = "USD/shares";

        let expected_result = Err(InvalidValue::new(
            "Measure",
            InvalidValueReason::ContainsSeparator('/'),
            "USD/shares",
        ));

        let result = Measure::new(text);

        assert_eq!(result, expected_result);
    }
}
