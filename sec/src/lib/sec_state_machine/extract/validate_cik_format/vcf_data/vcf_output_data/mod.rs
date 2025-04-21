use std::fmt;

use state_maschine::prelude::*;

mod cik;
pub use cik::Cik;

#[derive(Debug, Clone, PartialEq, PartialOrd, Hash, Eq, Ord)]
pub struct ValidateCikFormatOutputData {
    pub validated_cik: Cik,
}

impl StateData for ValidateCikFormatOutputData {
    fn get_state(&self) -> &Self {
        &self
    }

    fn update_state(&mut self, updates: Self::UpdateType) {
        self.validated_cik = updates;
    }

    type UpdateType = Cik;
}

impl fmt::Display for ValidateCikFormatOutputData {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "\tValid CIK: {}", self.validated_cik,)
    }
}
