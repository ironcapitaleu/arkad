pub mod state;
pub mod transition;

pub use state::State;
pub use transition::Transition;

#[non_exhaustive]
pub enum StateMachine {
    /// Invalid configuration of the state machine.
    InvalidConfiguration,

    /// State-internal error.
    State,

    /// Transtion related error.
    Transition,
}

