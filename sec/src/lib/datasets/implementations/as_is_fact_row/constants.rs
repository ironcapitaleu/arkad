//! # Constants
//!
//! The texts of the `value_kind` column and the type of the `value_numeric` column of
//! `as_is_facts`.

/// The `value_kind` of a numeric value.
pub const VALUE_KIND_NUMERIC: &str = "numeric";

/// The `value_kind` of a text value.
pub const VALUE_KIND_TEXT: &str = "text";

/// The `value_kind` of a nil value.
pub const VALUE_KIND_NIL: &str = "nil";

/// The digits of a `value_numeric` value.
pub const DECIMAL_PRECISION: u8 = 38;

/// The digits after the point of a `value_numeric` value.
pub const DECIMAL_SCALE: i8 = 10;

/// The unscaled digits of every `value_numeric` value are below this limit: `10^38`.
pub const DECIMAL_LIMIT: u128 = 10_u128.pow(38);
