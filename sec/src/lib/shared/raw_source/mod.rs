//! # Raw Source
//!
//! Provides the [`RawSource`] trait, which names the SEC file a
//! [`RawDocument`](crate::shared::raw_document::RawDocument) holds, and one implementation per
//! file.
//!
//! ## Modules
//!
//! - [`traits`]: The [`RawSource`] contract.
//! - [`implementations`]: One marker type per SEC file, such as [`RawCompanyFacts`].

pub mod implementations;
pub mod traits;

pub use implementations::{RawCompanyFacts, RawNotesArchive};
pub use traits::RawSource;
