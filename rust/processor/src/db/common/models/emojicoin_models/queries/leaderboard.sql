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
WITH melee AS (
    SELECT emojicoin_0_market_id, emojicoin_1_market_id
    FROM arena_info
    WHERE melee_id = $1
),
-- Get the latest transaction version (i.e. last transaction of this melee before the next one starts).
last_txn AS (
    SELECT transaction_version - 1 AS last_txn
    FROM arena_melee_events
    WHERE melee_id = $1 + 1
),
-- Get the total deposit amount for each user.
deposits AS (
    SELECT
        "user",
        SUM(input_amount) as deposits
    FROM arena_enter_events
    WHERE melee_id = $1
    GROUP BY "user"
),
-- Get the total withdrawal amount for each user.
withdrawals AS (
    SELECT
        "user",
        SUM(apt_proceeds(exit.*)) as withdrawals
    FROM arena_exit_events AS exit
    WHERE melee_id = $1
    AND transaction_version < (SELECT last_txn FROM last_txn)
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
        -- Here, we want this data at the end of the melee before exiting the melee
        -- for the final time so we only look for exits before the melee end.
        AND transaction_version < (SELECT last_txn FROM last_txn)
    ) AS a
    ORDER BY "user", transaction_version DESC, event_index DESC
),
-- Get wether the user last exited on emojicoin_0 or emojicoin_1.
last_exit_0 AS (
    SELECT DISTINCT ON ("user") "user", emojicoin_0_proceeds > 0 AS last_exit_0
    FROM arena_exit_events
    WHERE melee_id = $1
    ORDER BY "user", transaction_version DESC, event_index DESC
)
SELECT
    deposits."user",
    $1 AS melee_id,
    -- Proifts = Withdrawals in APT + current emojicoin balance converted to APT.
    COALESCE(withdrawals, 0) +
        ROUND(
            emojicoin_0_balance * price_at_txn(emojicoin_0_market_id, last_txn) +
            emojicoin_1_balance * price_at_txn(emojicoin_0_market_id, last_txn)
        ) AS profits,
    deposits AS losses,
    emojicoin_0_balance,
    emojicoin_1_balance,
    -- If user has no balance at the end of the melee, or he has, but there is
    -- an exit event that happened after the end of the melee, then set exited
    -- to true, otherwise to false.
    emojicoin_0_balance + emojicoin_1_balance = 0
    OR
    EXISTS(SELECT * FROM arena_exit_events WHERE transaction_version >= (select last_txn from last_txn))
    AS exited,
    last_exit_0,
    COALESCE(withdrawals, 0) AS withdrawals
FROM melee, last_txn, deposits
    NATURAL INNER JOIN last_balances
    NATURAL LEFT JOIN withdrawals
    NATURAL LEFT JOIN last_exit_0
ON CONFLICT
DO NOTHING;
