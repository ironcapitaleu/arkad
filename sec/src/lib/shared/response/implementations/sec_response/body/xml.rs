//! # XML Body
//!
//! Provides the [`XmlBody`], the body kind of a response that holds one XML file of a filing, for
//! example the XBRL instance or the label file.

use async_trait::async_trait;
use bytes::Bytes;
use storage::shared::Sha256Digest;

use crate::shared::content_type::ContentType;
use crate::shared::headers::Headers;
use crate::shared::response::implementations::sec_response::error::{
    ErrorReason, InvalidSecResponse,
};
use crate::shared::status_code::StatusCode;

use super::{ResponseBody, check_content_length};

/// The bytes of an XML file, exactly as the server sent them.
///
/// Accepts status 200 only, and the [`ContentType::Xml`] content type, which covers `text/xml`
/// and `application/xml`. If the `Content-Length` header is present, the body has exactly that
/// many bytes.
#[derive(Debug)]
pub struct XmlBody(Bytes);

#[async_trait]
impl ResponseBody for XmlBody {
    type Content = [u8];

    /// Checks that the status code is 200 and the content type is `Xml`.
    ///
    /// # Errors
    ///
    /// Returns an [`InvalidSecResponse`] with [`ErrorReason::InvalidStatusCode`] or
    /// [`ErrorReason::InvalidContentType`] for the first check that fails.
    fn check_status_and_content_type(
        status_code: StatusCode,
        content_type: &ContentType,
    ) -> Result<(), InvalidSecResponse> {
        if status_code != StatusCode::Ok {
            return Err(InvalidSecResponse::new(ErrorReason::InvalidStatusCode {
                status_code,
            }));
        }
        if *content_type != ContentType::Xml {
            return Err(InvalidSecResponse::new(ErrorReason::InvalidContentType {
                content_type: content_type.clone(),
            }));
        }
        Ok(())
    }

    /// Checks the body length against the `Content-Length` header and hashes the body.
    ///
    /// # Errors
    ///
    /// Returns an [`InvalidSecResponse`] with [`ErrorReason::InvalidContentLength`] if the body
    /// length does not match the `Content-Length` header.
    fn from_raw(
        headers: &Headers,
        raw_body: Vec<u8>,
    ) -> Result<(Self, Sha256Digest), InvalidSecResponse> {
        check_content_length(headers, &raw_body)?;
        let body_digest = Sha256Digest::of(&raw_body);
        Ok((Self(Bytes::from(raw_body)), body_digest))
    }

    /// Checks the body length, and hashes the body on the blocking thread pool with
    /// [`Sha256Digest::of_owned_async`], so the task polls other futures during the hash.
    ///
    /// # Errors
    ///
    /// Returns an [`InvalidSecResponse`] with [`ErrorReason::InvalidContentLength`] if the body
    /// length does not match the `Content-Length` header.
    async fn from_read(
        headers: &Headers,
        raw_body: Vec<u8>,
    ) -> Result<(Self, Sha256Digest), InvalidSecResponse> {
        check_content_length(headers, &raw_body)?;
        let (body_digest, raw_body) = Sha256Digest::of_owned_async(raw_body).await;
        Ok((Self(Bytes::from(raw_body)), body_digest))
    }

    fn content(&self) -> &Self::Content {
        &self.0
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use pretty_assertions::assert_eq;

    use crate::shared::content_type::ContentType;
    use crate::shared::headers::Headers;
    use crate::shared::response::SecResponse as SecResponseTrait;
    use crate::shared::response::implementations::sec_response::SecResponse;
    use crate::shared::response::implementations::sec_response::error::{
        ErrorReason, InvalidSecResponse,
    };
    use crate::shared::status_code::StatusCode;
    use crate::shared::url::Url;

    use super::XmlBody;

    const BODY: &[u8] = b"<xbrl></xbrl>";

    fn sample_url() -> Url {
        "https://www.sec.gov/Archives/edgar/data/3545/000000354526000047/alco-20260908_lab.xml"
            .parse()
            .expect("Given a hardcoded valid URL, parsing should always succeed")
    }

    fn headers(content_type: &str, content_length: Option<&str>) -> Headers {
        let mut raw = HashMap::from([("content-type".to_string(), content_type.to_string())]);
        if let Some(content_length) = content_length {
            raw.insert("content-length".to_string(), content_length.to_string());
        }
        Headers::new(raw)
    }

    fn xml_from_parts(
        headers: Headers,
        status_code: StatusCode,
    ) -> Result<SecResponse<XmlBody>, InvalidSecResponse> {
        let content_type = headers.content_type().clone();
        SecResponse::from_parts(
            sample_url(),
            headers,
            content_type,
            status_code,
            BODY.to_vec(),
        )
    }

    #[test]
    fn should_keep_the_body_unchanged_when_the_content_type_is_text_xml() {
        let headers = headers("text/xml", Some("13"));

        let expected_result = BODY;

        let response = xml_from_parts(headers, StatusCode::Ok).expect(
            "Given status 200, text/xml, and a matching length, building should always succeed",
        );
        let result = response.body();

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_keep_the_body_unchanged_when_the_content_type_is_application_xml() {
        let headers = headers("application/xml", None);

        let expected_result = BODY;

        let response = xml_from_parts(headers, StatusCode::Ok).expect(
            "Given status 200, application/xml, and no length header, building should always succeed",
        );
        let result = response.body();

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_return_invalid_status_code_when_the_status_is_404() {
        let headers = headers("text/xml", None);

        let expected_result = Some(InvalidSecResponse::new(ErrorReason::InvalidStatusCode {
            status_code: StatusCode::NotFound,
        }));

        let result = xml_from_parts(headers, StatusCode::NotFound).err();

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_return_invalid_content_type_when_the_content_type_is_html() {
        let headers = headers("text/html", None);

        let expected_result = Some(InvalidSecResponse::new(ErrorReason::InvalidContentType {
            content_type: ContentType::Html,
        }));

        let result = xml_from_parts(headers, StatusCode::Ok).err();

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_return_invalid_content_length_when_the_body_is_longer_than_the_header() {
        let headers = headers("text/xml", Some("4"));

        let expected_result = Some(InvalidSecResponse::new(ErrorReason::InvalidContentLength {
            content_length: "4".to_string(),
            body_length: BODY.len(),
        }));

        let result = xml_from_parts(headers, StatusCode::Ok).err();

        assert_eq!(result, expected_result);
    }

    const fn implements_auto_traits<T: Send + Sync + Unpin>() {}

    #[test]
    const fn should_implement_auto_traits_for_xml_body() {
        implements_auto_traits::<XmlBody>();
    }
}
