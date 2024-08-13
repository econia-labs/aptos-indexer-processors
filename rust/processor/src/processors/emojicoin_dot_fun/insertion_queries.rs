use diesel::{pg::Pg, query_builder::QueryFragment};

use crate::db::common::models::emojicoin_models::models::{
    chat_event::ChatEventModel, global_state_event::GlobalStateEventModel,
    liquidity_event::LiquidityEventModel, market_registration_event::MarketRegistrationEventModel,
    periodic_state_event::PeriodicStateEventModel, swap_event::SwapEventModel,
};
use crate::schema::{
    chat_events, global_state_events, liquidity_events, market_registration_events,
    periodic_state_events, swap_events,
};

pub fn insert_chat_events_query(
    items_to_insert: Vec<ChatEventModel>,
) -> (
    impl QueryFragment<Pg> + diesel::query_builder::QueryId + Send,
    Option<&'static str>,
) {
    (
        diesel::insert_into(chat_events::table)
            .values(items_to_insert)
            .on_conflict((chat_events::market_id, chat_events::market_nonce))
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
    (
        diesel::insert_into(liquidity_events::table)
            .values(items_to_insert)
            .on_conflict((liquidity_events::market_id, liquidity_events::market_nonce))
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
    (
        diesel::insert_into(swap_events::table)
            .values(items_to_insert)
            .on_conflict((swap_events::market_id, swap_events::market_nonce))
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
    (
        diesel::insert_into(market_registration_events::table)
            .values(items_to_insert)
            .on_conflict(market_registration_events::market_id)
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
    (
        diesel::insert_into(periodic_state_events::table)
            .values(items_to_insert)
            .on_conflict((
                periodic_state_events::market_id,
                periodic_state_events::period,
                periodic_state_events::market_nonce,
            ))
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
    (
        diesel::insert_into(global_state_events::table)
            .values(items_to_insert)
            .on_conflict(global_state_events::registry_nonce)
            .do_nothing(),
        None,
    )
}
