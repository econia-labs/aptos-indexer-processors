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
-- market. We can use the `market_nonce` here because they're inherently ordered
-- by time on a per-market basis.
CREATE INDEX sender_mkt_swap_hstry_idx ON swap_events (sender, market_id, market_nonce);

-- However, for querying a user's total transaction history across multiple
-- markets, the transaction version combined with the event index is the only
-- accurate representation of chronological ordering.
-- The event index is only used to sort events that have occurred within the
-- same transaction, i.e., it is a secondary key on the final chronological
-- sort; therefore, it does not need an index.
CREATE INDEX sender_all_swap_hstry_idx ON swap_events (sender, transaction_version);
CREATE INDEX sender_all_chat_hstry_idx ON chat_events (sender, transaction_version);
CREATE INDEX sender_all_pool_hstry_idx ON liquidity_events (sender, transaction_version);
CREATE INDEX sender_all_mkt_rgstr_hstry_idx ON market_registration_events (sender, transaction_version);

-- The only way to enforce that the subqueries filter by the sender/user's address
-- prior to the UNION ALLs is by making this a function.
CREATE FUNCTION user_emojicoin_txn_history(user text) RETURNS TABLE(
    event_data JSON
)
AS $$
WITH
    swaps AS (SELECT * FROM swap_events WHERE sender = $1),
    chats AS (SELECT * FROM chat_events WHERE sender = $1),
    liqs AS (SELECT * FROM liquidity_events WHERE sender = $1),
    regs AS (SELECT * FROM market_registration_events WHERE sender = $1)
SELECT row_to_json(events) AS event_data
FROM (
    SELECT transaction_version, event_index, swaps AS events FROM swaps
    UNION ALL
    SELECT transaction_version, event_index, chats AS events FROM chats
    UNION ALL
    SELECT transaction_version, event_index, liqs AS events FROM liqs
    UNION ALL
    SELECT transaction_version, event_index, regs AS events FROM regs
) AS all_events
ORDER BY transaction_version DESC, event_index DESC;
$$ LANGUAGE SQL;
