-- This file should undo anything in `up.sql`

DROP FUNCTION chunked_candlesticks_metadata(
  market_id NUMERIC,
  period period_type,
  chunk_size INTEGER
);
