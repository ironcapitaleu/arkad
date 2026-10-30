//! # Shared Utilities and Domain Types
//!
//! This module contains shared utilities and domain-specific types used throughout the SEC state machine library.
//! These components provide reusable logic and strongly-typed representations for common SEC concepts, such as the Central Index Key (CIK).
//!
//! ## Modules
//! - [`build_info`]: Provides the version and the git commit of the build, and the [`Producer`](storage::implementations::repository::filesystem::Producer) of a stored document.
//! - [`cik`]: Provides parsing, validation, and formatting utilities for SEC Central Index Keys (CIKs).
//! - [`daily_index_row`]: Provides [`DailyIndexRow`](daily_index_row::DailyIndexRow), one row of an EDGAR daily index, and the parser of the whole index.
//! - [`document_type`]: Provides [`SecDocumentType`](document_type::SecDocumentType), the types of document the SEC state machines store.
//! - [`document_storage`]: Provides [`FailedDocumentStorage`](document_storage::FailedDocumentStorage), the error for a document that the store did not keep.
//! - [`filing_ref`]: Provides [`FilingRef`](filing_ref::FilingRef), which names one filing to fetch.
//! - [`http_client`]: Provides utilities for creating and managing SEC API clients, including user agent handling.
//! - [`index_date`]: Provides [`IndexDate`](index_date::IndexDate), the date of one EDGAR daily index, and the parts of its URL.
//! - [`notes_archive_ref`]: Provides [`NotesArchiveRef`](notes_archive_ref::NotesArchiveRef), the period, key, size, and SHA-256 digest of one stored Notes file.
//! - [`notes_period`]: Provides [`NotesPeriod`](notes_period::NotesPeriod), the period of one SEC Financial Statement and Notes file, and its file name.
//! - [`raw_document`]: Provides [`RawDocument`](raw_document::RawDocument), one SEC file as the store keeps it.
//! - [`raw_repository`]: Provides [`RawRepository`](raw_repository::RawRepository), the bound on a repository of raw documents that a state context can hold.
//! - [`raw_source`]: Provides [`RawSource`](raw_source::RawSource), the trait that names the SEC file a raw document holds, and one implementation per file.
//! - [`rate_limiter`]: Provides a leaky-bucket rate limiter for pacing outgoing SEC API requests.
//! - [`request`]: Provides utilities for constructing SEC API requests, ensuring proper URL formatting.
//! - [`response`]: Provides utilities for handling HTTP responses from SEC endpoints.
//! - [`run_fields`]: Provides [`RunFields`](run_fields::RunFields), the metadata fields that name the run that produced a stored document.
//! - [`user_agent`]: Provides utilities for creating and validating SEC-compliant user agent strings.
//!
//! ## Usage
//! The types and functions in this module are intended to be used by state implementations, error handling, and data validation routines across the library.
//! For example, CIK parsing and validation is leveraged by the `extract/validate_cik_format` state and its input/output data modules.
//!
//! ## See Also
//! - [`crate::implementations`]: Concrete state and state machine implementations that use these shared utilities.
//! - [`crate::error`]: Error types that may reference shared domain types for detailed error reporting.

pub mod build_info;
pub mod cik;
pub mod content_type;
pub mod daily_index_row;
pub mod document_storage;
pub mod document_type;
pub mod filing_ref;
pub mod financial;
pub mod headers;
pub mod http_client;
pub mod index_date;
pub mod notes_archive_ref;
pub mod notes_period;
pub mod rate_limiter;
pub mod raw_document;
pub mod raw_repository;
pub mod raw_source;
pub mod request;
pub mod response;
pub mod run_fields;
pub mod status_code;
pub mod url;
pub mod user_agent;
