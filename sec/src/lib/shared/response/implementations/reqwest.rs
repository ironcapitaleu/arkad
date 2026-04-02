use reqwest::{Method, Response, Url};

use super::super::traits::InnerResponse;

impl InnerResponse for Response {
    type Method = Method;
    type Url = Url;
    type Body = String;
    type Header = String;
    type StatusCode = u16;
    type ContentType = String;
    type Error = String; // TODO:  check if needed, if not delete --- IGNORE ---

    /// Returns the HTTP method of the request.
    fn method(&self) -> &Self::Method {
        self.method()
    }

    /// Returns the URL endpoint of the HTTP request.
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
