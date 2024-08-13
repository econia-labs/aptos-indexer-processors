use crate::db::common::models::emojicoin_models::json_types::{LiquidityEvent, StateEvent};
use crate::db::common::models::emojicoin_models::utils::micros_to_naive_datetime;
use crate::db::common::models::emojicoin_models::{enums, json_types::TxnInfo};
use crate::schema::liquidity_events;
use bigdecimal::BigDecimal;
use field_count::FieldCount;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Deserialize, FieldCount, Identifiable, Insertable, Serialize)]
#[diesel(primary_key(market_id, market_nonce))]
#[diesel(table_name = liquidity_events)]
pub struct LiquidityEventModel {
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

    // Liquidity event data.
    provider: String,
    base_amount: i64,
    quote_amount: i64,
    lp_coin_amount: i64,
    liquidity_provided: bool,
    pro_rata_base_donation_claim_amount: i64,
    pro_rata_quote_donation_claim_amount: i64,

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
#[diesel(table_name = liquidity_events)]
pub struct LiquidityEventModelQuery {
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

    // Liquidity event data.
    provider: String,
    base_amount: i64,
    quote_amount: i64,
    lp_coin_amount: i64,
    liquidity_provided: bool,
    pro_rata_base_donation_claim_amount: i64,
    pro_rata_quote_donation_claim_amount: i64,

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

impl LiquidityEventModel {
    pub fn new(
        txn_info: TxnInfo,
        liquidity_event: LiquidityEvent,
        state_event: StateEvent,
    ) -> LiquidityEventModel {
        let StateEvent {
            market_metadata,
            state_metadata,
            clamm_virtual_reserves: clamm,
            cpamm_real_reserves: cpamm,
            lp_coin_supply,
            cumulative_stats: c_stats,
            instantaneous_stats: i_stats,
            last_swap,
            ..
        } = state_event;

        let LiquidityEvent {
            time,
            provider,
            base_amount,
            quote_amount,
            lp_coin_amount,
            liquidity_provided,
            pro_rata_base_donation_claim_amount,
            pro_rata_quote_donation_claim_amount,
            ..
        } = liquidity_event;

        LiquidityEventModel {
            // Transaction metadata.
            transaction_version: txn_info.version,
            sender: txn_info.sender.clone(),
            entry_function: txn_info.entry_function.clone(),
            transaction_timestamp: txn_info.timestamp,

            // Market and state metadata.
            market_id: liquidity_event.market_id,
            symbol_bytes: market_metadata.emoji_bytes,
            bump_time: micros_to_naive_datetime(time),
            market_nonce: liquidity_event.market_nonce,
            trigger: state_metadata.trigger,

            // Liquidity event data.
            provider,
            base_amount,
            quote_amount,
            lp_coin_amount,
            liquidity_provided,
            pro_rata_base_donation_claim_amount,
            pro_rata_quote_donation_claim_amount,

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