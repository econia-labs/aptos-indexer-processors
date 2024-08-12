-- This file should undo anything in `up.sql`
DROP VIEW IF EXISTS periodic_events_1d;
DROP VIEW IF EXISTS periodic_events_4h;
DROP VIEW IF EXISTS periodic_events_1h;
DROP VIEW IF EXISTS periodic_events_30m;
DROP VIEW IF EXISTS periodic_events_15m;
DROP VIEW IF EXISTS periodic_events_5m;
DROP VIEW IF EXISTS periodic_events_1m;
DROP INDEX IF EXISTS bump_time_idx;
DROP INDEX IF EXISTS prdc_evts_mkt_strt_idx;
DROP INDEX IF EXISTS bump_evts_trgr_mkt_btime_idx;
DROP INDEX IF EXISTS bump_evts_mkt_bytes_idx;
DROP TABLE IF EXISTS bump_events;
DROP TABLE IF EXISTS periodic_state_events;
DROP TABLE IF EXISTS global_state_events;
DROP TYPE IF EXISTS period_type;
DROP TYPE IF EXISTS trigger_type;
