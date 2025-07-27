use std::fmt;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum ConnectorKind {
    /// Connection used for batch extraction processes.
    BatchExtractor,
    /// Connection used for batch transformation processes.
    BatchTransformer,
    /// Connection used for batch loading processes.
    BatchLoader,
}

impl From<ConnectorKind> for String {
    fn from(val: ConnectorKind) -> Self {
        match val {
            ConnectorKind::BatchExtractor => "batch-extractor".to_string(),
            ConnectorKind::BatchTransformer => "batch-transformer".to_string(),
            ConnectorKind::BatchLoader => "batch-loader".to_string(),
        }
    }
}

impl fmt::Display for ConnectorKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", <Self as Into<String>>::into(*self))
    }
}

#[cfg(test)]
mod tests {
    use pretty_assertions::assert_eq;

    use super::*;

    #[test]
    fn should_return_correct_string_when_batch_extractor_into_string_is_called() {
        let connection_kind = ConnectorKind::BatchExtractor;

        let expected_result = "batch-extractor";

        let result: String = connection_kind.into();

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_return_correct_string_when_batch_transformer_into_string_is_called() {
        let connection_kind = ConnectorKind::BatchTransformer;

        let expected_result = "batch-transformer";

        let result: String = connection_kind.into();

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_return_correct_string_when_batch_loader_into_string_is_called() {
        let connection_kind = ConnectorKind::BatchLoader;

        let expected_result = "batch-loader";

        let result: String = connection_kind.into();

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_format_correctly_when_display_is_used() {
        let connection_kind = ConnectorKind::BatchExtractor;

        let expected_result = "batch-extractor";

        let result = format!("{connection_kind}");

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_be_equal_when_same_connection_kinds_are_compared() {
        let kind1 = ConnectorKind::BatchExtractor;
        let kind2 = ConnectorKind::BatchExtractor;

        assert_eq!(kind1, kind2);
    }

    #[test]
    fn should_not_be_equal_when_different_connection_kinds_are_compared() {
        let kind1 = ConnectorKind::BatchExtractor;
        let kind2 = ConnectorKind::BatchTransformer;

        assert_ne!(kind1, kind2);
    }
}
