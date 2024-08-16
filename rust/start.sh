#!/bin/bash

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
  number_concurrent_processing_tasks: 1
  transaction_filter:
    # Only allow transactions from these contract addresses
    # focus_contract_addresses:
    #   - "0x0"
    # Skip transactions from these sender addresses
    skip_sender_addresses:
      - "0x07"
    # Skip all transactions that aren't user transactions
    focus_user_transactions: true" > /app/config.yaml

/usr/local/bin/processor -c /app/config.yaml
