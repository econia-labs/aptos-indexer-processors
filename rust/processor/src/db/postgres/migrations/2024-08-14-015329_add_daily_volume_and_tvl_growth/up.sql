-- Your SQL goes here

CREATE OR REPLACE FUNCTION assert_arrays_equal_length(BIGINT[], NUMERIC[]) RETURNS void AS $$
BEGIN
    IF array_length($1, 1) != array_length($2, 1) THEN
        RAISE EXCEPTION 'Arrays are not of equal length';
    END IF;
END;
$$ LANGUAGE plpgsql;

CREATE OR REPLACE FUNCTION assert_arrays_equal_length(BIGINT[], BIGINT[]) RETURNS void AS $$
BEGIN
    IF array_length($1, 1) != array_length($2, 1) THEN
        RAISE EXCEPTION 'Arrays are not of equal length';
    END IF;
END;
$$ LANGUAGE plpgsql;

-- Note for the volume calculations below:
-- The array sizes are ignored by Postgres; they are there for documentation purposes.
-- There are 1440 minutes in a day; we store the volume and time in ms for each minute.
-- We store the time in ms here it's very simple to use it in the sum conditional:
-- For each (time, volume) pair, add the volume to the rolling sum if (time > now() - 86400000)
-- The market nonces are stored to make each array element totally unique in case we ever want
-- to run checks on the data integrity.
-- At some point it might be insightful to profile this with large unrolled datasets.
-- That is: no arrays, all rows, but with index-only scans/covering indexes, to see if
-- there's even a performance benefit with arrays.
-- We wouldn't even necessarily have to purge the data past 24h, just write indexes
-- for the query.
CREATE TABLE market_24h_rolling_1min_periods (
    market_id BIGINT NOT NULL,
    inserted_at TIMESTAMP NOT NULL DEFAULT NOW(),
    market_nonces BIGINT[1440] NOT NULL,
    period_volumes NUMERIC[1440] NOT NULL,
    start_times BIGINT[1440] NOT NULL, -- Microseconds.

    PRIMARY KEY (market_id),

    -- Check length with cardinality because `array_length` returns NULL if the array is empty.
    CONSTRAINT check_equal_array_cardinality CHECK (
        cardinality(market_nonces) = cardinality(period_volumes) AND
        cardinality(period_volumes) = cardinality(start_times)
    ),

    -- Check that the arrays are 1-dimensional since we check length with cardinality.
    CONSTRAINT check_array_dimensions CHECK (
        array_ndims(market_nonces) = 1 AND
        array_ndims(period_volumes) = 1 AND
        array_ndims(start_times) = 1
    )
);

CREATE INDEX mkt_rlng_24h_vol_idx
ON market_24h_rolling_1min_periods (market_id)
INCLUDE (period_volumes, start_times);

-- Concatenate, deduplicate, and filter the last 24 hours of 1min period data for a market
-- and insert it into the market_24h_rolling_1min_periods table.
CREATE OR REPLACE FUNCTION update_market_24h_rolling_1min_periods(
    p_market_id BIGINT,
    p_nonces BIGINT[],
    p_volumes NUMERIC[],
    p_times BIGINT[]
) RETURNS void AS $$
DECLARE 
    one_day_ago BIGINT;
BEGIN
    one_day_ago := (EXTRACT(EPOCH FROM NOW()) * 1000000 - 86400000000)::BIGINT;
    PERFORM assert_arrays_equal_length(p_nonces, p_volumes);
    PERFORM assert_arrays_equal_length(p_nonces, p_times);
    
    WITH table_values(n2, v2, t2) AS (
        SELECT 
            market_nonces AS n2,
            period_volumes AS v2,
            start_times AS t2
        FROM 
            market_24h_rolling_1min_periods
        WHERE 
            market_id = p_market_id
    ),
    -- If we don't do this, no rows are returned, and thus the next CTEs select nothing and
    -- the result is a row with empty arrays, even if the input arguments are populated.
    at_least_one_row_with_empty_arrays AS (
        SELECT 
            COALESCE((SELECT n2 FROM table_values), ARRAY[]::BIGINT[]) AS n2,
            COALESCE((SELECT v2 FROM table_values), ARRAY[]::NUMERIC[]) AS v2,
            COALESCE((SELECT t2 FROM table_values), ARRAY[]::BIGINT[]) AS t2
    ),
    flattened AS (
        SELECT 
            unnest(array_cat(p_nonces, n2)) AS nonces,
            unnest(array_cat(p_volumes, v2)) AS volumes,
            unnest(array_cat(p_times, t2)) AS times
        FROM
            at_least_one_row_with_empty_arrays
    ),
    -- Duplicate nonces may occur, so we filter them out.
    deduplicated_nonces AS (
        SELECT DISTINCT ON (nonces)
            nonces,
            volumes,
            times
        FROM 
            flattened
        -- Times should never be different, but in case they are if we rewrite, we'll keep the latest.
        -- This can also be useful for unit tests.
        ORDER BY (nonces) DESC
    ),
    -- Filter period_type that started more than 24h ago, also remove duplicates.
    filtered_by_time AS (
        SELECT DISTINCT ON (times)
            nonces,
            volumes,
            times
        FROM
            deduplicated_nonces
        WHERE
            times > one_day_ago
    ),
    -- Since we flattened the data and then re-aggregated it, we need to coalesce with an empty array again.
    non_nulled AS (
        SELECT 
            COALESCE(array_agg(nonces), ARRAY[]::BIGINT[]) AS final_nonces,
            COALESCE(array_agg(volumes), ARRAY[]::NUMERIC[]) AS final_volumes,
            COALESCE(array_agg(times), ARRAY[]::BIGINT[]) AS final_times
        FROM
            filtered_by_time
    )
    INSERT INTO market_24h_rolling_1min_periods (
        market_id,
        market_nonces,
        period_volumes,
        start_times
    ) VALUES (
        p_market_id,
        (SELECT (final_nonces) FROM non_nulled),
        (SELECT (final_volumes) FROM non_nulled),
        (SELECT (final_times) FROM non_nulled)
    )
    ON CONFLICT (market_id) DO UPDATE SET
        market_nonces = (SELECT (final_nonces) FROM non_nulled),
        period_volumes = (SELECT (final_volumes) FROM non_nulled),
        start_times = (SELECT (final_times) FROM non_nulled);
END;
$$ LANGUAGE plpgsql;


-- Calculate the 24h rolling volume for each market.
CREATE OR REPLACE VIEW market_rolling_24h_volume AS
WITH all_markets AS (
    SELECT DISTINCT market_id FROM market_24h_rolling_1min_periods
),
-- Sum all markets with non-expired times.
recent_volumes AS (
    SELECT 
        market_id,
        COALESCE(SUM(filtered_volume), 0::NUMERIC) AS volume
    FROM (
        SELECT 
            market_id,
            unnest(COALESCE(period_volumes, ARRAY[]::NUMERIC[])) as filtered_volume,
            unnest(COALESCE(start_times, ARRAY[]::BIGINT[])) as filtered_times
        FROM 
            market_24h_rolling_1min_periods
    ) AS unpacked
    WHERE
        -- 86400000000 is 24 hours in microseconds.
        filtered_times > (EXTRACT(EPOCH FROM NOW()) * 1000000 - 86400000000)::BIGINT
    GROUP BY
        market_id
),
-- Get the latest state tracker volume for each market, aka the unclosed 1min candle volume that hasn't
-- been emitted as a periodic state event yet.
latest_state_volumes AS (
    SELECT
        market_id,
        -- Don't include the volume in the state tracker if the bump time is older than 1 day.
        -- I think this means the volume calculation period is technically 24 hours + up to 1 minute.
        CASE
            WHEN bump_time > NOW() - INTERVAL '1 day'
            THEN COALESCE(volume_in_1m_state_tracker, 0::NUMERIC)
            ELSE 0::NUMERIC
        END AS volume_in_1m_state_tracker
    FROM
        market_latest_state_event
)
-- Left join zero volume markets with > 0 volume markets and latest state volumes, then sum the volumes.
SELECT 
    am.market_id,
    COALESCE(rv.volume, 0::NUMERIC) + COALESCE(lsv.volume_in_1m_state_tracker, 0::NUMERIC) AS total_volume
FROM 
    all_markets am
LEFT JOIN 
    recent_volumes rv ON am.market_id = rv.market_id
LEFT JOIN
    latest_state_volumes lsv ON am.market_id = lsv.market_id;
