//! # Fake Write Repository
//!
//! Provides [`FakeWriteRepository`], an in-memory [`WriteRepository`] of raw Company Facts
//! documents.

use std::cmp::Ordering;
use std::hash::{Hash, Hasher};
use std::sync::{Arc, Mutex};

use async_trait::async_trait;
use serde::ser::SerializeStruct;
use serde::{Serialize, Serializer};
use storage::{WriteError, WriteRepository};

use crate::shared::raw_document::RawDocument;
use crate::shared::raw_source::RawCompanyFacts;

/// An in-memory [`WriteRepository`] that records each persisted document, or rejects each write.
///
/// Clones share one record, so a test can keep a clone and read what a state persisted.
#[derive(Debug, Clone, Default)]
pub struct FakeWriteRepository {
    persisted: Arc<Mutex<Vec<RawDocument<RawCompanyFacts>>>>,
    rejection: Option<WriteError>,
}

impl FakeWriteRepository {
    /// Creates a [`FakeWriteRepository`] that accepts every write.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Creates a [`FakeWriteRepository`] that rejects every write with the given error.
    #[must_use]
    pub fn rejecting(error: WriteError) -> Self {
        Self {
            persisted: Arc::default(),
            rejection: Some(error),
        }
    }

    /// Returns a clone of every document persisted so far, in insertion order.
    ///
    /// # Panics
    ///
    /// Panics if a previous panic poisoned the internal mutex.
    #[must_use]
    pub fn persisted(&self) -> Vec<RawDocument<RawCompanyFacts>> {
        self.persisted
            .lock()
            .expect("No test panics while it holds the fake repository mutex")
            .clone()
    }
}

/// Serializes as `{ "kind": "fake", "rejection": null }`, or with the rejection message.
impl Serialize for FakeWriteRepository {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let mut state = serializer.serialize_struct("FakeWriteRepository", 2)?;
        state.serialize_field("kind", "fake")?;
        state.serialize_field(
            "rejection",
            &self.rejection.as_ref().map(ToString::to_string),
        )?;
        state.end()
    }
}

#[async_trait]
impl WriteRepository for FakeWriteRepository {
    type Record = RawDocument<RawCompanyFacts>;

    async fn persist(&self, record: Self::Record) -> Result<(), WriteError> {
        if let Some(error) = &self.rejection {
            return Err(error.clone());
        }
        self.persisted
            .lock()
            .expect("No test panics while it holds the fake repository mutex")
            .push(record);
        Ok(())
    }
}

// Deviation: the shared record has no value semantics, so two fakes compare by their rejection
// only. This satisfies the `Eq + Ord + Hash` bounds that a state context requires.

impl PartialEq for FakeWriteRepository {
    fn eq(&self, other: &Self) -> bool {
        self.rejection == other.rejection
    }
}

impl Eq for FakeWriteRepository {}

impl PartialOrd for FakeWriteRepository {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for FakeWriteRepository {
    fn cmp(&self, other: &Self) -> Ordering {
        self.rejection.cmp(&other.rejection)
    }
}

impl Hash for FakeWriteRepository {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.rejection.hash(state);
    }
}
