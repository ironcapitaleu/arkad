use super::sec_error::SecError;
use state_maschine::prelude::*;

pub trait SecState: State {
    /// Computes the output data for the SEC state.
    ///
    /// # Errors
    ///
    /// Returns a `SecError` if the output data cannot be computed.
    fn compute_output_data(&mut self) -> Result<(), SecError>;
}
