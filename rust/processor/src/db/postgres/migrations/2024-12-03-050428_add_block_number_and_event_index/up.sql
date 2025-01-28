-- Your SQL goes here
ALTER TABLE swap_events
        ADD COLUMN block_number NUMERIC NOT NULL,
        ADD COLUMN event_index NUMERIC NOT NULL;
CREATE INDEX se_block_num ON swap_events (block_number);

ALTER TABLE liquidity_events
        ADD COLUMN block_number NUMERIC NOT NULL,
        ADD COLUMN event_index NUMERIC NOT NULL;
CREATE INDEX le_block_num ON liquidity_events (block_number);
