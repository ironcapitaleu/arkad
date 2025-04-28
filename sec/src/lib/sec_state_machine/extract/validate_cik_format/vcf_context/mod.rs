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
    type UpdateType = ValidateCikFormatContextDataUpdater;
    fn get_context(&self) -> &Self {
        &self
    }

    fn update_context(&mut self, updates: Self::UpdateType) {
        if let Some(cik) = updates.raw_cik {
            self.raw_cik = cik;
        }
    }
}

impl Default for ValidateCikFormatContext {
    fn default() -> Self {
        Self::new("1067983")
    }
}

impl fmt::Display for ValidateCikFormatContext {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Raw CIK: {}", self.raw_cik)
    }
}

#[derive(Debug, Clone, PartialEq, PartialOrd, Hash, Eq, Ord)]
pub struct ValidateCikFormatContextDataUpdater {
    pub raw_cik: Option<String>,
}

pub struct ValidateCikFormatContextDataUpdaterBuilder {
    raw_cik: Option<String>,
}
impl ValidateCikFormatContextDataUpdaterBuilder {
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
    pub fn build(self) -> ValidateCikFormatContextDataUpdater {
        ValidateCikFormatContextDataUpdater { raw_cik: self.raw_cik }
    }
}

impl Default for ValidateCikFormatContextDataUpdaterBuilder {
    fn default() -> Self {
        Self::new()
    }
}