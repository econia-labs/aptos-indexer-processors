use crate::{
    db::common::models::emojicoin_models::models::{
        arena_enter_event::ArenaEnterEventModel, arena_exit_event::ArenaExitEventModel,
        arena_info::ArenaInfoModel, arena_melee_event::ArenaMeleeEventModel,
        arena_position::ArenaPositionModel, arena_swap_event::ArenaSwapEventModel,
        arena_vault_balance_update_event::ArenaVaultBalanceUpdateEventModel,
        chat_event::ChatEventModel, global_state_event::GlobalStateEventModel,
        liquidity_event::LiquidityEventModel,
        market_latest_state_event::MarketLatestStateEventModel,
        market_registration_event::MarketRegistrationEventModel,
        periodic_state_event::PeriodicStateEventModel, swap_event::SwapEventModel,
        user_liquidity_pools::UserLiquidityPoolsModel,
    },
    schema,
};
use bigdecimal::BigDecimal;
use diesel::{
    dsl::sql,
    pg::Pg,
    query_builder::QueryFragment,
    query_dsl::methods::{FilterDsl, SelectDsl},
    sql_types::{Bool, Nullable, Numeric},
    upsert::excluded,
    ExpressionMethods,
};
use num::Zero;

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

pub fn delete_unregistered_markets_query(
    items_to_remove: Vec<MarketRegistrationEventModel>,
) -> (
    impl QueryFragment<Pg> + diesel::query_builder::QueryId + Send,
    Option<&'static str>,
) {
    use schema::unregistered_markets::dsl::*;
    let data = items_to_remove
        .iter()
        .map(|e| e.symbol_bytes.clone())
        .collect::<Vec<_>>();
    (
        diesel::delete(unregistered_markets.filter(emojis.eq_any(data))),
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
                base_donation_claim_amount.eq(excluded(base_donation_claim_amount)),
                quote_donation_claim_amount.eq(excluded(quote_donation_claim_amount)),
                lp_coin_balance.eq(excluded(lp_coin_balance)),
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
                daily_tvl_per_lp_coin_growth.eq(excluded(daily_tvl_per_lp_coin_growth)),
                in_bonding_curve.eq(excluded(in_bonding_curve)),
                volume_in_1m_state_tracker.eq(excluded(volume_in_1m_state_tracker)),
                base_volume_in_1m_state_tracker.eq(excluded(base_volume_in_1m_state_tracker)),
            ))
            .filter(market_nonce.le(excluded(market_nonce))),
        None,
    )
}

pub fn insert_arena_melee_events_query(
    items_to_insert: Vec<ArenaMeleeEventModel>,
) -> (
    impl QueryFragment<Pg> + diesel::query_builder::QueryId + Send,
    Option<&'static str>,
) {
    use schema::arena_melee_events::dsl::*;
    (
        diesel::insert_into(schema::arena_melee_events::table)
            .values(items_to_insert)
            .on_conflict(melee_id)
            .do_nothing(),
        None,
    )
}

pub fn insert_arena_position_query(
    items_to_insert: Vec<ArenaPositionModel>,
) -> (
    impl QueryFragment<Pg> + diesel::query_builder::QueryId + Send,
    Option<&'static str>,
) {
    use schema::arena_position::dsl::*;
    (
        diesel::insert_into(schema::arena_position::table)
            .values(items_to_insert)
            .on_conflict((user, melee_id))
            .do_update()
            .set((
                open.eq(excluded(open)),
                emojicoin_0_balance.eq(emojicoin_0_balance + excluded(emojicoin_0_balance)),
                emojicoin_1_balance.eq(emojicoin_1_balance + excluded(emojicoin_1_balance)),
                deposits.eq(deposits + excluded(deposits)),
                match_amount.eq(match_amount + excluded(match_amount)),
                withdrawals.eq(withdrawals + excluded(withdrawals)),
                last_exit_0.eq(sql::<Nullable<Bool>>(
                    "COALESCE(EXCLUDED.last_exit_0, arena_position.last_exit_0)",
                )),
            )),
        None,
    )
}

pub fn insert_arena_info_query(
    info: Vec<ArenaInfoModel>,
) -> (
    impl QueryFragment<Pg> + diesel::query_builder::QueryId + Send,
    Option<&'static str>,
) {
    use schema::arena_info::dsl::*;
    (
        diesel::insert_into(schema::arena_info::table)
            .values(info)
            .on_conflict(melee_id)
            .do_nothing(),
        None,
    )
}

pub fn update_arena_info_enter_query(
    enter: ArenaEnterEventModel,
) -> impl QueryFragment<Pg> + diesel::query_builder::QueryId + Send {
    use schema::arena_info::dsl::*;
    diesel::insert_into(schema::arena_info::table)
        .values((
            melee_id.eq(enter.melee_id),
            volume.eq(enter.quote_volume.clone()),
            rewards_remaining.eq(-enter.match_amount.clone()),
            apt_locked.eq(enter.quote_volume.clone()),
        ))
        .on_conflict(melee_id)
        .do_update()
        .set((
            volume.eq(volume + enter.quote_volume.clone()),
            rewards_remaining.eq(rewards_remaining - enter.match_amount),
            apt_locked.eq(apt_locked + enter.quote_volume),
        ))
}

pub fn update_arena_info_swap_query(
    swap: ArenaSwapEventModel,
) -> impl QueryFragment<Pg> + diesel::query_builder::QueryId + Send {
    use schema::arena_info::dsl::*;
    diesel::insert_into(schema::arena_info::table)
        .values((
            melee_id.eq(swap.melee_id),
            volume.eq(swap.quote_volume.clone()),
            rewards_remaining.eq(BigDecimal::zero()),
            apt_locked.eq(BigDecimal::zero()),
        ))
        .on_conflict(melee_id)
        .do_update()
        .set((volume.eq(volume + swap.quote_volume),))
}

pub fn update_arena_info_exit_query(
    exit: ArenaExitEventModel,
) -> impl QueryFragment<Pg> + diesel::query_builder::QueryId + Send {
    use schema::arena_info::dsl::*;
    let locked = (exit.emojicoin_0_proceeds / exit.emojicoin_0_exchange_rate_base
        * exit.emojicoin_0_exchange_rate_quote
        + exit.emojicoin_1_proceeds / exit.emojicoin_1_exchange_rate_base
            * exit.emojicoin_1_exchange_rate_quote)
        .round(0);
    diesel::insert_into(schema::arena_info::table)
        .values((
            melee_id.eq(exit.melee_id),
            volume.eq(BigDecimal::zero()),
            rewards_remaining.eq(exit.tap_out_fee.clone()),
            apt_locked.eq(-locked.clone()),
        ))
        .on_conflict(melee_id)
        .do_update()
        .set((
            rewards_remaining.eq(rewards_remaining + exit.tap_out_fee),
            apt_locked.eq(sql::<Numeric>("GREATEST(arena_info.apt_locked - ")
                .bind::<Numeric, _>(locked)
                .sql(", 0)")),
        ))
}

#[derive(Clone, serde::Serialize, serde::Deserialize)]
pub struct ArenaLeaderboardHistoryParams {
    melee_id_value: BigDecimal,
    emojicoin_0_exchange_rate_base: BigDecimal,
    emojicoin_0_exchange_rate_quote: BigDecimal,
    emojicoin_1_exchange_rate_base: BigDecimal,
    emojicoin_1_exchange_rate_quote: BigDecimal,
}

pub fn insert_arena_leaderboard_history_query(
    params: ArenaLeaderboardHistoryParams,
) -> impl QueryFragment<Pg> + diesel::query_builder::QueryId + Send {
    use schema::arena_position::dsl::*;
    let ArenaLeaderboardHistoryParams {
        melee_id_value,
        emojicoin_0_exchange_rate_base,
        emojicoin_0_exchange_rate_quote,
        emojicoin_1_exchange_rate_base,
        emojicoin_1_exchange_rate_quote,
    } = params;
    let profits = withdrawals
        + sql::<Numeric>("ROUND(emojicoin_0_balance / ")
            .bind::<Numeric, _>(emojicoin_0_exchange_rate_base)
            .sql(" * ")
            .bind::<Numeric, _>(emojicoin_0_exchange_rate_quote)
            .sql(" + emojicoin_1_balance / ")
            .bind::<Numeric, _>(emojicoin_1_exchange_rate_base)
            .sql(" * ")
            .bind::<Numeric, _>(emojicoin_1_exchange_rate_quote)
            .sql(")");
    let data = arena_position.select((
        user,
        sql::<Numeric>("").bind::<Numeric, _>(melee_id_value),
        profits.clone(),
        deposits,
        sql::<Nullable<Bool>>("emojicoin_0_balance > 0"),
        emojicoin_0_balance,
        emojicoin_1_balance,
        sql::<Bool>("emojicoin_0_balance + emojicoin_1_balance = 0"),
        withdrawals,
    ));
    diesel::insert_into(schema::arena_leaderboard_history::table)
        .values(data)
        .into_columns((
            schema::arena_leaderboard_history::user,
            schema::arena_leaderboard_history::melee_id,
            schema::arena_leaderboard_history::profits,
            schema::arena_leaderboard_history::losses,
            schema::arena_leaderboard_history::last_exit_0,
            schema::arena_leaderboard_history::emojicoin_0_balance,
            schema::arena_leaderboard_history::emojicoin_1_balance,
            schema::arena_leaderboard_history::exited,
            schema::arena_leaderboard_history::withdrawals,
        ))
        .on_conflict((
            schema::arena_leaderboard_history::user,
            schema::arena_leaderboard_history::melee_id,
        ))
        .do_nothing()
}

pub fn insert_arena_enter_events_query(
    items_to_insert: Vec<ArenaEnterEventModel>,
) -> (
    impl QueryFragment<Pg> + diesel::query_builder::QueryId + Send,
    Option<&'static str>,
) {
    use schema::arena_enter_events::dsl::*;
    (
        diesel::insert_into(schema::arena_enter_events::table)
            .values(items_to_insert)
            .on_conflict((transaction_version, event_index))
            .do_nothing(),
        None,
    )
}

pub fn insert_arena_exit_events_query(
    items_to_insert: Vec<ArenaExitEventModel>,
) -> (
    impl QueryFragment<Pg> + diesel::query_builder::QueryId + Send,
    Option<&'static str>,
) {
    use schema::arena_exit_events::dsl::*;
    (
        diesel::insert_into(schema::arena_exit_events::table)
            .values(items_to_insert)
            .on_conflict((transaction_version, event_index))
            .do_nothing(),
        None,
    )
}

pub fn insert_arena_swap_events_query(
    items_to_insert: Vec<ArenaSwapEventModel>,
) -> (
    impl QueryFragment<Pg> + diesel::query_builder::QueryId + Send,
    Option<&'static str>,
) {
    use schema::arena_swap_events::dsl::*;
    (
        diesel::insert_into(schema::arena_swap_events::table)
            .values(items_to_insert)
            .on_conflict((transaction_version, event_index))
            .do_nothing(),
        None,
    )
}

pub fn insert_arena_vault_balance_update_events_query(
    items_to_insert: Vec<ArenaVaultBalanceUpdateEventModel>,
) -> (
    impl QueryFragment<Pg> + diesel::query_builder::QueryId + Send,
    Option<&'static str>,
) {
    use schema::arena_vault_balance_update_events::dsl::*;
    (
        diesel::insert_into(schema::arena_vault_balance_update_events::table)
            .values(items_to_insert)
            .on_conflict((transaction_version, event_index))
            .do_nothing(),
        None,
    )
}
