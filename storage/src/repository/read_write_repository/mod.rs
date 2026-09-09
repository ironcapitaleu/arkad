//! # Read Write Repository
//!
//! Provides [`ReadWriteRepository`], the one name for a store that offers both the read capability
//! and the write capability.

use crate::repository::read_repository::ReadRepository;
use crate::repository::write_repository::WriteRepository;

/// Names a store that reads and writes.
///
/// Adds no method of its own. A blanket implementation grants it to every type that implements
/// both [`ReadRepository`] and [`WriteRepository`], so a backend adapter writes no implementation
/// for it.
///
/// Both parent traits declare a `Record`, so `ReadWriteRepository<Record = ...>` names an
/// ambiguous associated type and does not compile. A caller that pins the record type binds the
/// two parent traits instead:
///
/// ```rust
/// # use storage::{ReadRepository, WriteRepository};
/// fn takes_a_store<S>(_store: S)
/// where
///     S: ReadRepository<Record = String, Key = String> + WriteRepository<Record = String>,
/// {
/// }
/// ```
///
/// # Required Traits
///
/// - [`ReadRepository`]: gives the store its `get` method.
/// - [`WriteRepository`]: gives the store its `persist` method.
pub trait ReadWriteRepository: ReadRepository + WriteRepository {}

impl<T: ReadRepository + WriteRepository> ReadWriteRepository for T {}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tests::fixtures::fake_store::FakeStore;

    const fn implements_read_write_repository<T: ReadWriteRepository>() {}

    #[test]
    const fn should_implement_read_write_repository_when_using_fake_store() {
        implements_read_write_repository::<FakeStore<String>>();
    }
}
