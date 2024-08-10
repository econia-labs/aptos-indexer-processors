use bigdecimal::BigDecimal;
use field_count::FieldCount;
use serde::{Deserialize, Serialize};

use crate::schema::periodic_state_events;

use super::enums::{PeriodicStateResolution, StateTrigger};

#[derive(Clone, Debug, Deserialize, FieldCount, Identifiable, Insertable, Serialize)]
#[diesel(primary_key(market_id, resolution, market_nonce))]
#[diesel(table_name = periodic_state_events)]
pub struct PeriodicStateEvent {
    // Transaction metadata.
    pub transaction_version: i64,
    pub sender: String,
    pub entry_function: String,
    pub inserted_at: chrono::NaiveDateTime,

    // Market metadata.
    pub market_id: i64,
    pub symbol_bytes: Vec<u8>,

    // State metadata.
    pub emit_time: chrono::NaiveDateTime,
    pub market_nonce: i64,
    pub trigger: StateTrigger,

    // Last swap data. The last swap can also be the event that triggered the periodic state event.
    pub last_swap_is_sell: bool,
    pub last_swap_avg_execution_price_q64: BigDecimal,
    pub last_swap_base_volume: BigDecimal,
    pub last_swap_quote_volume: BigDecimal,
    pub last_swap_nonce: i64,
    pub last_swap_emit_time: chrono::NaiveDateTime,

    // Periodic state metadata.
    pub resolution: PeriodicStateResolution,
    pub start_time: chrono::NaiveDateTime,

    // Periodic state event data.
    pub open_price_q64: BigDecimal,
    pub high_price_q64: BigDecimal,
    pub low_price_q64: BigDecimal,
    pub close_price_q64: BigDecimal,
    pub volume_base: BigDecimal,
    pub volume_quote: BigDecimal,
    pub integrator_fees: BigDecimal,
    pub pool_fees_base: BigDecimal,
    pub pool_fees_quote: BigDecimal,
    pub n_swaps: i64,
    pub n_chat_messages: i64,
    pub starts_in_bonding_curve: bool,
    pub ends_in_bonding_curve: bool,
    pub tvl_per_lp_coin_growth_q64: BigDecimal,
}
