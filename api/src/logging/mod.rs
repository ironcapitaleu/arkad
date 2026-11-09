//! # Logging
//!
//! Provides the JSON logger of the `api` binary and the [`ApiEvent`] values its logs carry. Each
//! line has the five fields `level`, `timestamp`, `event`, `message`, and `context`. `level` is
//! lowercase, and `context` is a JSON object.
//!
//! A call records `event`, `message`, and `context = %serde_json::json!({ … })`. `tracing` 0.1
//! records no nested value, so `context` reaches the logger as text, and [`JsonLog`] parses it
//! back into an object. Every other field of the call goes into `context` under its own name. If two
//! fields give the same key, the first one recorded wins.
//!
//! The logger is a copy of `sec/src/bin/shared/json_log.rs`. If you change it, change that file.

use std::fmt::{self, Display, Formatter};
use std::io::stdout;

use chrono::{SecondsFormat, Utc};
use serde::Serialize;
use serde_json::{Map, Value};
use tracing::field::{Field, Visit};
use tracing::{Event, Subscriber};
use tracing_appender::non_blocking::WorkerGuard;
use tracing_subscriber::filter::LevelFilter;
use tracing_subscriber::fmt::format::{DefaultFields, Writer};
use tracing_subscriber::fmt::{
    FmtContext, FormatEvent, FormatFields, MakeWriter, SubscriberBuilder,
};
use tracing_subscriber::registry::LookupSpan;

/// The events of the `api` binary, written into the `event` field of a log.
#[non_exhaustive]
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
    /// The server failed to answer a request, and the client got a `500`.
    RequestFailed,
}

impl Display for ApiEvent {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Self::Started => write!(f, "api_started"),
            Self::StartupFailed => write!(f, "api_startup_failed"),
            Self::Failed => write!(f, "api_failed"),
            Self::Stopped => write!(f, "api_stopped"),
            Self::RequestFailed => write!(f, "api_request_failed"),
        }
    }
}

/// Starts the JSON logger on stdout. The logger writes until the returned guard drops.
pub fn init_logger() -> WorkerGuard {
    let (non_blocking, guard) = tracing_appender::non_blocking(stdout());
    json_subscriber(non_blocking).init();
    guard
}

/// Returns a subscriber that writes each event to `writer` as one [`JsonLog`] line.
pub fn json_subscriber<W>(writer: W) -> SubscriberBuilder<DefaultFields, JsonLog, LevelFilter, W>
where
    W: for<'writer> MakeWriter<'writer> + Send + Sync + 'static,
{
    tracing_subscriber::fmt()
        .event_format(JsonLog)
        .with_writer(writer)
}

/// Formats an event as one JSON line with the five fields of a log line.
pub struct JsonLog;

impl<S, N> FormatEvent<S, N> for JsonLog
where
    S: Subscriber + for<'lookup> LookupSpan<'lookup>,
    N: for<'writer> FormatFields<'writer> + 'static,
{
    fn format_event(
        &self,
        _context: &FmtContext<'_, S, N>,
        mut writer: Writer<'_>,
        event: &Event<'_>,
    ) -> fmt::Result {
        let mut fields = LogFields::default();
        event.record(&mut fields);
        let line = LogLine {
            level: event.metadata().level().as_str().to_ascii_lowercase(),
            timestamp: Utc::now().to_rfc3339_opts(SecondsFormat::Micros, true),
            event: fields.event,
            message: fields.message,
            context: fields.context,
        };
        let json = serde_json::to_string(&line).map_err(|_| fmt::Error)?;
        writeln!(writer, "{json}")
    }
}

/// One log line. Serde writes the fields in the order of their declaration.
#[derive(Serialize)]
struct LogLine {
    level: String,
    timestamp: String,
    event: String,
    message: String,
    context: Map<String, Value>,
}

/// The fields of one event, sorted into `event`, `message`, and `context`.
#[derive(Default)]
struct LogFields {
    event: String,
    message: String,
    context: Map<String, Value>,
}

impl LogFields {
    /// Stores the text of `field`. The entries of a `context` object go into `context`.
    fn record_text(&mut self, field: &Field, text: String) {
        match field.name() {
            "event" => self.event = text,
            "message" => self.message = text,
            "context" => match serde_json::from_str(&text) {
                Ok(Value::Object(entries)) => {
                    for (key, value) in entries {
                        self.context.entry(key).or_insert(value);
                    }
                }
                _ => self.record_value(field, Value::String(text)),
            },
            _ => self.record_value(field, Value::String(text)),
        }
    }

    /// Stores `value` in `context` under the name of `field`, unless `context` holds that key.
    fn record_value(&mut self, field: &Field, value: Value) {
        self.context.entry(field.name()).or_insert(value);
    }
}

impl Visit for LogFields {
    fn record_str(&mut self, field: &Field, value: &str) {
        self.record_text(field, value.to_owned());
    }

    fn record_error(&mut self, field: &Field, value: &(dyn std::error::Error + 'static)) {
        self.record_text(field, value.to_string());
    }

    // `tracing` passes a field recorded with `%` to this method. The `Debug` form of that field is
    // its `Display` form.
    fn record_debug(&mut self, field: &Field, value: &dyn fmt::Debug) {
        self.record_text(field, format!("{value:?}"));
    }

    fn record_bool(&mut self, field: &Field, value: bool) {
        self.record_value(field, Value::from(value));
    }

    fn record_f64(&mut self, field: &Field, value: f64) {
        self.record_value(field, Value::from(value));
    }

    fn record_i64(&mut self, field: &Field, value: i64) {
        self.record_value(field, Value::from(value));
    }

    fn record_u64(&mut self, field: &Field, value: u64) {
        self.record_value(field, Value::from(value));
    }

    fn record_i128(&mut self, field: &Field, value: i128) {
        let value =
            i64::try_from(value).map_or_else(|_| Value::from(value.to_string()), Value::from);
        self.record_value(field, value);
    }

    fn record_u128(&mut self, field: &Field, value: u128) {
        let value =
            u64::try_from(value).map_or_else(|_| Value::from(value.to_string()), Value::from);
        self.record_value(field, value);
    }
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

    #[test]
    fn should_write_snake_case_name_for_request_failed_event() {
        let event = ApiEvent::RequestFailed;

        let expected_result = "api_request_failed";

        let result = event.to_string();

        assert_eq!(result, expected_result);
    }
}
