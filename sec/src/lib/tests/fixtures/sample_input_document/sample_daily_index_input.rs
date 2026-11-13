//! # Sample Daily Index Input
//!
//! Provides [`daily_index_inputs`], the inputs of a filing that one daily index listed.

use storage::Sha256Digest;
use storage::implementations::repository::filesystem::{DocumentKey, InputDocument};

/// Returns one [`InputDocument`] for the daily index `sec/daily-index/form.20260909.idx`, with the
/// SHA-256 of the bytes `index`.
///
/// # Panics
///
/// Never panics: the key is a hardcoded valid relative path.
#[must_use]
pub fn daily_index_inputs() -> Vec<InputDocument> {
    vec![InputDocument::new(
        DocumentKey::new("sec/daily-index/form.20260909.idx")
            .expect("Given a hardcoded valid relative path, the key should always build"),
        Sha256Digest::of(b"index"),
    )]
}
