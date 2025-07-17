//! # Configuration Management Module
//!
//! This module provides configuration management following 12-factor app principles.
//! It loads configuration from environment variables that are injected into the application
//! environment at runtime.
//!
//! ## Modules
//!
//! - [`error`]: Configuration error types and handling
//! - [`service`]: Configuration traits for service-specific configurations

pub mod error;
pub mod service;
