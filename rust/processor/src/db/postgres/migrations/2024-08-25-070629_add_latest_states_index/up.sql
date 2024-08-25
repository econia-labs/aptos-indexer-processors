-- Your SQL goes here

CREATE INDEX latest_states_idx
ON market_latest_state_event (bump_time DESC)
INCLUDE (market_id);
