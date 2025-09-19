FROM rabbitmq:4.1.2-management-alpine


# Copy configuration files from the correct directory structure
COPY infra/services/queues/batch-etl/config/ /etc/rabbitmq/

# Expose ports
EXPOSE 5672 15672

# Add healthcheck
HEALTHCHECK --interval=5s --timeout=3s --retries=2 --start-period=10s \
  CMD rabbitmq-diagnostics check_running

# Set container name via label
LABEL container.name="arkad-test-service"

# The base image already has the correct CMD, so we don't need to override it
