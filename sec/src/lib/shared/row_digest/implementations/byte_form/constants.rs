//! # Byte Form Constants
//!
//! The marker bytes that start each value of the byte form, as ADR 0033 fixes them.

/// The marker byte of a null value. No more bytes follow it.
pub const NULL_MARKER: u8 = 0x00;

/// The marker byte of a value that is not null. A length or a count follows it.
pub const VALUE_MARKER: u8 = 0x01;
