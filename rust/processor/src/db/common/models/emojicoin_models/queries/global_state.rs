use diesel::{ExpressionMethods, OptionalExtension, QueryDsl};
use diesel_async::RunQueryDsl;

use crate::{
    db::common::models::emojicoin_models::{
        db_types::global_state_events_model::GlobalStateEventModelQuery, enums::StateTrigger,
    },
    schema::{global_state_events, state_bumps},
    utils::database::DbPoolConnection,
};

impl GlobalStateEventModelQuery {
    pub async fn get_latest(conn: &mut DbPoolConnection<'_>) -> anyhow::Result<Option<Self>> {
        let res = global_state_events::table
            .select(global_state_events::all_columns)
            .order_by(global_state_events::registry_nonce.desc())
            .first::<Self>(conn)
            .await
            .optional();

        res.map_err(|e| {
            tracing::warn!("Error getting latest global state event: {:?}", e);
            anyhow::anyhow!("Error getting latest global state event: {:?}", e)
        })
    }
}

pub async fn get_num_global_events(conn: &mut DbPoolConnection<'_>) -> anyhow::Result<i64> {
    let res = global_state_events::table
        .select(diesel::dsl::count_star())
        .first::<i64>(conn)
        .await;

    res.map_err(|e| {
        tracing::warn!("Error getting number of markets: {:?}", e);
        anyhow::anyhow!("Error getting number of markets: {:?}", e)
    })
}

pub async fn get_num_bumps(conn: &mut DbPoolConnection<'_>) -> anyhow::Result<i64> {
    let res = state_bumps::table
        .select(diesel::dsl::count_star())
        .first::<i64>(conn)
        .await;

    res.map_err(|e| {
        tracing::warn!("Error getting number of bumps: {:?}", e);
        anyhow::anyhow!("Error getting number of bumps: {:?}", e)
    })
}

pub async fn get_num_markets(conn: &mut DbPoolConnection<'_>) -> anyhow::Result<i64> {
    state_bumps::table
        .filter(state_bumps::base_amount.eq(12))
        .count()
        .get_result::<i64>(conn)
        .await
        .map_err(|e| {
            tracing::warn!("Error getting number of markets: {:?}", e);
            anyhow::anyhow!("Error getting number of markets: {:?}", e)
        })
}
