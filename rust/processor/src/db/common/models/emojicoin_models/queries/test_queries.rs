use crate::{
    db::common::models::emojicoin_models::models::{
        chat_event::ChatEventQueryModel, global_state_event::GlobalStateEventQueryModel,
        liquidity_event::LiquidityEventQueryModel,
        market_registration_event::MarketRegistrationEventQueryModel,
        periodic_state_event::PeriodicStateEventQueryModel, swap_event::SwapEventQueryModel,
    },
    schema::chat_events,
    schema::global_state_events,
    schema::liquidity_events,
    schema::market_registration_events,
    schema::periodic_state_events,
    schema::swap_events,
    utils::database::DbPoolConnection,
};
use diesel::{ExpressionMethods, OptionalExtension, QueryDsl, QueryResult};
use diesel_async::RunQueryDsl;

impl GlobalStateEventQueryModel {
    pub async fn get_latest(conn: &mut DbPoolConnection<'_>) -> QueryResult<Option<Self>> {
        global_state_events::table
            .select(global_state_events::all_columns)
            .order_by(global_state_events::registry_nonce.desc())
            .first::<Self>(conn)
            .await
            .optional()
    }
}

impl ChatEventQueryModel {
    pub async fn get_latest_by_market(
        conn: &mut DbPoolConnection<'_>,
        market_id: i64,
    ) -> QueryResult<Vec<Self>> {
        chat_events::table
            .select(chat_events::all_columns)
            .filter(chat_events::market_id.eq(market_id))
            .order_by(chat_events::market_nonce.desc())
            .limit(100)
            .load::<Self>(conn)
            .await
    }
}

impl LiquidityEventQueryModel {
    pub async fn get_latest_by_market(
        conn: &mut DbPoolConnection<'_>,
        market_id: i64,
    ) -> QueryResult<Vec<Self>> {
        liquidity_events::table
            .select(liquidity_events::all_columns)
            .filter(liquidity_events::market_id.eq(market_id))
            .order_by(liquidity_events::market_nonce.desc())
            .limit(100)
            .load::<Self>(conn)
            .await
    }
}

impl SwapEventQueryModel {
    pub async fn get_latest_by_market(
        conn: &mut DbPoolConnection<'_>,
        market_id: i64,
    ) -> QueryResult<Vec<Self>> {
        swap_events::table
            .select(swap_events::all_columns)
            .filter(swap_events::market_id.eq(market_id))
            .order_by(swap_events::market_nonce.desc())
            .limit(100)
            .load::<Self>(conn)
            .await
    }
}

impl MarketRegistrationEventQueryModel {
    pub async fn get_latest(
        conn: &mut DbPoolConnection<'_>,
        market_id: i64,
    ) -> QueryResult<Vec<Self>> {
        market_registration_events::table
            .select(market_registration_events::all_columns)
            .filter(market_registration_events::market_id.eq(market_id))
            .order_by(market_registration_events::market_nonce.desc())
            .limit(100)
            .load::<Self>(conn)
            .await
    }
}

impl PeriodicStateEventQueryModel {
    pub async fn get_latest_by_market(
        conn: &mut DbPoolConnection<'_>,
        market_id: i64,
    ) -> QueryResult<Vec<Self>> {
        periodic_state_events::table
            .select(periodic_state_events::all_columns)
            .filter(periodic_state_events::market_id.eq(market_id))
            .order_by(periodic_state_events::market_nonce.desc())
            .limit(100)
            .load::<Self>(conn)
            .await
    }
}
