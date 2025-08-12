use std::fmt;

pub mod constants;

pub use constants::{BATCH_EXTRACTOR, BATCH_LOADER, BATCH_TRANSFORMER};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum ConnectorType {
    /// Used to create a connection for batch extraction processes.
    BatchExtractor,
    /// Used to create a connection for batch transformation processes.
    BatchTransformer,
    /// Used to create a connection for batch loading processes.
    BatchLoader,
}

impl From<ConnectorType> for String {
    fn from(val: ConnectorType) -> Self {
        match val {
            ConnectorType::BatchExtractor => BATCH_EXTRACTOR.to_string(),
            ConnectorType::BatchTransformer => BATCH_TRANSFORMER.to_string(),
            ConnectorType::BatchLoader => BATCH_LOADER.to_string(),
        }
    }
}

impl fmt::Display for ConnectorType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = match self {
            Self::BatchExtractor => BATCH_EXTRACTOR,
            Self::BatchTransformer => BATCH_TRANSFORMER,
            Self::BatchLoader => BATCH_LOADER,
        };
        write!(f, "{s}")
    }
}

#[cfg(test)]
mod tests {
    use pretty_assertions::assert_eq;

    use super::*;

    #[test]
    fn should_return_correct_string_when_batch_extractor_into_string_is_called() {
        let connector_type = ConnectorType::BatchExtractor;

        let expected_result = "batch-extractor";

        let result: String = connector_type.into();

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_return_correct_string_when_batch_transformer_into_string_is_called() {
        let connector_type = ConnectorType::BatchTransformer;

        let expected_result = "batch-transformer";

        let result: String = connector_type.into();

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_return_correct_string_when_batch_loader_into_string_is_called() {
        let connector_type = ConnectorType::BatchLoader;

        let expected_result = "batch-loader";

        let result: String = connector_type.into();

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_format_correctly_when_display_is_used() {
        let connector_type = ConnectorType::BatchExtractor;

        let expected_result = "batch-extractor";

        let result = format!("{connector_type}");

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_be_equal_when_same_connection_types_are_compared() {
        let connector_type1 = ConnectorType::BatchExtractor;
        let connector_type2 = ConnectorType::BatchExtractor;

        assert_eq!(connector_type1, connector_type2);
    }

    #[test]
    fn should_not_be_equal_when_different_connection_types_are_compared() {
        let connector_type1 = ConnectorType::BatchExtractor;
        let connector_type2 = ConnectorType::BatchTransformer;

        assert_ne!(connector_type1, connector_type2);
    }
}
