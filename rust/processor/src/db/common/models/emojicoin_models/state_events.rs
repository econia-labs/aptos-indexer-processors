use super::enums::StateTrigger;
use crate::schema::state_events;
use bigdecimal::BigDecimal;
use field_count::FieldCount;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Deserialize, FieldCount, Identifiable, Insertable, Serialize)]
#[diesel(primary_key(market_id, market_nonce))]
#[diesel(table_name = state_events)]
pub struct StateEvent {
    // Transaction metadata.
    pub transaction_version: i64,
    pub sender: String,
    pub entry_function: String,
    pub inserted_at: chrono::NaiveDateTime,

    // Market metadata.
    pub market_id: i64,
    pub symbol_bytes: Vec<u8>,

    // State metadata.
    pub bump_time: chrono::NaiveDateTime,
    pub market_nonce: i64,
    pub trigger: StateTrigger,

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
    pub last_swap_emit_time: chrono::NaiveDateTime,

    // Market registration & Swap data.
    pub integrator: String,
    pub integrator_fee: i64,

    // Swap event data.
    pub input_amount: i64,
    pub is_sell: bool,
    pub integrator_fee_rate_bps: i16,
    pub net_proceeds: i64,
    pub base_volume: i64,
    pub quote_volume: i64,
    pub avg_execution_price_q64: BigDecimal,
    pub pool_fee: i64,
    pub starts_in_bonding_curve: bool,
    pub results_in_state_transition: bool,

    // Liquidity event data.
    pub base_amount: i64,
    pub quote_amount: i64,
    pub lp_coin_amount: i64,
    pub liquidity_provided: bool,
    pub pro_rata_base_donation_claim_amount: i64,
    pub pro_rata_quote_donation_claim_amount: i64,

    // Chat event data.
    pub message: String,
    pub user_emojicoin_balance: i64,
    pub circulating_supply: i64,
    pub balance_as_fraction_of_circulating_supply_q64: BigDecimal,
}
