-- Your SQL goes here

DROP VIEW IF EXISTS price_feed;

-- This migration optimizes the price_feed view to improve query performance.
-- 
-- BACKGROUND:
-- The original price_feed implementation had significant performance issues:
-- 1. It scanned the entire swap_events table multiple times
-- 2. It calculated first_swap for all markets even though it was rarely needed
-- 3. It performed expensive operations before filtering the dataset
--
-- This adds a CASE statement to avoid calculating all `first_swap`s unless it's
-- actually necessary.
-- This reduces the query time by roughly 50%, tested using `EXPLAIN ANALYZE`
-- on the live `fallback` indexer.
--
-- It also adds a new field `delta_percentage` to allow sorting on the price delta
-- as a percentage.

CREATE VIEW price_feed AS
WITH markets AS (
    SELECT market_id
    FROM market_state
    WHERE daily_volume > 0
),
swap24 AS (
    SELECT DISTINCT ON (market_id)
        market_id,
        avg_execution_price_q64
    FROM swap_events
    WHERE transaction_timestamp <= CURRENT_TIMESTAMP - interval '1 day'
    AND market_id IN (SELECT market_id FROM markets)
    ORDER BY
        market_id,
        transaction_timestamp DESC
),
with_prices AS (
    SELECT
        latest_swap.*,
        CASE
            WHEN swap_open.avg_execution_price_q64 IS NULL THEN (
                SELECT avg_execution_price_q64
                FROM swap_events
                WHERE market_id = markets.market_id
                ORDER BY transaction_timestamp ASC
                LIMIT 1
            )
            ELSE swap_open.avg_execution_price_q64
        END AS open_price_q64,
        latest_swap.last_swap_avg_execution_price_q64 AS close_price_q64
    FROM markets
    INNER JOIN market_latest_state_event AS latest_swap ON markets.market_id = latest_swap.market_id
    LEFT JOIN swap24 AS swap_open ON markets.market_id = swap_open.market_id
    WHERE latest_swap.transaction_timestamp > CURRENT_TIMESTAMP - interval '1 day'
)
SELECT *,
    ((close_price_q64 / open_price_q64 * 100) - 100) AS delta_percentage
FROM with_prices;
