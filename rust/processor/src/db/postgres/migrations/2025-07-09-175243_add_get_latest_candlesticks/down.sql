-- This file should undo anything in `up.sql`

DROP FUNCTION get_market_latest_candlesticks(market_id NUMERIC);
DROP FUNCTION get_arena_latest_candlesticks(melee_id NUMERIC);
