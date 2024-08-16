-- This file should undo anything in `up.sql`

DROP INDEX mkt_rlng_24h_vol_idx;
DROP FUNCTION update_market_24h_rolling_1min_periods(
    p_market_id BIGINT,
    p_nonces BIGINT[],
    p_volumes NUMERIC[],
    p_times BIGINT[]
);
DROP FUNCTION assert_arrays_equal_length(BIGINT[], NUMERIC[]);
DROP FUNCTION assert_arrays_equal_length(BIGINT[], BIGINT[]);

DROP TABLE market_1m_periods_in_last_day;
DROP FUNCTION one_day_ago_micros();
DROP VIEW market_rolling_24h_volume;
DROP TABLE market_24h_rolling_1min_periods;
DROP INDEX mkt_1m_24h_idx;
DROP INDEX mkt_expired_1m_periods_idx;
