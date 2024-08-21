-- This file should undo anything in `up.sql`


ALTER TABLE swap_events
  DROP COLUMN balance_as_fraction_of_circulating_supply_before_q64,
  DROP COLUMN balance_as_fraction_of_circulating_supply_after_q64;

ALTER TABLE liquidity_events
RENAME COLUMN base_donation_claim_amount TO pro_rata_base_donation_claim_amount;

ALTER TABLE liquidity_events
RENAME COLUMN quote_donation_claim_amount TO pro_rata_quote_donation_claim_amount;

ALTER TABLE user_liquidity_pools
RENAME COLUMN base_donation_claim_amount TO pro_rata_base_donation_claim_amount;

ALTER TABLE user_liquidity_pools
RENAME COLUMN quote_donation_claim_amount TO pro_rata_quote_donation_claim_amount;

