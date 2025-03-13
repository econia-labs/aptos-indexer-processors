-- Your SQL goes here

-- The latest arena candlesticks, distinct on each `period`.
CREATE VIEW arena_latest_candlesticks AS (
    SELECT DISTINCT ON (melee_id, period) *
    FROM arena_candlesticks
    ORDER BY melee_id, period, start_time DESC
);
