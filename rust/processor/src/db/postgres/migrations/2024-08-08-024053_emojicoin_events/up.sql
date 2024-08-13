-- Your SQL goes here

-------------------------------------------------------------------------------
--
--                                   Enums
--
-------------------------------------------------------------------------------

CREATE TYPE triggers AS ENUM (
  'package_publication',
  'market_registration',
  'swap_buy',
  'swap_sell',
  'provide_liquidity',
  'remove_liquidity',
  'chat'
);

CREATE TYPE periods AS ENUM (
  'period_1m',  --     60_000_000 == 1 minute.
  'period_5m',  --    300_000_000 == 5 minutes.
  'period_15m', --    900_000_000 == 15 minutes.
  'period_30m', --  1_800_000_000 == 30 minutes.
  'period_1h',  --  3_600_000_000 == 1 hour.
  'period_4h',  -- 14_400_000_000 == 4 hours.
  'period_1d'   -- 86_400_000_000 == 1 day.
);

-------------------------------------------------------------------------------
--
--                                  Tables
--
-------------------------------------------------------------------------------

CREATE TABLE global_state_events (
  -- Transaction metadata.
  transaction_version BIGINT NOT NULL,
  sender VARCHAR(66) NOT NULL,
  entry_function VARCHAR(200),
  transaction_timestamp TIMESTAMP NOT NULL,
  inserted_at TIMESTAMP NOT NULL DEFAULT NOW(),

  -- Global state event data.
  emit_time TIMESTAMP NOT NULL,
  registry_nonce BIGINT NOT NULL,
  trigger triggers NOT NULL,
  cumulative_quote_volume NUMERIC NOT NULL,
  total_quote_locked NUMERIC NOT NULL,
  total_value_locked NUMERIC NOT NULL,
  market_cap NUMERIC NOT NULL,
  fully_diluted_value NUMERIC NOT NULL,
  cumulative_integrator_fees NUMERIC NOT NULL,
  cumulative_swaps BIGINT NOT NULL,
  cumulative_chat_messages BIGINT NOT NULL,

  PRIMARY KEY (registry_nonce)
);

CREATE TABLE periodic_state_events (
  -- Transaction metadata.
  transaction_version BIGINT NOT NULL,
  sender VARCHAR(66) NOT NULL,
  entry_function VARCHAR(200),
  transaction_timestamp TIMESTAMP NOT NULL,
  inserted_at TIMESTAMP NOT NULL DEFAULT NOW(),

  -- Market and state metadata.
  market_id BIGINT NOT NULL,
  symbol_bytes BYTEA NOT NULL,
  emit_time TIMESTAMP NOT NULL,
  market_nonce BIGINT NOT NULL,
  trigger triggers NOT NULL,

  -- Last swap data.
  last_swap_is_sell BOOLEAN NOT NULL,
  last_swap_avg_execution_price_q64 NUMERIC NOT NULL,
  last_swap_base_volume BIGINT NOT NULL,
  last_swap_quote_volume BIGINT NOT NULL,
  last_swap_nonce BIGINT NOT NULL,
  last_swap_time TIMESTAMP NOT NULL,

  -- Periodic state metadata.
  period periods NOT NULL,
  start_time TIMESTAMP NOT NULL,

  -- Periodic state event data.
  open_price_q64 NUMERIC NOT NULL,
  high_price_q64 NUMERIC NOT NULL,
  low_price_q64 NUMERIC NOT NULL,
  close_price_q64 NUMERIC NOT NULL,
  volume_base NUMERIC NOT NULL,
  volume_quote NUMERIC NOT NULL,
  integrator_fees NUMERIC NOT NULL,
  pool_fees_base NUMERIC NOT NULL,
  pool_fees_quote NUMERIC NOT NULL,
  n_swaps BIGINT NOT NULL,
  n_chat_messages BIGINT NOT NULL,
  starts_in_bonding_curve BOOLEAN NOT NULL,
  ends_in_bonding_curve BOOLEAN NOT NULL,
  tvl_per_lp_coin_growth_q64 NUMERIC NOT NULL,

  PRIMARY KEY (market_id, period, market_nonce)
);

CREATE TABLE market_registration_events (
  -- Transaction metadata.
  transaction_version BIGINT NOT NULL,
  sender VARCHAR(66) NOT NULL,
  entry_function VARCHAR(200),
  transaction_timestamp TIMESTAMP NOT NULL,
  inserted_at TIMESTAMP NOT NULL DEFAULT NOW(),

  -- Market and state metadata.
  market_id BIGINT NOT NULL,
  symbol_bytes BYTEA NOT NULL,
  bump_time TIMESTAMP NOT NULL,
  market_nonce BIGINT NOT NULL,
  trigger triggers NOT NULL,

  -- Market registration event data.
  registrant VARCHAR(66) NOT NULL,
  integrator VARCHAR(66) NOT NULL,
  integrator_fee BIGINT NOT NULL,

  PRIMARY KEY (market_id)
);

CREATE TABLE swap_events (
  -- Transaction metadata.
  transaction_version BIGINT NOT NULL,
  sender VARCHAR(66) NOT NULL,
  entry_function VARCHAR(200),
  transaction_timestamp TIMESTAMP NOT NULL,
  inserted_at TIMESTAMP NOT NULL DEFAULT NOW(),

  -- Market and state metadata.
  market_id BIGINT NOT NULL,
  symbol_bytes BYTEA NOT NULL,
  bump_time TIMESTAMP NOT NULL,
  market_nonce BIGINT NOT NULL,
  trigger triggers NOT NULL,

  -- Swap event data.
  swapper VARCHAR(66) NOT NULL,
  integrator VARCHAR(66) NOT NULL,
  integrator_fee BIGINT NOT NULL,
  input_amount BIGINT NOT NULL,
  is_sell BOOLEAN NOT NULL,
  integrator_fee_rate_bps SMALLINT NOT NULL,
  net_proceeds BIGINT NOT NULL,
  base_volume BIGINT NOT NULL,
  quote_volume BIGINT NOT NULL,
  avg_execution_price_q64 NUMERIC NOT NULL,
  pool_fee BIGINT NOT NULL,
  starts_in_bonding_curve BOOLEAN NOT NULL,
  results_in_state_transition BOOLEAN NOT NULL,

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

  PRIMARY KEY (market_id, market_nonce)
);

CREATE TABLE chat_events (
  -- Transaction metadata.
  transaction_version BIGINT NOT NULL,
  sender VARCHAR(66) NOT NULL,
  entry_function VARCHAR(200),
  transaction_timestamp TIMESTAMP NOT NULL,
  inserted_at TIMESTAMP NOT NULL DEFAULT NOW(),

  -- Market and state metadata.
  market_id BIGINT NOT NULL,
  symbol_bytes BYTEA NOT NULL,
  bump_time TIMESTAMP NOT NULL,
  market_nonce BIGINT NOT NULL,
  trigger triggers NOT NULL,

  -- Chat event data.
  "user" VARCHAR(66) NOT NULL,
  message TEXT NOT NULL,
  user_emojicoin_balance BIGINT NOT NULL,
  circulating_supply BIGINT NOT NULL,
  balance_as_fraction_of_circulating_supply_q64 NUMERIC NOT NULL,

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

  PRIMARY KEY (market_id, market_nonce)
);

CREATE TABLE liquidity_events (
  -- Transaction metadata.
  transaction_version BIGINT NOT NULL,
  sender VARCHAR(66) NOT NULL,
  entry_function VARCHAR(200),
  transaction_timestamp TIMESTAMP NOT NULL,
  inserted_at TIMESTAMP NOT NULL DEFAULT NOW(),

  -- Market and state metadata.
  market_id BIGINT NOT NULL,
  symbol_bytes BYTEA NOT NULL,
  bump_time TIMESTAMP NOT NULL,
  market_nonce BIGINT NOT NULL,
  trigger triggers NOT NULL,

  -- Liquidity event data.
  provider VARCHAR(66) NOT NULL,
  base_amount BIGINT NOT NULL,
  quote_amount BIGINT NOT NULL,
  lp_coin_amount BIGINT NOT NULL,
  liquidity_provided BOOLEAN NOT NULL,
  pro_rata_base_donation_claim_amount BIGINT NOT NULL,
  pro_rata_quote_donation_claim_amount BIGINT NOT NULL,

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

  PRIMARY KEY (market_id, market_nonce)
);

-------------------------------------------------------------------------------
--
--                                  Indexes
--
-------------------------------------------------------------------------------

-- Querying the swap events for a market, descending chronologically.
CREATE INDEX swaps_by_mkt_and_time_idx
ON swap_events (market_id, bump_time DESC);

-- Querying the chat events for a market, descending chronologically.
CREATE INDEX chats_by_mkt_and_time_idx
ON chat_events (market_id, bump_time DESC);

-- Querying the candlestick data for a market by its period,
-- descending chronologically by the candlestick open time.
CREATE UNIQUE INDEX prdc_evts_by_res_idx
ON periodic_state_events (market_id, period, start_time DESC);

-- Querying a user's liquidity pools.
CREATE INDEX user_lp_idx
ON liquidity_events (provider)
WHERE liquidity_provided = TRUE;