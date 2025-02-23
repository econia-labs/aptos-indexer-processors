-- Insert a snapshot of the leaderboard in arena_leaderboard_history for the
-- given melee ID.

-- The performance of this query has not been established, but should be
-- irrelevant, as arenas end once every 24 hours, and this script only runs
-- when an arena ends

INSERT INTO arena_leaderboard_history (
    "user",
    melee_id,
    profits,
    losses,
    emojicoin_0_balance,
    emojicoin_1_balance,
    exited,
    last_exit_0,
    withdrawals
)
-- Get the total deposit amount and last exit for each user.
WITH position AS (
    SELECT
        "user",
        deposits,
        last_exit_0
    FROM arena_position
    WHERE melee_id = $1
),
-- Get the total withdrawal amount for each user.
withdrawals AS (
    SELECT
        "user",
        SUM(apt_proceeds) as withdrawals
    FROM arena_exit_events
    WHERE melee_id = $1
    AND NOT after_end
    GROUP BY "user"
),
-- Get the last balance the user had at the end of the melee.
last_balances AS (
    SELECT DISTINCT ON("user")
        "user",
        emojicoin_0_balance,
        emojicoin_1_balance
    FROM (
        SELECT
            "user",
            transaction_version,
            event_index,
            emojicoin_0_proceeds AS emojicoin_0_balance,
            emojicoin_1_proceeds AS emojicoin_1_balance
        FROM arena_enter_events
        WHERE melee_id = $1
        UNION ALL
        SELECT
            "user",
            transaction_version,
            event_index,
            emojicoin_0_proceeds AS emojicoin_0_balance,
            emojicoin_1_proceeds AS emojicoin_1_balance
        FROM arena_swap_events
        WHERE melee_id = $1
        UNION ALL
        SELECT
            "user",
            transaction_version,
            event_index,
            0::numeric AS emojicoin_0_balance,
            0::numeric AS emojicoin_1_balance
        FROM arena_exit_events
        WHERE melee_id = $1
        AND NOT after_end
    ) AS a
    ORDER BY "user", transaction_version DESC, event_index DESC
)
SELECT
    position."user",
    $1 AS melee_id,
    -- Proifts = Withdrawals in APT + current emojicoin balance converted to APT.
    COALESCE(withdrawals, 0) +
        ROUND(
            emojicoin_0_balance * $2 +
            emojicoin_1_balance * $3
        ) AS profits,
    deposits AS losses,
    emojicoin_0_balance,
    emojicoin_1_balance,
    -- If user has no balance at the end of the melee, or he has, but there is
    -- an exit event that happened after the end of the melee, then set exited
    -- to true, otherwise to false.
    emojicoin_0_balance + emojicoin_1_balance = 0
    OR
    EXISTS(
        SELECT * FROM arena_exit_events AS aee
        WHERE aee."user" = position."user"
        AND melee_id = $1
        AND after_end
    )
    AS exited,
    last_exit_0,
    COALESCE(withdrawals, 0) AS withdrawals
FROM position
    NATURAL INNER JOIN last_balances
    NATURAL LEFT JOIN withdrawals;
