use crate::db::common::models::emojicoin_models::enums::{Periods, Triggers};
use crate::db::common::models::emojicoin_models::json_types::{
    InstantaneousStats, PeriodicStateTracker, TxnInfo,
};
use crate::db::common::models::emojicoin_models::utils::micros_to_naive_datetime;
use crate::db::common::models::emojicoin_models::{enums, json_types::MarketResource};
use crate::schema::market_latest_state_event;
use bigdecimal::{BigDecimal, Zero};
use field_count::FieldCount;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Deserialize, FieldCount, Identifiable, Insertable, Serialize)]
#[diesel(primary_key(market_id))]
#[diesel(table_name = market_latest_state_event)]
pub struct MarketLatestStateEventModel {
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
    pub trigger: enums::Triggers,

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

    daily_tvl_per_lp_coin_growth_q64: BigDecimal,
    in_bonding_curve: bool,
    volume_in_1m_state_tracker: BigDecimal,
}

impl MarketLatestStateEventModel {
    pub fn from_txn_and_market_resource(
        txn_info: TxnInfo,
        market: MarketResource,
        trigger: Triggers,
        instant_stats: InstantaneousStats,
    ) -> Self {
        let MarketResource {
            metadata,
            sequence_info,
            clamm_virtual_reserves,
            cpamm_real_reserves,
            lp_coin_supply,
            cumulative_stats,
            last_swap,
            periodic_state_trackers,
            ..
        } = market;

        // Note that we can examine the tracker for info here because it's the latest value on-chain.
        let (mut maybe_tracker_1m, mut maybe_tracker_1d) = (None, None);
        periodic_state_trackers.into_iter().for_each(|tracker| {
            if tracker.period == Periods::Period1M {
                maybe_tracker_1m = Some(tracker);
            } else if tracker.period == Periods::Period1D {
                maybe_tracker_1d = Some(tracker);
            }
        });

        let tracker_1m = maybe_tracker_1m.expect("Every market should have a PERIOD_1M tracker.");
        let tracker_1d = maybe_tracker_1d.expect("Every market should have a PERIOD_1D tracker.");

        Self {
            transaction_version: txn_info.version,
            sender: txn_info.sender,
            entry_function: txn_info.entry_function,
            transaction_timestamp: txn_info.timestamp,

            market_id: metadata.market_id,
            symbol_bytes: metadata.emoji_bytes,
            bump_time: micros_to_naive_datetime(sequence_info.last_bump_time),
            market_nonce: sequence_info.nonce,
            trigger,

            clamm_virtual_reserves_base: clamm_virtual_reserves.base,
            clamm_virtual_reserves_quote: clamm_virtual_reserves.quote,
            cpamm_real_reserves_base: cpamm_real_reserves.base,
            cpamm_real_reserves_quote: cpamm_real_reserves.quote,
            lp_coin_supply,
            cumulative_stats_base_volume: cumulative_stats.base_volume,
            cumulative_stats_quote_volume: cumulative_stats.quote_volume,
            cumulative_stats_integrator_fees: cumulative_stats.integrator_fees,
            cumulative_stats_pool_fees_base: cumulative_stats.pool_fees_base,
            cumulative_stats_pool_fees_quote: cumulative_stats.pool_fees_quote,
            cumulative_stats_n_swaps: cumulative_stats.n_swaps,
            cumulative_stats_n_chat_messages: cumulative_stats.n_chat_messages,
            instantaneous_stats_total_quote_locked: instant_stats.total_quote_locked,
            instantaneous_stats_total_value_locked: instant_stats.total_value_locked,
            instantaneous_stats_market_cap: instant_stats.market_cap,
            instantaneous_stats_fully_diluted_value: instant_stats.fully_diluted_value,
            last_swap_is_sell: last_swap.is_sell,
            last_swap_avg_execution_price_q64: last_swap.avg_execution_price_q64,
            last_swap_base_volume: last_swap.base_volume,
            last_swap_quote_volume: last_swap.quote_volume,
            last_swap_nonce: last_swap.nonce,
            last_swap_time: micros_to_naive_datetime(last_swap.time),

            daily_tvl_per_lp_coin_growth_q64: calculate_tvl_growth(tracker_1d),
            in_bonding_curve: tracker_1m.starts_in_bonding_curve
                || tracker_1m.ends_in_bonding_curve,
            volume_in_1m_state_tracker: tracker_1m.volume_quote,
        }
    }
}

pub fn calculate_tvl_growth(tracker_1d: PeriodicStateTracker) -> BigDecimal {
    let PeriodicStateTracker {
        tvl_to_lp_coin_ratio_start: start,
        tvl_to_lp_coin_ratio_end: end,
        ..
    } = tracker_1d;

    // tracker_1d.tvl_to_lp_coin_ratio_end - tracker_1d.tvl_to_lp_coin_ratio_start
    let a = BigDecimal::from(start.tvl);
    let b = BigDecimal::from(start.lp_coins);
    let c = BigDecimal::from(end.tvl);
    let d = BigDecimal::from(end.lp_coins);

    if a.is_zero() || b.is_zero() {
        BigDecimal::zero()
    } else {
        (b * c) / (a * d)
    }
}
