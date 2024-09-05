use crate::{
    db::common::models::emojicoin_models::{
        enums,
        json_types::{MarketRegistrationEvent, StateEvent, TxnInfo},
        utils::micros_to_naive_datetime,
    },
    schema::market_registration_events,
};
use field_count::FieldCount;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Deserialize, FieldCount, Identifiable, Insertable, Serialize)]
#[diesel(primary_key(market_id))]
#[diesel(table_name = market_registration_events)]
pub struct MarketRegistrationEventModel {
    // Transaction metadata.
    pub transaction_version: i64,
    pub sender: String,
    pub entry_function: Option<String>,
    pub transaction_timestamp: chrono::NaiveDateTime,

    // Market and state metadata.
    pub market_id: i64,
    pub symbol_bytes: Vec<u8>,
    pub bump_time: chrono::NaiveDateTime,
    pub market_nonce: i64,
    pub trigger: enums::Trigger,

    // Market registration event data.
    pub registrant: String,
    pub integrator: String,
    pub integrator_fee: i64,
}

impl MarketRegistrationEventModel {
    pub fn new(
        txn_info: TxnInfo,
        market_registration_event: MarketRegistrationEvent,
        state_event: StateEvent,
    ) -> MarketRegistrationEventModel {
        let MarketRegistrationEvent {
            market_metadata,
            time,
            registrant,
            integrator,
            integrator_fee,
            ..
        } = market_registration_event;

        MarketRegistrationEventModel {
            // Transaction metadata.
            transaction_version: txn_info.version,
            sender: txn_info.sender.clone(),
            entry_function: txn_info.entry_function.clone(),
            transaction_timestamp: txn_info.timestamp,

            // Market and state metadata.
            market_id: market_metadata.market_id,
            symbol_bytes: market_metadata.emoji_bytes,
            bump_time: micros_to_naive_datetime(time),
            market_nonce: state_event.state_metadata.market_nonce,
            trigger: state_event.state_metadata.trigger,

            // Market registration event data.
            registrant,
            integrator,
            integrator_fee,
        }
    }
}

/*
 The below is an example JSON response of the State event data emitted upon market registration.

 This data is here to clarify why the data model above is so small compared to the event data actually emitted
 upon market registration.

 This is because the only fields that change from market to market are:
   - emoji_bytes
   - market_address
   - market_id
   - bump_time

{
    "clamm_virtual_reserves": {
        "base": "49000000000000000",
        "quote": "400000000000"
    },
    "cpamm_real_reserves": {
        "base": "0",
        "quote": "0"
    },
    "cumulative_stats": {
        "base_volume": "0",
        "integrator_fees": "100000000",
        "n_chat_messages": "0",
        "n_swaps": "0",
        "pool_fees_base": "0",
        "pool_fees_quote": "0",
        "quote_volume": "0"
    },
    "instantaneous_stats": {
        "fully_diluted_value": "367346938775",
        "market_cap": "0",
        "total_quote_locked": "0",
        "total_value_locked": "0"
    },
    "last_swap": {
        "avg_execution_price_q64": "0",
        "base_volume": "0",
        "is_sell": false,
        "nonce": "0",
        "quote_volume": "0",
        "time": "0"
    },
    "lp_coin_supply": "0",
    "market_metadata": {
        "emoji_bytes": "0xf09fa5b9e298baefb88f",     <-- Unique per market.
        "market_address": "0x190a6cba6...0b3d7aac",  <-- Unique per market.
        "market_id": "1777"                          <-- Unique per market.
    },
    "state_metadata": {
        "bump_time": "1720313606499938",             <-- Unique per market.
        "market_nonce": "1",
        "trigger": 1
    }
}
*/
