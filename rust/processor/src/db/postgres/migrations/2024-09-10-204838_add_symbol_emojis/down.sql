-- This file should undo anything in `up.sql`

DROP VIEW market_latest_state;
DROP INDEX mkt_state_by_mkt_nonce_idx;
DROP INDEX mkt_state_by_mkt_cap_idx;
DROP INDEX mkt_state_by_all_time_volume_idx;
