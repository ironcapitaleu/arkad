use reqwest::Request;

use crate::shared::cik::Cik;

#[derive(Debug)]
pub struct SecRequest {
    pub inner: reqwest::Request,
}

impl SecRequest {
    pub fn new(cik: &Cik) -> Self {

        let url = format!("https://data.sec.gov/submissions/CIK{}.json", cik);
        Self {
            inner: Request::new(
                reqwest::Method::GET,
                reqwest::Url::parse(&url).expect("Hardcoded URL should always be valid."),
            ),
        }
    }

    pub fn request(&self) -> &Request {
        &self.inner
    }
}

impl Clone for SecRequest {
    fn clone(&self) -> Self {
        Self {
            inner: self
                .inner
                .try_clone()
                .expect("Failed to clone Request"),
        }
    }
}

impl PartialEq for SecRequest {
    fn eq(&self, other: &Self) -> bool {
        self.inner.url() == other.inner.url()
    }
}

impl PartialOrd for SecRequest {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.inner.url().cmp(other.inner.url()))
    }
}

impl Ord for SecRequest {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.inner.url().cmp(other.inner.url())
    }
}

impl Eq for SecRequest {}

impl std::hash::Hash for SecRequest {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.inner.url().hash(state);
    }
}

impl Default for SecRequest {
    fn default() -> Self {
        Self {
            inner: Request::new(reqwest::Method::GET, reqwest::Url::parse("https://httpbin.org/get").expect("Hardcoded URL should always be valid.")),
        }
    }
}