-- Your SQL goes here
ALTER TABLE arena_info ADD COLUMN emojicoin_0_symbols TEXT[];
ALTER TABLE arena_info ADD COLUMN emojicoin_1_symbols TEXT[];
ALTER TABLE arena_info ADD COLUMN emojicoin_0_market_id NUMERIC;
ALTER TABLE arena_info ADD COLUMN emojicoin_1_market_id NUMERIC;

UPDATE arena_info SET
    emojicoin_0_symbols = (SELECT symbol_emojis FROM market_registration_events AS mre WHERE mre.market_address = arena_info.emojicoin_0_market_address),
    emojicoin_1_symbols = (SELECT symbol_emojis FROM market_registration_events AS mre WHERE mre.market_address = arena_info.emojicoin_1_market_address),
    emojicoin_0_market_id = (SELECT market_id FROM market_registration_events AS mre WHERE mre.market_address = arena_info.emojicoin_0_market_address),
    emojicoin_1_market_id = (SELECT market_id FROM market_registration_events AS mre WHERE mre.market_address = arena_info.emojicoin_1_market_address);

ALTER TABLE arena_info ALTER COLUMN emojicoin_0_symbols SET NOT NULL;
ALTER TABLE arena_info ALTER COLUMN emojicoin_1_symbols SET NOT NULL;
ALTER TABLE arena_info ALTER COLUMN emojicoin_0_market_id SET NOT NULL;
ALTER TABLE arena_info ALTER COLUMN emojicoin_1_market_id SET NOT NULL;

CREATE FUNCTION arena_leaderboard_history_with_arena_info("user" text, "skip" int) RETURNS TABLE (
    melee_id NUMERIC,
    profits NUMERIC,
    losses NUMERIC,

    emojicoin_0_symbols TEXT[],
    emojicoin_1_symbols TEXT[],
    emojicoin_0_market_address TEXT,
    emojicoin_1_market_address TEXT,
    emojicoin_0_market_id NUMERIC,
    emojicoin_1_market_id NUMERIC,
    start_time TIMESTAMP,
    duration NUMERIC
)
AS $$
SELECT
    arena_leaderboard_history.melee_id,
    arena_leaderboard_history.profits,
    arena_leaderboard_history.losses,

    arena_info.emojicoin_0_symbols,
    arena_info.emojicoin_1_symbols,
    arena_info.emojicoin_0_market_address,
    arena_info.emojicoin_1_market_address,
    arena_info.emojicoin_0_market_id,
    arena_info.emojicoin_1_market_id,
    arena_info.start_time,
    arena_info.duration
FROM
    arena_leaderboard_history
INNER JOIN
    arena_info
ON
    arena_info.melee_id = arena_leaderboard_history.melee_id
WHERE "user" = $1
LIMIT 20
OFFSET $2
$$ LANGUAGE SQL;
