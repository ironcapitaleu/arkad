use state_maschine::prelude::State as SMState;

use super::sec_error::SecError;

pub mod context_data;
pub mod sec_state_data;

pub use context_data::ContextData;
pub use sec_state_data::SecStateData;

pub trait State: SMState {
    /// Computes the output data for the SEC state.
    ///
    /// # Errors
    ///
    /// Returns an `SecError` if the output data cannot be computed.
    fn compute_output_data(&mut self) -> Result<(), SecError>;
}
