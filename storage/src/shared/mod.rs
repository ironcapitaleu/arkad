//! # Shared
//!
//! Provides the types that the backends and the callers of the crate both use.
//!
//! ## Modules
//!
//! - [`account_id`]: The [`AccountId`], the key of an account, and the [`InvalidAccountId`] for a
//!   string that is not a UUID.
//! - [`account_plan`]: The [`AccountPlan`] an account is on, and the [`InvalidAccountPlan`] for a
//!   string that is not a plan.
//! - [`api_key_id`]: The [`ApiKeyId`], the key of one API key, and the [`InvalidApiKeyId`] for a
//!   string that is not a UUID.
//! - [`api_key_name`]: The [`ApiKeyName`], the label of one API key, and the
//!   [`InvalidApiKeyName`] for a string that is not a name.
//! - [`channel`]: The [`Channel`], the kind of credential of a `/v1` call, and the
//!   [`InvalidChannel`] for a string that is not a channel.
//! - [`company_id`]: The [`CompanyId`], the internal key of a company.
//! - [`company_identifier`]: The [`CompanyIdentifier`], a `(scheme, value)` pair that names a
//!   company, and its [`CompanyScheme`].
//! - [`key_environment`]: The [`KeyEnvironment`] an API key works in, and the
//!   [`InvalidKeyEnvironment`] for a string that is not an environment.
//! - [`listing_id`]: The [`ListingId`], the internal key of a listing.
//! - [`listing_identifier`]: The [`ListingIdentifier`], a `(scheme, value)` pair that names a
//!   listing, and its [`ListingScheme`].
//! - [`mic`]: The [`Mic`] of ISO 10383 that names an exchange, and the [`InvalidMic`] for a string
//!   that is not a MIC.
//! - [`sha256`]: The [`Sha256Digest`] of a byte slice, and the [`InvalidSha256Digest`] for a
//!   string that is not a digest.
//! - [`usage_key`]: The [`UsageKey`] of the call count of one account, channel, and UTC day, and
//!   the [`InvalidUsageKey`] for a string that is not a usage key.
//! - [`window`]: The [`Window`] that one quota limit counts over, and the [`InvalidWindow`] for a
//!   string that is not a window.

pub mod account_id;
pub mod account_plan;
pub mod api_key_id;
pub mod api_key_name;
pub mod channel;
pub mod company_id;
pub mod company_identifier;
pub mod key_environment;
pub mod listing_id;
pub mod listing_identifier;
pub mod mic;
pub mod sha256;
pub mod usage_key;
pub mod window;

pub use account_id::{AccountId, InvalidAccountId};
pub use account_plan::{AccountPlan, InvalidAccountPlan};
pub use api_key_id::{ApiKeyId, InvalidApiKeyId};
pub use api_key_name::{ApiKeyName, InvalidApiKeyName, MAX_API_KEY_NAME_CHARACTERS};
pub use channel::{Channel, InvalidChannel};
pub use company_id::CompanyId;
pub use company_identifier::{CompanyIdentifier, CompanyScheme};
pub use key_environment::{InvalidKeyEnvironment, KeyEnvironment};
pub use listing_id::ListingId;
pub use listing_identifier::{ListingIdentifier, ListingScheme};
pub use mic::{InvalidMic, Mic};
pub use sha256::{InvalidSha256Digest, Sha256Digest};
pub use usage_key::{InvalidUsageKey, InvalidUsageKeyReason, UsageKey};
pub use window::{InvalidWindow, Window};
