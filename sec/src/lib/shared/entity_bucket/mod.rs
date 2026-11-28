//! # Entity Bucket
//!
//! Provides [`entity_bucket`], the bucket of an entity in the rows of `candidates`, and
//! [`ENTITY_BUCKET_COUNT`], the number of buckets.
//!
//! A series task reads the rows of its buckets only. So the bucket of an entity must not change
//! from run to run, and every writer of `candidates` computes it with this function.

use twox_hash::XxHash3_64;
use xbrl::core::filing::EntityId;

/// The number of entity buckets. Each bucket is a number from 0 to 63.
pub const ENTITY_BUCKET_COUNT: u8 = 64;

/// The seed of the hash of an entity.
const SEED: u64 = 0;

/// Returns the bucket of `entity`: XXH3-64 with seed 0 over the UTF-8 bytes of the scheme, `:`,
/// and the ID, modulo [`ENTITY_BUCKET_COUNT`].
///
/// # Examples
///
/// ```rust
/// use sec::shared::entity_bucket::{ENTITY_BUCKET_COUNT, entity_bucket};
/// use xbrl::core::filing::EntityId;
///
/// let entity = EntityId::new("sec", "320193").expect(
///     "Given a hardcoded scheme and CIK, the creation of an entity ID should always succeed",
/// );
///
/// let expected_result = true;
///
/// let result = entity_bucket(&entity) < ENTITY_BUCKET_COUNT;
///
/// assert_eq!(result, expected_result);
/// ```
#[must_use]
pub fn entity_bucket(entity: &EntityId) -> u8 {
    let text = format!("{}:{}", entity.scheme(), entity.id());
    let hash = XxHash3_64::oneshot_with_seed(SEED, text.as_bytes());
    // 64 divides 256, so the lowest byte of the hash has the same remainder as the whole hash.
    hash.to_le_bytes()[0] % ENTITY_BUCKET_COUNT
}

#[cfg(test)]
mod tests {
    use pretty_assertions::assert_eq;

    use super::*;

    fn entity(scheme: &str, id: &str) -> EntityId {
        EntityId::new(scheme, id).expect(
            "Given a hardcoded scheme and ID, the creation of an entity ID should always succeed",
        )
    }

    // The expected buckets come from the reference implementation of XXH3-64 (the `xxhash`
    // package of Python): `sec:320193` hashes to 2757253951275582447, and `sec:1048911` to
    // 12058489201547411460.

    #[test]
    fn should_return_bucket_47_for_the_entity_sec_320193() {
        let apple = entity("sec", "320193");

        let expected_result = 47;

        let result = entity_bucket(&apple);

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_return_bucket_4_for_the_entity_of_the_fixture_filing() {
        let fixture_entity = entity("sec", "1048911");

        let expected_result = 4;

        let result = entity_bucket(&fixture_entity);

        assert_eq!(result, expected_result);
    }
}
