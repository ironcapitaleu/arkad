use std::fmt;

use state_maschine::prelude::*;

#[derive(Debug, Clone, PartialEq, PartialOrd, Hash, Eq, Ord)]
pub struct ValidateCikFormatContext {
    pub raw_cik: String,
}

impl ValidateCikFormatContext {
    pub fn new(cik: &(impl ToString + ?Sized)) -> Self {
        Self {
            raw_cik: cik.to_string(),
        }
    }
}

impl ContextData for ValidateCikFormatContext {
    type UpdateType = String;
    fn get_context(&self) -> &Self {
        &self
    }

    fn update_context(&mut self, updates: Self::UpdateType) {
        self.raw_cik = updates;
    }
}

impl Default for ValidateCikFormatContext {
    fn default() -> Self {
        Self::new("1067983")
    }
}

impl fmt::Display for ValidateCikFormatContext {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "CIK: {}", self.raw_cik)
    }
}
