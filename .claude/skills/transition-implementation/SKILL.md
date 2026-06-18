---
name: transition-implementation
description: >
  This skill should be used when the user asks to "implement a transition", "wire up the transition",
  "connect two states", "implement Transition trait", or needs to implement the actual transition
  logic between two states including SuperState wiring and binary integration.
version: 0.1.0
argument-hint: "[source-state] [target-state]"
allowed-tools: [Read, Write, Edit, Bash, AskUserQuestion]
---

# Transition Implementation Skill

## Purpose

Implement the Transition trait between two states, wire it into the SuperState, integrate into
applicable binaries, and update design documents. This follows the transition design phase.

## Prerequisites

Before implementing, verify:

1. **Source state exists** and has a defined output struct
2. **Target state exists** and has a defined input struct
3. **Transition design document exists** (from `/transition-design`)
4. **Both states compile** and pass their individual tests

## Implementation Workflow

### Phase 1: Implement Transition Trait

Use `into_parts()` to decompose the source state, extract what you need from its output and
context, and assemble the target state.

**Real example** (from `Extract<ExecuteSecRequest>` → `Transform<ParseCompanyFacts>`):

```rust
impl Transition<ExecuteSecRequest, ParseCompanyFacts> for ExtractSuperState<ExecuteSecRequest> {
    fn transition_to_next_state_sec(self) -> Result<Self::NewStateMachine, TransitionError> {
        let inner_state = self.into_current_state();
        let (_input, output, context) = inner_state.into_parts();

        let output_data = output.ok_or_else(|| {
            transition::MissingOutput::new(EXECUTE_SEC_REQUEST, PARSE_COMPANY_FACTS)
        })?;

        let sec_response = output_data.response;
        let cik = context.cik;

        Ok(TransformSuperState::<ParseCompanyFacts>::new(
            &sec_response,
            cik,
        ))
    }
}
```

**Generic pattern:**

```rust
impl Transition<SourceState, TargetState> for SuperState<SourceState> {
    fn transition_to_next_state_sec(self) -> Result<Self::NewStateMachine, TransitionError> {
        let inner_state = self.into_current_state();
        let (_input, output, context) = inner_state.into_parts();

        let output_data = output.ok_or_else(|| {
            transition::MissingOutput::new(SOURCE_STATE_NAME, TARGET_STATE_NAME)
        })?;

        // Extract fields from output and context
        let field_a = output_data.field_a;
        let shared_dep = context.shared_dep;

        // Construct target state
        let target_input = TargetStateInput::new(field_a);
        let target_context = TargetStateContext::new(shared_dep);
        let target_state = TargetState::new(target_input, target_context);

        Ok(SuperState {
            current_state: target_state,
            input: SuperStateData,
            output: None,
            context: self.context,
        })
    }
}
```

### Phase 2: Tests

Write tests for the transition:

1. **Happy path** — source with computed output transitions successfully
2. **Missing output** — source without computed output returns error
3. **Data integrity** — verify all mapped fields arrive correctly in target input
4. **Context propagation** — verify context carries forward correctly

```rust
#[test]
fn should_transition_from_source_to_target_when_output_computed() {
    let mut source = create_source_with_input_and_context();
    source.compute_output_data();
    let super_state = SuperState { current_state: source, /* ... */ };

    let expected_target_input = TargetStateInput::new(/* expected values */);

    let result = super_state.transition_to_next_state_sec();

    assert_eq!(result.unwrap().current_state.input_data(), &expected_target_input);
}

#[test]
fn should_fail_transition_when_source_output_not_computed() {
    let source = create_source_with_input_and_context();
    let super_state = SuperState { current_state: source, /* ... */ };

    let result = super_state.transition_to_next_state_sec();

    assert!(result.is_err());
}
```

### Phase 3: SuperState Wiring

1. **Add the transition to the SuperState's run logic** — ensure the state machine
   knows to transition from Source to Target after Source computes
2. **Update the async stream** — add the transition in the correct sequential position
3. **Verify the full pipeline** — Source compute → transition → Target exists in stream

### Phase 4: Binary Integration

1. **Identify affected binaries** — which `main()` functions use this SuperState?
2. **Wire the transition** — add it to the pipeline execution in correct order
3. **Test end-to-end** — run the binary and verify the transition executes

### Phase 5: Documentation Updates

1. **State diagram** — update mermaid state diagram to show the new transition arrow
2. **Class diagram** — update if new types were introduced for the mapping
3. **README** — update if the pipeline's capabilities changed

## Error Handling in Transitions

Transitions can fail. Follow the project's error conventions:

```rust
// For simple transitions (no complex mapping)
fn transition_to_next_state(self) -> Result<Self::NewStateMachine, &'static str> {
    let output = source_output.ok_or("Source state has no computed output")?;
    // ...
}

// For complex transitions (type conversions, validation)
fn transition_to_next_state(self) -> Result<Self::NewStateMachine, TransitionError> {
    let output = source_output.ok_or(TransitionError::MissingOutput {
        state: "SourceState".to_string(),
    })?;
    let mapped_field = TryFrom::try_from(output.field)
        .map_err(|e| TransitionError::FailedOutputConversion {
            reason: e.to_string(),
        })?;
    // ...
}
```

## Checklist

- [ ] Transition trait implemented
- [ ] Happy path test passes
- [ ] Missing output test passes
- [ ] Data integrity test passes
- [ ] SuperState run logic updated
- [ ] Async stream includes transition in correct order
- [ ] Affected binaries wired
- [ ] State diagram updated
- [ ] All tests pass (`cargo test`)
