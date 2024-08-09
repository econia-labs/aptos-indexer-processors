-- Your SQL goes here

-- NOTE 1: that in all tables, we use `emit_time` in lieu of a transaction timestamp because they will always be the same.
-- If this ever changes, we can always update the database schema.

-- We omit most state event data for periodic state events because we have no need for them. We can always add them later
-- or create a view to join them.

-- NOTE 2: I don't think it's possible for the sender to not be the user aka the first `&signer`, but I might be wrong
-- in the case of a multi-sig/governance script. Noting it here as a reminder to check during the review.
-- In any case, we use the `sender` field to store the following: the `registrant`, the `swapper`, the `user`, and the `provider`.

DO && BEGIN
  CREATE TYPE trigger_type AS ENUM (
    'PACKAGE_PUBLICATION', -- Emitted a single time alongside a Global State event.
    'MARKET_REGISTRATION',
    'SWAP_BUY',
    'SWAP_SELL',
    'PROVIDE_LIQUIDITY',
    'REMOVE_LIQUIDITY',
    'CHAT'
  );
EXCEPTION
  WHEN duplicate_object THEN NULL;
END;

DO && BEGIN
  CREATE TYPE periodic_state_resolution AS ENUM (
    '1m',  --     60_000_000 == 1 minute.
    '5m',  --    300_000_000 == 5 minutes.
    '15m', --    900_000_000 == 15 minutes.
    '30m', --  1_800_000_000 == 30 minutes.
    '1h',  --  3_600_000_000 == 1 hour.
    '4h',  -- 14_400_000_000 == 4 hours.
    '1d'   -- 86_400_000_000 == 1 day.
  );
EXCEPTION
  WHEN duplicate_object THEN NULL;
END;

CREATE TABLE IF NOT EXISTS periodic_state_events (
  -- Transaction metadata.
  transaction_version BIGINT NOT NULL,
  sender VARCHAR(66) NOT NULL, -- See note 2.
  entry_function_id_str text, -- NULL when called by a script.
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
  last_swap_base_volume NUMERIC NOT NULL,
  last_swap_quote_volume NUMERIC NOT NULL,
  last_swap_nonce BIGINT NOT NULL,
  last_swap_emit_time TIMESTAMP NOT NULL,

  -- Periodic state metadata.
  resolution periodic_state_resolution NOT NULL,
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
  PRIMARY KEY (market_id, resolution, market_nonce)
);

CREATE TABLE IF NOT EXISTS state_events (
  -- Transaction metadata.
  transaction_version BIGINT NOT NULL,
  sender VARCHAR(66) NOT NULL, -- See note 2.
  entry_function_id_str text, -- NULL when called by a script.
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
  instantaneous_total_value_locked NUMERIC NOT NULL,
  instantaneous_market_cap NUMERIC NOT NULL,
  instantaneous_fully_diluted_value NUMERIC NOT NULL,

  -- Last swap data. If the triggering event is a swap, this data is exactly the same as the swap data.
  last_swap_is_sell BOOLEAN NOT NULL,
  last_swap_avg_execution_price_q64 NUMERIC NOT NULL,
  last_swap_base_volume NUMERIC NOT NULL,
  last_swap_quote_volume NUMERIC NOT NULL,
  last_swap_nonce BIGINT NOT NULL,
  last_swap_emit_time TIMESTAMP NOT NULL,

  -------- Duplicated data --------
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

CREATE INDEX IF NOT EXISTS st_evts_mkt_bytes_index ON state_events (market_id, symbol_bytes);
CREATE INDEX IF NOT EXISTS st_evts_trgr_mkt_btime_index ON state_events (trigger, market_id, bump_time DESC);
CREATE INDEX IF NOT EXISTS pe_evts_mkt_res_etime_index ON periodic_state_events (market_id, resolution, emit_time DESC);
