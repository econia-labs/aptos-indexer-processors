-- Your SQL goes here

-- Retrieves each period's latest candlestick for a single market.
-- Note these are unrolled manually, since the `DISTINCT ON (period)`
-- query that seems obvious to use here is up to 100x slower.
CREATE FUNCTION market_latest_candlesticks(market_id NUMERIC)
RETURNS SETOF candlesticks
AS $$
  SELECT * FROM (
    (SELECT * FROM candlesticks WHERE market_id = $1 AND period = 'period_15s' ORDER BY start_time DESC LIMIT 1)
    UNION ALL
    (SELECT * FROM candlesticks WHERE market_id = $1 AND period = 'period_1m' ORDER BY start_time DESC LIMIT 1)
    UNION ALL
    (SELECT * FROM candlesticks WHERE market_id = $1 AND period = 'period_5m' ORDER BY start_time DESC LIMIT 1)
    UNION ALL
    (SELECT * FROM candlesticks WHERE market_id = $1 AND period = 'period_15m' ORDER BY start_time DESC LIMIT 1)
    UNION ALL
    (SELECT * FROM candlesticks WHERE market_id = $1 AND period = 'period_30m' ORDER BY start_time DESC LIMIT 1)
    UNION ALL
    (SELECT * FROM candlesticks WHERE market_id = $1 AND period = 'period_1h' ORDER BY start_time DESC LIMIT 1)
    UNION ALL
    (SELECT * FROM candlesticks WHERE market_id = $1 AND period = 'period_4h' ORDER BY start_time DESC LIMIT 1)
    UNION ALL
    (SELECT * FROM candlesticks WHERE market_id = $1 AND period = 'period_1d' ORDER BY start_time DESC LIMIT 1)
  ) as result;
$$ LANGUAGE SQL;

-- Retrieves each period's latest candlestick for a single melee.
-- Note these are unrolled manually, since the `DISTINCT ON (period)`
-- query that seems obvious to use here is up to 100x slower.
CREATE FUNCTION arena_latest_candlesticks(melee_id NUMERIC)
RETURNS SETOF arena_candlesticks
AS $$
  SELECT * FROM (
    (SELECT * FROM arena_candlesticks WHERE melee_id = $1 AND period = 'period_15s' ORDER BY start_time DESC LIMIT 1)
    UNION ALL
    (SELECT * FROM arena_candlesticks WHERE melee_id = $1 AND period = 'period_1m' ORDER BY start_time DESC LIMIT 1)
    UNION ALL
    (SELECT * FROM arena_candlesticks WHERE melee_id = $1 AND period = 'period_5m' ORDER BY start_time DESC LIMIT 1)
    UNION ALL
    (SELECT * FROM arena_candlesticks WHERE melee_id = $1 AND period = 'period_15m' ORDER BY start_time DESC LIMIT 1)
    UNION ALL
    (SELECT * FROM arena_candlesticks WHERE melee_id = $1 AND period = 'period_30m' ORDER BY start_time DESC LIMIT 1)
    UNION ALL
    (SELECT * FROM arena_candlesticks WHERE melee_id = $1 AND period = 'period_1h' ORDER BY start_time DESC LIMIT 1)
    UNION ALL
    (SELECT * FROM arena_candlesticks WHERE melee_id = $1 AND period = 'period_4h' ORDER BY start_time DESC LIMIT 1)
    UNION ALL
    (SELECT * FROM arena_candlesticks WHERE melee_id = $1 AND period = 'period_1d' ORDER BY start_time DESC LIMIT 1)
  ) as result;
$$ LANGUAGE SQL;
