//! # Configuration Management Module
//!
//! This module provides configuration management.
//! It loads configuration from environment variables that are injected into the application
//! environment at runtime.
//!
//! ## Modules
//! - [`traits`]: Core configuration traits for extensibility, integration, and service abstraction.
//! - [`implementations`]: Concrete configuration loaders and managers for environment, file, and runtime sources.
//! - [`shared`]: Shared domain types and utilities used across configuration logic.
//! - [`error`]: Configuration error types and handling for loading, parsing, and validation, with comprehensive error kinds and conversions.

pub mod error;

pub mod traits;

pub mod implementations;

pub mod shared;
