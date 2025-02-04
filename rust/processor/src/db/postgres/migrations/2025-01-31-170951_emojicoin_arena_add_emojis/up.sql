-- ============================================
-- Add duplicated metadata fields to arena info
-- ============================================

ALTER TABLE arena_info ADD COLUMN emojicoin_0_symbols TEXT[];
ALTER TABLE arena_info ADD COLUMN emojicoin_1_symbols TEXT[];
ALTER TABLE arena_info ADD COLUMN emojicoin_0_market_id NUMERIC;
ALTER TABLE arena_info ADD COLUMN emojicoin_1_market_id NUMERIC;

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

-- =================================================================
-- Add last exit and end holdings information to leaderboard history
-- =================================================================

ALTER TABLE arena_leaderboard_history ADD COLUMN last_exit TEXT;
ALTER TABLE arena_leaderboard_history ADD COLUMN emojicoin_0_balance NUMERIC NOT NULL;
ALTER TABLE arena_leaderboard_history ADD COLUMN emojicoin_1_balance NUMERIC NOT NULL;
ALTER TABLE arena_leaderboard_history ADD COLUMN exited BOOLEAN NOT NULL;

CREATE OR REPLACE FUNCTION save_leaderboard_history() RETURNS trigger AS $$
    BEGIN
        INSERT INTO arena_leaderboard_history
        SELECT
            "user",
            NEW.melee_id - 1,
            arena_leaderboard.profits,
            arena_leaderboard.losses,
            (
                WITH last_exit AS (
                    SELECT * FROM arena_exit_events AS aee
                    WHERE aee.melee_id = NEW.melee_id - 1
                    AND aee."user" = arena_leaderboard."user"
                    ORDER BY transaction_version DESC, event_index DESC
                    LIMIT 1
                ),
                melee AS (
                    SELECT * FROM arena_melee_events AS ame
                    WHERE ame.melee_id = NEW.melee_id - 1
                )
                SELECT
                    CASE
                        WHEN (SELECT last_exit.emojicoin_0_proceeds FROM last_exit) = 0 THEN (SELECT melee.emojicoin_1_market_address FROM melee)
                        WHEN (SELECT last_exit.emojicoin_1_proceeds FROM last_exit) = 0 THEN (SELECT melee.emojicoin_0_market_address FROM melee)
                        ELSE NULL -- aka never exited
                    END
            ),
            arena_leaderboard.emojicoin_0_balance,
            arena_leaderboard.emojicoin_1_balance,
            CASE WHEN arena_leaderboard.emojicoin_0_balance + arena_leaderboard.emojicoin_1_balance = 0 THEN true ELSE false END
        FROM arena_leaderboard;
        RETURN NEW;
    END;
$$ LANGUAGE plpgsql;

CREATE OR REPLACE FUNCTION update_leaderboard_history() RETURNS trigger AS $$
    BEGIN
        UPDATE arena_leaderboard_history
        SET exited = true
        WHERE arena_leaderboard_history.melee_id = NEW.melee_id
        AND arena_leaderboard_history."user" = NEW."user";
        RETURN NEW;
    END;
$$ LANGUAGE plpgsql;

CREATE OR REPLACE TRIGGER update_leaderboard_history_trigger BEFORE INSERT ON arena_exit_events
    FOR EACH ROW EXECUTE FUNCTION update_leaderboard_history();

-- ======================================
-- Add last exit information to positions
-- ======================================

ALTER TABLE arena_positions ADD COLUMN last_exit TEXT;
ALTER TABLE arena_positions ADD COLUMN match_amount NUMERIC NOT NULL;

CREATE OR REPLACE FUNCTION update_position_enter() RETURNS trigger AS $$
    BEGIN
        INSERT INTO arena_positions (
            "user",
            melee_id,
            open,
            emojicoin_0_balance,
            emojicoin_1_balance,
            withdrawals,
            deposits,
            match_amount
        ) VALUES (
            NEW."user",
            NEW.melee_id,
            true,
            NEW.emojicoin_0_proceeds,
            NEW.emojicoin_1_proceeds,
            0,
            NEW.input_amount + NEW.match_amount,
            CASE WHEN NEW.match_amount > 0 THEN true ELSE false END,
            NEW.match_amount
        )
        ON CONFLICT ("user", melee_id) DO
        UPDATE SET
            open = true,
            emojicoin_0_balance = arena_positions.emojicoin_0_balance + NEW.emojicoin_0_proceeds,
            emojicoin_1_balance = arena_positions.emojicoin_1_balance + NEW.emojicoin_1_proceeds,
            deposits = arena_positions.deposits + NEW.input_amount + NEW.match_amount,
            match_amount = arena_positions.match_amount + NEW.match_amount
        WHERE arena_positions."user" = NEW."user" AND arena_positions.melee_id = NEW.melee_id;
        RETURN NEW;
    END;
$$ LANGUAGE plpgsql;

CREATE OR REPLACE FUNCTION update_position_exit() RETURNS trigger AS $$
    BEGIN
        UPDATE arena_positions SET
            open = false,
            emojicoin_0_balance = 0,
            emojicoin_1_balance = 0,
            withdrawals = arena_positions.withdrawals
                + NEW.emojicoin_0_proceeds
                    / NEW.emojicoin_0_exchange_rate_base
                    * NEW.emojicoin_0_exchange_rate_quote
                + NEW.emojicoin_1_proceeds
                    / NEW.emojicoin_1_exchange_rate_base
                    * NEW.emojicoin_1_exchange_rate_quote,
            deposits = arena_positions.deposits + NEW.tap_out_fee,
            last_exit = CASE
                WHEN NEW.emojicoin_1_proceeds = 0 THEN (SELECT emojicoin_0_market_address FROM arena_melee_events AS ame WHERE ame.melee_id = NEW.melee_id)
                ELSE (SELECT emojicoin_1_market_address FROM arena_melee_events AS ame WHERE ame.melee_id = NEW.melee_id)
            END,
            match_amount = arena_positions.match_amount - NEW.tap_out_fee
        WHERE arena_positions."user" = NEW."user" AND arena_positions.melee_id = NEW.melee_id;
        RETURN NEW;
    END;
$$ LANGUAGE plpgsql;

CREATE OR REPLACE TRIGGER update_position_enter_trigger AFTER INSERT ON arena_enter_events
    FOR EACH ROW EXECUTE FUNCTION update_position_enter();

CREATE OR REPLACE TRIGGER update_position_exit_trigger AFTER INSERT ON arena_exit_events
    FOR EACH ROW EXECUTE FUNCTION update_position_exit();

-- =================================================================
-- Add new endpoint to get personal history with embedded melee info
-- =================================================================

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
