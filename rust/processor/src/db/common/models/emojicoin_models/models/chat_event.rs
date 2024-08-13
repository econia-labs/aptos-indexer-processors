use crate::db::common::models::emojicoin_models::json_types::{StateEvent, TxnInfo};
use crate::db::common::models::emojicoin_models::utils::micros_to_naive_datetime;
use crate::db::common::models::emojicoin_models::{enums, json_types::ChatEvent};
use crate::schema::chat_events;
use bigdecimal::BigDecimal;
use field_count::FieldCount;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Deserialize, FieldCount, Identifiable, Insertable, Serialize)]
#[diesel(primary_key(market_id, market_nonce))]
#[diesel(table_name = chat_events)]
pub struct ChatEventModel {
    // Transaction metadata.
    transaction_version: i64,
    sender: String,
    entry_function: Option<String>,
    transaction_timestamp: chrono::NaiveDateTime,

    // Market and state metadata.
    market_id: i64,
    symbol_bytes: Vec<u8>,
    bump_time: chrono::NaiveDateTime,
    market_nonce: i64,
    trigger: enums::Triggers,

    // Chat event data.
    user: String,
    message: String,
    user_emojicoin_balance: i64,
    circulating_supply: i64,
    balance_as_fraction_of_circulating_supply_q64: BigDecimal,

    // State event data.
    clamm_virtual_reserves_base: i64,
    clamm_virtual_reserves_quote: i64,
    cpamm_real_reserves_base: i64,
    cpamm_real_reserves_quote: i64,
    lp_coin_supply: BigDecimal,
    cumulative_stats_base_volume: BigDecimal,
    cumulative_stats_quote_volume: BigDecimal,
    cumulative_stats_integrator_fees: BigDecimal,
    cumulative_stats_pool_fees_base: BigDecimal,
    cumulative_stats_pool_fees_quote: BigDecimal,
    cumulative_stats_n_swaps: i64,
    cumulative_stats_n_chat_messages: i64,
    instantaneous_stats_total_quote_locked: i64,
    instantaneous_stats_total_value_locked: BigDecimal,
    instantaneous_stats_market_cap: BigDecimal,
    instantaneous_stats_fully_diluted_value: BigDecimal,
    last_swap_is_sell: bool,
    last_swap_avg_execution_price_q64: BigDecimal,
    last_swap_base_volume: i64,
    last_swap_quote_volume: i64,
    last_swap_nonce: i64,
    last_swap_time: chrono::NaiveDateTime,
}

// Need a queryable version of the model to include the `inserted_at` field, since it's populated at insertion time.
// Unfortunately, this is a limitation with `diesel`'s `insertable` derive macro.
#[derive(Clone, Debug, Identifiable, Queryable)]
#[diesel(primary_key(market_id, market_nonce))]
#[diesel(table_name = chat_events)]
pub struct ChatEventModelQuery {
    // Transaction metadata.
    transaction_version: i64,
    sender: String,
    entry_function: Option<String>,
    transaction_timestamp: chrono::NaiveDateTime,
    inserted_at: chrono::NaiveDateTime,

    // Market and state metadata.
    market_id: i64,
    symbol_bytes: Vec<u8>,
    bump_time: chrono::NaiveDateTime,
    market_nonce: i64,
    trigger: enums::Triggers,

    // Chat event data.
    user: String,
    message: String,
    user_emojicoin_balance: i64,
    circulating_supply: i64,
    balance_as_fraction_of_circulating_supply_q64: BigDecimal,

    // State event data.
    clamm_virtual_reserves_base: i64,
    clamm_virtual_reserves_quote: i64,
    cpamm_real_reserves_base: i64,
    cpamm_real_reserves_quote: i64,
    lp_coin_supply: BigDecimal,
    cumulative_stats_base_volume: BigDecimal,
    cumulative_stats_quote_volume: BigDecimal,
    cumulative_stats_integrator_fees: BigDecimal,
    cumulative_stats_pool_fees_base: BigDecimal,
    cumulative_stats_pool_fees_quote: BigDecimal,
    cumulative_stats_n_swaps: i64,
    cumulative_stats_n_chat_messages: i64,
    instantaneous_stats_total_quote_locked: i64,
    instantaneous_stats_total_value_locked: BigDecimal,
    instantaneous_stats_market_cap: BigDecimal,
    instantaneous_stats_fully_diluted_value: BigDecimal,
    last_swap_is_sell: bool,
    last_swap_avg_execution_price_q64: BigDecimal,
    last_swap_base_volume: i64,
    last_swap_quote_volume: i64,
    last_swap_nonce: i64,
    last_swap_time: chrono::NaiveDateTime,
}

impl ChatEventModel {
    pub fn new(
        txn_info: TxnInfo,
        chat_event: ChatEvent,
        state_event: StateEvent,
    ) -> ChatEventModel {
        let StateEvent {
            state_metadata,
            clamm_virtual_reserves: clamm,
            cpamm_real_reserves: cpamm,
            lp_coin_supply,
            cumulative_stats: c_stats,
            instantaneous_stats: i_stats,
            last_swap,
            ..
        } = state_event;

        let ChatEvent {
            user,
            market_metadata,
            message,
            user_emojicoin_balance,
            circulating_supply,
            balance_as_fraction_of_circulating_supply_q64,
            ..
        } = chat_event;

        ChatEventModel {
            // Transaction metadata.
            transaction_version: txn_info.version,
            sender: txn_info.sender.clone(),
            entry_function: txn_info.entry_function.clone(),
            transaction_timestamp: txn_info.timestamp,

            // Market and state metadata.
            market_id: market_metadata.market_id,
            symbol_bytes: market_metadata.emoji_bytes,
            bump_time: micros_to_naive_datetime(state_metadata.bump_time),
            market_nonce: state_metadata.market_nonce,
            trigger: state_metadata.trigger,

            // Chat event data.
            user,
            message,
            user_emojicoin_balance,
            circulating_supply,
            balance_as_fraction_of_circulating_supply_q64,

            // State event data.
            clamm_virtual_reserves_base: clamm.base,
            clamm_virtual_reserves_quote: clamm.quote,
            cpamm_real_reserves_base: cpamm.base,
            cpamm_real_reserves_quote: cpamm.quote,
            lp_coin_supply: lp_coin_supply.clone(),
            cumulative_stats_base_volume: c_stats.base_volume,
            cumulative_stats_quote_volume: c_stats.quote_volume,
            cumulative_stats_integrator_fees: c_stats.integrator_fees,
            cumulative_stats_pool_fees_base: c_stats.pool_fees_base,
            cumulative_stats_pool_fees_quote: c_stats.pool_fees_quote,
            cumulative_stats_n_swaps: c_stats.n_swaps,
            cumulative_stats_n_chat_messages: c_stats.n_chat_messages,
            instantaneous_stats_total_quote_locked: i_stats.total_quote_locked,
            instantaneous_stats_total_value_locked: i_stats.total_value_locked,
            instantaneous_stats_market_cap: i_stats.market_cap,
            instantaneous_stats_fully_diluted_value: i_stats.fully_diluted_value,
            last_swap_is_sell: last_swap.is_sell,
            last_swap_avg_execution_price_q64: last_swap.avg_execution_price_q64.clone(),
            last_swap_base_volume: last_swap.base_volume,
            last_swap_quote_volume: last_swap.quote_volume,
            last_swap_nonce: last_swap.nonce,
            last_swap_time: micros_to_naive_datetime(last_swap.time),
        }
    }
}