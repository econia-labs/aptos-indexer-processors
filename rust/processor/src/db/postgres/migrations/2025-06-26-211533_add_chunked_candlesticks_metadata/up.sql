-- Your SQL goes here

CREATE FUNCTION chunked_candlesticks_metadata(
  market_id NUMERIC,
  period period_type,
  chunk_size INTEGER
)
RETURNS TABLE (
  chunk_id INTEGER,
  first_start_time TIMESTAMP,
  last_start_time TIMESTAMP,
  num_items INTEGER
)
AS $$
  /*
   * Returns metadata for time-chunked candlesticks.
   *
   * Each "chunk" groups a consecutive range of `chunk_size` rows from the
   * `candlesticks` table for a given market and period, ordered by `start_time`.
   *
   * Parameters:
   * - market_id: the market to query
   * - period: the candlestick interval (e.g. 'period_1h')
   * - chunk_size: how many rows per chunk
   *
   * Output:
   * - chunk_id: numeric index of the chunk (0-based)
   * - first_start_time: earliest start_time in the chunk
   * - last_start_time: latest start_time in the chunk
   * - num_items: number of rows in this chunk
   *
   * This is useful for pagination, summarization, or caching of large candlestick datasets.
   * 
   * Note that this chunks data for *all* rows in the table, so it will not scale infinitely
   * as the number of candlesticks for a market increases. However, increasing the chunk size
   * can address this to some extent. Plus, there's no reason to reasonably expect a single
   * market will ever have more than a million rows any time soon. At a chunk size of 2,000
   * a million rows would only be 500 total chunks, and the current largest number of rows
   * for a single market/period is ~33,000.
   */
  WITH chunked_data AS (
    SELECT 
      start_time,
      -- The entire purpose of this function is to return stable (cacheable) metadata for a
      -- market's candlesticks, so to ensure consistent results, `start_time` must be in ASC
      -- order. Otherwise, new candlesticks would change the first/last start times for
      -- historical chunks, even if the chunk size is constant.
      (ROW_NUMBER() OVER (ORDER BY start_time ASC) - 1) / chunk_size AS chunk_id
    FROM candlesticks 
    WHERE market_id = $1 AND period = $2
  )
  SELECT 
    chunk_id,
    MIN(start_time) AS first_start_time,
    MAX(start_time) AS last_start_time,
    COUNT(*) AS num_items
  FROM chunked_data
  GROUP BY chunk_id
  ORDER BY chunk_id;
$$ LANGUAGE SQL;
