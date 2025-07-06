use std::fmt;

use state_maschine::prelude::StateData as SMStateData;

use crate::error::State as StateError;
use crate::shared::cik::Cik;
use crate::traits::state_machine::state::StateData;

#[derive(Debug, Clone, PartialEq, PartialOrd, Hash, Eq, Ord)]
pub struct PrepareSecRequestInputData {
    pub validated_cik: Cik,
    pub user_agent: String,
}

impl PrepareSecRequestInputData {
    #[must_use]
    pub const fn new(validated_cik: Cik, user_agent: String) -> Self {
        Self {
            validated_cik,
            user_agent,
        }
    }

    #[must_use]
    pub const fn validated_cik(&self) -> &Cik {
        &self.validated_cik
    }
    #[must_use]
    pub const fn user_agent(&self) -> &String {
        &self.user_agent
    }
}

impl StateData for PrepareSecRequestInputData {
    fn update_state(&mut self, updates: Self::UpdateType) -> Result<(), StateError> {
        if let Some(validated_cik) = updates.validated_cik {
            self.validated_cik = validated_cik;
        }
        if let Some(user_agent) = updates.user_agent {
            self.user_agent = user_agent;
        }
        Ok(())
    }
}

impl SMStateData for PrepareSecRequestInputData {
    type UpdateType = PrepareSecRequestInputDataUpdater;

    fn get_state(&self) -> &Self {
        self
    }

    fn update_state(&mut self, _updates: Self::UpdateType) {}
}

impl Default for PrepareSecRequestInputData {
    fn default() -> Self {
        Self {
            validated_cik: Cik::new("0001067983").expect("Hardcoded CIK should always be valid."),
            user_agent: String::new(),
        }
    }
}

impl fmt::Display for PrepareSecRequestInputData {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "\tValidated CIK: {}\nUser Agent: {}",
            self.validated_cik, self.user_agent
        )
    }
}

#[derive(Debug, Clone, PartialEq, PartialOrd, Hash, Eq, Ord)]
pub struct PrepareSecRequestInputDataUpdater {
    pub validated_cik: Option<Cik>,
    pub user_agent: Option<String>,
}

pub struct PrepareSecRequestInputDataUpdaterBuilder {
    pub validated_cik: Option<Cik>,
    pub user_agent: Option<String>,
}
impl PrepareSecRequestInputDataUpdaterBuilder {
    #[must_use]
    pub const fn new() -> Self {
        Self {
            validated_cik: None,
            user_agent: None,
        }
    }

    #[must_use]
    #[allow(clippy::missing_const_for_fn)]
    pub fn validated_cik(mut self, validated_cik: Cik, user_agent: String) -> Self {
        self.validated_cik = Some(validated_cik);
        self.user_agent = Some(user_agent);
        self
    }

    #[must_use]
    pub fn build(self) -> PrepareSecRequestInputDataUpdater {
        PrepareSecRequestInputDataUpdater {
            validated_cik: self.validated_cik,
            user_agent: self.user_agent,
        }
    }
}

impl Default for PrepareSecRequestInputDataUpdaterBuilder {
    fn default() -> Self {
        Self::new()
    }
}
