//! # Daily Index Repository
//!
//! Provides the [`DailyIndexRepository`] bound on the repository that the daily-index states hold.

use std::fmt::Debug;
use std::hash::Hash;

use serde::Serialize;
use storage::WriteRepository;

use crate::shared::raw_document::RawDocument;
use crate::shared::raw_source::RawDailyIndex;

/// A [`WriteRepository`] of raw daily indexes that a daily-index state context can hold.
///
/// A state context must implement the value traits and `Serialize`, so the repository must
/// implement them too.
pub trait DailyIndexRepository:
    WriteRepository<Record = RawDocument<RawDailyIndex>>
    + Debug
    + Clone
    + Eq
    + Ord
    + Hash
    + Send
    + Sync
    + Unpin
    + Serialize
{
}

impl<R> DailyIndexRepository for R where
    R: WriteRepository<Record = RawDocument<RawDailyIndex>>
        + Debug
        + Clone
        + Eq
        + Ord
        + Hash
        + Send
        + Sync
        + Unpin
        + Serialize
{
}
