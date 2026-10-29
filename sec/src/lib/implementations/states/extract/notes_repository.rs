//! # Notes Repository
//!
//! Provides the [`NotesRepository`] bound on the repository that the Notes extract states hold.

use std::fmt::Debug;
use std::hash::Hash;

/// A repository that a Notes extract state context can hold.
///
/// A state context must implement the value traits, so the repository must implement them too.
/// Every type that meets the bounds implements this trait. STA-338 adds the bound on the stored
/// record.
pub trait NotesRepository: Debug + Clone + Eq + Ord + Hash + Send + Sync + Unpin {}

impl<R> NotesRepository for R where R: Debug + Clone + Eq + Ord + Hash + Send + Sync + Unpin {}
