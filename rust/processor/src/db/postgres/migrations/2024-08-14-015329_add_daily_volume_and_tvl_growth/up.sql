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

CREATE OR REPLACE FUNCTION one_day_ago_micros() RETURNS BIGINT AS $$
    SELECT (EXTRACT(EPOCH FROM CURRENT_TIMESTAMP) * 1000000 - 86400000000)::BIGINT;
$$ LANGUAGE SQL IMMUTABLE;

CREATE TABLE market_1m_periods_in_last_day (
    market_id BIGINT NOT NULL,
    inserted_at TIMESTAMP NOT NULL DEFAULT NOW(),
    nonce BIGINT NOT NULL, -- Market nonce.
    volume NUMERIC NOT NULL, -- Quote volume.
    start_time BIGINT NOT NULL, -- In microseconds.

    PRIMARY KEY (market_id, nonce)
);

CREATE INDEX mkt_1m_24h_idx
ON market_1m_periods_in_last_day (market_id, start_time)
INCLUDE (volume)
WHERE start_time > one_day_ago_micros();

CREATE INDEX mkt_expired_1m_periods_idx
ON market_1m_periods_in_last_day (start_time)
WHERE start_time <= one_day_ago_micros();

-- Calculate the 24h rolling volume for each market.
CREATE VIEW market_daily_volume AS
WITH recent_volumes AS (
    SELECT 
        market_id,
        COALESCE(SUM(volume), 0::NUMERIC) AS volume
    FROM market_1m_periods_in_last_day
    WHERE
        start_time > one_day_ago_micros()
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
    lsv.market_id,
    COALESCE(rv.volume, 0::NUMERIC) + COALESCE(lsv.volume_in_1m_state_tracker, 0::NUMERIC) AS daily_volume
FROM 
    latest_state_volumes lsv
LEFT JOIN
    recent_volumes rv ON lsv.market_id = rv.market_id;
