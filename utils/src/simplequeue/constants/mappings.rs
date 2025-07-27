use std::collections::HashMap;
use std::sync::LazyLock;

use crate::simplequeue::channel::{ChannelConfig, ChannelType, QueueIdentifier};
use crate::simplequeue::connector::ConnectorKind;

/// Maps each ConnectorKind to a vector of ChannelConfig.
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
