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
/// Both parent traits have a `Record`. [`ReadRepository::Record`] is what `get` returns.
/// [`WriteRepository::Record`] is what `persist` takes. They are two separate slots that share a
/// name, and nothing forces them to hold the same type.
///
/// `ReadWriteRepository` has no `Record` of its own. It has the two it inherits from its parents.
/// That makes the difference between these three bounds:
///
/// - `S: ReadWriteRepository` compiles. It sets no record type, so there is nothing to
///   disambiguate.
/// - `S: ReadWriteRepository<Record = String>` does not compile. The compiler cannot tell which
///   of the two `Record`s that bound sets, and reports E0222.
/// - `S: ReadRepository<Record = String, Key = String> + WriteRepository<Record = String>`
///   compiles. Each `Record` is set on the trait that declares it.
///
/// Naming the type is separate from setting it. `S::Record` is ambiguous wherever both parents
/// are in scope, and reports E0221. That holds even where both are set to the same type. Write
/// `<S as ReadRepository>::Record` or `<S as WriteRepository>::Record`.
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
