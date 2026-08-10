# Arkad

Arkad is a production-grade financial data engineering framework written in Rust, built on a hierarchical finite state machine (HFSM) foundation. It provides invariant-based, type-safe, and testable ETL pipelines for processing financial data — starting with SEC filings — with predictable execution semantics, structured error handling, and support for asynchronous and parallel execution.

## Architecture

Arkad is structured as a Rust workspace with three crates:

### `state_maschine`

A general-purpose HFSM library providing the core trait abstractions and execution model that all pipelines are built on. Designed to be extended for domain-specific use cases without modification to the core.

### `sec`

Extends `state_maschine` for processing SEC filings — handling data acquisition, validation, transformation, and storage in a structured, type-safe pipeline.

### `xbrl`

The XBRL domain vocabulary: parsing the SEC JSON APIs, resolving concepts against the US-GAAP taxonomy, and validating financial statements against SFAC 6 invariants.

## Design

### ETL Pipeline as a Hierarchical State Machine

The SEC processing pipeline is modelled as a three-stage HFSM: `Extract`, `Transform`, and `Load`. Each super-state encapsulates its own internal states and transitions, enforcing clean separation of concerns and deterministic execution.

```mermaid
---
title: "SEC Data Processing Pipeline: State Machine Overview"
---
stateDiagram-v2
    direction LR
    [*] --> Extract
    state Extract {
        ValidateCikFormat --> PrepareSecRequest
        PrepareSecRequest --> ExecuteSecRequest
    }
    Extract --> Transform
    state Transform {
        ParseCompanyFacts --> CreateFinancialStatements
    }
    Transform --> Load
    state Load {
        StoreData
    }
    Load --> [*]
```

### Trait Hierarchy

Each pipeline state implements a layered trait system that enforces correctness at compile time. The SEC-specific traits (`State`, `StateData`, `Context`, `StateMachine`, `Transition`) extend the `SM`-prefixed base abstractions from `state_maschine`, enabling reuse of the execution engine while allowing full domain customisation.

```mermaid
---
title: "Sample `SEC` State: `SampleSecState`"
---
classDiagram
    class StateMachine~S: State~{
        <<trait>>
        %% SEC-specific StateMachine trait: `SMStateMachine<S> + Display`
    }

    class SMStateMachine~S: SMState~{
        <<trait>>
        %% Base StateMachine trait from `state_maschine`

        +current_state(&self) &S
        +current_state_mut(&mut self) &mut S
        +run(&mut self)
        +advance_state(&mut self)
    }

    class SuperState~S: State~ {
        <<trait>>
        %% SEC-specific SuperState trait: `State + StateMachine<S>`
    }

    class SMSuperState~S: SMState~ {
        <<trait>>
        %% Base SuperState trait from `state_maschine`
    }

    class SMState {
        <<trait>>
        %% Base State trait from `state_maschine`
        +type InputData: SMStateData
        +type OutputData: SMStateData
        +type Context: SMContext
        +state_name(&self) impl ToString
        +input_data(&self) &Self::InputData
        +compute_output_data(&mut self)
        +output_data(&self) Option~&Self::OutputData~
        +has_output_data_been_computed(&self) bool
        +context_data(&self) &Self::Context
    }

    class State {
        <<trait>>
        %% SEC-specific State trait: `SMState + Display`, async via `#[async_trait]`
        +compute_output_data_async(&mut self) async Result~(), StateError~
    }

    class SMStateData {
        <<trait>>
        %% Base StateData trait from `state_maschine`
        +type UpdateType
        +state(&self) &Self
        +update_state(&mut self, updates: Self::UpdateType)
    }

    class StateData {
        <<trait>>
        %% SEC-specific StateData trait
        +update_state(&mut self, updates: Self::UpdateType) Result~(), StateError~
    }

    class SMContext {
        <<trait>>
        %% Base Context trait from `state_maschine`
        +type UpdateType
        +context(&self) &Self
        +update_context(&mut self, updates: Self::UpdateType)
    }

    class Context {
        <<trait>>
        %% SEC-specific Context trait
        +can_retry(&self) bool
        +max_retries(&self) u32
    }

    class SMTransition~T, U~ {
        <<trait>>
        %% Base Transition trait from `state_maschine`: `StateMachine<T>`
        +type NewStateMachine: SMStateMachine~U~
        +transition_to_next_state(self) Result~Self::NewStateMachine, &'static str~
    }

    class Transition~T, U~ {
        <<trait>>
        %% SEC-specific Transition trait
        +transition_to_next_state_sec(self) Result~Self::NewStateMachine, TransitionError~
    }

    class NonTerminal {
        <<trait>>
        %% Marks a state machine as sitting at a non-terminal state
        +type Current: State + Serialize
        +type Next: State
    }

    class IntoStateMachineStream {
        <<trait>>
        %% Drives a state machine to completion as an async event stream
        +into_stream(self, execution_id: Uuid) StateMachineStream
    }

    class StateError {
        <<enum>>
        %% SEC-specific state errors, see `sec_error_handling.md`
        +InvalidCikFormat(InvalidCikFormat)
        +FailedRequestExecution(FailedRequestExecution)
        +IncompleteCompanyFacts(IncompleteCompanyFacts)
        +InvalidInput
        +InvalidContext
        +FailedOutputComputation
        +StateDataUpdateFailed
        +ContextUpdateFailed
    }

    class TransitionError {
        <<enum>>
        %% SEC-specific transition errors, see `sec_error_handling.md`
        +MissingOutput(MissingOutput)
        +FailedOutputConversion(FailedOutputConversion)
        +FailedContextConversion(FailedContextConversion)
    }

    class SampleSecState {
        <<struct>>
        %% A sample SEC State implementation, represents any 'SecState'
        -input: SampleSecStateInput
        -context: SampleSecStateContext
        -output: Option~SampleSecStateOutput~
        +new(input, context) Self
    }

    class SampleSecStateInput {
        <<struct>>
        %% Input for SampleSecState
        +input_data: String
    }

    class SampleSecStateOutput {
        <<struct>>
        %% Output for SampleSecState
        +output_data: String
    }

    class SampleSecStateContext {
        <<struct>>
        %% Context for SampleSecState
        +data: String
        +max_retries: u32
    }

    %% is-relationships
    StateMachine --> State : "is in a"
    SuperState --> StateMachine : "is a"
    SuperState --> State : "is a"

    %% SEC-specific trait inheritance
    StateMachine --> SMStateMachine : "extends"
    SuperState --> SMSuperState : "extends"
    State --> SMState : "extends"
    StateData --> SMStateData : "extends"
    Context --> SMContext : "extends"
    Transition --> SMTransition : "extends"

    %% Streaming
    NonTerminal --> State : "declares Current/Next as"
    IntoStateMachineStream --> NonTerminal : "blanket impl for"
    IntoStateMachineStream --> Transition : "blanket impl requires"

    %% Trait implementations
    SampleSecState --> State : "implements"
    SampleSecStateInput --> StateData : "implements"
    SampleSecStateOutput --> StateData : "implements"
    SampleSecStateContext --> Context : "implements"

    %% Struct relationships
    SampleSecState --> SampleSecStateInput : "has"
    SampleSecState --> SampleSecStateOutput : "has"
    SampleSecState --> SampleSecStateContext : "has"

    %% Error relationships
    State --> StateError : "can return"
    StateData --> StateError : "can return"
    Transition --> TransitionError : "can return"
```

### Error Type Hierarchy

Errors are modelled as a structured hierarchy — from top-level `ErrorKind` down through `StateMachine`, `State`, and `Transition` variants — with each layer wrapping strongly-typed domain errors. This makes all failure modes explicit, exhaustively matchable, and traceable to their origin.

```mermaid
---
title: "`sec` Error Type Hierarchy"
---
classDiagram
    class ErrorKind{
        <<enum>>
        %% Top-level error enum for all SEC state machine errors
        +StateMachine(StateMachine)
        +DowncastNotPossible
    }

    class StateMachine{
        <<enum>>
        %% Errors during state machine execution
        +InvalidConfiguration
        +State(State)
        +Transition(Transition)
    }

    class State {
        <<enum>>
        %% Errors from internal state operations
        +InvalidCikFormat(InvalidCikFormat)
        +FailedRequestExecution(FailedRequestExecution)
        +IncompleteCompanyFacts(IncompleteCompanyFacts)
        +InvalidInput
        +InvalidContext
        +FailedOutputComputation
        +StateDataUpdateFailed
        +ContextUpdateFailed
    }

    class Transition {
        <<enum>>
        %% Errors during state transitions
        +MissingOutput(MissingOutput)
        +FailedOutputConversion(FailedOutputConversion)
        +FailedContextConversion(FailedContextConversion)
    }

    class InvalidCikFormat{
        <<struct>>
        %% State-level wrapper for CIK validation errors
        +String state_name
        +CikError cik_error
    }

    class FailedRequestExecution{
        <<struct>>
        %% State-level wrapper for request execution errors
        +String state_name
        +FailedSecRequest domain_error
    }

    class IncompleteCompanyFacts{
        <<struct>>
        %% State-level error for missing XBRL concepts
        -String state_name
        -MissingFields missing_fields
    }

    class MissingFields{
        <<struct>>
        %% Newtype over the canonical names of the missing concepts
        -Vec~String~ fields
    }

    class MissingOutput{
        <<struct>>
        %% Transition-level error for missing output data
        +String source_state_name
        +String target_state_name
    }

    class FailedOutputConversion{
        <<struct>>
        %% Transition-level error for output-to-input conversion failure
        +String source_state_name
        +String target_state_name
    }

    class FailedContextConversion{
        <<struct>>
        %% Transition-level error for context conversion failure
        +String source_state_name
        +String target_state_name
    }

    class CikError{
        <<struct>>
        %% Domain error for invalid CIK format
        +InvalidCikReason reason
        +String invalid_cik
    }

    class InvalidCikReason{
        <<enum>>
        %% Why a string failed CIK validation
        +MaxLengthExceeded(usize cik_length)
        +ContainsNonNumericCharacters
    }

    class FailedSecRequest{
        <<struct>>
        %% Domain error for SEC request execution
        +SecClientErrorReason reason
    }

    class SecClientErrorReason{
        <<enum>>
        %% `sec_client::error::ErrorReason`, renamed here for diagram uniqueness
        +FailedRequestExecution(String details)
        +InvalidResponse(InvalidSecResponse source)
    }

    class InvalidSecResponse{
        <<struct>>
        %% Domain error for a response that failed SEC validation
        +SecResponseErrorReason reason
    }

    class SecResponseErrorReason{
        <<enum>>
        %% `sec_response::error::ErrorReason`, renamed here for diagram uniqueness
        +InvalidStatusCode(StatusCode status_code)
        +InvalidContentType(ContentType content_type)
        +InvalidBody(String details)
        +FailedBodyRead(String details)
    }

    %% Error hierarchy relationships
    ErrorKind <|-- StateMachine
    StateMachine <|-- State
    StateMachine <|-- Transition

    %% State error wraps domain errors
    State <|-- InvalidCikFormat
    State <|-- FailedRequestExecution
    State <|-- IncompleteCompanyFacts

    %% Transition error wraps specific errors
    Transition <|-- MissingOutput
    Transition <|-- FailedOutputConversion
    Transition <|-- FailedContextConversion

    %% State wrappers contain domain errors
    InvalidCikFormat --> CikError
    CikError --> InvalidCikReason
    IncompleteCompanyFacts --> MissingFields
    FailedRequestExecution --> FailedSecRequest
    FailedSecRequest --> SecClientErrorReason
    SecClientErrorReason --> InvalidSecResponse
    InvalidSecResponse --> SecResponseErrorReason
```

## Quality & Reliability

- **1,000+ unit tests** covering state transitions, input validation, error paths, and edge cases
- **Invariant-based validation** at every state boundary to prevent downstream data corruption
- **Async and parallel execution** via Tokio, preserving pipeline correctness and reproducibility
- **First-class CI** via GitHub Actions with automated testing, linting, and formatting checks
- **Devcontainer support** for reproducible development environments

## Getting Started

Make sure Rust is installed:

```bash
cargo --version
```

If you get a `command not found` error, install the Rust toolchain via [rustup](https://rustup.rs/) or your distro's package manager.

Clone the repository:

```bash
git clone https://github.com/ironcapitaleu/arkad.git
cd arkad
```

Run the full ETL pipeline (Extract + Transform) with structured JSON logging:

```bash
# All S&P 500 CIKs (paced by a rate limiter)
cargo run --features tracing-logging --bin stream_etl
```

## Contributing

See [CONTRIBUTING.md](.github/CONTRIBUTING.md) for guidelines. All contributions are welcome.
