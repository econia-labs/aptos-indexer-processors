-- Get the total proceeds of an exit event in APT.
CREATE FUNCTION apt_proceeds(exit arena_exit_events) RETURNS numeric
    LANGUAGE SQL
    IMMUTABLE
    RETURN ROUND(
        exit.emojicoin_0_proceeds / exit.emojicoin_0_exchange_rate_base * exit.emojicoin_0_exchange_rate_quote +
        exit.emojicoin_1_proceeds / exit.emojicoin_1_exchange_rate_base * exit.emojicoin_1_exchange_rate_quote
    );

-- Get the curve price of a market at a certain transaction version.
CREATE FUNCTION price_at_txn(market_id numeric, txn numeric) RETURNS numeric AS $$
    WITH latest_swap AS (
        SELECT
            transaction_version,
            event_index,
            lp_coin_supply,
            clamm_virtual_reserves_quote,
            clamm_virtual_reserves_base,
            cpamm_real_reserves_quote,
            cpamm_real_reserves_base
        FROM swap_events
        WHERE market_id = $1
        AND transaction_version <= $2
        ORDER BY transaction_version DESC, event_index DESC LIMIT 1
    ), latest_liquidity AS (
        SELECT
            transaction_version,
            event_index,
            lp_coin_supply,
            clamm_virtual_reserves_quote,
            clamm_virtual_reserves_base,
            cpamm_real_reserves_quote,
            cpamm_real_reserves_base
        FROM liquidity_events
        WHERE market_id = $1
        AND transaction_version <= $2
        ORDER BY transaction_version DESC, event_index DESC LIMIT 1
    ),
    market_reserves_before_txn AS (
        SELECT * FROM (
            SELECT * FROM latest_swap
            UNION
            SELECT * FROM latest_liquidity
        ) AS a ORDER BY transaction_version DESC, event_index DESC LIMIT 1
    ) SELECT
        CASE WHEN lp_coin_supply = 0
            THEN clamm_virtual_reserves_quote / clamm_virtual_reserves_base
            ELSE cpamm_real_reserves_quote / cpamm_real_reserves_base
        END
    FROM market_reserves_before_txn;
$$
    LANGUAGE SQL
    STABLE;
