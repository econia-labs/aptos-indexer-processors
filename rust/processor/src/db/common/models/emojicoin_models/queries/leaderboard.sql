-- Get the latest transaction version where we can find events related to this melee (except for exit events).
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
WITH last_txn AS(
    SELECT COALESCE((SELECT transaction_version - 1 FROM arena_melee_events WHERE melee_id = $1 + 1), 340282355920938453453374507431758211455) AS txn
),deposits AS ( -- Get the total deposit amount for each user.
    SELECT
        "user",
        SUM(input_amount) as deposits
    FROM arena_enter_events
    WHERE melee_id = $1
    GROUP BY "user"
),
withdrawals AS ( -- Get the total withdrawal amount for each user.
    SELECT ROUND(SUM(
        emojicoin_0_proceeds / emojicoin_0_exchange_rate_base * emojicoin_0_exchange_rate_quote +
        emojicoin_1_proceeds / emojicoin_1_exchange_rate_base * emojicoin_1_exchange_rate_quote
    )) as withdrawals,
    "user"
    FROM arena_exit_events
    WHERE melee_id = $1
    AND transaction_version < (SELECT txn FROM last_txn)
    GROUP BY "user"
),
last_event AS ( -- Get the latest event related to this arena (per user).
    SELECT
        "user",
        transaction_version,
        event_index,
        emojicoin_0_proceeds,
        emojicoin_1_proceeds
    FROM arena_enter_events
    WHERE melee_id = $1
    UNION
    SELECT
        "user",
        transaction_version,
        event_index,
        emojicoin_0_proceeds,
        emojicoin_1_proceeds
    FROM arena_swap_events
    WHERE melee_id = $1
    UNION
    SELECT
        "user",
        transaction_version,
        event_index,
        emojicoin_0_proceeds,
        emojicoin_1_proceeds
    FROM arena_exit_events
    WHERE melee_id = $1
    -- Here, we want his data at the end of the melee before exiting the melee
    -- for the final time so we only look for exits before the melee end.
    AND transaction_version < (SELECT txn FROM last_txn)
    ORDER BY "user", transaction_version DESC, event_index DESC
    LIMIT 1
),
last_balances AS ( -- Get the last balance the user had at the end of the melee.
    SELECT DISTINCT ON("user")
        "user",
        emojicoin_0_proceeds AS emojicoin_0_balance,
        emojicoin_1_proceeds AS emojicoin_1_balance
    FROM last_event
),
last_price_0 AS ( -- Get the last price of 0 at the  end of the melee.
    SELECT COALESCE((
        SELECT avg_execution_price_q64 / POW(2::numeric,64::numeric)
        FROM swap_events
        WHERE market_id = (SELECT emojicoin_0_market_id FROM arena_info WHERE melee_id = $1)
        AND transaction_version < (SELECT txn FROM last_txn)
        ORDER BY transaction_version DESC, event_index DESC
        LIMIT 1
    ), 0) AS price
),
last_price_1 AS ( -- Get the last price of 1 at the  end of the melee.
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
        SELECT
            last_balances.emojicoin_0_balance + last_balances.emojicoin_1_balance = 0
        OR
            EXISTS(SELECT * FROM arena_exit_events WHERE transaction_version >= (select txn from last_txn))
    ) AS exited,
    (
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
