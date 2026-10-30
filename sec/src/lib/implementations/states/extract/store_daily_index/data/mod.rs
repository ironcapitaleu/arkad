//! # Store Daily Index Data
//!
//! Groups the input and output data types for the
//! [`StoreDailyIndex`](super::StoreDailyIndex) state.
//!
//! ## Modules
//!
//! - [`input`]: The [`StoreDailyIndexInput`] holding the downloaded [`SecResponse<ZipBody>`](crate::shared::response::implementations::sec_response::SecResponse) and its [`IndexDate`](crate::shared::index_date::IndexDate).
//! - [`output`]: The [`StoreDailyIndexOutput`] holding the key and the text of the stored index.

pub mod input;
pub mod output;

pub use input::StoreDailyIndexInput;
pub use output::{StoreDailyIndexOutput, StoredDailyIndex};
