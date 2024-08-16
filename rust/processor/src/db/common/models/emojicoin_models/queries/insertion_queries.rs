use diesel::query_dsl::methods::FilterDsl;
use diesel::ExpressionMethods;
use diesel::{pg::Pg, query_builder::QueryFragment, upsert::excluded};

use crate::db::common::models::emojicoin_models::models::market_24h_rolling_volume::Market24HRolling1MinPeriodsModel;
use crate::db::common::models::emojicoin_models::models::{
    chat_event::ChatEventModel, global_state_event::GlobalStateEventModel,
    liquidity_event::LiquidityEventModel, market_latest_state_event::MarketLatestStateEventModel,
    market_registration_event::MarketRegistrationEventModel,
    periodic_state_event::PeriodicStateEventModel, swap_event::SwapEventModel,
    user_liquidity_pools::UserLiquidityPoolsModel,
};
use crate::schema;

pub fn insert_chat_events_query(
    items_to_insert: Vec<ChatEventModel>,
) -> (
    impl QueryFragment<Pg> + diesel::query_builder::QueryId + Send,
    Option<&'static str>,
) {
    use schema::chat_events::dsl::*;
    (
        diesel::insert_into(schema::chat_events::table)
            .values(items_to_insert)
            .on_conflict((market_id, market_nonce))
            .do_nothing(),
        None,
    )
}

pub fn insert_liquidity_events_query(
    items_to_insert: Vec<LiquidityEventModel>,
) -> (
    impl QueryFragment<Pg> + diesel::query_builder::QueryId + Send,
    Option<&'static str>,
) {
    use schema::liquidity_events::dsl::*;
    (
        diesel::insert_into(schema::liquidity_events::table)
            .values(items_to_insert)
            .on_conflict((market_id, market_nonce))
            .do_nothing(),
        None,
    )
}

pub fn insert_swap_events_query(
    items_to_insert: Vec<SwapEventModel>,
) -> (
    impl QueryFragment<Pg> + diesel::query_builder::QueryId + Send,
    Option<&'static str>,
) {
    use schema::swap_events::dsl::*;
    (
        diesel::insert_into(schema::swap_events::table)
            .values(items_to_insert)
            .on_conflict((market_id, market_nonce))
            .do_nothing(),
        None,
    )
}

pub fn insert_market_registration_events_query(
    items_to_insert: Vec<MarketRegistrationEventModel>,
) -> (
    impl QueryFragment<Pg> + diesel::query_builder::QueryId + Send,
    Option<&'static str>,
) {
    use schema::market_registration_events::dsl::*;
    (
        diesel::insert_into(schema::market_registration_events::table)
            .values(items_to_insert)
            .on_conflict(market_id)
            .do_nothing(),
        None,
    )
}

pub fn insert_periodic_state_events_query(
    items_to_insert: Vec<PeriodicStateEventModel>,
) -> (
    impl QueryFragment<Pg> + diesel::query_builder::QueryId + Send,
    Option<&'static str>,
) {
    use schema::periodic_state_events::dsl::*;
    (
        diesel::insert_into(schema::periodic_state_events::table)
            .values(items_to_insert)
            .on_conflict((market_id, period, market_nonce))
            .do_nothing(),
        None,
    )
}

pub fn insert_global_events(
    items_to_insert: Vec<GlobalStateEventModel>,
) -> (
    impl QueryFragment<Pg> + diesel::query_builder::QueryId + Send,
    Option<&'static str>,
) {
    use schema::global_state_events::dsl::*;
    (
        diesel::insert_into(schema::global_state_events::table)
            .values(items_to_insert)
            .on_conflict(registry_nonce)
            .do_nothing(),
        None,
    )
}

pub fn insert_user_liquidity_pools_query(
    items_to_insert: Vec<UserLiquidityPoolsModel>,
) -> (
    impl QueryFragment<Pg> + diesel::query_builder::QueryId + Send,
    Option<&'static str>,
) {
    use schema::user_liquidity_pools::dsl::*;
    (
        diesel::insert_into(schema::user_liquidity_pools::table)
            .values(items_to_insert)
            .on_conflict((provider, market_id))
            .do_update()
            .set((
                transaction_version.eq(excluded(transaction_version)),
                transaction_timestamp.eq(excluded(transaction_timestamp)),
                inserted_at.eq(excluded(inserted_at)),
                bump_time.eq(excluded(bump_time)),
                market_nonce.eq(excluded(market_nonce)),
                trigger.eq(excluded(trigger)),
                base_amount.eq(excluded(base_amount)),
                quote_amount.eq(excluded(quote_amount)),
                lp_coin_amount.eq(excluded(lp_coin_amount)),
                liquidity_provided.eq(excluded(liquidity_provided)),
                pro_rata_base_donation_claim_amount
                    .eq(excluded(pro_rata_base_donation_claim_amount)),
                pro_rata_quote_donation_claim_amount
                    .eq(excluded(pro_rata_quote_donation_claim_amount)),
            ))
            .filter(market_nonce.le(excluded(market_nonce))),
        None,
    )
}

pub fn insert_market_latest_state_event_query(
    items_to_insert: Vec<MarketLatestStateEventModel>,
) -> (
    impl QueryFragment<Pg> + diesel::query_builder::QueryId + Send,
    Option<&'static str>,
) {
    use schema::market_latest_state_event::dsl::*;
    (
        diesel::insert_into(schema::market_latest_state_event::table)
            .values(items_to_insert)
            .on_conflict(market_id)
            .do_update()
            .set((
                transaction_version.eq(excluded(transaction_version)),
                sender.eq(excluded(sender)),
                entry_function.eq(excluded(entry_function)),
                transaction_timestamp.eq(excluded(transaction_timestamp)),
                bump_time.eq(excluded(bump_time)),
                market_nonce.eq(excluded(market_nonce)),
                trigger.eq(excluded(trigger)),
                clamm_virtual_reserves_base.eq(excluded(clamm_virtual_reserves_base)),
                clamm_virtual_reserves_quote.eq(excluded(clamm_virtual_reserves_quote)),
                cpamm_real_reserves_base.eq(excluded(cpamm_real_reserves_base)),
                cpamm_real_reserves_quote.eq(excluded(cpamm_real_reserves_quote)),
                lp_coin_supply.eq(excluded(lp_coin_supply)),
                cumulative_stats_base_volume.eq(excluded(cumulative_stats_base_volume)),
                cumulative_stats_quote_volume.eq(excluded(cumulative_stats_quote_volume)),
                cumulative_stats_integrator_fees.eq(excluded(cumulative_stats_integrator_fees)),
                cumulative_stats_pool_fees_base.eq(excluded(cumulative_stats_pool_fees_base)),
                cumulative_stats_pool_fees_quote.eq(excluded(cumulative_stats_pool_fees_quote)),
                cumulative_stats_n_swaps.eq(excluded(cumulative_stats_n_swaps)),
                cumulative_stats_n_chat_messages.eq(excluded(cumulative_stats_n_chat_messages)),
                instantaneous_stats_total_quote_locked
                    .eq(excluded(instantaneous_stats_total_quote_locked)),
                instantaneous_stats_total_value_locked
                    .eq(excluded(instantaneous_stats_total_value_locked)),
                instantaneous_stats_market_cap.eq(excluded(instantaneous_stats_market_cap)),
                instantaneous_stats_fully_diluted_value
                    .eq(excluded(instantaneous_stats_fully_diluted_value)),
                last_swap_is_sell.eq(excluded(last_swap_is_sell)),
                last_swap_avg_execution_price_q64.eq(excluded(last_swap_avg_execution_price_q64)),
                last_swap_base_volume.eq(excluded(last_swap_base_volume)),
                last_swap_quote_volume.eq(excluded(last_swap_quote_volume)),
                last_swap_nonce.eq(excluded(last_swap_nonce)),
                last_swap_time.eq(excluded(last_swap_time)),
                daily_tvl_per_lp_coin_growth_q64.eq(excluded(daily_tvl_per_lp_coin_growth_q64)),
                in_bonding_curve.eq(excluded(in_bonding_curve)),
                volume_in_1m_state_tracker.eq(excluded(volume_in_1m_state_tracker)),
            ))
            .filter(market_nonce.le(excluded(market_nonce))),
        None,
    )
}

pub fn initialize_market_24h_rolling_1min_periods_query(
    market_ids: Vec<i64>,
) -> (
    impl QueryFragment<Pg> + diesel::query_builder::QueryId + Send,
    Option<&'static str>,
) {
    let items = market_ids
        .into_iter()
        .map(|m_id| Market24HRolling1MinPeriodsModel::new(m_id))
        .collect::<Vec<_>>();

    use schema::market_24h_rolling_1min_periods::dsl::*;
    (
        diesel::insert_into(schema::market_24h_rolling_1min_periods::table)
            .values(items)
            .on_conflict(market_id)
            .do_nothing(),
        None,
    )
}
