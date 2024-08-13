use crate::db::common::models::emojicoin_models::json_types::{StateEvent, SwapEvent};
use crate::db::common::models::emojicoin_models::utils::micros_to_naive_datetime;
use crate::db::common::models::emojicoin_models::{enums, json_types::TxnInfo};
use crate::schema::swap_events;
use bigdecimal::BigDecimal;
use field_count::FieldCount;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Deserialize, FieldCount, Identifiable, Insertable, Serialize)]
#[diesel(primary_key(market_id, market_nonce))]
#[diesel(table_name = swap_events)]
pub struct SwapEventModel {
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

    // Swap event data.
    swapper: String,
    integrator: String,
    integrator_fee: i64,
    input_amount: i64,
    is_sell: bool,
    integrator_fee_rate_bps: i16,
    net_proceeds: i64,
    base_volume: i64,
    quote_volume: i64,
    avg_execution_price_q64: BigDecimal,
    pool_fee: i64,
    starts_in_bonding_curve: bool,
    results_in_state_transition: bool,

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
#[diesel(table_name = swap_events)]
pub struct SwapEventModelQuery {
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

    // Swap event data.
    swapper: String,
    integrator: String,
    integrator_fee: i64,
    input_amount: i64,
    is_sell: bool,
    integrator_fee_rate_bps: i16,
    net_proceeds: i64,
    base_volume: i64,
    quote_volume: i64,
    avg_execution_price_q64: BigDecimal,
    pool_fee: i64,
    starts_in_bonding_curve: bool,
    results_in_state_transition: bool,

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

impl SwapEventModel {
    pub fn new(
        txn_info: TxnInfo,
        swap_event: SwapEvent,
        state_event: StateEvent,
    ) -> SwapEventModel {
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

        SwapEventModel {
            // Transaction metadata.
            transaction_version: txn_info.version,
            sender: txn_info.sender.clone(),
            entry_function: txn_info.entry_function.clone(),
            transaction_timestamp: txn_info.timestamp,

            // Market and state metadata.
            market_id: swap_event.market_id,
            symbol_bytes: market_metadata.emoji_bytes,
            bump_time: micros_to_naive_datetime(swap_event.time),
            market_nonce: swap_event.market_nonce,
            trigger: state_metadata.trigger,

            // Swap event data.
            swapper: swap_event.swapper,
            integrator: swap_event.integrator,
            integrator_fee: swap_event.integrator_fee,
            input_amount: swap_event.input_amount,
            is_sell: swap_event.is_sell,
            integrator_fee_rate_bps: swap_event.integrator_fee_rate_bps,
            net_proceeds: swap_event.net_proceeds,
            base_volume: swap_event.base_volume,
            quote_volume: swap_event.quote_volume,
            avg_execution_price_q64: swap_event.avg_execution_price_q64,
            pool_fee: swap_event.pool_fee,
            starts_in_bonding_curve: swap_event.starts_in_bonding_curve,
            results_in_state_transition: swap_event.results_in_state_transition,

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
