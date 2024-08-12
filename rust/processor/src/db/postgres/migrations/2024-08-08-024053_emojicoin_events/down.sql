-- This file should undo anything in `up.sql`
DROP VIEW chat_events;
DROP VIEW swap_events;
DROP VIEW market_registration_events;

DROP INDEX user_lp_idx;
DROP INDEX all_time_volume_idx;
DROP INDEX mkt_cap_idx;
DROP INDEX latest_bump_idx;
DROP INDEX mkts_with_pool_idx;
DROP INDEX prdc_evts_by_res_idx;
DROP INDEX chats_by_mkt_and_time_idx;
DROP INDEX swaps_by_mkt_and_time_idx;
DROP INDEX mkts_by_time_idx;

DROP TABLE state_bump_events;
DROP TABLE periodic_state_events;
DROP TABLE global_state_events;

DROP TYPE periods;
DROP TYPE triggers;
DROP TYPE event_names;
