-- Your SQL goes here

DROP VIEW arena_leaderboard;

-- The only changes here are the casts for POW(2,64) to NUMERIC (they previously weren't casted).
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
