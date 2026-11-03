//! # Stalled Server
//!
//! Provides local TCP servers that stop sending or close the connection, and a client that gives up
//! on them, for the tests of the request timeouts.

use std::future;
use std::net::SocketAddr;
use std::time::Duration;

use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};

/// Starts a server that accepts one connection, reads the request, writes `response_head`, and then
/// sends nothing.
///
/// The server holds the connection open until the test ends. Pass an empty `response_head` for a
/// server that never answers, or the status line and headers for a server that stalls in the body.
///
/// # Panics
///
/// Panics if the OS cannot bind a free port on the loopback interface.
pub async fn stalled_server(response_head: &'static [u8]) -> SocketAddr {
    serve_once(response_head, AfterHead::HoldOpen).await
}

/// Starts a server that accepts one connection, reads the request, writes `response_head`, and then
/// closes the connection.
///
/// Pass a head with a `Content-Length` for a body that ends early without a timeout.
///
/// # Panics
///
/// Panics if the OS cannot bind a free port on the loopback interface.
pub async fn closing_server(response_head: &'static [u8]) -> SocketAddr {
    serve_once(response_head, AfterHead::Close).await
}

/// What a server does after it writes the response head.
enum AfterHead {
    /// Keep the connection open and send nothing.
    HoldOpen,
    /// Close the connection.
    Close,
}

/// Starts a server that answers one connection with `response_head`, then acts as `after_head`
/// says.
async fn serve_once(response_head: &'static [u8], after_head: AfterHead) -> SocketAddr {
    let listener = TcpListener::bind("127.0.0.1:0").await.expect(
        "Port 0 on the loopback interface lets the OS pick a free port, so binding always succeeds",
    );
    let address = listener
        .local_addr()
        .expect("A bound listener always has a local address");
    tokio::spawn(async move {
        if let Ok((mut connection, _)) = listener.accept().await {
            read_request_head(&mut connection).await;
            let _ = connection.write_all(response_head).await;
            if matches!(after_head, AfterHead::HoldOpen) {
                future::pending::<()>().await;
            }
        }
    });
    address
}

/// Reads from `connection` until the blank line that ends the request head, or until it closes.
///
/// A test request has no body. So the server has read the whole request when the head ends, and a
/// close after the answer sends no reset.
async fn read_request_head(connection: &mut TcpStream) {
    let mut request = Vec::new();
    let mut chunk = [0; 1024];
    while !request.ends_with(b"\r\n\r\n") {
        match connection.read(&mut chunk).await {
            Ok(0) | Err(_) => return,
            Ok(read) => request.extend_from_slice(&chunk[..read]),
        }
    }
}

/// Returns a client that gives up on a response after 500 ms, and uses no proxy.
///
/// # Panics
///
/// Panics if `reqwest` cannot build the client, for example when the TLS backend fails to
/// initialize.
#[must_use]
pub fn impatient_client() -> reqwest::Client {
    reqwest::Client::builder()
        .read_timeout(Duration::from_millis(500))
        .no_proxy()
        .build()
        .expect("A builder with only a read timeout and no proxy fails only if the TLS backend cannot initialize")
}
