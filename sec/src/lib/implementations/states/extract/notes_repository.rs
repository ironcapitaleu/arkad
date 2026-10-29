//! # Notes Repository
//!
//! Provides the [`NotesRepository`] bound on the repository that the Notes extract states hold.

use std::fmt::Debug;
use std::hash::Hash;

use serde::Serialize;
use storage::WriteRepository;

use crate::shared::raw_document::RawDocument;
use crate::shared::raw_source::RawNotesArchive;

/// A [`WriteRepository`] of raw Notes archives that a Notes extract state context can hold.
///
/// A state context must implement the value traits and `Serialize`, so the repository must
/// implement them too.
pub trait NotesRepository:
    WriteRepository<Record = RawDocument<RawNotesArchive>>
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

impl<R> NotesRepository for R where
    R: WriteRepository<Record = RawDocument<RawNotesArchive>>
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
