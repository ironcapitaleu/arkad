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

Fetches raw SEC Company Facts JSON from the EDGAR API and writes it to the store. A stream of the super-state ends after StoreCompanyFacts:

1. **ValidateCikFormat** -- Validates and normalizes CIK strings to 10-digit zero-padded format.
2. **PrepareSecRequest** -- Creates the HTTP client and builds the SEC API request.
3. **ExecuteSecRequest** -- Executes the HTTP request and returns the validated SEC response.
4. **StoreCompanyFacts** -- Writes the raw body of the SEC response to the store and returns the key of the stored document. It is the last state of the stream.

Build the super-state at its entry state with `ExtractSuperState::builder()`. Set every field by name, in any order. `build()` exists only once the builder holds all four fields, so a `build()` call with a missing field does not compile:

```rust,ignore
let extract_state = ExtractSuperState::builder()
    .cik("1067983")
    .sec_client(sec_client)
    .execution_id(Uuid::new_v4())
    .repository(repository)
    .build();
```

### Transform SuperState

Validates, resolves, and structures the raw JSON response into typed financial domain objects:

1. **ParseCompanyFacts** -- Validates that the SEC response contains all required XBRL concepts (Revenue, Net Income, Total Assets, etc.), resolves company-specific XBRL key aliases, and parses observations into strongly-typed `CompanyData` with full data lineage.
2. **CreateFinancialStatements** -- (Scaffold) Will fan out `CompanyData` to multiple threads to create Balance Sheet, Income Statement, Cash Flow Statement, and Company Info.

## Running

```sh
# Basic extraction demo
cargo run --bin extraction

# Extract pipeline for all S&P 500 CIKs. Stores each response under ARKAD_STORAGE_ROOT
cargo run --features tracing-logging --bin stream_etl

# Same pipeline under the legacy binary name; differs only in log field layout
cargo run --features tracing-logging --bin stream_extract
```

Each binary builds a document store at startup. `ARKAD_STORAGE_ROOT` sets its root directory. If the variable is unset or empty, the root is `data`, relative to the working directory. The binary creates the root and its missing parents at startup. If it cannot, it stops at startup with an `io::Error`.

Each binary also builds an `SecClient` at startup. `ARKAD_SEC_USER_AGENT` sets the `User-Agent` of every SEC request. The SEC requires the form `Company Name email@domain.com`, for example `ARKAD_SEC_USER_AGENT="Acme Corp ops@acme.com"`. If the variable is unset or empty, the client sends the default `UserAgent`, which is `Test Company contact@test.com`. The binary then warns at startup, because the SEC can throttle a placeholder `User-Agent`. If the value is not valid UTF-8 or does not have this form, the binary stops at startup with a `UserAgentError`. If the value holds a character that an HTTP header cannot carry, it stops with a `reqwest` error.

### Container Image

The `Dockerfile` at the repository root builds an image that runs `stream_extract`. Build it from the repository root:

```sh
docker build -t arkad-extract .
```

The image sets no storage root, so the root is `/data` inside the container, and the documents go away with it. To keep them, mount a volume and point `ARKAD_STORAGE_ROOT` at it:

```sh
docker run --rm \
  -e ARKAD_SEC_USER_AGENT="Acme Corp ops@acme.com" \
  -e ARKAD_STORAGE_ROOT=/mnt/arkad \
  -v "$PWD/data:/mnt/arkad" \
  arkad-extract
```

The build reads the commit from `.git`, so `.dockerignore` and `.gcloudignore` keep it in the build context.
