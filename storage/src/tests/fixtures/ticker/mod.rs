//! # Ticker
//!
//! Provides [`count_ticks_until`], a future that counts how often the task polls it, and
//! [`LARGE_BODY_SIZE`], a body size whose hash takes many polls of the ticker.

use std::sync::atomic::{AtomicBool, Ordering};

/// The size in bytes of a body whose hash takes long enough for the ticker to tick many times.
pub const LARGE_BODY_SIZE: usize = 8 * 1024 * 1024;

/// Yields to the task until `is_done` is `true`, and returns the number of times it ran.
///
/// Joined with a future on one task, it ticks only while that future waits. A future that blocks
/// the task to its end lets it tick at most once.
pub async fn count_ticks_until(is_done: &AtomicBool) -> u64 {
    let mut ticks = 0;
    while !is_done.load(Ordering::Relaxed) {
        ticks += 1;
        tokio::task::yield_now().await;
    }
    ticks
}
