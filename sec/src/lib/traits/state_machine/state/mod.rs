use state_maschine::prelude::State as SMState;

use crate::error::State as StateError;

pub mod context_data;
pub mod state_data;

pub use context_data::ContextData;
pub use state_data::StateData;

pub trait State: SMState {
    /// Computes the output data for the SEC state.
    ///
    /// # Errors
    ///
    /// Returns an error convertible into a `StateError` if the output data computation fails.
    fn compute_output_data(&mut self) -> Result<(), impl Into<StateError>>;
}
