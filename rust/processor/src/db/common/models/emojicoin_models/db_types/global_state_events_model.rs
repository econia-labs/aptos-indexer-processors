use super::super::enums::StateTrigger;
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
    pub emit_time: chrono::NaiveDateTime,
    pub registry_nonce: i64,
    pub trigger: StateTrigger,
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
// Unfortunately, this is a limitation with `diesel`'s `insertable` derive macro, and it means we must have lots
// of duplicated code.
#[derive(Clone, Debug, Identifiable, Queryable)]
#[diesel(primary_key(registry_nonce))]
#[diesel(table_name = global_state_events)]
pub struct GlobalStateEventModelQuery {
    pub transaction_version: i64,
    pub sender: String,
    pub entry_function: Option<String>,
    pub emit_time: chrono::NaiveDateTime,
    pub registry_nonce: i64,
    pub trigger: StateTrigger,
    pub cumulative_quote_volume: BigDecimal,
    pub total_quote_locked: BigDecimal,
    pub total_value_locked: BigDecimal,
    pub market_cap: BigDecimal,
    pub fully_diluted_value: BigDecimal,
    pub cumulative_integrator_fees: BigDecimal,
    pub cumulative_swaps: i64,
    pub cumulative_chat_messages: i64,

    // Database metadata.
    pub inserted_at: chrono::NaiveDateTime,
}
