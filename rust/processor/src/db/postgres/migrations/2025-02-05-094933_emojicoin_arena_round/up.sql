-- Your SQL goes here
CREATE OR REPLACE FUNCTION update_position_exit() RETURNS trigger AS $$
    BEGIN
        UPDATE arena_positions SET
            open = false,
            emojicoin_0_balance = 0,
            emojicoin_1_balance = 0,
            withdrawals = arena_positions.withdrawals
                -- diff here
                + ROUND(NEW.emojicoin_0_proceeds
                    / NEW.emojicoin_0_exchange_rate_base
                    * NEW.emojicoin_0_exchange_rate_quote)
                -- diff here
                + ROUND(NEW.emojicoin_1_proceeds
                    / NEW.emojicoin_1_exchange_rate_base
                    * NEW.emojicoin_1_exchange_rate_quote),
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

UPDATE arena_positions SET withdrawals = ROUND(arena_positions.withdrawals);
UPDATE arena_leaderboard_history SET profits = ROUND(arena_leaderboard_history.profits);

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
            ROUND(emojicoin_0_balance * COALESCE((SELECT * FROM price_emojicoin_0), 0::numeric)) +
            ROUND(emojicoin_1_balance * COALESCE((SELECT * FROM price_emojicoin_1), 0::numeric)) AS profits,
        withdrawals,
        deposits AS losses
    FROM arena_positions WHERE melee_id = (SELECT melee_id FROM melee)
)
SELECT
    *,
    profits / losses * 100 - 100 AS pnl_percent,
    profits - losses AS pnl_octas
FROM realized_position;

ALTER TABLE arena_leaderboard_history ADD COLUMN withdrawals NUMERIC;

UPDATE arena_leaderboard_history
SET withdrawals = COALESCE((
    SELECT SUM(
        ROUND(aee.emojicoin_0_proceeds
            / aee.emojicoin_0_exchange_rate_base
            * aee.emojicoin_0_exchange_rate_quote)
        + ROUND(aee.emojicoin_1_proceeds
            / aee.emojicoin_1_exchange_rate_base
            * aee.emojicoin_1_exchange_rate_quote)
    )
    FROM arena_exit_events AS aee
    WHERE aee.melee_id = arena_leaderboard_history.melee_id
    AND aee."user" = arena_leaderboard_history."user"
    AND transaction_version <= COALESCE((
        SELECT transaction_version
        FROM arena_melee_events AS ame
        WHERE ame.melee_id = aee.melee_id + 1
    ), 340282366920938463463374607431768211456)
), 0);

ALTER TABLE arena_leaderboard_history ALTER COLUMN withdrawals SET NOT NULL;

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
            CASE WHEN arena_leaderboard.emojicoin_0_balance + arena_leaderboard.emojicoin_1_balance = 0 THEN true ELSE false END,
            withdrawals
        FROM arena_leaderboard;
        RETURN NEW;
    END;
$$ LANGUAGE plpgsql;

DROP FUNCTION arena_leaderboard_history_with_arena_info;
CREATE FUNCTION arena_leaderboard_history_with_arena_info("user" text, "skip" int) RETURNS TABLE (
    melee_id NUMERIC,
    profits NUMERIC,
    losses NUMERIC,
    last_exit TEXT,
    emojicoin_0_balance NUMERIC,
    emojicoin_1_balance NUMERIC,
    exited BOOLEAN,
    withdrawals NUMERIC,

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
    arena_leaderboard_history.last_exit,
    arena_leaderboard_history.emojicoin_0_balance,
    arena_leaderboard_history.emojicoin_1_balance,
    arena_leaderboard_history.exited,
    arena_leaderboard_history.withdrawals,

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
ORDER BY arena_leaderboard_history.melee_id
LIMIT 20
OFFSET $2
$$ LANGUAGE SQL;
