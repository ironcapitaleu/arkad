use state_maschine::prelude::*;
use std::fmt;

#[derive(Debug, Clone, PartialEq, PartialOrd, Hash, Eq, Ord)]
pub struct ValidateCikFormatInputData {
    pub raw_cik: String,
}

impl StateData for ValidateCikFormatInputData {
    fn get_state(&self) -> &Self {
        &self
    }

    fn update_state(&mut self, updates: Self::UpdateType) {
        self.raw_cik = updates;
    }

    type UpdateType = String;
}

impl Default for ValidateCikFormatInputData {
    fn default() -> Self {
        Self {
            raw_cik: "1067983".to_string(),
        }
    }
}

impl fmt::Display for ValidateCikFormatInputData {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "\tCIK: {}", self.raw_cik,)
    }
}
