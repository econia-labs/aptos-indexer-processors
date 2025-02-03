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

CREATE OR REPLACE FUNCTION create_melee_info() RETURNS trigger AS $$
    BEGIN
        WITH market0 AS (
            SELECT * FROM market_registration_events WHERE market_address = NEW.emojicoin_0_market_address
        ), market1 AS (
            SELECT * FROM market_registration_events WHERE market_address = NEW.emojicoin_1_market_address
        )
        INSERT INTO arena_info (
            melee_id,
            volume,
            rewards_remaining,
            apt_locked,
            emojicoin_0_market_address,
            emojicoin_1_market_address,
            start_time,
            duration,
            max_match_percentage,
            max_match_amount,
            emojicoin_0_symbols,
            emojicoin_1_symbols,
            emojicoin_0_market_id,
            emojicoin_1_market_id
        ) VALUES (
            NEW.melee_id,
            0,
            NEW.available_rewards,
            0,
            NEW.emojicoin_0_market_address,
            NEW.emojicoin_1_market_address,
            NEW.start_time,
            NEW.duration,
            NEW.max_match_percentage,
            NEW.max_match_amount,
            (SELECT symbol_emojis FROM market0),
            (SELECT symbol_emojis FROM market1),
            (SELECT market_id FROM market0),
            (SELECT market_id FROM market1)
        )
        ON CONFLICT (melee_id) DO
        UPDATE SET
            rewards_remaining = arena_info.rewards_remaining + NEW.available_rewards,
            emojicoin_0_market_address = NEW.emojicoin_0_market_address,
            emojicoin_1_market_address = NEW.emojicoin_1_market_address,
            start_time = NEW.start_time,
            duration = NEW.duration,
            max_match_percentage = NEW.max_match_percentage,
            max_match_amount = NEW.max_match_amount,
            emojicoin_0_symbols = (SELECT symbol_emojis FROM market0),
            emojicoin_1_symbols = (SELECT symbol_emojis FROM market1),
            emojicoin_0_market_id = (SELECT market_id FROM market0),
            emojicoin_1_market_id = (SELECT market_id FROM market1)
        WHERE arena_info.melee_id = NEW.melee_id;
        RETURN NEW;
    END;
$$ LANGUAGE plpgsql;

CREATE OR REPLACE TRIGGER create_melee_info_trigger AFTER INSERT ON arena_melee_events
    FOR EACH ROW EXECUTE FUNCTION create_melee_info();
