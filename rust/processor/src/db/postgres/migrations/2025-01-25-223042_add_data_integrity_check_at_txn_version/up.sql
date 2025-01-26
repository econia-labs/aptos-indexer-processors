-- Your SQL goes here

-- This function atomically returns the state of all market states at a single
-- point in time- specifically, when the transaction version is equal to the
-- `last_success_version` returned.
--
-- It's possible to verify the data integrity of the database to some extent
-- by comparing the values of global on-chain state against these aggregate
-- values.
--
-- Note that the latest global state event data in the database isn't returned here
-- because it's not the latest global state, it's the latest *emitted* global state.
--
-- To verify the data integrity at a specific transaction version, retrieve the
-- on-chain global state resource where the transaction version specified is the
-- `last_success_version` that this function returns, and then compare all values.
CREATE FUNCTION aggregate_market_data() RETURNS TABLE(
  -- `processor_status` columns at the exact time of the query.
  last_success_version BIGINT,
  last_updated TIMESTAMP,
  last_transaction_timestamp TIMESTAMP,

  -- Aggregate number of markets.
  num_markets BIGINT,
  num_markets_in_bonding_curve BIGINT,
  num_markets_post_bonding_curve BIGINT,

  -- Globally aggregated market state data.
  aggregate_quote_volume NUMERIC,
  aggregate_total_quote_locked NUMERIC,
  aggregate_total_value_locked NUMERIC,
  aggregate_market_cap NUMERIC,
  aggregate_fully_diluted_value NUMERIC,
  aggregate_integrator_fees NUMERIC,
  aggregate_num_swaps BIGINT,
  aggregate_num_chat_messages BIGINT,
  aggregate_market_nonces BIGINT
)
AS $$
WITH aggregate_market_states AS (
    SELECT
        COUNT(*) as num_markets,
        COUNT(*) FILTER (WHERE in_bonding_curve = true) as num_markets_in_bonding_curve,
        COUNT(*) FILTER (WHERE in_bonding_curve = false) as num_markets_post_bonding_curve,
        SUM(cumulative_stats_quote_volume) as aggregate_quote_volume,
        SUM(instantaneous_stats_total_quote_locked) as aggregate_total_quote_locked,
        SUM(instantaneous_stats_total_value_locked) as aggregate_total_value_locked,
        SUM(instantaneous_stats_market_cap) as aggregate_market_cap,
        SUM(instantaneous_stats_fully_diluted_value) as aggregate_fully_diluted_value,
        SUM(cumulative_stats_integrator_fees) as aggregate_integrator_fees,
        SUM(cumulative_stats_n_swaps) as aggregate_num_swaps,
        SUM(cumulative_stats_n_chat_messages) as aggregate_num_chat_messages,
        SUM(market_nonce) as aggregate_market_nonces
    FROM market_state
)
SELECT
    ps.last_success_version,
    ps.last_updated,
    ps.last_transaction_timestamp,
    agg_ms.num_markets,
    agg_ms.num_markets_in_bonding_curve,
    agg_ms.num_markets_post_bonding_curve,
    agg_ms.aggregate_quote_volume,
    agg_ms.aggregate_total_quote_locked,
    agg_ms.aggregate_total_value_locked,
    agg_ms.aggregate_market_cap,
    agg_ms.aggregate_fully_diluted_value,
    agg_ms.aggregate_integrator_fees,
    agg_ms.aggregate_num_swaps,
    agg_ms.aggregate_num_chat_messages,
    agg_ms.aggregate_market_nonces
FROM
    processor_status as ps,
    aggregate_market_states as agg_ms;
$$ LANGUAGE SQL;
