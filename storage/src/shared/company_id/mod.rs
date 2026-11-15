//! # Company ID
//!
//! Provides [`CompanyId`], the internal key of a company. Every fact, identifier row, and claim of
//! a company keys on it. ADR 0017 (`docs/adr/0017-company-id-is-a-uuid.md`) records why the key is
//! a UUID.

use std::fmt::{self, Display, Formatter};

use uuid::Uuid;

/// The internal key of a company: an opaque UUID.
///
/// The load assigns a [`CompanyId`] once per company, and nothing reuses or changes it (ADR 0001,
/// "Design Rules" 4). The value carries no meaning: a CIK or an LEI is an identifier of the
/// company, not its key. To find the key of an identifier, ask an
/// [`EntityResolver`](crate::EntityResolver).
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct CompanyId(Uuid);

impl CompanyId {
    /// Wraps a UUID that the load assigned to a company, or that a store read back.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use storage::CompanyId;
    /// use uuid::Uuid;
    ///
    /// let uuid = Uuid::from_u128(0x0193_0000_0000_7000_8000_0000_0000_0001);
    ///
    /// let expected_result = uuid;
    ///
    /// let result = CompanyId::from_uuid(uuid).as_uuid();
    ///
    /// assert_eq!(result, expected_result);
    /// ```
    #[must_use]
    pub const fn from_uuid(uuid: Uuid) -> Self {
        Self(uuid)
    }

    /// Returns the UUID, for a store that writes the key.
    #[must_use]
    pub const fn as_uuid(&self) -> Uuid {
        self.0
    }
}

impl Display for CompanyId {
    /// Writes the UUID in its hyphenated lowercase form.
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[cfg(test)]
mod tests {
    use std::fmt::Debug;
    use std::hash::Hash;

    use pretty_assertions::assert_eq;

    use super::*;

    const fn implements_auto_traits<T: Sized + Send + Sync + Unpin>() {}
    #[test]
    const fn should_implement_auto_traits_for_company_id() {
        implements_auto_traits::<CompanyId>();
    }

    const fn implements_value_traits<
        T: Debug + Clone + Copy + PartialEq + Eq + PartialOrd + Ord + Hash,
    >() {
    }
    #[test]
    const fn should_implement_value_traits_for_company_id() {
        implements_value_traits::<CompanyId>();
    }

    #[test]
    fn should_return_the_wrapped_uuid_for_a_company_id() {
        let uuid = Uuid::from_u128(1);

        let expected_result = Uuid::from_u128(1);

        let result = CompanyId::from_uuid(uuid).as_uuid();

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_display_the_hyphenated_uuid_for_a_company_id() {
        let company_id = CompanyId::from_uuid(Uuid::from_u128(1));

        let expected_result = "00000000-0000-0000-0000-000000000001";

        let result = company_id.to_string();

        assert_eq!(result, expected_result);
    }
}
