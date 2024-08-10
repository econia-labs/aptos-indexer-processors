use crate::db::common::models::emojicoin_models::enums::StateTrigger;
use crate::schema::state_bumps;
use bigdecimal::BigDecimal;
use field_count::FieldCount;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Deserialize, FieldCount, Identifiable, Insertable, Serialize)]
#[diesel(primary_key(market_id, market_nonce))]
#[diesel(table_name = state_bumps)]
pub struct StateBumpModel {
    // Transaction metadata.
    pub transaction_version: i64,
    pub sender: String,
    pub entry_function: Option<String>,

    // Market metadata.
    pub market_id: i64,
    pub symbol_bytes: Vec<u8>,

    // State metadata.
    pub bump_time: chrono::NaiveDateTime,
    pub market_nonce: i64,
    pub trigger: StateTrigger,

    // ---------------- State data ----------------
    // Flattened `cpamm_virtual_reserves`.
    pub clamm_virtual_reserves_base: i64,
    pub clamm_virtual_reserves_quote: i64,
    // Flattened `clamm_real_reserves`.
    pub cpamm_real_reserves_base: i64,
    pub cpamm_real_reserves_quote: i64,
    pub lp_coin_supply: BigDecimal,
    // Flattened `cumulative_stats`.
    pub cumulative_base_volume: BigDecimal,
    pub cumulative_quote_volume: BigDecimal,
    pub cumulative_integrator_fees: BigDecimal,
    pub cumulative_pool_fees_base: BigDecimal,
    pub cumulative_pool_fees_quote: BigDecimal,
    pub cumulative_n_swaps: i64,
    pub cumulative_n_chat_messages: i64,
    // Flattened `instantaneous_stats`.
    pub instantaneous_stats_total_quote_locked: i64,
    pub instantaneous_total_value_locked: BigDecimal,
    pub instantaneous_market_cap: BigDecimal,
    pub instantaneous_fully_diluted_value: BigDecimal,

    // Flattened `last_swap`. The last swap can also be the event that triggered the periodic state event.
    pub last_swap_is_sell: bool,
    pub last_swap_avg_execution_price_q64: BigDecimal,
    pub last_swap_base_volume: BigDecimal,
    pub last_swap_quote_volume: BigDecimal,
    pub last_swap_nonce: i64,
    pub last_swap_time: chrono::NaiveDateTime,

    // Market registration & Swap data.
    pub integrator: Option<String>,
    pub integrator_fee: Option<i64>,

    // Swap event data.
    pub input_amount: Option<i64>,
    pub is_sell: Option<bool>,
    pub integrator_fee_rate_bps: Option<i16>,
    pub net_proceeds: Option<i64>,
    pub base_volume: Option<i64>,
    pub quote_volume: Option<i64>,
    pub avg_execution_price_q64: Option<BigDecimal>,
    pub pool_fee: Option<i64>,
    pub starts_in_bonding_curve: Option<bool>,
    pub results_in_state_transition: Option<bool>,

    // Liquidity event data.
    pub base_amount: Option<i64>,
    pub quote_amount: Option<i64>,
    pub lp_coin_amount: Option<i64>,
    pub liquidity_provided: Option<bool>,
    pub pro_rata_base_donation_claim_amount: Option<i64>,
    pub pro_rata_quote_donation_claim_amount: Option<i64>,

    // Chat event data.
    pub message: Option<String>,
    pub user_emojicoin_balance: Option<i64>,
    pub circulating_supply: Option<i64>,
    pub balance_as_fraction_of_circulating_supply_q64: Option<BigDecimal>,
}

// Need a queryable version of the model to include the `inserted_at` field, since it's populated at insertion time.
// Unfortunately, this is a limitation with `diesel`'s `insertable` derive macro, and it means we must have lots
// of duplicated code.
#[derive(Clone, Debug, Identifiable, Queryable)]
#[diesel(primary_key(market_id, market_nonce))]
#[diesel(table_name = state_bumps)]
pub struct StateBumpModelQuery {
    // Transaction metadata.
    pub transaction_version: i64,
    pub sender: String,
    pub entry_function: Option<String>,

    // Market metadata.
    pub market_id: i64,
    pub symbol_bytes: Vec<u8>,

    // State metadata.
    pub bump_time: chrono::NaiveDateTime,
    pub market_nonce: i64,
    pub trigger: StateTrigger,

    // State data.
    pub clamm_virtual_reserves_base: i64,
    pub clamm_virtual_reserves_quote: i64,
    pub cpamm_real_reserves_base: i64,
    pub cpamm_real_reserves_quote: i64,
    pub lp_coin_supply: BigDecimal,
    pub cumulative_base_volume: BigDecimal,
    pub cumulative_quote_volume: BigDecimal,
    pub cumulative_integrator_fees: BigDecimal,
    pub cumulative_pool_fees_base: BigDecimal,
    pub cumulative_pool_fees_quote: BigDecimal,
    pub cumulative_n_swaps: i64,
    pub cumulative_n_chat_messages: i64,
    pub instantaneous_stats_total_quote_locked: i64,
    pub instantaneous_total_value_locked: BigDecimal,
    pub instantaneous_market_cap: BigDecimal,
    pub instantaneous_fully_diluted_value: BigDecimal,

    // Last swap data. The last swap can also be the event that triggered the periodic state event.
    pub last_swap_is_sell: bool,
    pub last_swap_avg_execution_price_q64: BigDecimal,
    pub last_swap_base_volume: BigDecimal,
    pub last_swap_quote_volume: BigDecimal,
    pub last_swap_nonce: i64,
    pub last_swap_time: chrono::NaiveDateTime,

    // Market registration & Swap data.
    pub integrator: Option<String>,
    pub integrator_fee: Option<i64>,

    // Swap event data.
    pub input_amount: Option<i64>,
    pub is_sell: Option<bool>,
    pub integrator_fee_rate_bps: Option<i16>,
    pub net_proceeds: Option<i64>,
    pub base_volume: Option<i64>,
    pub quote_volume: Option<i64>,
    pub avg_execution_price_q64: Option<BigDecimal>,
    pub pool_fee: Option<i64>,
    pub starts_in_bonding_curve: Option<bool>,
    pub results_in_state_transition: Option<bool>,

    // Liquidity event data.
    pub base_amount: Option<i64>,
    pub quote_amount: Option<i64>,
    pub lp_coin_amount: Option<i64>,
    pub liquidity_provided: Option<bool>,
    pub pro_rata_base_donation_claim_amount: Option<i64>,
    pub pro_rata_quote_donation_claim_amount: Option<i64>,

    // Chat event data.
    pub message: Option<String>,
    pub user_emojicoin_balance: Option<i64>,
    pub circulating_supply: Option<i64>,
    pub balance_as_fraction_of_circulating_supply_q64: Option<BigDecimal>,

    // Database metadata.
    pub inserted_at: chrono::NaiveDateTime,
}
