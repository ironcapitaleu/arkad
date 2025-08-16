pub mod constants;

pub use constants::{CONNECTOR_CONFIG_MAP, EMPTY_VEC};

use crate::queue::shared::{ChannelConfig, ChannelType, ConnectorType, QueueIdentifier};

pub struct Mapper;

impl Mapper {
    pub fn retrieve_channel_configs_for_connector(
        connector: &ConnectorType,
    ) -> &'static Vec<ChannelConfig> {
        CONNECTOR_CONFIG_MAP.get(connector).unwrap_or(&EMPTY_VEC)
    }

    pub fn retrieve_accessible_queues_for_connector(
        connector: &ConnectorType,
    ) -> Vec<QueueIdentifier> {
        CONNECTOR_CONFIG_MAP
            .get(connector)
            .map(|configs| {
                configs
                    .iter()
                    .map(|config| config.queue_identifier)
                    .collect::<std::collections::HashSet<_>>()
                    .into_iter()
                    .collect()
            })
            .unwrap_or_default()
    }

    pub fn list_connector_permissions_on_queue(
        connector: &ConnectorType,
        queue_id: &QueueIdentifier,
    ) -> Vec<ChannelType> {
        CONNECTOR_CONFIG_MAP
            .get(connector)
            .map(|configs| {
                configs
                    .iter()
                    .filter(|config| config.queue_identifier == *queue_id)
                    .map(|config| config.channel_type)
                    .collect()
            })
            .unwrap_or_default()
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashSet;

    use pretty_assertions::assert_eq;

    use super::*;
    use crate::queue::shared::{ChannelType, ConnectorType, QueueIdentifier};

    #[test]
    fn should_retrieve_correct_list_of_queue_ids_when_passing_batch_extractor_connector_type() {
        let connector_type = ConnectorType::BatchExtractor;

        let expected_result = vec![QueueIdentifier::BatchExtractor];

        let result = Mapper::retrieve_accessible_queues_for_connector(&connector_type);

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_retrieve_correct_list_of_queue_ids_when_passing_batch_transformer_connector_type() {
        let connector_type = ConnectorType::BatchTransformer;

        let expected_result: HashSet<QueueIdentifier> = vec![
            QueueIdentifier::BatchExtractor,
            QueueIdentifier::BatchTransformer,
        ]
        .into_iter()
        .collect();

        let result: HashSet<QueueIdentifier> =
            Mapper::retrieve_accessible_queues_for_connector(&connector_type)
                .into_iter()
                .collect();

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_retrieve_correct_list_of_queue_ids_when_passing_batch_loader_connector_type() {
        let connector_type = ConnectorType::BatchLoader;

        let expected_result: HashSet<QueueIdentifier> = vec![QueueIdentifier::BatchTransformer]
            .into_iter()
            .collect();

        let result: HashSet<QueueIdentifier> =
            Mapper::retrieve_accessible_queues_for_connector(&connector_type)
                .into_iter()
                .collect();

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_retrieve_correct_permissions_when_passing_batch_extractor_connector_type_and_batch_extractor_queue_id()
     {
        let connector_type = ConnectorType::BatchExtractor;
        let queue_identifier = QueueIdentifier::BatchExtractor;

        let expected_result = vec![ChannelType::Producer];

        let result =
            Mapper::list_connector_permissions_on_queue(&connector_type, &queue_identifier);

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_retrieve_correct_permissions_when_passing_batch_extractor_connector_type_and_batch_transformer_queue_id()
     {
        let connector_type = ConnectorType::BatchExtractor;
        let queue_identifier = QueueIdentifier::BatchTransformer;

        let expected_result = vec![];

        let result =
            Mapper::list_connector_permissions_on_queue(&connector_type, &queue_identifier);

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_retrieve_correct_permissions_when_passing_batch_extractor_connector_type_and_batch_loader_queue_id()
     {
        let connector_type = ConnectorType::BatchExtractor;
        let queue_identifier = QueueIdentifier::BatchLoader;

        let expected_result = vec![];

        let result =
            Mapper::list_connector_permissions_on_queue(&connector_type, &queue_identifier);

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_retrieve_correct_permissions_when_passing_batch_transformer_connector_type_and_batch_extractor_queue_id()
     {
        let connector_type = ConnectorType::BatchTransformer;
        let queue_identifier = QueueIdentifier::BatchExtractor;

        let expected_result = vec![ChannelType::Consumer];

        let result =
            Mapper::list_connector_permissions_on_queue(&connector_type, &queue_identifier);

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_retrieve_correct_permissions_when_passing_batch_transformer_connector_type_and_batch_transformer_queue_id()
     {
        let connector_type = ConnectorType::BatchTransformer;
        let queue_identifier = QueueIdentifier::BatchTransformer;

        let expected_result = vec![ChannelType::Producer];

        let result =
            Mapper::list_connector_permissions_on_queue(&connector_type, &queue_identifier);

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_retrieve_correct_permissions_when_passing_batch_transformer_connector_type_and_batch_loader_queue_id()
     {
        let connector_type = ConnectorType::BatchTransformer;
        let queue_identifier = QueueIdentifier::BatchLoader;

        let expected_result = vec![];

        let result =
            Mapper::list_connector_permissions_on_queue(&connector_type, &queue_identifier);

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_retrieve_correct_permissions_when_passing_batch_loader_connector_type_and_batch_extractor_queue_id()
     {
        let connector_type = ConnectorType::BatchLoader;
        let queue_identifier = QueueIdentifier::BatchExtractor;

        let expected_result = vec![];

        let result =
            Mapper::list_connector_permissions_on_queue(&connector_type, &queue_identifier);

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_retrieve_correct_permissions_when_passing_batch_loader_connector_type_and_batch_transformer_queue_id()
     {
        let connector_type = ConnectorType::BatchLoader;
        let queue_identifier = QueueIdentifier::BatchTransformer;

        let expected_result = vec![ChannelType::Consumer];

        let result =
            Mapper::list_connector_permissions_on_queue(&connector_type, &queue_identifier);

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_retrieve_correct_permissions_when_passing_batch_loader_connector_type_and_batch_loader_queue_id()
     {
        let connector_type = ConnectorType::BatchLoader;
        let queue_identifier = QueueIdentifier::BatchLoader;

        let expected_result = vec![];

        let result =
            Mapper::list_connector_permissions_on_queue(&connector_type, &queue_identifier);

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_return_correct_channel_configs_when_passing_batch_extractor_connector() {
        let connector_type = ConnectorType::BatchExtractor;

        let expected_result: HashSet<ChannelConfig> = vec![ChannelConfig {
            channel_type: ChannelType::Producer,
            queue_identifier: QueueIdentifier::BatchExtractor,
        }]
        .into_iter()
        .collect();

        let result: HashSet<ChannelConfig> =
            Mapper::retrieve_channel_configs_for_connector(&connector_type)
                .iter()
                .copied()
                .collect();

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_return_correct_channel_configs_when_passing_batch_transformer_connector() {
        let connector_type = ConnectorType::BatchTransformer;

        let expected_result: HashSet<ChannelConfig> = vec![
            ChannelConfig {
                channel_type: ChannelType::Consumer,
                queue_identifier: QueueIdentifier::BatchExtractor,
            },
            ChannelConfig {
                channel_type: ChannelType::Producer,
                queue_identifier: QueueIdentifier::BatchTransformer,
            },
        ]
        .into_iter()
        .collect();

        let result: HashSet<ChannelConfig> =
            Mapper::retrieve_channel_configs_for_connector(&connector_type)
                .iter()
                .copied()
                .collect();

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_return_correct_channel_configs_when_passing_batch_loader_connector() {
        let connector_type = ConnectorType::BatchLoader;

        let expected_result: HashSet<ChannelConfig> = vec![ChannelConfig {
            channel_type: ChannelType::Consumer,
            queue_identifier: QueueIdentifier::BatchTransformer,
        }]
        .into_iter()
        .collect();

        let result: HashSet<ChannelConfig> =
            Mapper::retrieve_channel_configs_for_connector(&connector_type)
                .iter()
                .copied()
                .collect();

        assert_eq!(result, expected_result);
    }
}
