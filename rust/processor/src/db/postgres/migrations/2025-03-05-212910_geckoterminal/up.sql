CREATE VIEW geckoterminal_events AS
SELECT
    sender,
    block_number,
    transaction_timestamp,
    transaction_version,
    event_index,
    market_address,
    is_sell,
    net_proceeds,
    input_amount,
    avg_execution_price_q64,
    lp_coin_supply,
    clamm_virtual_reserves_base,
    clamm_virtual_reserves_quote,
    cpamm_real_reserves_base,
    cpamm_real_reserves_quote,
    0 as "base_amount",
    0 as "quote_amount",
    'swap' AS "event_type"
FROM swap_events
UNION ALL
SELECT
    sender,
    block_number,
    transaction_timestamp,
    transaction_version,
    event_index,
    market_address,
    false,
    0,
    0,
    0,
    lp_coin_supply,
    clamm_virtual_reserves_base,
    clamm_virtual_reserves_quote,
    cpamm_real_reserves_base,
    cpamm_real_reserves_quote,
    base_amount,
    quote_amount,
    'swap'
FROM liquidity_events;

CREATE VIEW geckoterminal_latest_block AS
WITH lasts AS (
    (
        SELECT block_number, transaction_timestamp
        FROM liquidity_events
        ORDER BY block_number DESC
        LIMIT 1
    )
    UNION ALL
    (
        SELECT block_number, transaction_timestamp
        FROM swap_events
        ORDER BY block_number DESC
        LIMIT 1
    )
)
SELECT block_number, transaction_timestamp
FROM lasts
ORDER BY block_number DESC
LIMIT 1;
