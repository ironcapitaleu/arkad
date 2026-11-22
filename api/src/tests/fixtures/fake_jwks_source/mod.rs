//! # Fake JWKS Source
//!
//! Provides [`FakeJwksSource`], an in-memory [`JwksSource`] test double that serves the key sets a
//! test sets up, and counts the fetches.

use std::collections::VecDeque;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, Mutex, PoisonError};

use async_trait::async_trait;
use jsonwebtoken::jwk::JwkSet;

use crate::implementations::supabase_token_verifier::JwksSource;
use crate::traits::token_verifier::TokenError;

/// An in-memory [`JwksSource`] test double.
///
/// Each fetch takes the next answer of the list. The last answer repeats for every later fetch.
/// Built with [`FakeJwksSource::hanging_after`], it never answers after a number of fetches.
#[derive(Debug)]
pub struct FakeJwksSource {
    answers: Mutex<VecDeque<Result<JwkSet, TokenError>>>,
    fetches: Arc<AtomicUsize>,
    hang_after: Option<usize>,
}

impl FakeJwksSource {
    /// Creates a source that gives `answers` in order and then repeats the last one.
    ///
    /// # Panics
    ///
    /// Panics if `answers` is empty.
    #[must_use]
    pub fn answering(answers: Vec<Result<JwkSet, TokenError>>) -> Self {
        assert!(
            !answers.is_empty(),
            "A fake source needs at least one answer"
        );
        Self {
            answers: Mutex::new(answers.into()),
            fetches: Arc::new(AtomicUsize::new(0)),
            hang_after: None,
        }
    }

    /// Returns the source that never answers once it has answered `count` fetches.
    #[must_use]
    pub const fn hanging_after(mut self, count: usize) -> Self {
        self.hang_after = Some(count);
        self
    }

    /// Returns the counter of fetches, which stays readable after the source moves into a verifier.
    #[must_use]
    pub fn fetches(&self) -> Arc<AtomicUsize> {
        Arc::clone(&self.fetches)
    }
}

#[async_trait]
impl JwksSource for FakeJwksSource {
    async fn fetch(&self) -> Result<JwkSet, TokenError> {
        let answered = self.fetches.fetch_add(1, Ordering::SeqCst);
        if self.hang_after.is_some_and(|count| answered >= count) {
            return std::future::pending().await;
        }
        let mut answers = self.answers.lock().unwrap_or_else(PoisonError::into_inner);
        if answers.len() > 1 {
            answers
                .pop_front()
                .expect("Given more than one answer, taking the first should always succeed")
        } else {
            answers[0].clone()
        }
    }
}
