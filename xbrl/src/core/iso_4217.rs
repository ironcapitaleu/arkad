//! # ISO 4217 Currency Codes
//!
//! Provides [`CURRENCY_CODES`], the codes of the current ISO 4217 currencies, and
//! [`HISTORIC_CURRENCY_CODES`], the codes of withdrawn ISO 4217 currencies.
//!
//! Sources, both XML files of the maintenance agency SIX:
//!
//! - List One (current currencies), published 2026-09-17, SHA-256
//!   `33139b438657d1cee116ba737807ea71d19d6de4b90f799a09c56f0cc6a1b0ff`:
//!   <https://www.six-group.com/dam/download/financial-information/data-center/iso-currrency/lists/list-one.xml>
//! - List Three (historic denominations), published 2026-01-01, SHA-256
//!   `98fde2423cdb916dd59dcf5fe96222edad8fa198d865c1c83dbc464b9cc52387`:
//!   <https://www.six-group.com/dam/download/financial-information/data-center/iso-currrency/lists/list-three.xml>
//!
//! The tables leave out the codes that are not money: fund codes, the List One entries with no
//! minor unit (`N.A.`), such as `XAU`, `XDR`, `XTS`, and `XXX`, and the List Three codes that
//! start with `X`, such as `XEU`. A code of both lists is only in [`CURRENCY_CODES`]. Do not edit
//! the tables by hand. The script in the description of pull request #479 writes them.

/// The codes of the current ISO 4217 currencies, sorted, for example `EUR` and `USD`.
pub const CURRENCY_CODES: &[&str] = &[
    "AED", "AFN", "ALL", "AMD", "AOA", "ARS", "AUD", "AWG", "AZN", "BAM", "BBD", "BDT", "BHD",
    "BIF", "BMD", "BND", "BOB", "BRL", "BSD", "BTN", "BWP", "BYN", "BZD", "CAD", "CDF", "CHF",
    "CLP", "CNY", "COP", "CRC", "CUP", "CVE", "CZK", "DJF", "DKK", "DOP", "DZD", "EGP", "ERN",
    "ETB", "EUR", "FJD", "FKP", "GBP", "GEL", "GHS", "GIP", "GMD", "GNF", "GTQ", "GYD", "HKD",
    "HNL", "HTG", "HUF", "IDR", "ILS", "INR", "IQD", "IRR", "ISK", "JMD", "JOD", "JPY", "KES",
    "KGS", "KHR", "KMF", "KPW", "KRW", "KWD", "KYD", "KZT", "LAK", "LBP", "LKR", "LRD", "LSL",
    "LYD", "MAD", "MDL", "MGA", "MKD", "MMK", "MNT", "MOP", "MRU", "MUR", "MVR", "MWK", "MXN",
    "MYR", "MZN", "NAD", "NGN", "NIO", "NOK", "NPR", "NZD", "OMR", "PAB", "PEN", "PGK", "PHP",
    "PKR", "PLN", "PYG", "QAR", "RON", "RSD", "RUB", "RWF", "SAR", "SBD", "SCR", "SDG", "SEK",
    "SGD", "SHP", "SLE", "SOS", "SRD", "SSP", "STN", "SVC", "SYP", "SZL", "THB", "TJS", "TMT",
    "TND", "TOP", "TRY", "TTD", "TWD", "TZS", "UAH", "UGX", "USD", "UYU", "UZS", "VED", "VES",
    "VND", "VUV", "WST", "XAF", "XCD", "XCG", "XOF", "XPF", "YER", "ZAR", "ZMW", "ZWG",
];

/// The codes of withdrawn ISO 4217 currencies, sorted, for example `DEM` and `HRK`.
pub const HISTORIC_CURRENCY_CODES: &[&str] = &[
    "ADP", "AFA", "ALK", "ANG", "AOK", "AON", "AOR", "ARA", "ARP", "ARY", "ATS", "AYM", "AZM",
    "BAD", "BEC", "BEF", "BEL", "BGJ", "BGK", "BGL", "BGN", "BOP", "BRB", "BRC", "BRE", "BRN",
    "BRR", "BUK", "BYB", "BYR", "CHC", "CSD", "CSJ", "CSK", "CUC", "CYP", "DDM", "DEM", "ECS",
    "ECV", "EEK", "ESA", "ESB", "ESP", "FIM", "FRF", "GEK", "GHC", "GHP", "GNE", "GNS", "GQE",
    "GRD", "GWE", "GWP", "HRD", "HRK", "IEP", "ILP", "ILR", "ISJ", "ITL", "LAJ", "LSM", "LTL",
    "LTT", "LUC", "LUF", "LUL", "LVL", "LVR", "MGF", "MLF", "MRO", "MTL", "MTP", "MVQ", "MXP",
    "MZE", "MZM", "NIC", "NLG", "PEH", "PEI", "PES", "PLZ", "PTE", "RHD", "ROK", "ROL", "RUR",
    "SDD", "SDP", "SIT", "SKK", "SLL", "SRG", "STD", "SUR", "TJR", "TMM", "TPE", "TRL", "UAK",
    "UGS", "UGW", "USS", "UYN", "UYP", "VEB", "VEF", "VNC", "YDD", "YUD", "YUM", "YUN", "ZAL",
    "ZMK", "ZRN", "ZRZ", "ZWC", "ZWD", "ZWL", "ZWN", "ZWR",
];
