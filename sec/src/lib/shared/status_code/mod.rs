use std::fmt;

/// HTTP status code classification for SEC API responses.
///
/// `StatusCode` represents specific HTTP status codes that are relevant to
/// SEC API interactions. Known codes are modeled as explicit variants, while
/// any other valid HTTP status code is captured by the `Other` variant.
///
/// # Examples
///
/// ```rust
/// use sec::shared::status_code::StatusCode;
///
/// let status = StatusCode::from_u16(200);
/// assert_eq!(status, StatusCode::Ok);
/// assert_eq!(status.as_u16(), 200);
/// assert_eq!(status.to_string(), "200");
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[non_exhaustive]
pub enum StatusCode {
    /// 100 Continue.
    Continue,

    /// 200 OK.
    Ok,

    /// 301 Moved Permanently.
    MovedPermanently,

    /// 400 Bad Request.
    BadRequest,

    /// 403 Forbidden.
    Forbidden,

    /// 404 Not Found.
    NotFound,

    /// 429 Too Many Requests.
    TooManyRequests,

    /// 500 Internal Server Error.
    InternalServerError,

    /// 503 Service Unavailable.
    ServiceUnavailable,

    /// Any other valid HTTP status code not explicitly modeled.
    Other(u16),
}

impl StatusCode {
    /// Creates a `StatusCode` from a raw `u16` value.
    #[must_use]
    pub const fn from_u16(code: u16) -> Self {
        match code {
            100 => Self::Continue,
            200 => Self::Ok,
            301 => Self::MovedPermanently,
            400 => Self::BadRequest,
            403 => Self::Forbidden,
            404 => Self::NotFound,
            429 => Self::TooManyRequests,
            500 => Self::InternalServerError,
            503 => Self::ServiceUnavailable,
            _ => Self::Other(code),
        }
    }

    /// Returns the raw `u16` status code value.
    #[must_use]
    pub const fn as_u16(&self) -> u16 {
        match self {
            Self::Continue => 100,
            Self::Ok => 200,
            Self::MovedPermanently => 301,
            Self::BadRequest => 400,
            Self::Forbidden => 403,
            Self::NotFound => 404,
            Self::TooManyRequests => 429,
            Self::InternalServerError => 500,
            Self::ServiceUnavailable => 503,
            Self::Other(code) => *code,
        }
    }

    /// Returns `true` if the status code is informational (100–199).
    #[must_use]
    pub const fn is_informational(&self) -> bool {
        matches!(self.as_u16(), 100..=199)
    }

    /// Returns `true` if the status code is a success (200–299).
    #[must_use]
    pub const fn is_success(&self) -> bool {
        matches!(self.as_u16(), 200..=299)
    }

    /// Returns `true` if the status code is a redirection (300–399).
    #[must_use]
    pub const fn is_redirection(&self) -> bool {
        matches!(self.as_u16(), 300..=399)
    }

    /// Returns `true` if the status code is a client error (400–499).
    #[must_use]
    pub const fn is_client_error(&self) -> bool {
        matches!(self.as_u16(), 400..=499)
    }

    /// Returns `true` if the status code is a server error (500–599).
    #[must_use]
    pub const fn is_server_error(&self) -> bool {
        matches!(self.as_u16(), 500..=599)
    }
}

impl From<reqwest::StatusCode> for StatusCode {
    fn from(status: reqwest::StatusCode) -> Self {
        Self::from_u16(status.as_u16())
    }
}

impl fmt::Display for StatusCode {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.as_u16())
    }
}

#[cfg(test)]
mod tests {
    use pretty_assertions::assert_eq;

    use super::*;

    #[test]
    fn should_return_continue_when_code_is_100() {
        let code: u16 = 100;

        let expected_result = StatusCode::Continue;

        let result = StatusCode::from_u16(code);

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_return_ok_when_code_is_200() {
        let code: u16 = 200;

        let expected_result = StatusCode::Ok;

        let result = StatusCode::from_u16(code);

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_return_moved_permanently_when_code_is_301() {
        let code: u16 = 301;

        let expected_result = StatusCode::MovedPermanently;

        let result = StatusCode::from_u16(code);

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_return_bad_request_when_code_is_400() {
        let code: u16 = 400;

        let expected_result = StatusCode::BadRequest;

        let result = StatusCode::from_u16(code);

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_return_forbidden_when_code_is_403() {
        let code: u16 = 403;

        let expected_result = StatusCode::Forbidden;

        let result = StatusCode::from_u16(code);

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_return_not_found_when_code_is_404() {
        let code: u16 = 404;

        let expected_result = StatusCode::NotFound;

        let result = StatusCode::from_u16(code);

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_return_too_many_requests_when_code_is_429() {
        let code: u16 = 429;

        let expected_result = StatusCode::TooManyRequests;

        let result = StatusCode::from_u16(code);

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_return_internal_server_error_when_code_is_500() {
        let code: u16 = 500;

        let expected_result = StatusCode::InternalServerError;

        let result = StatusCode::from_u16(code);

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_return_service_unavailable_when_code_is_503() {
        let code: u16 = 503;

        let expected_result = StatusCode::ServiceUnavailable;

        let result = StatusCode::from_u16(code);

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_return_other_when_code_is_not_explicitly_modeled() {
        let code: u16 = 201;

        let expected_result = StatusCode::Other(201);

        let result = StatusCode::from_u16(code);

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_return_raw_code_when_calling_as_u16_on_named_variant() {
        let status = StatusCode::NotFound;

        let expected_result = 404_u16;

        let result = status.as_u16();

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_return_raw_code_when_calling_as_u16_on_other_variant() {
        let status = StatusCode::Other(201);

        let expected_result = 201_u16;

        let result = status.as_u16();

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_return_true_when_checking_is_informational_on_continue() {
        let status = StatusCode::Continue;

        assert!(status.is_informational());
    }

    #[test]
    fn should_return_true_when_checking_is_success_on_ok() {
        let status = StatusCode::Ok;

        assert!(status.is_success());
    }

    #[test]
    fn should_return_true_when_checking_is_success_on_other_success_code() {
        let status = StatusCode::Other(204);

        assert!(status.is_success());
    }

    #[test]
    fn should_return_false_when_checking_is_success_on_client_error() {
        let status = StatusCode::NotFound;

        assert!(!status.is_success());
    }

    #[test]
    fn should_return_true_when_checking_is_redirection_on_moved_permanently() {
        let status = StatusCode::MovedPermanently;

        assert!(status.is_redirection());
    }

    #[test]
    fn should_return_true_when_checking_is_client_error_on_too_many_requests() {
        let status = StatusCode::TooManyRequests;

        assert!(status.is_client_error());
    }

    #[test]
    fn should_return_true_when_checking_is_server_error_on_service_unavailable() {
        let status = StatusCode::ServiceUnavailable;

        assert!(status.is_server_error());
    }

    #[test]
    fn should_display_numeric_code_when_formatting_named_variant() {
        let status = StatusCode::Ok;

        let expected_result = "200";

        let result = status.to_string();

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_display_numeric_code_when_formatting_other_variant() {
        let status = StatusCode::Other(201);

        let expected_result = "201";

        let result = status.to_string();

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_convert_from_reqwest_status_code_when_using_from() {
        let reqwest_status = reqwest::StatusCode::NOT_FOUND;

        let expected_result = StatusCode::NotFound;

        let result = StatusCode::from(reqwest_status);

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_convert_to_other_from_reqwest_status_code_when_code_is_not_modeled() {
        let reqwest_status = reqwest::StatusCode::CREATED;

        let expected_result = StatusCode::Other(201);

        let result = StatusCode::from(reqwest_status);

        assert_eq!(result, expected_result);
    }
}
