# SEC

A modular state machine library for processing SEC filings in Rust. It provides type-safe, testable workflows for extracting data from the SEC EDGAR API, transforming it into structured financial domain types, and (eventually) loading it into storage.

## Pipeline Overview

The SEC ETL pipeline is composed of hierarchical SuperStates, each containing multiple inner states:

```text
Extract                          Transform                        Load (planned)
  ValidateCikFormat                ParseCompanyFacts                StoreData
  PrepareSecRequest                CreateFinancialStatements
  ExecuteSecRequest
  StoreCompanyFacts
```

### Extract SuperState

Fetches raw SEC Company Facts JSON from the EDGAR API and writes it to the store:

1. **ValidateCikFormat** -- Validates and normalizes CIK strings to 10-digit zero-padded format.
2. **PrepareSecRequest** -- Creates the HTTP client and builds the SEC API request.
3. **ExecuteSecRequest** -- Executes the HTTP request and returns the validated SEC response.
4. **StoreCompanyFacts** -- Writes the raw body of the SEC response to the store and returns the key of the stored document. A transition leads into this state from ExecuteSecRequest, but the pipeline stream does not run it.

### Transform SuperState

Validates, resolves, and structures the raw JSON response into typed financial domain objects:

1. **ParseCompanyFacts** -- Validates that the SEC response contains all required XBRL concepts (Revenue, Net Income, Total Assets, etc.), resolves company-specific XBRL key aliases, and parses observations into strongly-typed `CompanyData` with full data lineage.
2. **CreateFinancialStatements** -- (Scaffold) Will fan out `CompanyData` to multiple threads to create Balance Sheet, Income Statement, Cash Flow Statement, and Company Info.

## Running

```sh
# Basic extraction demo
cargo run --bin extraction

# Full ETL pipeline (Extract + Transform) for all S&P 500 CIKs
cargo run --features tracing-logging --bin stream_etl

# Same pipeline under the legacy binary name; differs only in log field layout
cargo run --features tracing-logging --bin stream_extract
```

Each binary builds a document store at startup. `ARKAD_STORAGE_ROOT` sets its root directory. If the variable is unset or empty, the root is `data`, relative to the working directory.
