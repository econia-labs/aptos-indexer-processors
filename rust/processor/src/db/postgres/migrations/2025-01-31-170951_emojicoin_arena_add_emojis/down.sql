-- This file should undo anything in `up.sql`
DROP FUNCTION arena_leaderboard_history_with_arena_info;
ALTER TABLE arena_info DROP COLUMN emojicoin_0_symbols;
ALTER TABLE arena_info DROP COLUMN emojicoin_1_symbols;
ALTER TABLE arena_info DROP COLUMN emojicoin_0_market_id;
ALTER TABLE arena_info DROP COLUMN emojicoin_1_market_id;
