---
name: state-scaffold
description: >
  Use when creating a new state, scaffolding state boilerplate, or adding a new
  state to the state machine. Generates the full directory structure, structs,
  trait implementations, and test suite through a guided questionnaire.
version: 0.1.0
---

# State Scaffold Skill

## Purpose

Generate the complete boilerplate for a new state in the SEC state machine. This includes the directory structure, all required structs, trait implementations, Display impls, and the full test suite.

## Questionnaire

Before generating code, gather these inputs from the user:

1. **State name** — e.g., `ValidateCikFormat`, `ExecuteSecRequest`
2. **Input fields** — what data does this state receive? (field names, types, sources)
3. **Context** — shared dependencies (clients, config, max_retries)
4. **Output fields** — what does this state produce? (field names, types)
5. **Source state** — which state transitions into this one? (for Transition impl)
6. **Async?** — does compute_output_data need async? (almost always yes for SEC)
7. **SuperState** — which state machine does this belong to?

## Generated Directory Structure

```text
{snake_case_state_name}/
├── mod.rs              # State struct + State trait impl + Display + tests
├── constants.rs        # pub const STATE_NAME: &str = "..."
├── data/
│   ├── mod.rs          # pub use input::*; pub use output::*;
│   ├── input/
│   │   └── mod.rs      # Input struct + StateData + Updater + Builder + tests
│   └── output/
│       └── mod.rs      # Output struct + StateData + Updater + Builder + tests
└── context/
    └── mod.rs          # Context struct + Context + Updater + Builder + Display + tests
```

## Generated Code Per Struct (Input, Output, Context)

For each data struct, generate:

1. **Core struct** with derives: `Debug, Clone, PartialEq, PartialOrd, Hash, Eq, Ord, Serialize`
2. **Constructor** (`new(...)`)
3. **StateData/Context trait impl** (SEC version with `Result` + state_machine version)
4. **Updater struct** — all fields wrapped in `Option`
5. **UpdaterBuilder** — const `new()`, fluent setters, `build()` method, `Default` impl
6. **Display impl**
7. **Test module** containing:
   - Auto-trait tests: Send, Sync, Unpin, Sized, Clone, Debug, Hash, Eq, Ord, PartialEq, PartialOrd
   - Thread safety test
   - `should_return_reference_to_self`
   - `should_create_different_with_custom_data`
   - `should_update_field_when_update_contains_value` (one per field)
   - `should_leave_unchanged_on_empty_update`

## Generated State Struct (mod.rs)

1. **State struct** with `input`, `context`, `output: Option<Output>`
2. **Constructor** (`new(input, context)`)
3. **`into_parts()`** method for transition decomposition
4. **SEC State trait impl** (`compute_output_data_async` — stub returning `Ok(())`)
5. **state_machine State trait impl** (blocking wrapper with tokio runtime detection)
6. **Display impl** (formatted summary of all fields)
7. **Transition impl** from source state
8. **Test module:**
   - `should_return_state_name`
   - `should_return_input_data`
   - `should_return_context_data`
   - `should_compute_output_data_async` (tokio test)
   - `should_compute_output_data_sync`
   - Auto-trait tests for the state struct itself

## Key Patterns

### Async/Sync Dual Implementation

```rust
// SEC trait (async, fallible)
#[async_trait]
impl State for MyState {
    async fn compute_output_data_async(&mut self) -> Result<(), StateError> {
        todo!("Implement domain logic")
    }
}

// state_machine trait (sync, infallible wrapper)
impl SMState for MyState {
    fn compute_output_data(&mut self) {
        let result = if let Ok(handle) = tokio::runtime::Handle::try_current() {
            tokio::task::block_in_place(|| handle.block_on(self.compute_output_data_async()))
        } else {
            tokio::runtime::Runtime::new()
                .expect("Tokio runtime creation should always succeed when no runtime is active")
                .block_on(self.compute_output_data_async())
        };
        if let Err(e) = result {
            panic!("compute_output_data failed: {e}")
        }
    }
}
```

### Transition Pattern

```rust
impl Transition<SourceState, NewState> for SuperState<SourceState> {
    type NewStateMachine = SuperState<NewState>;

    fn transition_to_next_state(self) -> Result<Self::NewStateMachine, &'static str> {
        let (source_input, source_output, source_context) = self.current_state.into_parts();
        let output = source_output.ok_or("Source state has no computed output")?;

        let new_input = NewStateInput::new(/* fields from output */);
        let new_context = NewStateContext::new(/* shared deps from source_context */);
        let new_state = NewState::new(new_input, new_context);

        Ok(SuperState { current_state: new_state, ..self })
    }
}
```

### Auto-Trait Test Pattern

```rust
const fn implements_auto_traits<T: Sized + Send + Sync + Unpin>() {}

#[test]
const fn should_implement_auto_traits() {
    implements_auto_traits::<MyStateInput>();
}

const fn implements_send<T: Send>() {}
const fn implements_sync<T: Sync>() {}

#[test]
const fn should_be_thread_safe() {
    implements_send::<MyStateInput>();
    implements_sync::<MyStateInput>();
}
```

## Documentation Updates

After scaffolding a new state, update the following:

- **Design docs** (e.g., `design/` mermaid diagrams): Add the new state to any state machine diagrams, transition flow charts, or UML class diagrams
- **README.md**: Update any state listings, architecture overviews, or module descriptions that reference available states
- **SuperState design doc**: If the state belongs to a SuperState, update its mermaid sequence/state diagram to include the new state and its transitions

Always verify existing diagrams still reflect reality after adding the new state.

## Binary Integration

After scaffolding, assess whether the new state should be added to one or more binaries:

- **Identify applicable binaries** — not all states belong in every binary, but most will appear in at least one
- **Add the state to the SuperState's async stream in the correct order** — it must execute after its source state and before the next state in the pipeline
- **Wire transitions** — ensure the binary's run loop or stream handles the transition from the previous state into this one and from this one into the next
- **Respect Streamable trait** — if the SuperState implements streaming, ensure the new state integrates into the async stream (yields progress, can be polled, respects cancellation)

If the state is not yet ready for binary integration (e.g., `compute_output_data_async` is still a stub), note it as a TODO for the `state-implementation` phase.

## What This Skill Does NOT Generate

- Domain logic inside `compute_output_data_async` (use `state-implementation` skill)
- Domain-specific error types (introduced during implementation)
- Integration tests (added during implementation)
- Dependency injection setup beyond Context stubs
