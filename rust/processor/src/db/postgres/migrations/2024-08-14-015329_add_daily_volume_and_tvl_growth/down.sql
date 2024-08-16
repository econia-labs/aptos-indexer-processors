-- This file should undo anything in `up.sql`

DROP FUNCTION assert_arrays_equal_length(BIGINT[], NUMERIC[]);
DROP FUNCTION assert_arrays_equal_length(BIGINT[], BIGINT[]);

DROP INDEX mkt_1m_24h_idx;
DROP INDEX mkt_expired_1m_periods_idx;
DROP VIEW market_daily_volume;
DROP TABLE market_1m_periods_in_last_day;
DROP FUNCTION one_day_ago_micros();
