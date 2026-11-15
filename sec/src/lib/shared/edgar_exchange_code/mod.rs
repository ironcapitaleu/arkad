//! # EDGAR Exchange Code
//!
//! Provides [`mic_of`], the map from an EDGAR exchange code to the [`Mic`] of the exchange it
//! names.
//!
//! `dei:SecurityExchangeName` holds an EDGAR exchange code, such as `NASDAQ`. The codes are the
//! enumeration `edgarExchangeCodeItemType` of the `dei` schema. The MICs come from the ISO 10383
//! list of 2026-11-09.

use std::collections::HashMap;
use std::sync::LazyLock;

use storage::Mic;

/// Maps each EDGAR exchange code that names an exchange to the MIC of that exchange.
///
/// `NONE` names no exchange, so the map leaves it out.
static MIC_BY_EDGAR_EXCHANGE_CODE: LazyLock<HashMap<&'static str, &'static str>> =
    LazyLock::new(|| {
        HashMap::from([
            ("NYSE", "XNYS"),
            ("NASDAQ", "XNAS"),
            // CHX and NYSETX name one exchange: the Chicago Stock Exchange, now NYSE Texas.
            ("CHX", "XCHI"),
            ("BOX", "XBOX"),
            ("BX", "XBOS"),
            ("C2", "C2OX"),
            ("CBOE", "XCBO"),
            ("CboeBYX", "BATY"),
            ("CboeBZX", "BATS"),
            ("CboeEDGA", "EDGA"),
            ("CboeEDGX", "EDGX"),
            ("GEMX", "GMNI"),
            ("IEX", "IEXG"),
            ("ISE", "XISX"),
            ("MIAX", "XMIO"),
            ("MRX", "MCRY"),
            ("NYSEAMER", "XASE"),
            ("NYSEArca", "ARCX"),
            ("NYSENAT", "XCIS"),
            ("NYSETX", "XCHI"),
            // The MIC of the exchange MIAX PEARL, LLC. Its equities segment has the MIC `EPRL`.
            ("PEARL", "MPRL"),
            ("Phlx", "XPHL"),
        ])
    });

/// Returns the MIC of the exchange that an EDGAR exchange code names, or `None`.
///
/// The code matches exactly, case included, so `Phlx` maps and `PHLX` does not. `NONE` names no
/// exchange, so it returns `None`. A code outside the enumeration also returns `None`, and the
/// caller keeps the code as filed.
///
/// # Examples
///
/// ```rust
/// use sec::shared::edgar_exchange_code::mic_of;
///
/// let code = "NASDAQ";
///
/// let expected_result = Some("XNAS".to_owned());
///
/// let result = mic_of(code).map(|mic| mic.to_string());
///
/// assert_eq!(result, expected_result);
/// ```
#[must_use]
pub fn mic_of(code: &str) -> Option<Mic> {
    // Every MIC of the map is valid. The test of every code of the enumeration checks it.
    Mic::new(*MIC_BY_EDGAR_EXCHANGE_CODE.get(code)?).ok()
}

#[cfg(test)]
mod tests {
    use pretty_assertions::assert_eq;

    use super::*;

    /// The values of `edgarExchangeCodeItemType` in the vendored `dei` schema, in schema order.
    const DEI_EXCHANGE_CODES: [&str; 23] = [
        "NONE", "NYSE", "NASDAQ", "CHX", "BOX", "BX", "C2", "CBOE", "CboeBYX", "CboeBZX",
        "CboeEDGA", "CboeEDGX", "GEMX", "IEX", "ISE", "MIAX", "MRX", "NYSEAMER", "NYSEArca",
        "NYSENAT", "NYSETX", "PEARL", "Phlx",
    ];

    /// Returns the `value` of each `xs:enumeration` in the type `edgarExchangeCodeItemType`.
    fn enumeration_of_the_vendored_schema() -> Vec<String> {
        let path = concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../xbrl/taxonomy/dei/2026/dei-2026.xsd"
        );
        let schema = std::fs::read_to_string(path).expect(
            "Given the dei schema vendored in the workspace, reading it should always succeed",
        );
        let start = schema
            .find(r#"name="edgarExchangeCodeItemType""#)
            .expect("Given the 2026 dei schema, it should always define edgarExchangeCodeItemType");
        let length = schema[start..]
            .find("</xs:complexType>")
            .expect("Given the 2026 dei schema, edgarExchangeCodeItemType should always be closed");
        schema[start..start + length]
            .split(r#"<xs:enumeration value=""#)
            .skip(1)
            .map(|rest| rest.split('"').next().unwrap_or_default().to_owned())
            .collect()
    }

    #[test]
    fn should_list_every_code_of_the_enumeration_for_the_vendored_dei_schema() {
        let expected_result = DEI_EXCHANGE_CODES;

        let result = enumeration_of_the_vendored_schema();

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_hold_every_exchange_code_but_none_for_the_map() {
        let map = &*MIC_BY_EDGAR_EXCHANGE_CODE;

        let expected_result = vec![
            "BOX", "BX", "C2", "CBOE", "CHX", "CboeBYX", "CboeBZX", "CboeEDGA", "CboeEDGX", "GEMX",
            "IEX", "ISE", "MIAX", "MRX", "NASDAQ", "NYSE", "NYSEAMER", "NYSEArca", "NYSENAT",
            "NYSETX", "PEARL", "Phlx",
        ];

        let mut result: Vec<&str> = map.keys().copied().collect();
        result.sort_unstable();

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_map_each_code_to_its_mic_for_every_code_of_the_dei_enumeration() {
        let codes = DEI_EXCHANGE_CODES;

        let expected_result = vec![
            ("NONE", None),
            ("NYSE", Some("XNYS")),
            ("NASDAQ", Some("XNAS")),
            ("CHX", Some("XCHI")),
            ("BOX", Some("XBOX")),
            ("BX", Some("XBOS")),
            ("C2", Some("C2OX")),
            ("CBOE", Some("XCBO")),
            ("CboeBYX", Some("BATY")),
            ("CboeBZX", Some("BATS")),
            ("CboeEDGA", Some("EDGA")),
            ("CboeEDGX", Some("EDGX")),
            ("GEMX", Some("GMNI")),
            ("IEX", Some("IEXG")),
            ("ISE", Some("XISX")),
            ("MIAX", Some("XMIO")),
            ("MRX", Some("MCRY")),
            ("NYSEAMER", Some("XASE")),
            ("NYSEArca", Some("ARCX")),
            ("NYSENAT", Some("XCIS")),
            ("NYSETX", Some("XCHI")),
            ("PEARL", Some("MPRL")),
            ("Phlx", Some("XPHL")),
        ];

        let mics: Vec<Option<String>> = codes
            .iter()
            .map(|code| mic_of(code).map(|mic| mic.to_string()))
            .collect();
        let result: Vec<(&str, Option<&str>)> = codes
            .iter()
            .copied()
            .zip(mics.iter().map(Option::as_deref))
            .collect();

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_return_none_when_the_code_is_none() {
        let code = "NONE";

        let expected_result = None;

        let result = mic_of(code);

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_return_none_when_the_code_is_not_in_the_enumeration() {
        let code = "LSE";

        let expected_result = None;

        let result = mic_of(code);

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_return_none_when_the_code_differs_in_case() {
        let code = "PHLX";

        let expected_result = None;

        let result = mic_of(code);

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_return_none_when_the_code_is_empty() {
        let code = "";

        let expected_result = None;

        let result = mic_of(code);

        assert_eq!(result, expected_result);
    }
}
