```mermaid
---
title: "Sample `SEC` State: `SampleSecState`"
---
classDiagram
    class SecStateMachine~S: SecState~{
        <<trait>>
        %% SEC-specific SecStateMachine trait
    }

    class StateMachine~S: State~{
        <<trait>>
        %% Base StateMachine trait from `state_maschine`

        +get_current_state(&self) &S
        +get_current_state_mut(&mut self) &mut S
        +advance_state(&mut self)
        +run(&mut self)
    }
    class SecSuperState~S: SecState~ {
        <<trait>>
        %% SEC-specific SuperState trait
    }   

    class SuperState~S: State~ {
        <<trait>>
        %% Base SuperState trait from `state_maschine`
    }   

    class State {
        <<trait>>
        %% Base State trait from `state_maschine`
        +type InputData: StateData
        +type OutputData: StateData
        +type Context: ContextData
        +get_state_name(&self) impl ToString
        +get_input_data(&self) &Self::InputData
        +compute_output_data(&mut self)
        +get_output_data(&self) Option<&Self::OutputData>
        +get_context_data(&self) &Self::Context
    }

    class SecState {
        <<trait>>
        %% SEC-specific State trait
        +compute_output_data(&mut self, Result<(), SecError>)
    }

    class StateData {
        <<trait>>
        %% Base StateData trait from `state_maschine`
        +type UpdateType
        +get_state(&self) &Self
        +update_state(&mut self, updates: Self::UpdateType)
    }

    class SecStateData {
        <<trait>>
        %% SEC-specific StateData trait
        +update_state(&mut self, updates: Self::UpdateType) Result<(), SecError>
    }

    class ContextData {
        <<trait>>
        %% Base ContextData trait from `state_maschine`
        +type UpdateType
        +get_context(&self) &Self
        +update_context(&mut self, updates: Self::UpdateType)
    }

    class SecContextData {
        <<trait>>
        %% SEC-specific ContextData trait
        +can_retry(&self) bool
        +get_max_retries(&self) u32
    }

    class SecError {
        <<enum>>
        %% SEC-specific errors
        InvalidCikFormat(String)
    }

    class SampleSecState {
        <<struct>>
        %% A sample SecState implementation, represents any 'SecState'
        -input: SampleSecStateInputData
        -context: SampleSecStateContext
        -output: Option<SampleSecStateOutputData>
        +new(input, context) Self
    }

    class SampleSecStateInputData {
        <<struct>>
        %% Input data for SampleSecState
        +raw_cik: String
    }

    class SampleSecStateOutputData {
        <<struct>>
        %% Output data for SampleSecState
        +validated_cik: Cik
    }

    class Cik {
        <<struct>>
        %% A validated CIK structure
        -value: String
        +new(cik) Result<Self, SecError>
    }

    class SampleSecStateContext {
        <<struct>>
        %% Context for SampleSecState
        +raw_cik: String
        +max_retries: u32
    }

    %% is-relationships
    SecStateMachine --> SecState : "is in a"
    SecSuperState --> SecStateMachine : "is a"
    SecSuperState --> SecState :  "is a"


    %% SEC-specific trait inheritance
    SecStateMachine --> StateMachine : "extends"
    SecSuperState --> SuperState : "extends"
    SecState --> State : "extends"
    SecStateData --> StateData : "extends"
    SecContextData --> ContextData : "extends"

    %% Trait implementations
    
    SampleSecState --> SecState : "implements"
    SampleSecStateInputData --> SecStateData : "implements"
    SampleSecStateOutputData --> SecStateData : "implements"
    SampleSecStateContext --> SecContextData : "implements"

    %% Struct relationships
    SampleSecState --> SampleSecStateInputData : "has"
    SampleSecState --> SampleSecStateOutputData : "has"
    SampleSecState --> SampleSecStateContext : "has"
    SampleSecStateOutputData --> Cik : "has"

    %% Error relationships
    SecState --> SecError : "can return"
    SecStateData --> SecError : "can return"
    Cik --> SecError : "can return"     
    ```