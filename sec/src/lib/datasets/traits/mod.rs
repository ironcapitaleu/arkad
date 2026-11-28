//! # Traits
//!
//! Provides [`ArrowRow`], the contract of a row type of one dataset, and [`PartOrder`], the order
//! of its rows in a part.

pub mod arrow_row;
pub mod part_order;

pub use arrow_row::ArrowRow;
pub use part_order::PartOrder;
