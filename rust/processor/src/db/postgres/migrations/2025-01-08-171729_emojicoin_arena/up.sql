-- Raw events

CREATE TABLE arena_melee_events (
    transaction_version BIGINT NOT NULL,
    event_index BIGINT NOT NULL,
    sender VARCHAR(66) NOT NULL,
    entry_function VARCHAR(200),
    transaction_timestamp TIMESTAMP NOT NULL,
    inserted_at TIMESTAMP NOT NULL DEFAULT NOW(),

    melee_id NUMERIC NOT NULL PRIMARY KEY,
    emojicoin_0_market_address TEXT NOT NULL,
    emojicoin_1_market_address TEXT NOT NULL,
    start_time NUMERIC NOT NULL,
    duration NUMERIC NOT NULL,
    max_match_percentage NUMERIC NOT NULL,
    max_match_amount NUMERIC NOT NULL,
    available_rewards NUMERIC NOT NULL
);

CREATE TABLE arena_enter_events (
    transaction_version BIGINT NOT NULL,
    event_index BIGINT NOT NULL,
    sender VARCHAR(66) NOT NULL,
    entry_function VARCHAR(200),
    transaction_timestamp TIMESTAMP NOT NULL,
    inserted_at TIMESTAMP NOT NULL DEFAULT NOW(),

    "user" TEXT NOT NULL,
    melee_id NUMERIC NOT NULL,
    input_amount NUMERIC NOT NULL,
    quote_volume NUMERIC NOT NULL,
    integrator_fee NUMERIC NOT NULL,
    match_amount NUMERIC NOT NULL,
    emojicoin_0_proceeds NUMERIC NOT NULL,
    emojicoin_1_proceeds NUMERIC NOT NULL,
    emojicoin_0_exchange_rate_base NUMERIC NOT NULL,
    emojicoin_0_exchange_rate_quote NUMERIC NOT NULL,
    emojicoin_1_exchange_rate_base NUMERIC NOT NULL,
    emojicoin_1_exchange_rate_quote NUMERIC NOT NULL,

    PRIMARY KEY (transaction_version, event_index)
);

CREATE TABLE arena_exit_events (
    transaction_version BIGINT NOT NULL,
    event_index BIGINT NOT NULL,
    sender VARCHAR(66) NOT NULL,
    entry_function VARCHAR(200),
    transaction_timestamp TIMESTAMP NOT NULL,
    inserted_at TIMESTAMP NOT NULL DEFAULT NOW(),

    "user" TEXT NOT NULL,
    melee_id NUMERIC NOT NULL,
    tap_out_fee NUMERIC NOT NULL,
    emojicoin_0_proceeds NUMERIC NOT NULL,
    emojicoin_1_proceeds NUMERIC NOT NULL,
    emojicoin_0_exchange_rate_base NUMERIC NOT NULL,
    emojicoin_0_exchange_rate_quote NUMERIC NOT NULL,
    emojicoin_1_exchange_rate_base NUMERIC NOT NULL,
    emojicoin_1_exchange_rate_quote NUMERIC NOT NULL,

    PRIMARY KEY (transaction_version, event_index)
);

CREATE TABLE arena_swap_events (
    transaction_version BIGINT NOT NULL,
    event_index BIGINT NOT NULL,
    sender VARCHAR(66) NOT NULL,
    entry_function VARCHAR(200),
    transaction_timestamp TIMESTAMP NOT NULL,
    inserted_at TIMESTAMP NOT NULL DEFAULT NOW(),

    "user" TEXT NOT NULL,
    melee_id NUMERIC NOT NULL,
    quote_volume NUMERIC NOT NULL,
    integrator_fee NUMERIC NOT NULL,
    emojicoin_0_proceeds NUMERIC NOT NULL,
    emojicoin_1_proceeds NUMERIC NOT NULL,
    emojicoin_0_exchange_rate_base NUMERIC NOT NULL,
    emojicoin_0_exchange_rate_quote NUMERIC NOT NULL,
    emojicoin_1_exchange_rate_base NUMERIC NOT NULL,
    emojicoin_1_exchange_rate_quote NUMERIC NOT NULL,

    PRIMARY KEY (transaction_version, event_index)
);

CREATE TABLE arena_vault_balance_update_events (
    transaction_version BIGINT NOT NULL,
    event_index BIGINT NOT NULL,
    sender VARCHAR(66) NOT NULL,
    entry_function VARCHAR(200),
    transaction_timestamp TIMESTAMP NOT NULL,
    inserted_at TIMESTAMP NOT NULL DEFAULT NOW(),

    new_balance NUMERIC NOT NULL,

    PRIMARY KEY (transaction_version, event_index)
);

-- Derived data

CREATE TABLE arena_positions (
    "user" TEXT NOT NULL,
    melee_id NUMERIC NOT NULL,
    open BOOL NOT NULL,
    emojicoin_0_balance NUMERIC NOT NULL,
    emojicoin_1_balance NUMERIC NOT NULL,
    withdrawals NUMERIC NOT NULL,
    deposits NUMERIC NOT NULL,

    PRIMARY KEY ("user", melee_id)
);

CREATE TABLE arena_leaderboard_history (
    "user" TEXT NOT NULL,
    melee_id NUMERIC NOT NULL,
    profits NUMERIC NOT NULL,
    losses NUMERIC NOT NULL,

    PRIMARY KEY ("user", melee_id)
);

-- Functions

CREATE VIEW arena_leaderboard AS
WITH melee AS (
    SELECT * FROM arena_melee_events ORDER BY melee_id DESC LIMIT 1
), price_emojicoin_0 AS (
    SELECT avg_execution_price_q64::numeric / POW(2,64) AS price FROM swap_events
    WHERE market_address = (SELECT emojicoin_0_market_address FROM arena_melee_events WHERE melee_id = (SELECT melee_id FROM melee))
    ORDER BY market_nonce DESC
    LIMIT 1
), price_emojicoin_1 AS (
    SELECT avg_execution_price_q64::numeric / POW(2,64) AS price FROM swap_events
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

-- Triggers to update derived data

-- Insert a new position in the positions table.
--
-- If position already exists, it means that this is a top off and we handle it
-- as such.
CREATE FUNCTION update_position_enter() RETURNS trigger AS $$
    BEGIN
        INSERT INTO arena_positions (
            "user",
            melee_id,
            open,
            emojicoin_0_balance,
            emojicoin_1_balance,
            withdrawals,
            deposits
        ) VALUES (
            NEW."user",
            NEW.melee_id,
            true,
            NEW.emojicoin_0_proceeds,
            NEW.emojicoin_1_proceeds,
            0,
            NEW.input_amount + NEW.match_amount
        )
        ON CONFLICT ("user", melee_id) DO
        UPDATE SET
            open = true,
            emojicoin_0_balance = arena_positions.emojicoin_0_balance + NEW.emojicoin_0_proceeds,
            emojicoin_1_balance = arena_positions.emojicoin_1_balance + NEW.emojicoin_1_proceeds,
            deposits = arena_positions.deposits + NEW.input_amount + NEW.match_amount
        WHERE arena_positions."user" = NEW."user" AND arena_positions.melee_id = NEW.melee_id;
        RETURN NEW;
    END;
$$ LANGUAGE plpgsql;

-- Mark the position as closed and update withdrawals and deposits.
CREATE FUNCTION update_position_exit() RETURNS trigger AS $$
    BEGIN
        UPDATE arena_positions SET
            open = false,
            emojicoin_0_balance = 0,
            emojicoin_1_balance = 0,
            withdrawals = arena_positions.withdrawals + NEW.emojicoin_0_proceeds * NEW.emojicoin_0_exchange_rate_base / NEW.emojicoin_0_exchange_rate_quote + NEW.emojicoin_1_proceeds * NEW.emojicoin_1_exchange_rate_base / NEW.emojicoin_1_exchange_rate_quote,
            deposits = arena_positions.deposits + NEW.tap_out_fee
        WHERE arena_positions."user" = NEW."user" AND arena_positions.melee_id = NEW.melee_id;
        RETURN NEW;
    END;
$$ LANGUAGE plpgsql;

-- Update the emojicoin balances according to the swap data.
CREATE FUNCTION update_position_swap() RETURNS trigger AS $$
    BEGIN
        UPDATE arena_positions SET
            emojicoin_0_balance = NEW.emojicoin_0_proceeds,
            emojicoin_1_balance = NEW.emojicoin_1_proceeds
        WHERE arena_positions."user" = NEW."user" AND arena_positions.melee_id = NEW.melee_id;
        RETURN NEW;
    END;
$$ LANGUAGE plpgsql;

-- Save a snapshot of the leaderboard in the leaderboard history table.
CREATE FUNCTION save_leaderboard_history() RETURNS trigger AS $$
    BEGIN
        INSERT INTO arena_leaderboard_history
        SELECT
            "user",
            NEW.melee_id - 1,
            profits,
            losses
        FROM arena_leaderboard;
        RETURN NEW;
    END;
$$ LANGUAGE plpgsql;

CREATE TRIGGER update_position_enter_trigger AFTER INSERT ON arena_enter_events
    FOR EACH ROW EXECUTE FUNCTION update_position_enter();

CREATE TRIGGER update_position_exit_trigger AFTER INSERT ON arena_exit_events
    FOR EACH ROW EXECUTE FUNCTION update_position_exit();

CREATE TRIGGER update_position_swap_trigger AFTER INSERT ON arena_swap_events
    FOR EACH ROW EXECUTE FUNCTION update_position_swap();

CREATE TRIGGER save_leaderboard_history_trigger BEFORE INSERT ON arena_melee_events
    FOR EACH ROW EXECUTE FUNCTION save_leaderboard_history();

-- Indices

CREATE INDEX latest_swap_by_market_address ON swap_events (market_address, market_nonce DESC);
