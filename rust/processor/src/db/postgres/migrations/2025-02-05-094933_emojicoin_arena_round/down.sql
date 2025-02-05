-- This file should undo anything in `up.sql`
CREATE OR REPLACE FUNCTION update_position_exit() RETURNS trigger AS $$
    BEGIN
        UPDATE arena_positions SET
            open = false,
            emojicoin_0_balance = 0,
            emojicoin_1_balance = 0,
            withdrawals = arena_positions.withdrawals
                -- diff here
                + NEW.emojicoin_0_proceeds
                    / NEW.emojicoin_0_exchange_rate_base
                    * NEW.emojicoin_0_exchange_rate_quote
                -- diff here
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

CREATE OR REPLACE TRIGGER update_position_exit_trigger AFTER INSERT ON arena_exit_events
    FOR EACH ROW EXECUTE FUNCTION update_position_exit();

DROP FUNCTION arena_leaderboard_history_with_arena_info;
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

DROP VIEW arena_leaderboard;
CREATE VIEW arena_leaderboard AS
WITH melee AS (
    SELECT * FROM arena_melee_events ORDER BY melee_id DESC LIMIT 1
), price_emojicoin_0 AS (
    SELECT avg_execution_price_q64 / POW(2,64)::NUMERIC AS price FROM swap_events
    WHERE market_address = (SELECT emojicoin_0_market_address FROM arena_melee_events WHERE melee_id = (SELECT melee_id FROM melee))
    ORDER BY market_nonce DESC
    LIMIT 1
), price_emojicoin_1 AS (
    SELECT avg_execution_price_q64 / POW(2,64)::NUMERIC AS price FROM swap_events
    WHERE market_address = (SELECT emojicoin_1_market_address FROM arena_melee_events WHERE melee_id = (SELECT melee_id FROM melee))
    ORDER BY market_nonce DESC
    LIMIT 1
), realized_position AS (
    SELECT
        "user",
        open,
        emojicoin_0_balance,
        emojicoin_1_balance,
        withdrawals +
            emojicoin_0_balance * (SELECT * FROM price_emojicoin_0) +
            emojicoin_1_balance * (SELECT * FROM price_emojicoin_1) AS profits,
        deposits AS losses
    FROM arena_positions WHERE melee_id = (SELECT melee_id FROM melee)
)
SELECT
    *,
    profits / losses * 100 - 100 AS pnl_percent,
    profits - losses AS pnl_octas
FROM realized_position;

ALTER TABLE arena_leaderboard_history DROP COLUMN withdrawals;
