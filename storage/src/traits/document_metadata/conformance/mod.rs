//! # Conformance
//!
//! Provides [`shape_violations`], the check that metadata serializes in the shape
//! [`DocumentMetadata`] requires, and the [`ShapeViolation`] it reports.

use serde_json::Value;

use super::DocumentMetadata;

/// Enum representing one way serialized metadata differs from the shape [`DocumentMetadata`]
/// requires.
#[non_exhaustive]
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum ShapeViolation {
    /// The metadata does not serialize to JSON.
    FailedSerialization {
        /// The serializer's message.
        reason: String,
    },
    /// The metadata serializes to a JSON value that is not an object.
    InvalidShape,
    /// A shared field is not at the top level under its exact name.
    MissingSharedKey {
        /// The name the shared field must have.
        key: &'static str,
    },
    /// A shared field holds a value that differs from what its getter returns.
    UnexpectedSharedValue {
        /// The name of the shared field.
        key: &'static str,
    },
    /// The stored `kind` differs from [`DocumentMetadata::KIND`].
    UnexpectedKind {
        /// The kind the metadata type defines.
        expected: &'static str,
    },
    /// The `context` field is not a nested JSON object.
    InvalidContext,
}

/// Returns every way the serialized metadata differs from the shape [`DocumentMetadata`] requires.
///
/// Serializes the metadata to JSON, then checks that:
/// - each shared field sits at the top level under the name of its getter, and holds the value
///   that its getter returns.
/// - the stored `kind` equals [`DocumentMetadata::KIND`].
/// - `context` is a nested JSON object.
///
/// An empty list means the metadata has the required shape.
///
/// # Examples
///
/// ```rust
/// # use chrono::{DateTime, Utc};
/// # use serde::{Deserialize, Serialize};
/// # use storage::traits::document_metadata::conformance::shape_violations;
/// # use storage::{DocumentMetadata, InputDocument, METADATA_VERSION, Producer};
/// # use uuid::Uuid;
/// #
/// # #[derive(Serialize, Deserialize)]
/// # struct FetchContext { url: String }
/// #
/// # #[derive(Serialize, Deserialize)]
/// # struct FetchMetadata {
/// #     metadata_version: u32,
/// #     kind: String,
/// #     media_type: String,
/// #     bytes: u64,
/// #     sha256: String,
/// #     produced_at: DateTime<Utc>,
/// #     execution_id: Uuid,
/// #     producer: Producer,
/// #     inputs: Vec<InputDocument>,
/// #     context: FetchContext,
/// # }
/// #
/// # impl DocumentMetadata for FetchMetadata {
/// #     type Context = FetchContext;
/// #     const KIND: &'static str = "raw_fetch.v1";
/// #     fn metadata_version(&self) -> u32 { self.metadata_version }
/// #     fn kind(&self) -> &str { &self.kind }
/// #     fn media_type(&self) -> &str { &self.media_type }
/// #     fn bytes(&self) -> u64 { self.bytes }
/// #     fn sha256(&self) -> &str { &self.sha256 }
/// #     fn produced_at(&self) -> DateTime<Utc> { self.produced_at }
/// #     fn execution_id(&self) -> Uuid { self.execution_id }
/// #     fn producer(&self) -> &Producer { &self.producer }
/// #     fn inputs(&self) -> &[InputDocument] { &self.inputs }
/// #     fn context(&self) -> &Self::Context { &self.context }
/// # }
/// let metadata = FetchMetadata {
///     metadata_version: METADATA_VERSION,
///     kind: FetchMetadata::KIND.to_owned(),
///     media_type: "application/json".to_owned(),
///     bytes: 2,
///     sha256: "44136fa355b3678a1146ad16f7e8649e94fb4fc21fe77e8310c060f61caaff8a".to_owned(),
///     produced_at: DateTime::<Utc>::UNIX_EPOCH,
///     execution_id: Uuid::nil(),
///     producer: Producer::new("Fetch", "0.1.0", "63c1f59"),
///     inputs: Vec::new(),
///     context: FetchContext { url: "https://example.com/data.json".to_owned() },
/// };
///
/// let expected_result = Vec::new();
///
/// let result = shape_violations(&metadata);
///
/// assert_eq!(result, expected_result);
/// ```
#[must_use]
pub fn shape_violations<M: DocumentMetadata>(metadata: &M) -> Vec<ShapeViolation> {
    let (serialized, shared_values) = match serialize(metadata) {
        Ok(values) => values,
        Err(error) => {
            return vec![ShapeViolation::FailedSerialization {
                reason: error.to_string(),
            }];
        }
    };
    let Value::Object(fields) = serialized else {
        return vec![ShapeViolation::InvalidShape];
    };

    let mut violations = Vec::new();
    for (key, expected) in shared_values {
        match fields.get(key) {
            None => violations.push(ShapeViolation::MissingSharedKey { key }),
            Some(found) if *found != expected => {
                violations.push(ShapeViolation::UnexpectedSharedValue { key });
            }
            Some(_) => {}
        }
    }
    if fields.get("kind").and_then(Value::as_str) != Some(M::KIND) {
        violations.push(ShapeViolation::UnexpectedKind { expected: M::KIND });
    }
    if !fields.get("context").is_some_and(Value::is_object) {
        violations.push(ShapeViolation::InvalidContext);
    }
    violations
}

/// The serialized metadata, and each shared field's name with its getter's serialized value.
type SerializedMetadata = (Value, [(&'static str, Value); 10]);

/// Serializes the whole metadata, and each getter on its own.
fn serialize<M: DocumentMetadata>(metadata: &M) -> Result<SerializedMetadata, serde_json::Error> {
    let shared_values = [
        (
            "metadata_version",
            serde_json::to_value(metadata.metadata_version())?,
        ),
        ("kind", serde_json::to_value(metadata.kind())?),
        ("media_type", serde_json::to_value(metadata.media_type())?),
        ("bytes", serde_json::to_value(metadata.bytes())?),
        ("sha256", serde_json::to_value(metadata.sha256())?),
        ("produced_at", serde_json::to_value(metadata.produced_at())?),
        (
            "execution_id",
            serde_json::to_value(metadata.execution_id())?,
        ),
        ("producer", serde_json::to_value(metadata.producer())?),
        ("inputs", serde_json::to_value(metadata.inputs())?),
        ("context", serde_json::to_value(metadata.context())?),
    ];
    Ok((serde_json::to_value(metadata)?, shared_values))
}

#[cfg(test)]
mod tests {
    use chrono::{DateTime, Utc};
    use pretty_assertions::assert_eq;
    use serde::{Deserialize, Serialize};
    use uuid::Uuid;

    use super::*;
    use crate::shared::input_document::InputDocument;
    use crate::shared::producer::Producer;
    use crate::tests::fixtures::sample_document::{SampleContext, SampleMetadata, sample_metadata};

    /// Metadata whose JSON form is set freely, while its getters return the sample metadata.
    ///
    /// A test edits the JSON form to break one rule of the shape and leaves the getters as they are.
    #[derive(Debug, Serialize, Deserialize)]
    #[serde(transparent)]
    struct LooseMetadata {
        stored: Value,
        #[serde(skip, default = "sample_metadata")]
        getters: SampleMetadata,
    }

    impl LooseMetadata {
        /// Creates metadata that serializes as the sample metadata, edited by `edit`.
        fn edited(edit: impl FnOnce(&mut Value)) -> Self {
            let mut stored = serde_json::to_value(sample_metadata())
                .expect("Given the sample metadata, serialization should always succeed");
            edit(&mut stored);
            Self {
                stored,
                getters: sample_metadata(),
            }
        }
    }

    impl DocumentMetadata for LooseMetadata {
        type Context = SampleContext;

        const KIND: &'static str = SampleMetadata::KIND;

        fn metadata_version(&self) -> u32 {
            self.getters.metadata_version()
        }

        fn kind(&self) -> &str {
            self.getters.kind()
        }

        fn media_type(&self) -> &str {
            self.getters.media_type()
        }

        fn bytes(&self) -> u64 {
            self.getters.bytes()
        }

        fn sha256(&self) -> &str {
            self.getters.sha256()
        }

        fn produced_at(&self) -> DateTime<Utc> {
            self.getters.produced_at()
        }

        fn execution_id(&self) -> Uuid {
            self.getters.execution_id()
        }

        fn producer(&self) -> &Producer {
            self.getters.producer()
        }

        fn inputs(&self) -> &[InputDocument] {
            self.getters.inputs()
        }

        fn context(&self) -> &Self::Context {
            self.getters.context()
        }
    }

    #[test]
    fn should_return_no_violation_for_sample_metadata() {
        let metadata = sample_metadata();

        let expected_result: Vec<ShapeViolation> = Vec::new();

        let result = shape_violations(&metadata);

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_report_a_missing_shared_key_when_a_shared_field_is_renamed() {
        let metadata = LooseMetadata::edited(|stored| {
            let sha256 = stored["sha256"].take();
            stored["sha_256"] = sha256;
            stored
                .as_object_mut()
                .expect("Given serialized struct metadata, the value should always be an object")
                .remove("sha256");
        });

        let expected_result = vec![ShapeViolation::MissingSharedKey { key: "sha256" }];

        let result = shape_violations(&metadata);

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_report_an_unexpected_shared_value_when_a_field_differs_from_its_getter() {
        let metadata = LooseMetadata::edited(|stored| stored["metadata_version"] = 2.into());

        let expected_result = vec![ShapeViolation::UnexpectedSharedValue {
            key: "metadata_version",
        }];

        let result = shape_violations(&metadata);

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_report_an_unexpected_kind_when_the_stored_kind_differs_from_the_kind_constant() {
        let mut metadata = LooseMetadata::edited(|stored| stored["kind"] = "other_kind.v1".into());
        metadata.getters.kind = "other_kind.v1".to_owned();

        let expected_result = vec![ShapeViolation::UnexpectedKind {
            expected: "sample_document.v1",
        }];

        let result = shape_violations(&metadata);

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_report_an_invalid_context_when_the_context_is_flattened_into_the_top_level() {
        let metadata = LooseMetadata::edited(|stored| {
            let fields = stored
                .as_object_mut()
                .expect("Given serialized struct metadata, the value should always be an object");
            let context = fields.remove("context");
            fields.insert(
                "source".to_owned(),
                context.map_or(Value::Null, |mut context| context["source"].take()),
            );
        });

        let expected_result = vec![
            ShapeViolation::MissingSharedKey { key: "context" },
            ShapeViolation::InvalidContext,
        ];

        let result = shape_violations(&metadata);

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_report_an_invalid_shape_when_the_metadata_serializes_to_a_list() {
        let metadata = LooseMetadata::edited(|stored| *stored = Value::Array(Vec::new()));

        let expected_result = vec![ShapeViolation::InvalidShape];

        let result = shape_violations(&metadata);

        assert_eq!(result, expected_result);
    }
}
