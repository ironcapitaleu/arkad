//! # Reqwest Inner Request
//!
//! Implements [`InnerRequest`] for [`reqwest::Request`], binding the transport's method and URL
//! types and delegating to `reqwest`.

use reqwest::{Method, Request, Url};

use super::super::traits::InnerRequest;

impl InnerRequest for Request {
    /// The [`reqwest::Method`] type.
    type Method = Method;
    /// The [`reqwest::Url`] type.
    type Url = Url;

    fn new(method: Method, url: Url) -> Self {
        Self::new(method, url)
    }

    fn method(&self) -> &Self::Method {
        self.method()
    }

    fn url(&self) -> &Self::Url {
        self.url()
    }
}

#[cfg(test)]
mod tests {
    use pretty_assertions::assert_eq;
    use reqwest::{Method, Request, Url};

    use crate::shared::request::traits::inner::InnerRequest;
    #[test]
    fn should_create_same_request_with_same_method_when_using_trait_constructor() {
        let method = Method::GET;
        let url = Url::parse("https://example.com").expect("Hardcoded URL should always be valid");

        let expected_result = Request::new(method.clone(), url.clone()).method().clone();

        let result = <Request as InnerRequest>::new(method, url).method().clone();

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_create_same_request_with_same_url_when_using_trait_constructor() {
        let method = Method::GET;
        let url = Url::parse("https://example.com").expect("Hardcoded URL should always be valid");

        let expected_result = Request::new(method.clone(), url.clone()).url().clone();

        let result = <Request as InnerRequest>::new(method, url).url().clone();

        assert_eq!(result, expected_result);
    }
}
