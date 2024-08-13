-- Your SQL goes here

CREATE TABLE latest_market_state (
  -- Transaction metadata.
  transaction_version BIGINT NOT NULL,
  sender VARCHAR(66) NOT NULL,
  entry_function VARCHAR(200),
  transaction_timestamp TIMESTAMP NOT NULL,
  inserted_at TIMESTAMP NOT NULL DEFAULT NOW(),

  -- Market and state metadata.
  market_id BIGINT NOT NULL,
  symbol_bytes BYTEA NOT NULL,
  bump_time TIMESTAMP NOT NULL, -- Note that bump and emit time are interchangeable.
  market_nonce BIGINT NOT NULL,
  trigger triggers NOT NULL,

  -- State event data.
  clamm_virtual_reserves_base BIGINT NOT NULL,
  clamm_virtual_reserves_quote BIGINT NOT NULL,
  cpamm_real_reserves_base BIGINT NOT NULL,
  cpamm_real_reserves_quote BIGINT NOT NULL,
  lp_coin_supply NUMERIC NOT NULL,
  cumulative_stats_base_volume NUMERIC NOT NULL,
  cumulative_stats_quote_volume NUMERIC NOT NULL,
  cumulative_stats_integrator_fees NUMERIC NOT NULL,
  cumulative_stats_pool_fees_base NUMERIC NOT NULL,
  cumulative_stats_pool_fees_quote NUMERIC NOT NULL,
  cumulative_stats_n_swaps BIGINT NOT NULL,
  cumulative_stats_n_chat_messages BIGINT NOT NULL,
  instantaneous_stats_total_quote_locked BIGINT NOT NULL,
  instantaneous_stats_total_value_locked NUMERIC NOT NULL,
  instantaneous_stats_market_cap NUMERIC NOT NULL,
  instantaneous_stats_fully_diluted_value NUMERIC NOT NULL,
  last_swap_is_sell BOOLEAN NOT NULL,
  last_swap_avg_execution_price_q64 NUMERIC NOT NULL,
  last_swap_base_volume BIGINT NOT NULL,
  last_swap_quote_volume BIGINT NOT NULL,
  last_swap_nonce BIGINT NOT NULL,
  last_swap_time TIMESTAMP NOT NULL,
  
  -- Querying all post-bonding curve markets. i.e., markets with liquidity pools.
  tvl_per_lp_coin_growth_q64 NUMERIC,
  in_bonding_curve BOOLEAN NOT NULL,
  daily_volume NUMERIC NOT NULL,

  PRIMARY KEY (market_id)
);

CREATE INDEX mkts_in_bonding_curve_idx
ON latest_market_state (in_bonding_curve, market_id, market_nonce DESC)
WHERE in_bonding_curve = TRUE;

CREATE INDEX mkts_daily_volume_idx
ON latest_market_state (daily_volume DESC, market_id)
WHERE daily_volume != 0;

CREATE TABLE user_liquidity_pools (
  provider VARCHAR(66) NOT NULL,
  transaction_version BIGINT NOT NULL,
  transaction_timestamp TIMESTAMP NOT NULL,
  inserted_at TIMESTAMP NOT NULL DEFAULT NOW(),

  market_id BIGINT NOT NULL,
  symbol_bytes BYTEA NOT NULL,
  bump_time TIMESTAMP NOT NULL,
  market_nonce BIGINT NOT NULL,
  trigger triggers NOT NULL,

  -- Liquidity event data.
  base_amount BIGINT NOT NULL,
  quote_amount BIGINT NOT NULL,
  lp_coin_amount BIGINT NOT NULL,
  liquidity_provided BOOLEAN NOT NULL,
  pro_rata_base_donation_claim_amount BIGINT NOT NULL,
  pro_rata_quote_donation_claim_amount BIGINT NOT NULL,

  PRIMARY KEY (provider, market_id)
);

CREATE INDEX user_lp_pools_idx
ON user_liquidity_pools (provider, market_id, market_nonce DESC);
