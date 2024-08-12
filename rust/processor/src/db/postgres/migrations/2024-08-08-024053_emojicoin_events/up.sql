-- Your SQL goes here

-------------------------------------------------------------------------------
--
--                                   Enums
--
-------------------------------------------------------------------------------

CREATE TYPE event_names AS ENUM (
  'market_registration',
  'swap',
  'chat',
  'liquidity'
);

CREATE TYPE triggers AS ENUM (
  'package_publication', -- Emitted a single time alongside a Global State event.
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

  -- Market metadata.
  market_id BIGINT NOT NULL,
  symbol_bytes BYTEA NOT NULL,

  -- State metadata.
  emit_time TIMESTAMP NOT NULL,
  market_nonce BIGINT NOT NULL,
  trigger triggers NOT NULL,

  -- Last swap data. The last swap can also be the event that triggered the periodic state event.
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

CREATE TABLE bump_events (
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
  trigger triggers NOT NULL,

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
  -- All bump events have these in some form.
  event_name event_names NOT NULL,
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

-------------------------------------------------------------------------------
--
--                                  Indexes
--
-------------------------------------------------------------------------------

-- Create partial indexes for the common queries. Unique markets, swaps by market and time, and chats by market and time.
CREATE INDEX mkts_by_time_idx
ON bump_events (bump_time DESC)
WHERE event_name = 'market_registration';

CREATE INDEX swaps_by_mkt_and_time_idx
ON bump_events (market_id, bump_time DESC)
WHERE event_name = 'swap';

CREATE INDEX chats_by_mkt_and_time_idx
ON bump_events (market_id, bump_time DESC)
WHERE event_name = 'chat';

-- Querying the candlestick data for a market by its period resolution.
CREATE INDEX prdc_evts_by_res_idx
ON periodic_state_events (market_id, period, start_time DESC);

-- Querying all post-bonding curve markets. i.e., markets with liquidity pools.
CREATE UNIQUE INDEX mkts_with_pool_idx
ON bump_events (market_id)
WHERE results_in_state_transition = TRUE;

-- Sorting by bump order.
CREATE INDEX latest_bump_idx
ON bump_events (market_id, market_nonce DESC);

-- Sorting by market cap, descending.
CREATE INDEX mkt_cap_idx
ON bump_events (instantaneous_stats_market_cap DESC);

-- Sorting by time volume, descending.
CREATE INDEX all_time_volume_idx
ON bump_events (cumulative_quote_volume DESC);

-- Querying a user's liquidity pools.
CREATE INDEX user_lp_idx
ON bump_events (user_address)
WHERE event_name = 'liquidity' AND liquidity_provided = TRUE;

-------------------------------------------------------------------------------
--
--                                   Views
--
-------------------------------------------------------------------------------

-- Split the bump events into views for the event types.
CREATE VIEW market_registration_events AS
SELECT * FROM bump_events
WHERE event_name = 'market_registration'::event_names
ORDER BY bump_time DESC;

CREATE VIEW swap_events AS
SELECT * FROM bump_events
WHERE event_name = 'swap'::event_names
ORDER BY bump_time DESC;


CREATE VIEW chat_events AS
SELECT * FROM bump_events
WHERE event_name = 'chat'::event_names
ORDER BY bump_time DESC;
