-- Your SQL goes here
CREATE FUNCTION user_pools(provider text) RETURNS TABLE(
  transaction_version BIGINT,
  sender VARCHAR(66),
  entry_function VARCHAR(200),
  transaction_timestamp TIMESTAMP,
  inserted_at TIMESTAMP,

  -- Market and state metadata.
  market_id BIGINT,
  symbol_bytes BYTEA,
  symbol_emojis TEXT[],
  bump_time TIMESTAMP, -- Note that bump and emit time are interchangeable.
  market_nonce BIGINT,
  trigger trigger_type,
  market_address VARCHAR(66),

  -- State event data.
  clamm_virtual_reserves_base BIGINT,
  clamm_virtual_reserves_quote BIGINT,
  cpamm_real_reserves_base BIGINT,
  cpamm_real_reserves_quote BIGINT,
  lp_coin_supply NUMERIC,
  cumulative_stats_base_volume NUMERIC,
  cumulative_stats_quote_volume NUMERIC,
  cumulative_stats_integrator_fees NUMERIC,
  cumulative_stats_pool_fees_base NUMERIC,
  cumulative_stats_pool_fees_quote NUMERIC,
  cumulative_stats_n_swaps BIGINT,
  cumulative_stats_n_chat_messages BIGINT,
  instantaneous_stats_total_quote_locked BIGINT,
  instantaneous_stats_total_value_locked NUMERIC,
  instantaneous_stats_market_cap NUMERIC,
  instantaneous_stats_fully_diluted_value NUMERIC,
  last_swap_is_sell BOOLEAN,
  last_swap_avg_execution_price_q64 NUMERIC,
  last_swap_base_volume BIGINT,
  last_swap_quote_volume BIGINT,
  last_swap_nonce BIGINT,
  last_swap_time TIMESTAMP,

  -- Querying all post-bonding curve markets. i.e., markets with liquidity pools.
  daily_tvl_per_lp_coin_growth_q64 NUMERIC,
  in_bonding_curve BOOLEAN,
  volume_in_1m_state_tracker NUMERIC,

  lp_coin_balance BIGINT
)
AS $$
SELECT mlse.*, ulp.lp_coin_balance
FROM
    market_latest_state_event AS mlse,
    user_liquidity_pools AS ulp
WHERE mlse.market_id = ulp.market_id
AND ulp.provider = $1
AND lp_coin_balance <> 0
$$ LANGUAGE SQL;