//! # Raw Source
//!
//! Provides the [`RawSource`] trait, which names the SEC file a
//! [`RawDocument`](crate::shared::raw_document::RawDocument) holds, and one implementation per
//! file.
//!
//! ## Modules
//!
//! - [`traits`]: The [`RawSource`] contract, and [`RawFileSource`] for SEC files that arrive as
//!   raw bytes.
//! - [`implementations`]: One marker type per SEC file, such as [`RawCompanyFacts`].

pub mod implementations;
pub mod traits;

pub use implementations::{RawCompanyFacts, RawDailyIndex, RawNotesArchive, RawSubmissions};
pub use traits::{RawFileSource, RawSource};
