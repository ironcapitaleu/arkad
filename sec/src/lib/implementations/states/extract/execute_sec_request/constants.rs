use crate::shared::http_client::RespondsWith;
use crate::shared::request::implementations::sec_request::CompanyFacts;

/// Human-readable name of the state for the [`CompanyFacts`] kind, used in error messages and
/// logging. Each kind names its state in [`RespondsWith::EXECUTE_STATE_NAME`].
pub const STATE_NAME: &str = CompanyFacts::EXECUTE_STATE_NAME;
