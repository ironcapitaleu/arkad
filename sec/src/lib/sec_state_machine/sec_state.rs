use super::sec_error::SecError;
use state_maschine::prelude::*;

pub trait SecState: State {
    fn compute_output_data(&mut self) -> Result<(), SecError>;
}
