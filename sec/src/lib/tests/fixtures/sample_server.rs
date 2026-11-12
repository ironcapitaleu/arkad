//! # Sample Server
//!
//! Provides local TCP servers that stop sending, close the connection, or send a gzip body, and a
//! client that gives up on them, for the tests of the request timeouts and of gzip bodies.

use std::future;
use std::io::Write;
use std::net::SocketAddr;
use std::time::Duration;

use flate2::Compression;
use flate2::write::GzEncoder;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};

/// The name of the request header that lists the accepted encodings, in lowercase.
const ACCEPT_ENCODING_HEADER: &str = "accept-encoding:";

/// The value of `Content-Encoding` for a gzip body.
const GZIP_ENCODING: &str = "gzip";

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
    serve_once(move |_| response_head.to_vec(), AfterResponse::HoldOpen).await
}

/// Starts a server that accepts one connection, reads the request, writes `response`, and then
/// closes the connection.
///
/// Pass a head with a `Content-Length` for a body that ends early without a timeout.
///
/// # Panics
///
/// Panics if the OS cannot bind a free port on the loopback interface.
pub async fn closing_server(response: impl Into<Vec<u8>>) -> SocketAddr {
    let response = response.into();
    serve_once(move |_| response, AfterResponse::Close).await
}

/// Starts a server that answers one request with status 200, `content_type`, and `body`, and then
/// closes the connection.
///
/// If the request asks for gzip, the server sends `body` gzip-encoded, with
/// `Content-Encoding: gzip`. Otherwise it sends `body` unchanged. `Content-Length` states the
/// number of bytes on the wire in both cases, as the SEC does.
///
/// # Panics
///
/// Panics if the OS cannot bind a free port on the loopback interface.
pub async fn gzip_server(content_type: &'static str, body: &'static [u8]) -> SocketAddr {
    let answer = move |request_head: &[u8]| {
        if asks_for_gzip(request_head) {
            gzip_response(content_type, &gzip(body))
        } else {
            response(content_type, None, body)
        }
    };
    serve_once(answer, AfterResponse::Close).await
}

/// Returns `body` gzip-encoded.
///
/// # Panics
///
/// Panics if the gzip encoder fails to write to its [`Vec`].
#[must_use]
pub fn gzip(body: &[u8]) -> Vec<u8> {
    let mut encoder = GzEncoder::new(Vec::new(), Compression::default());
    encoder
        .write_all(body)
        .expect("A gzip encoder that writes to a Vec never fails");
    encoder
        .finish()
        .expect("A gzip encoder that writes to a Vec never fails")
}

/// Returns a response with status 200, `content_type`, `Content-Encoding: gzip`, and `gzip_body`
/// unchanged, with its length as `Content-Length`.
///
/// Pass a broken `gzip_body` to test the decoder.
#[must_use]
pub fn gzip_response(content_type: &str, gzip_body: &[u8]) -> Vec<u8> {
    response(content_type, Some(GZIP_ENCODING), gzip_body)
}

/// Returns a response with status 200, `content_type`, the optional `content_encoding`, and `body`,
/// with the length of `body` as `Content-Length`.
fn response(content_type: &str, content_encoding: Option<&str>, body: &[u8]) -> Vec<u8> {
    let encoding_header = content_encoding
        .map(|encoding| format!("Content-Encoding: {encoding}\r\n"))
        .unwrap_or_default();
    let mut response = format!(
        "HTTP/1.1 200 OK\r\nContent-Type: {content_type}\r\n{encoding_header}Content-Length: {}\r\n\r\n",
        body.len()
    )
    .into_bytes();
    response.extend_from_slice(body);
    response
}

/// Returns `true` if the `Accept-Encoding` header of `request_head` lists gzip.
fn asks_for_gzip(request_head: &[u8]) -> bool {
    String::from_utf8_lossy(request_head)
        .to_ascii_lowercase()
        .lines()
        .filter_map(|line| line.strip_prefix(ACCEPT_ENCODING_HEADER))
        .flat_map(|encodings| encodings.split(','))
        .any(|encoding| encoding.trim() == GZIP_ENCODING)
}

/// What a server does after it writes the response.
enum AfterResponse {
    /// Keep the connection open and send nothing.
    HoldOpen,
    /// Close the connection.
    Close,
}

/// Starts a server that answers one connection with the bytes `answer` returns for the request
/// head, then acts as `after_response` says.
async fn serve_once(
    answer: impl FnOnce(&[u8]) -> Vec<u8> + Send + 'static,
    after_response: AfterResponse,
) -> SocketAddr {
    let listener = TcpListener::bind("127.0.0.1:0").await.expect(
        "Port 0 on the loopback interface lets the OS pick a free port, so binding always succeeds",
    );
    let address = listener
        .local_addr()
        .expect("A bound listener always has a local address");
    tokio::spawn(async move {
        if let Ok((mut connection, _)) = listener.accept().await {
            let request_head = read_request_head(&mut connection).await;
            let _ = connection.write_all(&answer(&request_head)).await;
            if matches!(after_response, AfterResponse::HoldOpen) {
                future::pending::<()>().await;
            }
        }
    });
    address
}

/// Reads from `connection` until the blank line that ends the request head, or until it closes,
/// and returns the bytes read.
///
/// A test request has no body. So the server has read the whole request when the head ends, and a
/// close after the answer sends no reset.
async fn read_request_head(connection: &mut TcpStream) -> Vec<u8> {
    let mut request = Vec::new();
    let mut chunk = [0; 1024];
    while !request.ends_with(b"\r\n\r\n") {
        match connection.read(&mut chunk).await {
            Ok(0) | Err(_) => break,
            Ok(read) => request.extend_from_slice(&chunk[..read]),
        }
    }
    request
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
