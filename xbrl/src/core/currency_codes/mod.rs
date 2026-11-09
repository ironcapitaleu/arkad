//! # Currency Codes
//!
//! Provides the tables of currency codes that tell a currency unit from any other unit.
//!
//! The measures of a unit carry no `iso4217:` prefix, so the text alone cannot tell `USD` from a
//! custom measure such as `BOE`. [`Unit::is_currency`](super::unit::Unit::is_currency) looks the
//! measure up in these tables instead. The tables are compiled in, so no file is read at run time.
//!
//! ## Modules
//!
//! - [`iso_4217`]: The codes of the current and withdrawn ISO 4217 currencies, generated from the
//!   lists of the maintenance agency SIX.

pub mod iso_4217;
