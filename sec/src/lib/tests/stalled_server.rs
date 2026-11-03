//! # Stalled Server
//!
//! Provides a local TCP server that stops sending, for the tests of the request timeouts.

use std::future;
use std::net::SocketAddr;

use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpListener;

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
    let listener = TcpListener::bind("127.0.0.1:0").await.expect(
        "Port 0 on the loopback interface lets the OS pick a free port, so binding always succeeds",
    );
    let address = listener
        .local_addr()
        .expect("A bound listener always has a local address");
    tokio::spawn(async move {
        if let Ok((mut connection, _)) = listener.accept().await {
            // One read takes the whole request, because a test request is a few hundred bytes.
            let mut request = [0; 4096];
            let _ = connection.read(&mut request).await;
            let _ = connection.write_all(response_head).await;
            future::pending::<()>().await;
        }
    });
    address
}
