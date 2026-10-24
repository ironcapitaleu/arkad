//! # Traits
//!
//! Provides the traits that set what a [`FilesystemRepository`] can do.
//!
//! ## Modules
//!
//! - [`access`]: The access markers and the traits that connect them to the ports.
//! - [`document`]: The [`Document`](document::Document) trait every stored document kind
//!   implements.
//!
//! [`FilesystemRepository`]: crate::implementations::repository::filesystem::FilesystemRepository

pub mod access;
pub mod document;
