use std::io;
use std::pin::Pin;
use std::task::{Context, Poll};

use async_trait::async_trait;
use bytes::Bytes;
use http_body::{Body, Frame};
use reqwest::ResponseBuilderExt;

use crate::shared::http_client::traits::InnerClient;

/// A fake HTTP client that always returns a `reqwest::Response` built from fixed parts.
///
/// Used for testing code that reads a real `reqwest::Response` without a network call. Each call
/// builds a new response, because `reqwest::Response` is not `Clone`. The response URL is the
/// request URL.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct AlwaysRespondingHttpClient {
    /// The HTTP status code of every response.
    pub status: u16,
    /// The header names and values of every response.
    pub headers: Vec<(&'static str, String)>,
    /// The body of every response.
    pub body: Vec<u8>,
    /// Whether every response sends its whole body or stops early.
    pub delivery: Delivery,
}

/// How a fake response delivers its body.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Delivery {
    /// The response sends the whole body.
    Full,
    /// The body stops before its first byte, as on a reset connection.
    StopsEarly,
}

#[async_trait]
impl InnerClient for AlwaysRespondingHttpClient {
    type Request = reqwest::Request;
    type Response = reqwest::Response;
    type Error = reqwest::Error;

    async fn execute_request(&self, request: Self::Request) -> Result<Self::Response, Self::Error> {
        let mut builder = http::Response::builder()
            .status(self.status)
            .url(request.url().clone());
        for (name, value) in &self.headers {
            builder = builder.header(*name, value);
        }
        let body = match self.delivery {
            Delivery::Full => reqwest::Body::from(self.body.clone()),
            Delivery::StopsEarly => reqwest::Body::wrap(StoppedBody),
        };
        let response = builder
            .body(body)
            .expect("Given a valid status code and header names, building the response should always succeed");
        Ok(reqwest::Response::from(response))
    }
}

/// A response body whose first read fails with a reset connection.
struct StoppedBody;

impl Body for StoppedBody {
    type Data = Bytes;
    type Error = io::Error;

    fn poll_frame(
        self: Pin<&mut Self>,
        _context: &mut Context<'_>,
    ) -> Poll<Option<Result<Frame<Self::Data>, Self::Error>>> {
        Poll::Ready(Some(Err(io::Error::from(io::ErrorKind::ConnectionReset))))
    }
}
