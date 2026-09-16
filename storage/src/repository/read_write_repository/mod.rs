//! # Read Write Repository
//!
//! Provides [`ReadWriteRepository`], the one name for a store that both reads and writes.

use crate::repository::read_repository::ReadRepository;
use crate::repository::write_repository::WriteRepository;

/// Names a store that both reads and writes.
///
/// Adds no method of its own. A blanket implementation grants it to every type that implements
/// both [`ReadRepository`] and [`WriteRepository`], so a backend writes no implementation for it.
///
/// # Setting the Record Type
///
/// `ReadWriteRepository<Record = String>` does not compile. Both parent traits declare a
/// `Record`, so the compiler cannot tell which one that bound means. Bind the parents instead:
/// `S: ReadRepository<Record = String, Key = String> + WriteRepository<Record = String>`.
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
    use crate::tests::fixtures::fake_read_write_repository::FakeReadWriteRepository;

    const fn implements_read_write_repository<T: ReadWriteRepository>() {}

    #[test]
    const fn should_implement_read_write_repository_when_using_fake_read_write_repository() {
        implements_read_write_repository::<FakeReadWriteRepository<String>>();
    }
}
