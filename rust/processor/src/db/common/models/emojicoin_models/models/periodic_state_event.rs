use super::super::enums::{PeriodType, Trigger};
use super::super::utils::micros_to_naive_datetime;
use crate::db::common::models::emojicoin_models::json_types::{
    LastSwap, PeriodicStateEvent, TxnInfo,
};
use crate::schema::periodic_state_events;
use bigdecimal::BigDecimal;
use field_count::FieldCount;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Deserialize, FieldCount, Identifiable, Insertable, Serialize)]
#[diesel(primary_key(market_id, period, market_nonce))]
#[diesel(table_name = periodic_state_events)]
pub struct PeriodicStateEventModel {
    // Transaction metadata.
    pub transaction_version: i64,
    pub sender: String,
    pub entry_function: Option<String>,
    pub transaction_timestamp: chrono::NaiveDateTime,

    // Market metadata.
    pub market_id: i64,
    pub symbol_bytes: Vec<u8>,

    // State metadata.
    pub emit_time: chrono::NaiveDateTime,
    pub market_nonce: i64,
    pub trigger: Trigger,

    // Last swap data. The last swap can also be the event that triggered the periodic state event.
    pub last_swap_is_sell: bool,
    pub last_swap_avg_execution_price_q64: BigDecimal,
    pub last_swap_base_volume: i64,
    pub last_swap_quote_volume: i64,
    pub last_swap_nonce: i64,
    pub last_swap_time: chrono::NaiveDateTime,

    // Periodic state metadata.
    pub period: PeriodType,
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

// Need a queryable version of the model to include the `inserted_at` field, since it's populated at insertion time.
// Unfortunately, this is a limitation with `diesel`'s `insertable` derive macro, and it means we must have lots
// of duplicated code.
#[derive(Clone, Debug, Identifiable, Queryable)]
#[diesel(primary_key(market_id, period, market_nonce))]
#[diesel(table_name = periodic_state_events)]
pub struct PeriodicStateEventModelQuery {
    // Transaction metadata.
    pub transaction_version: i64,
    pub sender: String,
    pub entry_function: Option<String>,
    pub inserted_at: chrono::NaiveDateTime,
    pub transaction_timestamp: chrono::NaiveDateTime,

    // Market metadata.
    pub market_id: i64,
    pub symbol_bytes: Vec<u8>,

    // State metadata.
    pub emit_time: chrono::NaiveDateTime,
    pub market_nonce: i64,
    pub trigger: Trigger,

    // Flattened `last_swap`. The last swap can also be the event that triggered the periodic state event.
    pub last_swap_is_sell: bool,
    pub last_swap_avg_execution_price_q64: BigDecimal,
    pub last_swap_base_volume: i64,
    pub last_swap_quote_volume: i64,
    pub last_swap_nonce: i64,
    pub last_swap_time: chrono::NaiveDateTime,

    // Periodic state metadata.
    pub period: PeriodType,
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

// Converting from our strongly typed, previously JSON data to the database model.
impl PeriodicStateEventModel {
    pub fn from_periodic_events(
        txn_info: TxnInfo,
        periodic_state_events: Vec<PeriodicStateEvent>,
        last_swap: LastSwap,
    ) -> Vec<PeriodicStateEventModel> {
        periodic_state_events
            .into_iter()
            .map(|ps_event| PeriodicStateEventModel {
                transaction_version: txn_info.version,
                sender: txn_info.sender.clone(),
                entry_function: txn_info.entry_function.clone(),
                transaction_timestamp: txn_info.timestamp,
                market_id: ps_event.market_metadata.market_id,
                symbol_bytes: ps_event.market_metadata.emoji_bytes,
                emit_time: micros_to_naive_datetime(ps_event.periodic_state_metadata.emit_time),
                market_nonce: ps_event.periodic_state_metadata.emit_market_nonce,
                trigger: ps_event.periodic_state_metadata.trigger,
                last_swap_is_sell: last_swap.is_sell,
                last_swap_avg_execution_price_q64: last_swap.avg_execution_price_q64.clone(),
                last_swap_base_volume: last_swap.base_volume,
                last_swap_quote_volume: last_swap.quote_volume,
                last_swap_nonce: last_swap.nonce,
                last_swap_time: micros_to_naive_datetime(last_swap.time),
                period: ps_event.periodic_state_metadata.period,
                start_time: micros_to_naive_datetime(ps_event.periodic_state_metadata.start_time),
                open_price_q64: ps_event.open_price_q64,
                high_price_q64: ps_event.high_price_q64,
                low_price_q64: ps_event.low_price_q64,
                close_price_q64: ps_event.close_price_q64,
                volume_base: ps_event.volume_base,
                volume_quote: ps_event.volume_quote,
                integrator_fees: ps_event.integrator_fees,
                pool_fees_base: ps_event.pool_fees_base,
                pool_fees_quote: ps_event.pool_fees_quote,
                n_swaps: ps_event.n_swaps,
                n_chat_messages: ps_event.n_chat_messages,
                starts_in_bonding_curve: ps_event.starts_in_bonding_curve,
                ends_in_bonding_curve: ps_event.ends_in_bonding_curve,
                tvl_per_lp_coin_growth_q64: ps_event.tvl_per_lp_coin_growth_q64,
            })
            .collect()
    }
}
