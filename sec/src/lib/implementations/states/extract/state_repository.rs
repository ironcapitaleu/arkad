//! # State Repository
//!
//! Provides the [`StateRepository`] bound on a repository that a state holds without writing to
//! it.

use std::fmt::Debug;
use std::hash::Hash;

use serde::Serialize;
use storage::WriteRepository;

/// A [`WriteRepository`] of any record that a state context can hold.
///
/// A state context must implement the value traits and `Serialize`, so the repository must
/// implement them too. Every type that meets the bounds implements this trait, including every
/// [`RawCompanyFactsRepository`](super::store_company_facts::RawCompanyFactsRepository) and every
/// [`NotesRepository`](super::notes_repository::NotesRepository).
pub trait StateRepository:
    WriteRepository + Debug + Clone + Eq + Ord + Hash + Send + Sync + Unpin + Serialize
{
}

impl<R> StateRepository for R where
    R: WriteRepository + Debug + Clone + Eq + Ord + Hash + Send + Sync + Unpin + Serialize
{
}
