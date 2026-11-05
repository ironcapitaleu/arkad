//! # JSON Log Tests
//!
//! Runs the JSON logger of the binaries into a buffer, and parses the first line it writes. The
//! logger is a module of each binary, so this test compiles it as a private module.
//!
//! ## External Dependencies
//!
//! None.

// The tests write to a buffer, not to stdout, so `init_logger` would fire `dead_code`.
#[expect(dead_code)]
#[path = "../src/bin/shared/json_log.rs"]
mod json_log;

use std::io::{self, Write};
use std::sync::{Arc, Mutex};

use pretty_assertions::assert_eq;
use serde_json::{Value, json};
use tracing_subscriber::fmt::MakeWriter;

use json_log::json_subscriber;

const LOCK_EXPECT: &str =
    "Given a writer that never panics while it holds the lock, the lock should never be poisoned";

/// A writer that keeps every byte in memory.
#[derive(Clone, Default)]
struct Buffer(Arc<Mutex<Vec<u8>>>);

impl Write for Buffer {
    fn write(&mut self, bytes: &[u8]) -> io::Result<usize> {
        self.0.lock().expect(LOCK_EXPECT).extend_from_slice(bytes);
        Ok(bytes.len())
    }

    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}

impl MakeWriter<'_> for Buffer {
    type Writer = Self;

    fn make_writer(&self) -> Self::Writer {
        self.clone()
    }
}

/// Runs `log` under the JSON logger, and returns its first line as JSON.
fn first_line(log: impl FnOnce()) -> Value {
    let buffer = Buffer::default();
    tracing::subscriber::with_default(json_subscriber(buffer.clone()).finish(), log);
    let bytes = buffer.0.lock().expect(LOCK_EXPECT).clone();
    let text = String::from_utf8(bytes)
        .expect("Given a logger that writes only JSON, the output should always be UTF-8");
    let line = text
        .lines()
        .next()
        .expect("Given a closure that logs one event, the logger should always write a line");
    serde_json::from_str(line)
        .expect("Given a logger that writes JSON lines, each line should always parse as JSON")
}

#[test]
fn should_write_context_as_object_when_context_is_recorded_as_display() {
    let context = json!({ "root": "/data", "attempt": 2 });

    let expected_result = json!({ "root": "/data", "attempt": 2 });

    let result = first_line(|| {
        tracing::error!(event = "startup_failed", message = "Failed", context = %context);
    });

    assert_eq!(result["context"], expected_result);
}

#[test]
fn should_write_level_in_lowercase_when_event_is_an_error() {
    let message = "Failed";

    let expected_result = json!("error");

    let result = first_line(|| tracing::error!(event = "startup_failed", message = %message));

    assert_eq!(result["level"], expected_result);
}

#[test]
fn should_write_exactly_the_five_fields_when_event_has_extra_fields() {
    let env_var = "ARKAD_SEC_USER_AGENT";

    let expected_result = vec!["context", "event", "level", "message", "timestamp"];

    let line = first_line(|| {
        tracing::warn!(
            event = "default_user_agent",
            message = "Warning",
            env_var = env_var
        );
    });
    let result: Vec<&str> = line
        .as_object()
        .expect("Given a logger that writes JSON objects, each line should always be an object")
        .keys()
        .map(String::as_str)
        .collect();

    assert_eq!(result, expected_result);
}

#[test]
fn should_move_flat_fields_into_context_when_event_has_no_context_field() {
    let root = "/data";
    let successes: u64 = 3;
    let duration_ms: u128 = 1500;

    let expected_result = json!({ "root": "/data", "successes": 3, "duration_ms": 1500 });

    let result = first_line(|| {
        tracing::info!(
            event = "batch_complete",
            message = "Done",
            root = %root,
            successes = successes,
            duration_ms = duration_ms,
        );
    });

    assert_eq!(result["context"], expected_result);
}

#[test]
fn should_write_empty_context_when_event_has_no_other_fields() {
    let message = "Done";

    let expected_result = json!({});

    let result = first_line(|| tracing::info!(event = "batch_complete", message = %message));

    assert_eq!(result["context"], expected_result);
}

#[test]
fn should_keep_context_text_under_context_key_when_context_is_not_an_object() {
    let context = "not an object";

    let expected_result = json!({ "context": "not an object" });

    let result = first_line(|| {
        tracing::info!(event = "batch_complete", message = "Done", context = %context);
    });

    assert_eq!(result["context"], expected_result);
}

#[test]
fn should_write_implicit_message_when_event_has_a_format_string() {
    let count = 3;

    let expected_result = json!("Stored 3 files");

    let result = first_line(|| tracing::info!(event = "batch_complete", "Stored {count} files"));

    assert_eq!(result["message"], expected_result);
}
