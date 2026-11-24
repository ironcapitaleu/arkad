//! # API Binary
//!
//! Starts the API: reads the configuration, fetches the public keys for login tokens, binds the
//! listener on `$PORT`, and serves the router until a shutdown signal arrives.

use std::process::ExitCode;
use std::sync::Arc;

use api::AppState;
use api::config::{Config, InternalDocs};
use api::implementations::SupabaseTokenVerifier;
use api::logging::{ApiEvent, init_logger};
use serde_json::json;
use tokio::net::TcpListener;
use tokio::signal::unix::{SignalKind, signal};

#[tokio::main]
async fn main() -> ExitCode {
    let _guard = init_logger();

    let config = match Config::from_env() {
        Ok(config) => config,
        Err(error) => {
            tracing::error!(
                event = %ApiEvent::StartupFailed,
                message = %format!(
                    "[FailedConfigRead] Failed to read the configuration, Caused by: {error}"
                ),
                context = %json!({ "env_var": error.name() }),
            );
            return ExitCode::FAILURE;
        }
    };

    let Some(verifier) = create_token_verifier(config.supabase_url()).await else {
        return ExitCode::FAILURE;
    };
    let verifier = Arc::new(verifier);
    let _refresh_task = SupabaseTokenVerifier::spawn_refresh(&verifier);
    let state = AppState::new(verifier);

    let address = config.socket_addr();
    let listener = match TcpListener::bind(address).await {
        Ok(listener) => listener,
        Err(error) => {
            tracing::error!(
                event = %ApiEvent::StartupFailed,
                message = %format!(
                    "[FailedListenerBinding] Failed to bind the listener, Reason: '{error}'"
                ),
                context = %json!({ "address": address.to_string() }),
            );
            return ExitCode::FAILURE;
        }
    };

    tracing::info!(
        event = %ApiEvent::Started,
        message = %"The API listens for requests",
        context = %json!({
            "address": address.to_string(),
            "internal_docs": config.internal_docs() == InternalDocs::Enabled,
        }),
    );

    if let Err(error) = axum::serve(listener, api::router(config.internal_docs(), state))
        .with_graceful_shutdown(shutdown_signal())
        .await
    {
        tracing::error!(
            event = %ApiEvent::Failed,
            message = %format!("[FailedServing] Failed to serve requests, Reason: '{error}'"),
            context = %json!({ "address": address.to_string() }),
        );
        return ExitCode::FAILURE;
    }

    tracing::info!(
        event = %ApiEvent::Stopped,
        message = %"The API stopped after a shutdown signal",
        context = %json!({ "address": address.to_string() }),
    );
    ExitCode::SUCCESS
}

/// Fetches the public keys of the Supabase project at `supabase_url` and creates the verifier.
///
/// Logs the failure and returns `None` if the HTTP client or the fetch fails.
async fn create_token_verifier(supabase_url: &str) -> Option<SupabaseTokenVerifier> {
    let client = match reqwest::Client::builder().build() {
        Ok(client) => client,
        Err(error) => {
            tracing::error!(
                event = %ApiEvent::StartupFailed,
                message = %format!(
                    "[FailedClientCreation] Failed to create the HTTP client, Reason: '{error}'"
                ),
                context = %json!({}),
            );
            return None;
        }
    };
    match SupabaseTokenVerifier::fetch(&client, supabase_url).await {
        Ok(verifier) => Some(verifier),
        Err(error) => {
            tracing::error!(
                event = %ApiEvent::StartupFailed,
                message = %format!(
                    "[FailedTokenVerifierCreation] Failed to create the login token verifier, Caused by: {error}"
                ),
                context = %json!({ "supabase_url": supabase_url }),
            );
            None
        }
    }
}

/// Completes on `Ctrl+C` or on `SIGTERM`.
async fn shutdown_signal() {
    let mut terminate = signal(SignalKind::terminate())
        .expect("Given a Unix process with a Tokio runtime, registering the SIGTERM handler should always succeed");
    tokio::select! {
        _ = tokio::signal::ctrl_c() => {},
        _ = terminate.recv() => {},
    }
}
