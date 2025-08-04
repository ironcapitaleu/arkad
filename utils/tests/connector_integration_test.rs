use lapin::Connection as LapinConnection;
use tracing::{error, info, warn};
use tracing_test::traced_test;

use utils::simplequeue::connector::{ConnectorKind};
use utils::simplequeue::traits::Connection as ConnectionTrait;
use utils::simplequeue::ConnectorBuilder;

/// Integration test for Connector's create_connection method using real LapinConnection.
/// 
/// This test verifies that the connector properly establishes connections
/// and logs the connection results with actual RabbitMQ connection.
/// 
/// Note: This test requires a running RabbitMQ instance for successful connection.
#[tokio::test]
#[traced_test]
#[ignore = "requires RabbitMQ running"]
async fn should_successfully_create_connection_and_log_result_when_using_lapin_connection() {
    // Arrange
    // Try to establish a basic connection using the same credentials as the connector
    let basic_uri = "amqp://admin:admin123@localhost:5672/%2F";
    let lapin_connection = match LapinConnection::connect(basic_uri, lapin::ConnectionProperties::default()).await {
        Ok(conn) => conn,
        Err(e) => {
            warn!(
                severity = "WARN", 
                message = "Could not establish basic connection for test",
                error = %e,
                uri = basic_uri
            );
            return; // Skip test if connection fails
        }
    };
    let connector = ConnectorBuilder::new()
        .user("admin")
        .password("admin123")
        .host("localhost")
        .port(5672u16)
        .vhost("/")
        .connector_kind(ConnectorKind::BatchExtractor)
        .build();

    // Define
    let expected_uri = "amqp://admin:admin123@localhost:5672/%2F";

    // Act
    info!(
        severity = "INFO",
        message = "Starting connection test with valid inner connection",
        connector_kind = ?connector.kind(),
        host = connector.host(),
        port = connector.port(),
        user = connector.user(),
        vhost = connector.vhost()
    );

    let result = connector.create_connection(lapin_connection).await;

    // Assert
    match &result {
        Ok(connection) => {
            info!(
                severity = "INFO",
                message = "Successfully created DAWG connection",
                uri = expected_uri,
                connection_established = true,
                connector_user = connection.connector().user(),
                connector_host = connection.connector().host()
            );
        }
        Err(error) => {
            error!(
                severity = "ERROR",
                message = "Unexpected connection failure",
                error = %error,
                uri = expected_uri
            );
        }
    }

    assert!(result.is_ok());
}
