//! # Sample Input Document
//!
//! Provides sample [`InputDocument`](storage::implementations::repository::filesystem::InputDocument)
//! lists for the metadata `inputs` of stored documents.
//!
//! ## Modules
//!
//! - [`sample_daily_index_input`]: [`daily_index_inputs`], the inputs of a filing that one daily
//!   index listed.

pub mod sample_daily_index_input;

pub use sample_daily_index_input::daily_index_inputs;
