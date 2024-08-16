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
DROP VIEW market_rolling_24h_volume;
DROP TABLE market_24h_rolling_1min_periods;
DROP TABLE market_latest_1d_tvl_lp_coin_growth;
