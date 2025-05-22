use state_maschine;

use super::sec_error::SecError;

pub mod sec_context_data;
pub mod sec_state_data;

pub use sec_context_data::SecContextData;
pub use sec_state_data::SecStateData;

pub trait State: state_maschine::prelude::State {
    /// Computes the output data for the SEC state.
    ///
    /// # Errors
    ///
    /// Returns a `SecError` if the output data cannot be computed.
    fn compute_output_data(&mut self) -> Result<(), SecError>;
}
