-- This file should undo anything in `up.sql`

DROP FUNCTION market_latest_candlesticks(market_id NUMERIC);
DROP FUNCTION arena_latest_candlesticks(melee_id NUMERIC);
