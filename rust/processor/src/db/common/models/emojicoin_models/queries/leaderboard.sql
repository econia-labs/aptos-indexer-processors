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
-- Get the latest transaction version (i.e. last transaction of this melee before the next one starts).
WITH last_txn AS(
    SELECT COALESCE(
        (SELECT transaction_version - 1 FROM arena_melee_events WHERE melee_id = $1 + 1),
        -- If no such transaction found (aka melee not ended), just take
        -- U128::MAX to basically take all transactions into account. This
        -- should not happen, as for this to run, the melee must have
        -- ended, and thus another melee event with a higher melee ID
        -- should exits.
        340282355920938453453374507431758211455
    ) AS txn
), deposits AS ( -- Get the total deposit amount for each user.
    SELECT
        "user",
        SUM(input_amount) as deposits
    FROM arena_enter_events
    WHERE melee_id = $1
    GROUP BY "user"
), withdrawals AS ( -- Get the total withdrawal amount for each user.
    SELECT ROUND(SUM(
        emojicoin_0_proceeds / emojicoin_0_exchange_rate_base * emojicoin_0_exchange_rate_quote +
        emojicoin_1_proceeds / emojicoin_1_exchange_rate_base * emojicoin_1_exchange_rate_quote
    )) as withdrawals,
    "user"
    FROM arena_exit_events
    WHERE melee_id = $1
    AND transaction_version < (SELECT txn FROM last_txn)
    GROUP BY "user"
), last_event AS ( -- Get the latest event related to this arena (per user).
    SELECT
        "user",
        transaction_version,
        event_index,
        emojicoin_0_proceeds AS emojicoin_0_balance,
        emojicoin_1_proceeds AS emojicoin_1_balance
    FROM arena_enter_events
    WHERE melee_id = $1
    UNION
    SELECT
        "user",
        transaction_version,
        event_index,
        emojicoin_0_proceeds AS emojicoin_0_balance,
        emojicoin_1_proceeds AS emojicoin_1_balance
    FROM arena_swap_events
    WHERE melee_id = $1
    UNION
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
    AND transaction_version < (SELECT txn FROM last_txn)
), last_balances AS ( -- Get the last balance the user had at the end of the melee.
    SELECT DISTINCT ON("user")
        "user",
        emojicoin_0_balance,
        emojicoin_1_balance
    FROM last_event
    ORDER BY "user", transaction_version DESC, event_index DESC
), last_price_0 AS ( -- Get the last price of 0 at the end of the melee.
    SELECT COALESCE((
        SELECT avg_execution_price_q64 / POW(2::numeric,64::numeric)
        FROM swap_events
        WHERE market_id = (SELECT emojicoin_0_market_id FROM arena_info WHERE melee_id = $1)
        AND transaction_version < (SELECT txn FROM last_txn)
        ORDER BY transaction_version DESC, event_index DESC
        LIMIT 1
    ), 0) AS price
), last_price_1 AS ( -- Get the last price of 1 at the end of the melee.
    SELECT COALESCE((
        SELECT avg_execution_price_q64 / POW(2::numeric,64::numeric)
        FROM swap_events
        WHERE market_id = (SELECT emojicoin_1_market_id FROM arena_info WHERE melee_id = $1)
        AND transaction_version < (SELECT txn FROM last_txn)
        ORDER BY transaction_version DESC, event_index DESC
        LIMIT 1
    ), 0) AS price
)
SELECT
    enter."user",
    $1 AS melee_id,
    COALESCE(withdrawals, 0) + ROUND(last_balances.emojicoin_0_balance * last_price_0.price + last_balances.emojicoin_1_balance * last_price_1.price) AS profits,
    deposits.deposits AS losses,
    last_balances.emojicoin_0_balance,
    last_balances.emojicoin_1_balance,
    (
        -- If user has no balance at the end of the melee,
        SELECT
            last_balances.emojicoin_0_balance + last_balances.emojicoin_1_balance = 0
        -- or he has, but there is an exit event that happened after the end of the melee,
        OR
            EXISTS(SELECT * FROM arena_exit_events WHERE transaction_version >= (select txn from last_txn))
        -- then set exited to true, otherwise to false.
    ) AS exited,
    ( -- Check wether the user last exited on emojicoin_0 or emojicoin_1
        SELECT exit.emojicoin_0_proceeds > 0
        FROM arena_exit_events AS exit
        WHERE exit.melee_id = $1
        AND exit."user" = enter."user"
        ORDER BY transaction_version DESC, event_index DESC
        LIMIT 1
    ) AS last_exit_0,
    COALESCE(withdrawals, 0) AS withdrawals
FROM last_price_0, last_price_1, (
    SELECT DISTINCT "user" FROM arena_enter_events WHERE melee_id = $1
) AS enter
INNER JOIN deposits ON enter.user = deposits.user
LEFT JOIN withdrawals ON enter.user = withdrawals.user
INNER JOIN last_balances ON enter.user = last_balances.user
ON CONFLICT
DO NOTHING;
