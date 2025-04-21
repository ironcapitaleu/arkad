use std::fmt;

use state_maschine::prelude::*;

#[derive(Debug, Clone, PartialEq, PartialOrd, Hash, Eq, Ord)]
pub struct ValidateCikFormatContext {
    // status: Status,
    pub given_cik: String,
}

// #[derive(Debug, Clone, PartialEq, PartialOrd, Hash, Eq, Ord)]
// enum Status {
//     PreValidation,
//     PostValidation,
// }

impl ValidateCikFormatContext {
    pub fn new(cik: &(impl ToString + ?Sized)) -> Self {
        Self {
            given_cik: cik.to_string(),
        }
    }
}

impl ContextData for ValidateCikFormatContext {
    type UpdateType = String;
    fn get_context(&self) -> &Self {
        &self
    }

    fn update_context(&mut self, updates: Self::UpdateType) {
        self.given_cik = updates;
    }
}

impl Default for ValidateCikFormatContext {
    fn default() -> Self {
        Self::new("1067983")
    }
}

impl fmt::Display for ValidateCikFormatContext {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "CIK: {}", self.given_cik)
    }
}
