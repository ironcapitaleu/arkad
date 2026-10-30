use crate::shared::request::implementations::sec_request::{CompanyFacts, RequestKind};

/// Human-readable name of the state for the [`CompanyFacts`] kind, used in error messages and
/// logging. Each kind names its state in [`RequestKind::PREPARE_STATE_NAME`].
pub const STATE_NAME: &str = CompanyFacts::PREPARE_STATE_NAME;
