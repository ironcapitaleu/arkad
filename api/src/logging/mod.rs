//! # Logging
//!
//! Provides the JSON logger of the `api` binary and the [`ApiEvent`] values its logs carry.
//!
//! Every log has the five fields `level`, `timestamp`, `event`, `message`, and `context`.

use std::fmt::{self, Display, Formatter};
use std::io::stdout;

use tracing_appender::non_blocking::WorkerGuard;
use tracing_subscriber::fmt::format::FmtSpan;

/// The events of the `api` binary, written into the `event` field of a log.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ApiEvent {
    /// The listener is bound and the API serves requests.
    Started,
    /// The configuration or the listener failed, so the API did not start.
    StartupFailed,
    /// The API stopped with an error.
    Failed,
    /// The API stopped after a shutdown signal.
    Stopped,
}

impl Display for ApiEvent {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Self::Started => write!(f, "api_started"),
            Self::StartupFailed => write!(f, "api_startup_failed"),
            Self::Failed => write!(f, "api_failed"),
            Self::Stopped => write!(f, "api_stopped"),
        }
    }
}

/// Starts the JSON logger on stdout. The logger writes until the returned guard drops.
pub fn init_logger() -> WorkerGuard {
    let (non_blocking, guard) = tracing_appender::non_blocking(stdout());
    tracing_subscriber::fmt()
        .json()
        .with_span_events(FmtSpan::NONE)
        .with_target(false)
        .with_current_span(false)
        .flatten_event(true)
        .with_writer(non_blocking)
        .init();
    guard
}

#[cfg(test)]
mod tests {
    use pretty_assertions::assert_eq;

    use super::*;

    #[test]
    fn should_write_snake_case_name_for_started_event() {
        let event = ApiEvent::Started;

        let expected_result = "api_started";

        let result = event.to_string();

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_write_snake_case_name_for_startup_failed_event() {
        let event = ApiEvent::StartupFailed;

        let expected_result = "api_startup_failed";

        let result = event.to_string();

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_write_snake_case_name_for_failed_event() {
        let event = ApiEvent::Failed;

        let expected_result = "api_failed";

        let result = event.to_string();

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_write_snake_case_name_for_stopped_event() {
        let event = ApiEvent::Stopped;

        let expected_result = "api_stopped";

        let result = event.to_string();

        assert_eq!(result, expected_result);
    }
}
