use crate::db::common::models::emojicoin_models::{
    enums,
    json_types::{GlobalStateEvent, TxnInfo},
    utils::micros_to_naive_datetime,
};
use crate::schema::global_state_events;
use bigdecimal::BigDecimal;
use field_count::FieldCount;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Deserialize, FieldCount, Identifiable, Insertable, Serialize)]
#[diesel(primary_key(registry_nonce))]
#[diesel(table_name = global_state_events)]
pub struct GlobalStateEventModel {
    pub transaction_version: i64,
    pub sender: String,
    pub entry_function: Option<String>,
    pub transaction_timestamp: chrono::NaiveDateTime,
    pub emit_time: chrono::NaiveDateTime,
    pub registry_nonce: i64,
    pub trigger: enums::Triggers,
    pub cumulative_quote_volume: BigDecimal,
    pub total_quote_locked: BigDecimal,
    pub total_value_locked: BigDecimal,
    pub market_cap: BigDecimal,
    pub fully_diluted_value: BigDecimal,
    pub cumulative_integrator_fees: BigDecimal,
    pub cumulative_swaps: i64,
    pub cumulative_chat_messages: i64,
}

// Need a queryable version of the model to include the `inserted_at` field, since it's populated at insertion time.
// Unfortunately, this is a limitation with `diesel`'s `insertable` derive macro.
#[derive(Identifiable, Queryable)]
#[diesel(primary_key(registry_nonce))]
#[diesel(table_name = global_state_events)]
pub struct GlobalStateEventModelQuery {
    pub transaction_version: i64,
    pub sender: String,
    pub entry_function: Option<String>,
    pub transaction_timestamp: chrono::NaiveDateTime,
    pub inserted_at: chrono::NaiveDateTime,
    pub emit_time: chrono::NaiveDateTime,
    pub registry_nonce: i64,
    pub trigger: enums::Triggers,
    pub cumulative_quote_volume: BigDecimal,
    pub total_quote_locked: BigDecimal,
    pub total_value_locked: BigDecimal,
    pub market_cap: BigDecimal,
    pub fully_diluted_value: BigDecimal,
    pub cumulative_integrator_fees: BigDecimal,
    pub cumulative_swaps: i64,
    pub cumulative_chat_messages: i64,
}

impl GlobalStateEventModel {
    pub fn new(txn_info: TxnInfo, global_state_event: GlobalStateEvent) -> Self {
        GlobalStateEventModel {
            transaction_version: txn_info.version,
            sender: txn_info.sender,
            entry_function: txn_info.entry_function,
            transaction_timestamp: txn_info.timestamp,
            emit_time: micros_to_naive_datetime(global_state_event.emit_time),
            registry_nonce: global_state_event.registry_nonce,
            trigger: global_state_event.trigger,
            cumulative_quote_volume: global_state_event.cumulative_quote_volume,
            total_quote_locked: global_state_event.total_quote_locked,
            total_value_locked: global_state_event.total_value_locked,
            market_cap: global_state_event.market_cap,
            fully_diluted_value: global_state_event.fully_diluted_value,
            cumulative_integrator_fees: global_state_event.cumulative_integrator_fees,
            cumulative_swaps: global_state_event.cumulative_swaps,
            cumulative_chat_messages: global_state_event.cumulative_chat_messages,
        }
    }
}
