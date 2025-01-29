-- Your SQL goes here

-- This function atomically returns the state of all market states at a single
-- point in time- specifically, when the transaction version is equal to the
-- `last_emojicoin_transaction_version` returned.
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
-- `last_emojicoin_transaction_version` that this function returns, and then compare
-- all values.
--
-- NOTE: `last_success_version` from the `processor_status` is not synced with
-- new event data inserted and can actually be behind the highest emojicoin
-- transaction version. To properly get the last successfully processed and inserted
-- emojicoin version, it's necessary to get the max transaction version among
-- all transaction versions.
CREATE FUNCTION aggregate_market_state() RETURNS TABLE(
  last_emojicoin_transaction_version BIGINT,

  -- The following columns are structured to match all the `registry_view` fields.
  cumulative_chat_messages NUMERIC,
  cumulative_integrator_fees NUMERIC,
  cumulative_quote_volume NUMERIC,
  cumulative_swaps NUMERIC,
  fully_diluted_value NUMERIC,
  last_bump_time TIMESTAMP,
  market_cap NUMERIC,
  n_markets NUMERIC,
  nonce NUMERIC,
  total_quote_locked NUMERIC,
  total_value_locked NUMERIC,
  
  n_markets_in_bonding_curve NUMERIC,
  n_markets_post_bonding_curve NUMERIC
)
AS $$
SELECT
    MAX(transaction_version) as last_emojicoin_transaction_version,

    -- The following columns mirror the `registry_view` return value structure.
    SUM(cumulative_stats_n_chat_messages) as cumulative_chat_messages,
    SUM(cumulative_stats_integrator_fees) as cumulative_integrator_fees,
    SUM(cumulative_stats_quote_volume) as cumulative_quote_volume,
    SUM(cumulative_stats_n_swaps) as cumulative_swaps,
    SUM(instantaneous_stats_fully_diluted_value) as fully_diluted_value,
    MAX(bump_time) as last_bump_time, 
    SUM(instantaneous_stats_market_cap) as market_cap,
    COUNT(*) as n_markets,
    -- Add one to account for `init_module` incrementing the registry nonce without
    -- incrementing a market's `market_nonce`.
    SUM(market_nonce) + 1 as nonce,
    SUM(instantaneous_stats_total_quote_locked) as total_quote_locked,
    SUM(instantaneous_stats_total_value_locked) as total_value_locked,

    COUNT(*) FILTER (WHERE in_bonding_curve = true) as n_markets_in_bonding_curve,
    COUNT(*) FILTER (WHERE in_bonding_curve = false) as n_markets_post_bonding_curve
FROM market_state
$$ LANGUAGE SQL;
