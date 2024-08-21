#!/bin/bash

STARTING_VERSION=$(psql -d "$DATABASE_URL" -t -c "SELECT last_success_version FROM processor_status;" 2>/dev/null | tr -d '[:space:]')
if [ -z "$STARTING_VERSION" ]; then
  STARTING_VERSION=0
fi
export STARTING_VERSION

echo "health_check_port: 8084
server_config:
  processor_config:
    type: emojicoin_processor
  postgres_connection_string: $DATABASE_URL
  indexer_grpc_data_service_address: $GRPC_DATA_SERVICE_URL
  indexer_grpc_http2_ping_interval_in_secs: 60
  indexer_grpc_http2_ping_timeout_in_secs: 10
  auth_token: $GRPC_AUTH_TOKEN
  starting_version: $STARTING_VERSION
  # BE CAREFUL CHANGING THIS VALUE. Concurrent processing may result in undefined behavior.
  number_concurrent_processing_tasks: 1
  transaction_filter:
    focus_user_transactions: true" > /app/config.yaml

/usr/local/bin/processor --config-path /app/config.yaml
