use std::borrow::Borrow;

use super::super::enums::Trigger;
use crate::db::common::models::emojicoin_models::json_types::{BumpEvent, StateEvent, TxnInfo};
use crate::db::common::models::emojicoin_models::utils::micros_to_naive_datetime;
use crate::schema::bump_events;
use bigdecimal::BigDecimal;
use field_count::FieldCount;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Deserialize, FieldCount, Identifiable, Insertable, Serialize)]
#[diesel(primary_key(market_id, market_nonce))]
#[diesel(table_name = bump_events)]
pub struct BumpEventModel {
    // Transaction metadata.
    pub transaction_version: i64,
    pub sender: String,
    pub entry_function: Option<String>,
    pub transaction_timestamp: chrono::NaiveDateTime,

    // Market metadata.
    pub market_id: i64,
    pub symbol_bytes: Vec<u8>,

    // State metadata.
    pub bump_time: chrono::NaiveDateTime,
    pub market_nonce: i64,
    pub trigger: Trigger,

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
    pub instantaneous_stats_total_value_locked: BigDecimal,
    pub instantaneous_stats_market_cap: BigDecimal,
    pub instantaneous_stats_fully_diluted_value: BigDecimal,

    // Flattened `last_swap`. The last swap can also be the event that triggered the periodic state event.
    pub last_swap_is_sell: bool,
    pub last_swap_avg_execution_price_q64: BigDecimal,
    pub last_swap_base_volume: i64,
    pub last_swap_quote_volume: i64,
    pub last_swap_nonce: i64,
    pub last_swap_time: chrono::NaiveDateTime,

    //------ Data in multiple event types --------
    // All bump events have a user, either a `registrant`, a `swapper`, a `provider`, or a `user`.
    pub user_address: String,

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
#[diesel(table_name = bump_events)]
pub struct BumpEventModelQuery {
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
    pub bump_time: chrono::NaiveDateTime,
    pub market_nonce: i64,
    pub trigger: Trigger,

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
    pub instantaneous_stats_total_value_locked: BigDecimal,
    pub instantaneous_stats_market_cap: BigDecimal,
    pub instantaneous_stats_fully_diluted_value: BigDecimal,

    // Last swap data. The last swap can also be the event that triggered the periodic state event.
    pub last_swap_is_sell: bool,
    pub last_swap_avg_execution_price_q64: BigDecimal,
    pub last_swap_base_volume: BigDecimal,
    pub last_swap_quote_volume: BigDecimal,
    pub last_swap_nonce: i64,
    pub last_swap_time: chrono::NaiveDateTime,

    //------ Data in multiple event types --------
    // All bump events have a user, either a `registrant`, a `swapper`, a `provider`, or a `user`.
    pub user_address: String,

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

// Converting from our strongly typed, previously JSON data to the database model.
impl BumpEventModel {
    pub fn from_bump_and_state_event(
        txn_info: TxnInfo,
        bump_event: BumpEvent,
        state_event: StateEvent,
    ) -> BumpEventModel {
        let StateEvent {
            state_metadata,
            market_metadata,
            clamm_virtual_reserves: clamm,
            cpamm_real_reserves: cpamm,
            lp_coin_supply,
            cumulative_stats: c_stats,
            instantaneous_stats: i_stats,
            last_swap,
            ..
        } = state_event;

        let (integrator, integrator_fee) = match bump_event.borrow() {
            BumpEvent::Swap(e) => (Some(e.integrator.clone()), Some(e.integrator_fee)),
            BumpEvent::MarketRegistration(e) => {
                (Some(e.integrator.clone()), Some(e.integrator_fee))
            },
            _ => (None, None),
        };

        let (
            input_amount,
            is_sell,
            integrator_fee_rate_bps,
            net_proceeds,
            base_volume,
            quote_volume,
            avg_execution_price_q64,
            pool_fee,
            starts_in_bonding_curve,
            results_in_state_transition,
        ) = match bump_event.borrow() {
            BumpEvent::Swap(e) => (
                Some(e.input_amount),
                Some(e.is_sell),
                Some(e.integrator_fee_rate_bps),
                Some(e.net_proceeds),
                Some(e.base_volume),
                Some(e.quote_volume),
                Some(e.avg_execution_price_q64.clone()),
                Some(e.pool_fee),
                Some(e.starts_in_bonding_curve),
                Some(e.results_in_state_transition),
            ),
            _ => (None, None, None, None, None, None, None, None, None, None),
        };

        let (
            base_amount,
            quote_amount,
            lp_coin_amount,
            liquidity_provided,
            pro_rata_base_donation_claim_amount,
            pro_rata_quote_donation_claim_amount,
        ) = match bump_event.borrow() {
            BumpEvent::Liquidity(e) => (
                Some(e.base_amount),
                Some(e.quote_amount),
                Some(e.lp_coin_amount),
                Some(e.liquidity_provided),
                Some(e.pro_rata_base_donation_claim_amount),
                Some(e.pro_rata_quote_donation_claim_amount),
            ),
            _ => (None, None, None, None, None, None),
        };

        let (
            message,
            user_emojicoin_balance,
            circulating_supply,
            balance_as_fraction_of_circulating_supply_q64,
        ) = match bump_event.borrow() {
            BumpEvent::Chat(e) => (
                Some(e.message.clone()),
                Some(e.user_emojicoin_balance),
                Some(e.circulating_supply),
                Some(e.balance_as_fraction_of_circulating_supply_q64.clone()),
            ),
            _ => (None, None, None, None),
        };

        let user_address = match bump_event.borrow() {
            BumpEvent::Swap(e) => e.swapper.clone(),
            BumpEvent::MarketRegistration(e) => e.registrant.clone(),
            BumpEvent::Liquidity(e) => e.provider.clone(),
            BumpEvent::Chat(e) => e.user.clone(),
        };

        BumpEventModel {
            transaction_version: txn_info.version,
            sender: txn_info.sender.clone(),
            entry_function: txn_info.entry_function.clone(),
            transaction_timestamp: txn_info.timestamp,
            market_id: market_metadata.market_id,
            symbol_bytes: market_metadata.emoji_bytes.clone(),
            bump_time: micros_to_naive_datetime(state_metadata.bump_time),
            market_nonce: state_metadata.market_nonce,
            trigger: state_metadata.trigger,
            last_swap_is_sell: last_swap.is_sell,
            last_swap_avg_execution_price_q64: last_swap.avg_execution_price_q64.clone(),
            last_swap_base_volume: last_swap.base_volume,
            last_swap_quote_volume: last_swap.quote_volume,
            last_swap_nonce: last_swap.nonce,
            last_swap_time: micros_to_naive_datetime(last_swap.time),
            clamm_virtual_reserves_base: clamm.base,
            clamm_virtual_reserves_quote: clamm.quote,
            cpamm_real_reserves_base: cpamm.base,
            cpamm_real_reserves_quote: cpamm.quote,
            lp_coin_supply,
            cumulative_base_volume: c_stats.base_volume,
            cumulative_quote_volume: c_stats.quote_volume,
            cumulative_integrator_fees: c_stats.integrator_fees,
            cumulative_pool_fees_base: c_stats.pool_fees_base,
            cumulative_pool_fees_quote: c_stats.pool_fees_quote,
            cumulative_n_swaps: c_stats.n_swaps,
            cumulative_n_chat_messages: c_stats.n_chat_messages,
            instantaneous_stats_total_quote_locked: i_stats.total_quote_locked,
            instantaneous_stats_total_value_locked: i_stats.total_value_locked,
            instantaneous_stats_market_cap: i_stats.market_cap,
            instantaneous_stats_fully_diluted_value: i_stats.fully_diluted_value,
            user_address,
            // Market registration & Swap data.
            integrator,
            integrator_fee,
            // Swap event data.
            input_amount,
            is_sell,
            integrator_fee_rate_bps,
            net_proceeds,
            base_volume,
            quote_volume,
            avg_execution_price_q64,
            pool_fee,
            starts_in_bonding_curve,
            results_in_state_transition,
            // Liquidity event data.
            base_amount,
            quote_amount,
            lp_coin_amount,
            liquidity_provided,
            pro_rata_base_donation_claim_amount,
            pro_rata_quote_donation_claim_amount,
            // Chat event data.
            message,
            user_emojicoin_balance,
            circulating_supply,
            balance_as_fraction_of_circulating_supply_q64,
        }
    }
}
