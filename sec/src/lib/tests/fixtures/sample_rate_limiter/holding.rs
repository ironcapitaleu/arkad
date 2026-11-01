use std::cmp::Ordering as CmpOrdering;
use std::hash::{Hash, Hasher};
use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};

use async_trait::async_trait;

use crate::shared::rate_limiter::traits::RateLimiter;

/// How a [`HoldingRateLimiter`] holds a caller.
#[derive(Debug, Clone, Copy)]
enum Hold {
    /// The first call yields to the executor this many times. Every later call returns at once.
    FirstTurnYields(usize),
    /// Every call waits forever.
    Forever,
}

/// A fake rate limiter that counts each call to `await_turn` and holds some callers.
///
/// Used for testing the order and the number of callers that run at once. Clones share one
/// counter.
#[derive(Debug, Clone)]
pub struct HoldingRateLimiter {
    /// The number of calls to `await_turn`, shared by all clones.
    pub turns: Arc<AtomicUsize>,
    hold: Hold,
}

impl HoldingRateLimiter {
    /// Creates a [`HoldingRateLimiter`] whose first caller yields `yields` times, so it finishes
    /// after the callers that start after it.
    #[must_use]
    pub fn first_turn_yields(yields: usize) -> Self {
        Self {
            turns: Arc::default(),
            hold: Hold::FirstTurnYields(yields),
        }
    }

    /// Creates a [`HoldingRateLimiter`] whose callers wait forever.
    #[must_use]
    pub fn forever() -> Self {
        Self {
            turns: Arc::default(),
            hold: Hold::Forever,
        }
    }
}

#[async_trait]
impl RateLimiter for HoldingRateLimiter {
    async fn await_turn(&self) {
        let turn = self.turns.fetch_add(1, Ordering::SeqCst);
        match self.hold {
            Hold::FirstTurnYields(yields) if turn == 0 => {
                for _ in 0..yields {
                    tokio::task::yield_now().await;
                }
            }
            Hold::FirstTurnYields(_) => {}
            Hold::Forever => std::future::pending().await,
        }
    }
}

// Deviation: a client needs `Eq`, `Ord`, and `Hash` to implement `SecClient`. The count changes
// through `&self`, so every limiter compares equal and adds nothing to a hash. Same rule as
// `CountingRateLimiter`.

impl PartialEq for HoldingRateLimiter {
    fn eq(&self, _other: &Self) -> bool {
        true
    }
}

impl Eq for HoldingRateLimiter {}

impl PartialOrd for HoldingRateLimiter {
    fn partial_cmp(&self, other: &Self) -> Option<CmpOrdering> {
        Some(self.cmp(other))
    }
}

impl Ord for HoldingRateLimiter {
    fn cmp(&self, _other: &Self) -> CmpOrdering {
        CmpOrdering::Equal
    }
}

impl Hash for HoldingRateLimiter {
    fn hash<H: Hasher>(&self, _state: &mut H) {}
}
