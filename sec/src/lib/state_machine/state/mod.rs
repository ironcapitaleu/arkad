use super::sec_error::SecError;
use state_maschine::prelude::*;

pub mod sec_context_data;
pub mod sec_state_data;

pub use sec_context_data::SecContextData;
pub use sec_state_data::SecStateData;

pub trait SecState: State {
    /// Computes the output data for the SEC state.
    ///
    /// # Errors
    ///
    /// Returns a `SecError` if the output data cannot be computed.
    fn compute_output_data(&mut self) -> Result<(), SecError>;
}
