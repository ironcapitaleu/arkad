use lapin::Connection as LapinConnection;
use tracing::{error, info, warn};
use tracing_test::traced_test;
use pretty_assertions::assert_eq;

use utils::simplequeue::ConnectorBuilder;
use utils::simplequeue::connector::ConnectorKind;
use utils::simplequeue::traits::Connection as ConnectionTrait;

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
    // Try to establish a basic connection using the credentials from the connector
    let connector = ConnectorBuilder::new()
        .user("admin")
        .password("admin123")
        .host("localhost")
        .port(5672u16)
        .vhost("/")
        .connector_kind(ConnectorKind::BatchExtractor)
        .build();
    
    // Test the connector's URI
    let expected_uri = "amqp://admin:admin123@localhost:5672/%2F";

    let actual_uri = connector.uri();
    
    assert_eq!(actual_uri, expected_uri, "Connector URI does not match expected URI");

    // Establish a connection using the connetor's URI
    let lapin_connection =
        match LapinConnection::connect(&actual_uri, lapin::ConnectionProperties::default()).await {
            Ok(conn) => conn,
            Err(e) => {
                warn!(
                    severity = "WARN",
                    message = "Could not establish basic connection for test",
                    error = %e,
                    uri = actual_uri
                );
                return; // Skip test if connection fails
            }
        };

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
                uri = actual_uri,
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
