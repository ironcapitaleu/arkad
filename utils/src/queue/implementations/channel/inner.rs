//! Implementations of the [`InnerChannel`] trait for custom and foreign types.
//!
//! This module provides [`InnerChannel`] trait implementations for types defined outside this crate (foreign types),
//! such as [`lapin::Channel`]. These implementations enable the use of external channel types with the [`ChannelBuilder`](crate::queue::implementations::channel::ChannelBuilder)
//! pattern, allowing seamless integration and construction of channels within this library's abstractions.
//!
//! By centralizing these implementations here, we ensure that all supported custom and foreign channel types can be used
//! transparently with the builder API, promoting extensibility and maintainability.
use lapin::Channel as LapinChannel;

use crate::queue::traits::InnerChannel;

impl InnerChannel for LapinChannel {
    fn serve(&self) {
        println!("LapinChannel is serving.");
    }
}
