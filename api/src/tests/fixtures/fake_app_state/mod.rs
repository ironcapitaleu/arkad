//! # Fake App State
//!
//! Provides [`fake_app_state`], an [`AppState`] for tests whose routes check no login token.

use std::sync::Arc;

use crate::routes::AppState;
use crate::tests::fixtures::fake_token_verifier::FakeTokenVerifier;

/// Returns an [`AppState`] with a default [`FakeTokenVerifier`], which rejects every token.
#[must_use]
pub fn fake_app_state() -> AppState {
    AppState::new(Arc::new(FakeTokenVerifier::default()))
}
