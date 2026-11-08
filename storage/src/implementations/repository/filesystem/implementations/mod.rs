//! # Implementations
//!
//! Provides the [`FilesystemRepositoryBuilder`](builder::FilesystemRepositoryBuilder), the file
//! layout constants, and the repository ports for [`FilesystemRepository`].
//!
//! ## Modules
//!
//! - [`builder`]: The [`FilesystemRepositoryBuilder`](builder::FilesystemRepositoryBuilder) and the
//!   markers for its missing steps.
//! - [`constants`]: The constants that fix the file layout, such as
//!   [`METADATA_SUFFIX`](constants::METADATA_SUFFIX).
//! - `key_walk`: The sorted walk over the directories under a
//!   [`KeyPrefix`](crate::implementations::repository::filesystem::KeyPrefix).
//! - `read_repository`: Implements [`ReadRepository`](crate::ReadRepository).
//! - `write_repository`: Implements [`WriteRepository`](crate::WriteRepository).
//!
//! [`FilesystemRepository`]: crate::implementations::repository::filesystem::FilesystemRepository

pub mod builder;
pub mod constants;
mod key_walk;
mod read_repository;
mod write_repository;
