use diesel::{query_builder::QueryFragment, ExpressionMethods, OptionalExtension, QueryDsl};
use diesel_async::{
    pooled_connection::{bb8::PooledConnection, AsyncDieselConnectionManager},
    AsyncPgConnection, RunQueryDsl,
};

use crate::{
    db::common::models::emojicoin_models::db_types::global_state_events_model::GlobalStateEventModelQuery,
    schema::global_state_events,
    utils::database::{ArcDbPool, Backend},
};

impl GlobalStateEventModelQuery {
    pub async fn get(pool: ArcDbPool) -> diesel::QueryResult<Vec<Self>> {
        let conn = &mut pool.get().await.map_err(|e| {
            tracing::warn!("Error getting connection from pool: {:?}", e);
            diesel::result::Error::DatabaseError(
                diesel::result::DatabaseErrorKind::UnableToSendCommand,
                Box::new(e.to_string()),
            )
        })?;

        let query = global_state_events::table
            .select(global_state_events::all_columns)
            .order_by(global_state_events::registry_nonce.desc())
            .limit(1);

        let res = query.load::<Self>(conn).await;

        if let Err(ref e) = res {
            let debug_string = diesel::debug_query::<Backend, _>(&query).to_string();
            tracing::warn!("Error running query: {:?}\n{:?}", e, debug_string);
        }

        res
    }
}

pub async fn connection_helper<U>(pool: ArcDbPool, query: U)
where
    U: QueryFragment<Backend> + diesel::query_builder::QueryId + Send,
{
    let conn = &mut pool.get().await.map_err(|e| {
        tracing::warn!("Error getting connection from pool: {:?}", e);
        diesel::result::Error::DatabaseError(
            diesel::result::DatabaseErrorKind::UnableToSendCommand,
            Box::new(e.to_string()),
        )
    });

    let res = query.load::<U::Output>(conn).await;
}
