#!/bin/sh

psql "$DATABASE_URL" -c '\copy processor_status to /processor_status.csv csv'

if [ -s "/processor_status.csv" ];then
    export STARTING_VERSION="$(cut -d, -f2 /processor_status.csv)"
fi

rm /processor_status.csv

echo "health_check_port: 8084
server_config:
  processor_config:
    type: events_processor
    contract_address: \"$CONTRACT_ADDRESS\"
  postgres_connection_string: $DATABASE_URL
  indexer_grpc_data_service_address: $GRPC_DATA_SERVICE_URL
  indexer_grpc_http2_ping_interval_in_secs: 60
  indexer_grpc_http2_ping_timeout_in_secs: 10
  auth_token: $GRPC_AUTH_TOKEN
  starting_version: $STARTING_VERSION" > config.yaml

/usr/local/bin/processor --config-path config.yaml
