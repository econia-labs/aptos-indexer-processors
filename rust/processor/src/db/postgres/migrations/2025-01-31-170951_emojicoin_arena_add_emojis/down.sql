-- This file should undo anything in `up.sql`
DROP FUNCTION arena_leaderboard_history_with_arena_info;
ALTER TABLE arena_info DROP COLUMN emojicoin_0_symbols;
ALTER TABLE arena_info DROP COLUMN emojicoin_1_symbols;
ALTER TABLE arena_info DROP COLUMN emojicoin_0_market_id;
ALTER TABLE arena_info DROP COLUMN emojicoin_1_market_id;

CREATE OR REPLACE FUNCTION create_melee_info() RETURNS trigger AS $$
    BEGIN
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
            max_match_amount
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
            NEW.max_match_amount
        )
        ON CONFLICT (melee_id) DO
        UPDATE SET
            rewards_remaining = arena_info.rewards_remaining + NEW.available_rewards,
            emojicoin_0_market_address = NEW.emojicoin_0_market_address,
            emojicoin_1_market_address = NEW.emojicoin_1_market_address,
            start_time = NEW.start_time,
            duration = NEW.duration,
            max_match_percentage = NEW.max_match_percentage,
            max_match_amount = NEW.max_match_amount
        WHERE arena_info.melee_id = NEW.melee_id;
        RETURN NEW;
    END;
$$ LANGUAGE plpgsql;

CREATE OR REPLACE TRIGGER create_melee_info_trigger AFTER INSERT ON arena_melee_events
    FOR EACH ROW EXECUTE FUNCTION create_melee_info();


ALTER TABLE arena_leaderboard_history DROP COLUMN last_exit;
ALTER TABLE arena_leaderboard_history DROP COLUMN emojicoin_0_balance;
ALTER TABLE arena_leaderboard_history DROP COLUMN emojicoin_1_balance;

CREATE OR REPLACE FUNCTION save_leaderboard_history() RETURNS trigger AS $$
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

CREATE OR REPLACE TRIGGER save_leaderboard_history_trigger BEFORE INSERT ON arena_melee_events
    FOR EACH ROW EXECUTE FUNCTION save_leaderboard_history();

ALTER TABLE arena_positions DROP COLUMN last_exit;

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
            deposits = arena_positions.deposits + NEW.tap_out_fee
        WHERE arena_positions."user" = NEW."user" AND arena_positions.melee_id = NEW.melee_id;
        RETURN NEW;
    END;
$$ LANGUAGE plpgsql;

CREATE OR REPLACE TRIGGER update_position_exit_trigger AFTER INSERT ON arena_exit_events
    FOR EACH ROW EXECUTE FUNCTION update_position_exit();
