//! # Fake Write Repository
//!
//! Provides [`FakeRawRepository`], an in-memory [`WriteRepository`] of raw documents of one
//! source, and its aliases [`FakeWriteRepository`], [`FakeNotesRepository`],
//! [`FakeDailyIndexRepository`], [`FakeFilingIndexRepository`], [`FakeSubmissionsRepository`], [`FakeXbrlInstanceRepository`], and
//! [`FakeXbrlLabelsRepository`].

use std::cmp::Ordering;
use std::fmt;
use std::hash::{Hash, Hasher};
use std::sync::{Arc, Mutex};

use async_trait::async_trait;
use serde::ser::SerializeStruct;
use serde::{Serialize, Serializer};
use storage::{WriteError, WriteRepository};

use crate::shared::raw_document::RawDocument;
use crate::shared::raw_source::{
    RawCompanyFacts, RawDailyIndex, RawFilingIndex, RawNotesArchive, RawSource, RawSubmissions,
    RawXbrlInstance, RawXbrlLabels,
};

/// An in-memory [`WriteRepository`] that records each persisted document, or rejects each write.
///
/// Clones share one record, so a test can keep a clone and read what a state persisted.
#[derive(Debug, Clone)]
pub struct FakeRawRepository<S: RawSource> {
    persisted: Arc<Mutex<Vec<RawDocument<S>>>>,
    rejection: Option<WriteError>,
}

/// A [`FakeRawRepository`] of raw Company Facts documents.
pub type FakeWriteRepository = FakeRawRepository<RawCompanyFacts>;

/// A [`FakeRawRepository`] of raw Notes archives.
pub type FakeNotesRepository = FakeRawRepository<RawNotesArchive>;

/// A [`FakeRawRepository`] of raw daily indexes.
pub type FakeDailyIndexRepository = FakeRawRepository<RawDailyIndex>;

/// A [`FakeRawRepository`] of raw folder indexes of filings.
pub type FakeFilingIndexRepository = FakeRawRepository<RawFilingIndex>;

/// A [`FakeRawRepository`] of raw `submissions` documents.
pub type FakeSubmissionsRepository = FakeRawRepository<RawSubmissions>;

/// A [`FakeRawRepository`] of raw XBRL instances.
pub type FakeXbrlInstanceRepository = FakeRawRepository<RawXbrlInstance>;

/// A [`FakeRawRepository`] of raw label files.
pub type FakeXbrlLabelsRepository = FakeRawRepository<RawXbrlLabels>;

impl<S: RawSource> Default for FakeRawRepository<S> {
    fn default() -> Self {
        Self {
            persisted: Arc::default(),
            rejection: None,
        }
    }
}

impl<S: RawSource> FakeRawRepository<S> {
    /// Creates a [`FakeRawRepository`] that accepts every write.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Creates a [`FakeRawRepository`] that rejects every write with the given error.
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
    pub fn persisted(&self) -> Vec<RawDocument<S>> {
        self.persisted
            .lock()
            .expect("No test panics while it holds the fake repository mutex")
            .clone()
    }
}

/// Formats as `Fake write repository`, or with `, rejecting writes with <error>` when it rejects.
impl<S: RawSource> fmt::Display for FakeRawRepository<S> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &self.rejection {
            None => write!(f, "Fake write repository"),
            Some(error) => write!(f, "Fake write repository, rejecting writes with {error}"),
        }
    }
}

/// Serializes as `{ "kind": "fake", "rejection": null }`, or with the rejection message.
impl<Source: RawSource> Serialize for FakeRawRepository<Source> {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let mut state = serializer.serialize_struct("FakeRawRepository", 2)?;
        state.serialize_field("kind", "fake")?;
        state.serialize_field(
            "rejection",
            &self.rejection.as_ref().map(ToString::to_string),
        )?;
        state.end()
    }
}

#[async_trait]
impl<S: RawSource + Send + Sync> WriteRepository for FakeRawRepository<S> {
    type Record = RawDocument<S>;

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

impl<S: RawSource> PartialEq for FakeRawRepository<S> {
    fn eq(&self, other: &Self) -> bool {
        self.rejection == other.rejection
    }
}

impl<S: RawSource> Eq for FakeRawRepository<S> {}

impl<S: RawSource> PartialOrd for FakeRawRepository<S> {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl<S: RawSource> Ord for FakeRawRepository<S> {
    fn cmp(&self, other: &Self) -> Ordering {
        self.rejection.cmp(&other.rejection)
    }
}

impl<S: RawSource> Hash for FakeRawRepository<S> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.rejection.hash(state);
    }
}
