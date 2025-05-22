pub mod state_machine;
pub use state_machine::StateMachine;
#[non_exhaustive]
pub enum ErrorKind {
    /// State machine related error.
    StateMachine,
}
