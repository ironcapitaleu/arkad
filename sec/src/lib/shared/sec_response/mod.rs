use reqwest::Response;

#[derive(Debug)]
pub struct SecResponse {
    pub inner: Response,
}

impl SecResponse {
    #[must_use]
    /// Creates a new [`SecResponse`] which wraps a [`Response`].
    pub const fn new(inner: Response) -> Self {
        Self { inner }
    }

    #[must_use] pub const fn response(&self) -> &Response {
        &self.inner
    }
}

impl PartialEq for SecResponse {
    fn eq(&self, other: &Self) -> bool {
        self.inner.url() == other.inner.url()
    }
}

impl PartialOrd for SecResponse {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for SecResponse {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.inner.url().cmp(other.inner.url())
    }
}

impl Eq for SecResponse {}

impl std::hash::Hash for SecResponse {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.inner.url().hash(state);
    }
}
