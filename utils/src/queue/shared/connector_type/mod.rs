pub mod constants;

pub use constants::{BATCH_EXTRACTOR, BATCH_LOADER, BATCH_TRANSFORMER};

pub enum ConnectorType {
    BatchExtractor,
    BatchTransformer,
    BatchLoader,
}

#[cfg(test)]
mod tests {
    use pretty_assertions::assert_eq;

    use super::*;

    #[test]
    fn should_have_correct_value_when_batch_extractor_constant_is_used() {
        let expected_result = "batch-extractor";

        let result = BATCH_EXTRACTOR;

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_have_correct_value_when_batch_transformer_constant_is_used() {
        let expected_result = "batch-transformer";

        let result = BATCH_TRANSFORMER;

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_have_correct_value_when_batch_loader_constant_is_used() {
        let expected_result = "batch-loader";

        let result = BATCH_LOADER;

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_not_be_equal_when_different_constants_are_compared() {
        let unexpected_result = BATCH_TRANSFORMER;

        let result = BATCH_EXTRACTOR;

        assert_ne!(result, unexpected_result);
    }
}
