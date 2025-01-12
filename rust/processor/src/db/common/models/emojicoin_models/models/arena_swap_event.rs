use crate::{
    db::common::models::emojicoin_models::json_types::{ArenaSwapEvent, TxnInfo},
    schema::arena_swap_events,
};
use field_count::FieldCount;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Deserialize, FieldCount, Identifiable, Insertable, Serialize)]
#[diesel(primary_key(transaction_version, event_index))]
#[diesel(table_name = arena_swap_events)]
pub struct ArenaSwapEventModel {
    // Transaction metadata.
    pub transaction_version: i64,
    pub event_index: i64,
    pub sender: String,
    pub entry_function: Option<String>,
    pub transaction_timestamp: chrono::NaiveDateTime,

    pub user: String,
    pub melee_id: i64,
    pub quote_volume: i64,
    pub integrator_fee: i64,

    pub emojicoin_0_proceeds: i64,
    pub emojicoin_1_proceeds: i64,
    pub emojicoin_0_exchange_rate_base: i64,
    pub emojicoin_0_exchange_rate_quote: i64,
    pub emojicoin_1_exchange_rate_base: i64,
    pub emojicoin_1_exchange_rate_quote: i64,
}

impl ArenaSwapEventModel {
    pub fn new(txn_info: TxnInfo, arena_swap_event: ArenaSwapEvent) -> ArenaSwapEventModel {
        ArenaSwapEventModel {
            // Transaction metadata.
            transaction_version: txn_info.version,
            event_index: arena_swap_event.event_index,
            sender: txn_info.sender.clone(),
            entry_function: txn_info.entry_function.clone(),
            transaction_timestamp: txn_info.timestamp,

            user: arena_swap_event.user,
            melee_id: arena_swap_event.melee_id,
            quote_volume: arena_swap_event.quote_volume,
            integrator_fee: arena_swap_event.integrator_fee,

            emojicoin_0_proceeds: arena_swap_event.emojicoin_0_proceeds,
            emojicoin_1_proceeds: arena_swap_event.emojicoin_1_proceeds,
            emojicoin_0_exchange_rate_base: arena_swap_event.emojicoin_0_exchange_rate.base,
            emojicoin_0_exchange_rate_quote: arena_swap_event.emojicoin_0_exchange_rate.quote,
            emojicoin_1_exchange_rate_base: arena_swap_event.emojicoin_1_exchange_rate.base,
            emojicoin_1_exchange_rate_quote: arena_swap_event.emojicoin_1_exchange_rate.quote,
        }
    }
}
