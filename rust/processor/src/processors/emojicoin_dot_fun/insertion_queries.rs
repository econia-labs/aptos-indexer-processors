use diesel::ExpressionMethods;
use diesel::{pg::Pg, query_builder::QueryFragment, upsert::excluded};

use crate::db::common::models::emojicoin_models::models::{
    chat_event::ChatEventModel, global_state_event::GlobalStateEventModel,
    liquidity_event::LiquidityEventModel, market_registration_event::MarketRegistrationEventModel,
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
            )),
        Some(" WHERE user_liquidity_pools.market_nonce <= EXCLUDED.market_nonce "),
    )
}
