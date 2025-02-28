-- Your SQL goes here

ALTER TABLE chat_events
    ADD COLUMN event_index BIGINT NOT NULL;
ALTER TABLE market_registration_events
    ADD COLUMN event_index BIGINT NOT NULL;
    
--------------------------------------------------------------------------------
-- For these queries, "sender" is used as the representation of a user instead
-- of the emitted fields that represent the interacting account's address;
-- i.e., the "swapper", "user", "provider", and "registrant". 
--
-- Most often, these two fields match; however, objects and resource accounts
-- are often used in place of the transaction "sender" account as a proxy to
-- interact with the contract, meaning that the actual user intending to
-- interact with the contract is often only the "sender" and *not*` the
-- address in the corresponding event field.
--
-- It is possible that this assumption could be wrong, in that the "sender"
-- and individual "swapper"/"user"/"provider"/"registrant" fields don't match,
-- but the actual user is *not* the sender but in the fact the one in the event
-- field. A specific example of this would be if an account (the "sender") signs
-- and sends a multi-sig script or wrapper contract in which multiple other
-- accounts sign separate swap/chat/register/liquidity transactions.
--
-- However, the difference between "sender" and the corresponding event field
-- falls under the scope of application logic and is thus left to be dealt
-- with there, not here.
--------------------------------------------------------------------------------

-- For querying a user's chronologically descending trade history for a single
-- market. Event index is not included because the number of events per txn
-- is generally less than 100. 
CREATE INDEX sender_mkt_swap_hstry_idx ON swap_events (sender, market_id, transaction_version);

-- For querying all of a user's activity of a certain type. Event index not
-- included again for the same reason as mentioned above.
CREATE INDEX sender_all_swap_hstry_idx ON swap_events (sender, transaction_version);
CREATE INDEX sender_all_chat_hstry_idx ON chat_events (sender, transaction_version);
CREATE INDEX sender_all_pool_hstry_idx ON liquidity_events (sender, transaction_version);
CREATE INDEX sender_all_mkt_rgstr_hstry_idx ON market_registration_events (sender, transaction_version);
