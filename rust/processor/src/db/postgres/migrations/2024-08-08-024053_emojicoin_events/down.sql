-- This file should undo anything in `up.sql`
DROP INDEX IF EXISTS pe_evts_mkt_res_etime_index;
DROP INDEX IF EXISTS st_bmps_trgr_mkt_btime_index;
DROP INDEX IF EXISTS st_bmps_mkt_bytes_index;
DROP TABLE IF EXISTS state_bumps;
DROP TABLE IF EXISTS periodic_state_events;
DROP TABLE IF EXISTS global_state_events;
DROP TYPE IF EXISTS periodic_state_resolution;
DROP TYPE IF EXISTS state_trigger;
