-- This file should undo anything in `up.sql`
ALTER TABLE market_registration_events
        DROP COLUMN block_number;
