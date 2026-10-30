//! # Raw Repository Trait
//!
//! Provides [`RawRepository`], the bound on a repository that writes the
//! [`RawDocument`]s of one [`RawSource`].

use std::fmt::{Debug, Display};
use std::hash::Hash;

use serde::Serialize;
use storage::WriteRepository;

use crate::shared::raw_document::RawDocument;
use crate::shared::raw_source::RawSource;

/// A [`WriteRepository`] of the [`RawDocument`]s of the source `S` that a state context can hold.
///
/// A state context must implement the value traits, `Serialize` and `Display`, so the repository
/// must implement them too. Every type that meets the bounds implements this trait, for example
/// `FilesystemRepository<Write, RawDocument<S>>`.
pub trait RawRepository<S: RawSource>:
    WriteRepository<Record = RawDocument<S>>
    + Debug
    + Display
    + Clone
    + Eq
    + Ord
    + Hash
    + Unpin
    + Serialize
{
}

impl<S: RawSource, R> RawRepository<S> for R where
    R: WriteRepository<Record = RawDocument<S>>
        + Debug
        + Display
        + Clone
        + Eq
        + Ord
        + Hash
        + Unpin
        + Serialize
{
}

#[cfg(test)]
mod tests {
    use storage::implementations::repository::filesystem::{FilesystemRepository, Write};

    use super::*;
    use crate::shared::raw_source::{RawCompanyFacts, RawDailyIndex, RawNotesArchive};
    use crate::tests::fixtures::sample_write_repository::{
        FakeDailyIndexRepository, FakeNotesRepository, FakeWriteRepository,
    };

    const fn implements_raw_repository<S: RawSource, T: RawRepository<S>>() {}

    #[test]
    const fn should_implement_raw_repository_for_filesystem_repository() {
        implements_raw_repository::<
            RawCompanyFacts,
            FilesystemRepository<Write, RawDocument<RawCompanyFacts>>,
        >();
    }

    #[test]
    const fn should_implement_raw_repository_for_fake_write_repository() {
        implements_raw_repository::<RawCompanyFacts, FakeWriteRepository>();
    }

    #[test]
    const fn should_implement_raw_repository_for_fake_notes_repository() {
        implements_raw_repository::<RawNotesArchive, FakeNotesRepository>();
    }

    #[test]
    const fn should_implement_raw_repository_for_fake_daily_index_repository() {
        implements_raw_repository::<RawDailyIndex, FakeDailyIndexRepository>();
    }
}
