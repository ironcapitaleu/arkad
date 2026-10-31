//! # Filing Document Responses
//!
//! Provides [`FilingDocumentResponses`], the responses to a request for the XBRL instance and the
//! label file of one filing.

use std::fmt::{self, Display, Formatter};

use serde::Serialize;

use crate::shared::response::implementations::sec_response::{SecResponse, XmlBody};

/// The response for the XBRL instance of one filing, and the response for its label file if the
/// filing has one.
///
/// Equality, ordering, and hashing compare both responses.
#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize)]
pub struct FilingDocumentResponses {
    instance: SecResponse<XmlBody>,
    labels: Option<SecResponse<XmlBody>>,
}

impl FilingDocumentResponses {
    /// Creates a new [`FilingDocumentResponses`] from the response for the instance and the
    /// response for the label file.
    #[must_use]
    pub const fn new(instance: SecResponse<XmlBody>, labels: Option<SecResponse<XmlBody>>) -> Self {
        Self { instance, labels }
    }

    /// Returns the response for the XBRL instance.
    #[must_use]
    pub const fn instance(&self) -> &SecResponse<XmlBody> {
        &self.instance
    }

    /// Returns the response for the label file, or `None` if the filing has no label file.
    #[must_use]
    pub const fn labels(&self) -> Option<&SecResponse<XmlBody>> {
        self.labels.as_ref()
    }
}

impl Display for FilingDocumentResponses {
    /// Writes the status code and the URL of each response, for example
    /// `Instance: 200 https://…_htm.xml, Label File: None`.
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(f, "Instance: {}, Label File: ", self.instance)?;
        match &self.labels {
            Some(labels) => write!(f, "{labels}"),
            None => write!(f, "None"),
        }
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use pretty_assertions::assert_eq;

    use crate::shared::content_type::ContentType;
    use crate::shared::headers::Headers;
    use crate::shared::response::implementations::sec_response::{SecResponse, XmlBody};
    use crate::shared::status_code::StatusCode;

    use super::FilingDocumentResponses;

    const INSTANCE_URL: &str =
        "https://www.sec.gov/Archives/edgar/data/3545/000000354526000047/alco-20260908_htm.xml";
    const LABELS_URL: &str =
        "https://www.sec.gov/Archives/edgar/data/3545/000000354526000047/alco-20260908_lab.xml";

    fn xml_response(url: &str) -> SecResponse<XmlBody> {
        SecResponse::from_parts(
            url.parse()
                .expect("Given a hardcoded valid URL, parsing should always succeed"),
            Headers::new(HashMap::new()),
            ContentType::Xml,
            StatusCode::Ok,
            b"<xbrl></xbrl>".to_vec(),
        )
        .expect("Given status 200 and an XML content type, building should always succeed")
    }

    #[test]
    fn should_display_both_responses_when_the_filing_has_a_label_file() {
        let responses = FilingDocumentResponses::new(
            xml_response(INSTANCE_URL),
            Some(xml_response(LABELS_URL)),
        );

        let expected_result = format!("Instance: 200 {INSTANCE_URL}, Label File: 200 {LABELS_URL}");

        let result = responses.to_string();

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_display_none_for_the_label_file_when_the_filing_has_none() {
        let responses = FilingDocumentResponses::new(xml_response(INSTANCE_URL), None);

        let expected_result = format!("Instance: 200 {INSTANCE_URL}, Label File: None");

        let result = responses.to_string();

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_serialize_the_instance_and_the_labels_for_filing_document_responses() {
        let responses = FilingDocumentResponses::new(xml_response(INSTANCE_URL), None);

        let expected_result = vec!["instance", "labels"];

        let value = serde_json::to_value(&responses)
            .expect("Given responses of strings and digests, serialization should always succeed");
        let result: Vec<&str> = value
            .as_object()
            .map(|fields| fields.keys().map(String::as_str).collect())
            .unwrap_or_default();

        assert_eq!(result, expected_result);
    }

    const fn implements_state_output_traits<
        T: std::fmt::Debug
            + std::fmt::Display
            + Eq
            + Ord
            + std::hash::Hash
            + serde::Serialize
            + Send
            + Sync
            + Unpin,
    >() {
    }

    #[test]
    const fn should_implement_state_output_traits_for_filing_document_responses() {
        implements_state_output_traits::<FilingDocumentResponses>();
    }
}
