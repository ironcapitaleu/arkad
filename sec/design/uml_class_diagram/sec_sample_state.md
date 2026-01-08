```mermaid
---
title: "Sample `SEC` State: `SampleState`"
---
classDiagram
    class StateMachine~S: State~{
        <<trait>>
        %% SEC-specific StateMachine trait
    }

    class SMStateMachine~S: SMState~{
        <<trait>>
        %% Base StateMachine trait from `state_maschine`

        +get_current_state(&self) &S
        +get_current_state_mut(&mut self) &mut S
        +advance_state(&mut self)
        +run(&mut self)
    }
    class SuperState~S: State~ {
        <<trait>>
        %% SEC-specific SuperState trait
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
        +get_state_name(&self) impl ToString
        +get_input_data(&self) &Self::InputData
        +compute_output_data(&mut self)
        +get_output_data(&self) Option<&Self::OutputData>
        +get_context_data(&self) &Self::Context
    }

    class State {
        <<trait>>
        %% SEC-specific State trait
        +compute_output_data(&mut self, Result<(), StateError>)
    }

    class SMStateData {
        <<trait>>
        %% Base StateData trait from `state_maschine`
        +type UpdateType
        +get_state(&self) &Self
        +update_state(&mut self, updates: Self::UpdateType)
    }

    class StateData {
        <<trait>>
        %% SEC-specific StateData trait
        +update_state(&mut self, updates: Self::UpdateType) Result<(), StateError>
    }

    class SMContext {
        <<trait>>
        %% Base Context trait from `state_maschine`
        +type UpdateType
        +get_context(&self) &Self
        +update_context(&mut self, updates: Self::UpdateType)
    }

    class Context {
        <<trait>>
        %% SEC-specific Context trait
        +can_retry(&self) bool
        +get_max_retries(&self) u32
    }

    class StateError {
        <<enum>>
        %% SEC-specific errors.
        -InvalidCikFormat
        -InvalidInput
        -InvalidContext
        -FailedOutputComputation
        -StateDataUpdateFailed
        -ContextUpdateFailed
    }

    class SampleState {
        <<struct>>
        %% A sample SecState implementation, represents any 'SecState'
        -input: SampleSecStateInput
        -context: SampleSecStateContext
        -output: Option~SampleSecStateOutput~
        +new(input, context) Self
    }

    class SampleStateInputData {
        <<struct>>
        %% Input data for SampleSecState
        +input_data: String
    }

    class SampleStateOutputData {
        <<struct>>
        %% Output data for SampleSecState
        +output_data: String
    }

    class SampleStateContext {
        <<struct>>
        %% Context for SampleSecState
        +context_data: String
        +max_retries: u32
    }

    %% is-relationships
    StateMachine --> State : "is in a"
    SuperState --> StateMachine : "is a"
    SuperState --> State :  "is a"


    %% SEC-specific trait inheritance
    StateMachine --> SMStateMachine : "extends"
    SuperState --> SMSuperState : "extends"
    State --> SMState : "extends"
    StateData --> SMStateData : "extends"
    Context --> SMContext : "extends"

    %% Trait implementations
    
    SampleState --> State : "implements"
    SampleStateInputData --> StateData : "implements"
    SampleStateOutputData --> StateData : "implements"
    SampleStateContext --> Context : "implements"

    %% Struct relationships
    SampleState --> SampleStateInputData : "has"
    SampleState --> SampleStateOutputData : "has"
    SampleState --> SampleStateContext : "has"

    %% Error relationships
    State --> StateError : "can return"
    StateData --> StateError : "can return" 
    ```
