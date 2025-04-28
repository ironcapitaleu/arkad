use state_maschine::prelude::*;
use std::fmt;

#[derive(Debug, Clone, PartialEq, PartialOrd, Hash, Eq, Ord)]
pub struct ValidateCikFormatInputData {
    pub raw_cik: String,
}

impl StateData for ValidateCikFormatInputData {
    type UpdateType = ValidateCikFormatInputDataUpdater;

    fn get_state(&self) -> &Self {
        &self
    }

    fn update_state(&mut self, updates: Self::UpdateType) {
        if let Some(cik) = updates.raw_cik {
            self.raw_cik = cik;
        }
    }
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

#[derive(Debug, Clone, PartialEq, PartialOrd, Hash, Eq, Ord)]
pub struct ValidateCikFormatInputDataUpdater {
    pub raw_cik: Option<String>,
}

pub struct ValidateCikFormatInputDataUpdaterBuilder {
    raw_cik: Option<String>,
}
impl ValidateCikFormatInputDataUpdaterBuilder {
    #[must_use]
    pub const fn new() -> Self {
        Self { raw_cik: None }
    }

    #[must_use]
    #[allow(clippy::missing_const_for_fn)]
    pub fn cik(mut self, cik: String) -> Self {
        self.raw_cik = Some(cik);
        self
    }

    #[must_use]
    pub fn build(self) -> ValidateCikFormatInputDataUpdater {
        ValidateCikFormatInputDataUpdater {
            raw_cik: self.raw_cik,
        }
    }
}

impl Default for ValidateCikFormatInputDataUpdaterBuilder {
    fn default() -> Self {
        Self::new()
    }
}
