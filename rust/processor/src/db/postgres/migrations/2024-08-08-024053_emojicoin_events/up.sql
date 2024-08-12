-- Your SQL goes here

-------------------------------------------------------------------------------
--
--                                   Enums
--
-------------------------------------------------------------------------------

CREATE TYPE trigger_type AS ENUM (
  'package_publication', -- Emitted a single time alongside a Global State event.
  'market_registration',
  'swap_buy',
  'swap_sell',
  'provide_liquidity',
  'remove_liquidity',
  'chat'
);

CREATE TYPE period_type AS ENUM (
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

CREATE TABLE IF NOT EXISTS global_state_events (
  -- Transaction metadata.
  transaction_version BIGINT NOT NULL,
  sender VARCHAR(66) NOT NULL,
  entry_function VARCHAR(200),
  transaction_timestamp TIMESTAMP NOT NULL,
  inserted_at TIMESTAMP NOT NULL DEFAULT NOW(),

  -- Global state event data.
  emit_time TIMESTAMP NOT NULL,
  registry_nonce BIGINT NOT NULL,
  trigger trigger_type NOT NULL,
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

CREATE TABLE IF NOT EXISTS periodic_state_events (
  -- Transaction metadata.
  transaction_version BIGINT NOT NULL,
  sender VARCHAR(66) NOT NULL,
  entry_function VARCHAR(200),
  transaction_timestamp TIMESTAMP NOT NULL,
  inserted_at TIMESTAMP NOT NULL DEFAULT NOW(),

  -- Market metadata.
  market_id BIGINT NOT NULL,
  symbol_bytes BYTEA NOT NULL,

  -- State metadata.
  emit_time TIMESTAMP NOT NULL,
  market_nonce BIGINT NOT NULL,
  trigger trigger_type NOT NULL,

  -- Last swap data. The last swap can also be the event that triggered the periodic state event.
  last_swap_is_sell BOOLEAN NOT NULL,
  last_swap_avg_execution_price_q64 NUMERIC NOT NULL,
  last_swap_base_volume BIGINT NOT NULL,
  last_swap_quote_volume BIGINT NOT NULL,
  last_swap_nonce BIGINT NOT NULL,
  last_swap_time TIMESTAMP NOT NULL,

  -- Periodic state metadata.
  period period_type NOT NULL,
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

CREATE TABLE IF NOT EXISTS bump_events (
  -- Transaction metadata.
  transaction_version BIGINT NOT NULL,
  sender VARCHAR(66) NOT NULL,
  entry_function VARCHAR(200),
  transaction_timestamp TIMESTAMP NOT NULL,
  inserted_at TIMESTAMP NOT NULL DEFAULT NOW(),

  -- Market metadata.
  market_id BIGINT NOT NULL,
  symbol_bytes BYTEA NOT NULL,

  -- State metadata.
  bump_time TIMESTAMP NOT NULL,
  market_nonce BIGINT NOT NULL,
  trigger trigger_type NOT NULL,

  -- State event data.
  clamm_virtual_reserves_base BIGINT NOT NULL,
  clamm_virtual_reserves_quote BIGINT NOT NULL,
  cpamm_real_reserves_base BIGINT NOT NULL,
  cpamm_real_reserves_quote BIGINT NOT NULL,
  lp_coin_supply NUMERIC NOT NULL,
  cumulative_base_volume NUMERIC NOT NULL,
  cumulative_quote_volume NUMERIC NOT NULL,
  cumulative_integrator_fees NUMERIC NOT NULL,
  cumulative_pool_fees_base NUMERIC NOT NULL,
  cumulative_pool_fees_quote NUMERIC NOT NULL,
  cumulative_n_swaps BIGINT NOT NULL,
  cumulative_n_chat_messages BIGINT NOT NULL,
  instantaneous_stats_total_quote_locked BIGINT NOT NULL,
  instantaneous_stats_total_value_locked NUMERIC NOT NULL,
  instantaneous_stats_market_cap NUMERIC NOT NULL,
  instantaneous_stats_fully_diluted_value NUMERIC NOT NULL,

  -- Last swap data. If the triggering event is a swap, this data is exactly the same as the swap data.
  last_swap_is_sell BOOLEAN NOT NULL,
  last_swap_avg_execution_price_q64 NUMERIC NOT NULL,
  last_swap_base_volume BIGINT NOT NULL,
  last_swap_quote_volume BIGINT NOT NULL,
  last_swap_nonce BIGINT NOT NULL,
  last_swap_time TIMESTAMP NOT NULL,

  -------- Data in multiple event types --------
  -- All bump events have a user, either a `registrant`, a `swapper`, a `provider`, or a `user`.
  user_address VARCHAR(66) NOT NULL,

  -- Market registration & Swap data.
  integrator VARCHAR(66),
  integrator_fee BIGINT,

  -------- Trigger/event type-specific data --------
  -- Swap event data.
  input_amount BIGINT,
  is_sell BOOLEAN,
  integrator_fee_rate_bps SMALLINT,
  net_proceeds BIGINT,
  base_volume BIGINT,
  quote_volume BIGINT,
  avg_execution_price_q64 NUMERIC,
  pool_fee BIGINT,
  starts_in_bonding_curve BOOLEAN,
  results_in_state_transition BOOLEAN,

  -- Liquidity event data.
  base_amount BIGINT,
  quote_amount BIGINT,
  lp_coin_amount BIGINT,
  liquidity_provided BOOLEAN,
  pro_rata_base_donation_claim_amount BIGINT,
  pro_rata_quote_donation_claim_amount BIGINT,

  -- Chat event data.
  message TEXT,
  user_emojicoin_balance BIGINT,
  circulating_supply BIGINT,
  balance_as_fraction_of_circulating_supply_q64 NUMERIC,

  PRIMARY KEY (market_id, market_nonce)
);

CREATE INDEX IF NOT EXISTS bump_evts_mkt_bytes_idx ON bump_events (market_id, symbol_bytes);
CREATE INDEX IF NOT EXISTS bump_evts_trgr_mkt_btime_idx ON bump_events (market_id, trigger, bump_time DESC);
CREATE INDEX IF NOT EXISTS prdc_evts_mkt_strt_idx ON periodic_state_events (market_id, period, start_time DESC);
CREATE INDEX IF NOT EXISTS bump_time_idx ON bump_events (bump_time DESC);

-------------------------------------------------------------------------------
--
--                                   Views
--
-------------------------------------------------------------------------------

-------------------------------------------------------------------------------
-- Periodic state events views, one for each period.
CREATE VIEW periodic_events_1m AS
SELECT * FROM periodic_state_events
WHERE period = 'period_1m'::period_type;

CREATE VIEW periodic_events_5m AS
SELECT * FROM periodic_state_events
WHERE period = 'period_5m'::period_type;

CREATE VIEW periodic_events_15m AS
SELECT * FROM periodic_state_events
WHERE period = 'period_15m'::period_type;

CREATE VIEW periodic_events_30m AS
SELECT * FROM periodic_state_events
WHERE period = 'period_30m'::period_type;

CREATE VIEW periodic_events_1h AS
SELECT * FROM periodic_state_events
WHERE period = 'period_1h'::period_type;

CREATE VIEW periodic_events_4h AS
SELECT * FROM periodic_state_events
WHERE period = 'period_4h'::period_type;

CREATE VIEW periodic_events_1d AS
SELECT * FROM periodic_state_events
WHERE period = 'period_1d'::period_type;
