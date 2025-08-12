use std::collections::HashMap;
use std::sync::LazyLock;

use crate::queue::implementations::channel::{ChannelConfig, ChannelType, QueueIdentifier};
use crate::queue::implementations::connector::ConnectorKind;

/// Maps each [`ConnectorKind`] to a vector of [`ChannelConfig`].
pub static CONNECTOR_CONFIG_MAP: LazyLock<HashMap<ConnectorKind, Vec<ChannelConfig>>> =
    LazyLock::new(|| {
        let mut m = HashMap::new();
        m.insert(
            ConnectorKind::BatchExtractor,
            vec![ChannelConfig {
                channel_type: ChannelType::Producer,
                queue_identifier: QueueIdentifier::BatchExtractor,
            }],
        );
        m.insert(
            ConnectorKind::BatchTransformer,
            vec![
                ChannelConfig {
                    channel_type: ChannelType::Consumer,
                    queue_identifier: QueueIdentifier::BatchExtractor,
                },
                ChannelConfig {
                    channel_type: ChannelType::Producer,
                    queue_identifier: QueueIdentifier::BatchTransformer,
                },
            ],
        );
        m.insert(
            ConnectorKind::BatchLoader,
            vec![ChannelConfig {
                channel_type: ChannelType::Consumer,
                queue_identifier: QueueIdentifier::BatchTransformer,
            }],
        );
        m
    });

#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::assert_eq;

    #[test]
    fn should_map_batch_extractor_connector_to_producer_access_for_batch_extractor_queue() {
        let configs = CONNECTOR_CONFIG_MAP
            .get(&ConnectorKind::BatchExtractor)
            .expect("`ConnectorKind::BatchExtractor` should always have a config.");

        let expected_result = 1;
        let result = configs.len();
        assert_eq!(result, expected_result);

        let has_producer_access_to_batch_extractor = configs.iter().any(|config| {
            matches!(config.channel_type, ChannelType::Producer)
                && matches!(config.queue_identifier, QueueIdentifier::BatchExtractor)
        });
        assert!(has_producer_access_to_batch_extractor);
    }

    #[test]
    fn should_map_batch_transformer_connector_to_consumer_and_producer_access() {
        let configs = CONNECTOR_CONFIG_MAP
            .get(&ConnectorKind::BatchTransformer)
            .expect("`ConnectorKind::BatchTransformer` should always have a config.");

        let expected_result = 2;
        let result = configs.len();
        assert_eq!(result, expected_result);

        let has_consumer_access_to_batch_extractor = configs.iter().any(|config| {
            matches!(config.channel_type, ChannelType::Consumer)
                && matches!(config.queue_identifier, QueueIdentifier::BatchExtractor)
        });
        assert!(has_consumer_access_to_batch_extractor);

        let has_producer_access_to_batch_transformer = configs.iter().any(|config| {
            matches!(config.channel_type, ChannelType::Producer)
                && matches!(config.queue_identifier, QueueIdentifier::BatchTransformer)
        });
        assert!(has_producer_access_to_batch_transformer);
    }

    #[test]
    fn should_map_batch_loader_connector_to_consumer_access_for_batch_transformer_queue() {
        let configs = CONNECTOR_CONFIG_MAP
            .get(&ConnectorKind::BatchLoader)
            .expect("`ConnectorKind::BatchLoader` should always have a config.");

        let expected_result = 1;
        let result = configs.len();
        assert_eq!(result, expected_result);

        let has_consumer_access_to_batch_transformer = configs.iter().any(|config| {
            matches!(config.channel_type, ChannelType::Consumer)
                && matches!(config.queue_identifier, QueueIdentifier::BatchTransformer)
        });
        assert!(has_consumer_access_to_batch_transformer);
    }
}
