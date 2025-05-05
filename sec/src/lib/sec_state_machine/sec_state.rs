use state_maschine::prelude::*;
use super::sec_error::SecError;

pub trait SecState: State {
    fn try_compute_output_data(&mut self) -> Result<(), SecError>;
}