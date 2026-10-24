use std::io::{Read, Write};
use std::net::TcpListener;
use std::thread;

/// Returns a real `reqwest::Response` with the given status line, content type, and body.
///
/// A one-shot server on a loopback port answers a single request, so the test needs no outside
/// network and no extra HTTP crate.
///
/// # Panics
///
/// Panics if the local socket cannot be bound or the request fails. A test has no way to recover
/// from either.
pub async fn fake_reqwest_response(
    status_line: &str,
    content_type: &str,
    body: &[u8],
) -> reqwest::Response {
    let listener = TcpListener::bind("127.0.0.1:0")
        .expect("Given a free local port, binding a listener should always succeed");
    let address = listener
        .local_addr()
        .expect("Given a bound listener, reading its address should always succeed");
    let mut message = format!(
        "HTTP/1.1 {status_line}\r\ncontent-type: {content_type}\r\ncontent-length: {}\r\nconnection: close\r\n\r\n",
        body.len()
    )
    .into_bytes();
    message.extend_from_slice(body);

    let server = thread::spawn(move || {
        let (mut stream, _) = listener
            .accept()
            .expect("Given a client that connects, accepting the connection should always succeed");
        // Reads the whole request head. A close with unread bytes sends a reset, not a normal close.
        let mut request = Vec::new();
        let mut chunk = [0_u8; 1024];
        while !request.ends_with(b"\r\n\r\n") {
            let read = stream
                .read(&mut chunk)
                .expect("Given an open connection, reading the request should always succeed");
            if read == 0 {
                break;
            }
            request.extend_from_slice(&chunk[..read]);
        }
        stream
            .write_all(&message)
            .expect("Given an open connection, writing the response should always succeed");
    });

    let client = reqwest::Client::builder()
        .no_proxy()
        .build()
        .expect("Given default settings, building a reqwest client should always succeed");
    let response = client
        .get(format!("http://{address}/"))
        .send()
        .await
        .expect("Given a local server that answers, the request should always succeed");
    server
        .join()
        .expect("Given a server thread that does not panic, joining it should always succeed");
    response
}
