/// String constant for batch extractor connector kind.
pub const BATCH_EXTRACTOR: &str = "batch-extractor";

/// String constant for batch transformer connector kind.
pub const BATCH_TRANSFORMER: &str = "batch-transformer";

/// String constant for batch loader connector kind.
pub const BATCH_LOADER: &str = "batch-loader";

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
