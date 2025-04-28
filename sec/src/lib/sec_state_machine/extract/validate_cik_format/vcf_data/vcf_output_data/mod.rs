use std::fmt;

use state_maschine::prelude::*;

pub mod cik;
pub use cik::Cik;

#[derive(Debug, Clone, PartialEq, PartialOrd, Hash, Eq, Ord)]
pub struct ValidateCikFormatOutputData {
    pub validated_cik: Cik,
}

impl StateData for ValidateCikFormatOutputData {
    type UpdateType = ValidateCikFormatOutputDataUpdater;

    fn get_state(&self) -> &Self {
        &self
    }

    fn update_state(&mut self, updates: Self::UpdateType) {
        if let Some(cik) = updates.cik {
            self.validated_cik = Cik::new(&cik);
        }
    }
}

impl fmt::Display for ValidateCikFormatOutputData {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "\tValid CIK: {}", self.validated_cik,)
    }
}

#[derive(Debug, Clone, PartialEq, PartialOrd, Hash, Eq, Ord)]
pub struct ValidateCikFormatOutputDataUpdater {
    pub cik: Option<Cik>,
}

pub struct ValidateCikFormatOutputDataUpdaterBuilder {
    cik: Option<Cik>,
}
impl ValidateCikFormatOutputDataUpdaterBuilder {
    #[must_use]
    pub const fn new() -> Self {
        Self { cik: None }
    }

    #[must_use]
    #[allow(clippy::missing_const_for_fn)]
    pub fn cik(mut self, cik: &(impl ToString + ?Sized)) -> Self {
        self.cik = Some(Cik::new(cik));
        self
    }

    #[must_use]
    pub fn build(self) -> ValidateCikFormatOutputDataUpdater {
        ValidateCikFormatOutputDataUpdater { cik: self.cik }
    }
}

impl Default for ValidateCikFormatOutputDataUpdaterBuilder {
    fn default() -> Self {
        Self::new()
    }
}