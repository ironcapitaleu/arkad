# Queue Connection & Channel Design

## Architecture Overview

```mermaid
---
title: "Queue Connection & Channel Design"
---

classDiagram
    %% ConnectionKind enum for hardcoded connection types
    class ConnectionKind {
        <<enum>>
        +BatchExtractor
        +BatchTransformer
        +BatchLoader
    }

    %% Connector holds connection parameters (host, port, vhost, etc.)
    class Connector {
        <<struct>>
        +user: String
        +password: String
        +host: String
        +port: u16
        +vhost: String
        +kind: ConnectionKind
        +uri() String
        +create_connection() Result~Connection, ConnectionFailed~
    }

    %% Connection struct holds the lapin connection and connector info
    class Connection {
        <<struct>>
        +inner: LapinConnection
        +connector: Connector
        +name() String
        +create_producer_channel(queue_name: String) Result~ProducerChannel, ChannelError~
        +create_consumer_channel(queue_name: String) Result~ConsumerChannel, ChannelError~
        +is_connected() bool
        +close() Result~(), ConnectionFailed~
    }

    %% ProducerChannel knows the queue name and connection
    class ProducerChannel {
        <<struct>>
        +inner: LapinChannel
        +queue_name: String
        +connection_kind: ConnectionKind
        +send(message: &[u8]) Result~(), PublishError~
        +declare_queue() Result~(), QueueError~
    }

    %% ConsumerChannel knows the queue name and connection
    class ConsumerChannel {
        <<struct>>
        +inner: LapinChannel
        +queue_name: String
        +connection_kind: ConnectionKind
        +receive() Result~Message, ConsumeError~
        +declare_queue() Result~(), QueueError~
        +start_consuming() Result~Consumer, ConsumeError~
    }

    %% Error types
    class ErrorKind {
        <<enum>>
        +ConnectionFailed(ConnectionFailed)
        +ChannelError(ChannelError)
        +PublishError(PublishError)
        +ConsumeError(ConsumeError)
        +QueueError(QueueError)
    }

    %% Relationships
    ConnectionKind <|-- Connector : has
    Connector <|-- Connection : created_with
    Connection o-- ProducerChannel : creates
    Connection o-- ConsumerChannel : creates
    ProducerChannel --> ConnectionKind : logs_with
    ConsumerChannel --> ConnectionKind : logs_with
    ErrorKind o-- ConnectionFailed : contains
    ErrorKind o-- ChannelError : contains
    ErrorKind o-- PublishError : contains
    ErrorKind o-- ConsumeError : contains
    ErrorKind o-- QueueError : contains
```

## Usage Examples

### Basic Connection and Channel Creation

```rust
use utils::queue::{Connector, ConnectionKind, ConnectorBuilder};

// Create a connector with hardcoded connection type
let connector = ConnectorBuilder::new()
    .user("admin")
    .password("secret")
    .host("localhost")
    .port(5672)
    .vhost("/")
    .kind(ConnectionKind::BatchxExtractor)
    .build();

// Establish connection
let connection = connector.create_connection().await?;

// Create producer channel
let producer = connection.create_producer_channel("batch.extraction.results").await?;
producer.declare_queue().await?;
producer.send(b"Hello, batch!").await?;

// Create consumer channel
let consumer = connection.create_consumer_channel("batch.extraction.results").await?;
consumer.declare_queue().await?;
let message = consumer.receive().await?;
```

### Shared Connection Across Threads

```rust
use std::sync::Arc;
use tokio::task;

let connection = Arc::new(connector.create_connection().await?);

// Producer thread
let producer_connection = Arc::clone(&connection);
let producer_handle = task::spawn(async move {
    let producer = producer_connection
        .create_producer_channel("worker.tasks")
        .await?;
    producer.send(b"task data").await
});

// Consumer thread
let consumer_connection = Arc::clone(&connection);
let consumer_handle = task::spawn(async move {
    let consumer = consumer_connection
        .create_consumer_channel("worker.tasks")
        .await?;
    consumer.receive().await
});

// Wait for both threads
let (producer_result, consumer_result) = tokio::join!(producer_handle, consumer_handle);
```
