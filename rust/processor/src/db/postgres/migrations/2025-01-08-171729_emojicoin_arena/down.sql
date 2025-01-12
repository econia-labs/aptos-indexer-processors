-- This file should undo anything in `up.sql`

DROP INDEX latest_swap_by_market_address;

DROP TRIGGER update_position_enter_trigger ON arena_enter_events;
DROP TRIGGER update_position_exit_trigger ON arena_exit_events;
DROP TRIGGER update_position_swap_trigger ON arena_swap_events;
DROP TRIGGER snapshot_leaderboard_trigger ON arena_melee_events;

DROP FUNCTION update_position_enter;
DROP FUNCTION update_position_exit;
DROP FUNCTION update_position_swap;
DROP FUNCTION snapshot_leaderboard;

DROP FUNCTION arena_leaderboard;

DROP TABLE arena_melee_events;
DROP TABLE arena_enter_events;
DROP TABLE arena_exit_events;
DROP TABLE arena_swap_events;
DROP TABLE arena_vault_balance_update_events;
DROP TABLE arena_leaderboard_history;
DROP TABLE arena_positions;
