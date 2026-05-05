# Arkad

Arkad is a production-grade financial data engineering framework written in Rust, built on a hierarchical finite state machine (HFSM) foundation. It provides invariant-based, type-safe, and testable ETL pipelines for processing financial data — starting with SEC filings — with predictable execution semantics, structured error handling, and support for asynchronous and parallel execution.

## Architecture

Arkad is structured as a Rust workspace with two crates:

### `state_maschine`

A general-purpose HFSM library providing the core trait abstractions and execution model that all pipelines are built on. Designed to be extended for domain-specific use cases without modification to the core.

### `sec`

Extends `state_maschine` for processing SEC filings — handling data acquisition, validation, transformation, and storage in a structured, type-safe pipeline.

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

Each pipeline state implements a layered trait system that enforces correctness at compile time. The SEC-specific `State`, `StateData`, and `Context` traits extend base abstractions from `state_maschine`, enabling reuse of the execution engine while allowing full domain customisation.

```mermaid
---
title: "Sample `SEC` State: `SampleState`"
---
classDiagram
    class StateMachine~S: State~{
        <<trait>>
    }
    class SMStateMachine~S: SMState~{
        <<trait>>
        +get_current_state(&self) &S
        +get_current_state_mut(&mut self) &mut S
        +advance_state(&mut self)
        +run(&mut self)
    }
    class SuperState~S: State~ {
        <<trait>>
    }   
    class SMSuperState~S: SMState~ {
        <<trait>>
    }   
    class SMState {
        <<trait>>
        +type InputData: SMStateData
        +type OutputData: SMStateData
        +type Context: SMContext
        +get_state_name(&self) impl ToString
        +get_input_data(&self) &Self::InputData
        +compute_output_data(&mut self)
        +get_output_data(&self) Option~&Self::OutputData~
        +get_context_data(&self) &Self::Context
    }
    class State {
        <<trait>>
        +compute_output_data(&mut self, Result~(), StateError~)
    }
    class SMStateData {
        <<trait>>
        +type UpdateType
        +get_state(&self) &Self
        +update_state(&mut self, updates: Self::UpdateType)
    }
    class StateData {
        <<trait>>
        +update_state(&mut self, updates: Self::UpdateType) Result~(), StateError~
    }
    class SMContext {
        <<trait>>
        +type UpdateType
        +get_context(&self) &Self
        +update_context(&mut self, updates: Self::UpdateType)
    }
    class Context {
        <<trait>>
        +can_retry(&self) bool
        +get_max_retries(&self) u32
    }
    class SampleState {
        <<struct>>
        -input: SampleStateInput
        -context: SampleStateContext
        -output: Option~SampleStateOutput~
        +new(input, context) Self
    }
    class SampleStateInput {
        <<struct>>
        +input_data: String
    }
    class SampleStateOutput {
        <<struct>>
        +output_data: String
    }
    class SampleStateContext {
        <<struct>>
        +context_data: String
        +max_retries: u32
    }
    StateMachine --> State : "is in a"
    SuperState --> StateMachine : "is a"
    SuperState --> State : "is a"
    StateMachine --> SMStateMachine : "extends"
    SuperState --> SMSuperState : "extends"
    State --> SMState : "extends"
    StateData --> SMStateData : "extends"
    Context --> SMContext : "extends"
    SampleState --> State : "implements"
    SampleStateInput --> StateData : "implements"
    SampleStateOutput --> StateData : "implements"
    SampleStateContext --> Context : "implements"
    SampleState --> SampleStateInput : "has"
    SampleState --> SampleStateOutput : "has"
    SampleState --> SampleStateContext : "has"
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
        +StateMachine(StateMachine)
        +DowncastNotPossible
    }
    class StateMachine{
        <<enum>>
        +State(State)
        +Transition(Transition)
        +InvalidConfiguration
    }
    class State {
        <<enum>>
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
        +MissingOutput(MissingOutput)
        +FailedOutputConversion
        +FailedContextConversion
    }
    class InvalidCikFormat{
        <<struct>>
        +String state_name
        +CikError domain_error
    }
    class FailedRequestExecution{
        <<struct>>
        +String state_name
        +SecRequestError domain_error
    }
    class IncompleteCompanyFacts{
        <<struct>>
        +String state_name
        +Vec~String~ missing_fields
    }
    class MissingOutput{
        <<struct>>
        +String super_state_name
        +String target_state_name
    }
    class CikError{
        <<struct>>
        +InvalidCikReason reason
        +String invalid_cik
    }
    class SecRequestError{
        <<struct>>
        +SecRequestErrorReason reason
    }
    ErrorKind <|-- StateMachine
    StateMachine <|-- State
    StateMachine <|-- Transition
    State <|-- InvalidCikFormat
    State <|-- FailedRequestExecution
    State <|-- IncompleteCompanyFacts
    Transition <|-- MissingOutput
    InvalidCikFormat --> CikError
    FailedRequestExecution --> SecRequestError
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
# All S&P 500 CIKs (3 concurrent)
cargo run --features tracing-logging --bin stream_etl
```

## Contributing

See [CONTRIBUTING.md](CONTRIBUTING.md) for guidelines. All contributions are welcome.
