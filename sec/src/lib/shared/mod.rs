//! # Shared Utilities and Domain Types
//!
//! This module contains shared utilities and domain-specific types used throughout the SEC state machine library.
//! These components provide reusable logic and strongly-typed representations for common SEC concepts, such as the Central Index Key (CIK).
//!
//! ## Modules
//! - [`accession_number`]: Provides [`AccessionNumber`](accession_number::AccessionNumber), the accession number of one SEC filing.
//! - [`build_info`]: Provides the version and the git commit of the build, and the [`Producer`](storage::implementations::repository::filesystem::Producer) of a stored document.
//! - [`cik`]: Provides parsing, validation, and formatting utilities for SEC Central Index Keys (CIKs).
//! - [`company_data`]: Provides [`CompanyData`](company_data::CompanyData), the as-is facts and the filings of one company.
//! - [`document_storage`]: Provides [`FailedDocumentStorage`](document_storage::FailedDocumentStorage), the error for a document that the store did not keep.
//! - [`edgar_exchange_code`]: Provides [`mic_of`](edgar_exchange_code::mic_of), the map from an EDGAR exchange code to the MIC of the exchange it names.
//! - [`entity_bucket`]: Provides [`entity_bucket`](entity_bucket::entity_bucket), the bucket of an entity in the rows of `candidates`, and the bucket count.
//! - [`filing_context`]: Provides [`FilingContext`](filing_context::FilingContext), the context of every state of the filing machine, and [`RegistryHandle`](filing_context::RegistryHandle), the taxonomy registry inside it.
//! - [`filing_documents`]: Provides [`FilingDocuments`](filing_documents::FilingDocuments), the XBRL instance and the label file of one filing, picked from its folder index.
//! - [`filing_documents_ref`]: Provides [`FilingDocumentsRef`](filing_documents_ref::FilingDocumentsRef), the filing and the keys of its stored XBRL instance and label file.
//! - [`filing_input`]: Provides [`FilingInput`](filing_input::FilingInput), the input of the filing machine for one Notes filing or one instance filing.
//! - [`filing_ref`]: Provides [`FilingRef`](filing_ref::FilingRef), which names one filing to fetch.
//! - [`form`]: Provides [`Form`](form::Form), the SEC form types a data point can originate from, and the forms both ingestion paths keep.
//! - [`form_index_row`]: Provides [`FormIndexRow`](form_index_row::FormIndexRow), one row of an EDGAR daily index or full index, and the parser of the whole index.
//! - [`http_client`]: Provides utilities for creating and managing SEC API clients, including user agent handling.
//! - [`index_date`]: Provides [`IndexDate`](index_date::IndexDate), the date of one EDGAR daily index, and the parts of its URL.
//! - [`notes_archive_ref`]: Provides [`NotesArchiveRef`](notes_archive_ref::NotesArchiveRef), the period, key, size, and SHA-256 digest of one stored Notes file.
//! - [`notes_period`]: Provides [`NotesPeriod`](notes_period::NotesPeriod), the period of one SEC Financial Statement and Notes file, and its file name.
//! - [`parsed_filing`]: Provides [`ParsedFiling`](parsed_filing::ParsedFiling), the as-is output of one filing from either parser.
//! - [`quarter`]: Provides [`Quarter`](quarter::Quarter), one calendar quarter of one year, which names the folder of an EDGAR daily index or full index.
//! - [`raw_document`]: Provides [`RawDocument`](raw_document::RawDocument), one SEC file as the store keeps it.
//! - [`raw_repository`]: Provides [`RawRepository`](raw_repository::RawRepository), the bound on a repository of raw documents that a state context can hold.
//! - [`raw_source`]: Provides [`RawSource`](raw_source::RawSource), the trait that names the SEC file a raw document holds, and one implementation per file.
//! - [`rate_limiter`]: Provides a leaky-bucket rate limiter for pacing outgoing SEC API requests.
//! - [`request`]: Provides utilities for constructing SEC API requests, ensuring proper URL formatting.
//! - [`response`]: Provides utilities for handling HTTP responses from SEC endpoints.
//! - [`row_digest`]: Provides [`content_digest`](row_digest::content_digest), the SHA-256 of the rows of a write unit, and [`ByteForm`](row_digest::ByteForm), the byte form of the values it hashes.
//! - [`run_fields`]: Provides [`RunFields`](run_fields::RunFields), the metadata fields that name the run that produced a stored document.
//! - [`source_counts`]: Provides [`SourceCounts`](source_counts::SourceCounts), the source rows of one filing and the rows the parse dropped, by reason.
//! - [`source_document`]: Provides [`SourceDocument`](source_document::SourceDocument), the raw document that the rows of one filing come from.
//! - [`standardized_filing`]: Provides [`StandardizedFiling`](standardized_filing::StandardizedFiling), the as-is output of one filing with its candidates and its `Conflict` findings.
//! - [`stored_form_index`]: Provides [`StoredFormIndex`](stored_form_index::StoredFormIndex), the key, the SHA-256, and the text of one stored daily index or full index.
//! - [`transform_version`]: Provides [`TRANSFORM_VERSION`](transform_version::TRANSFORM_VERSION), the version of the transform output that this build writes.
//! - [`user_agent`]: Provides utilities for creating and validating SEC-compliant user agent strings.
//!
//! ## Usage
//! The types and functions in this module are intended to be used by state implementations, error handling, and data validation routines across the library.
//! For example, CIK parsing and validation is leveraged by the `extract/validate_cik_format` state and its input/output data modules.
//!
//! ## See Also
//! - [`crate::implementations`]: Concrete state and state machine implementations that use these shared utilities.
//! - [`crate::error`]: Error types that may reference shared domain types for detailed error reporting.

pub mod accession_number;
pub mod build_info;
pub mod cik;
pub mod company_data;
pub mod content_type;
pub mod document_storage;
pub mod edgar_exchange_code;
pub mod entity_bucket;
pub mod filing_context;
pub mod filing_documents;
pub mod filing_documents_ref;
pub mod filing_input;
pub mod filing_ref;
pub mod form;
pub mod form_index_row;
pub mod headers;
pub mod http_client;
pub mod index_date;
pub mod notes_archive_ref;
pub mod notes_period;
pub mod parsed_filing;
pub mod quarter;
pub mod rate_limiter;
pub mod raw_document;
pub mod raw_repository;
pub mod raw_source;
pub mod request;
pub mod response;
pub mod row_digest;
pub mod run_fields;
pub mod source_counts;
pub mod source_document;
pub mod standardized_filing;
pub mod status_code;
pub mod stored_form_index;
pub mod transform_version;
pub mod url;
pub mod user_agent;
