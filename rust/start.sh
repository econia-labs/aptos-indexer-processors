#!/bin/bash

# Attempt to get the latest processed transaction version from the database.
psql $DATABASE_URL -c '\copy processor_status to /app/out.csv csv'

# If the file is not empty, then the database has been initialized and there
# is a starting version to use. In this case, overwrite the environment
# variable set by the container with the value from the database.
if [ -s "/app/out.csv" ];then
    export STARTING_VERSION=$(cut -d, -f2 /app/out.csv)
fi
rm /app/out.csv

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
