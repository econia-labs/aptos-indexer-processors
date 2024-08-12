use diesel::{ExpressionMethods, OptionalExtension, QueryDsl, QueryResult};
use diesel_async::RunQueryDsl;

use crate::{
    db::common::models::emojicoin_models::models::global_state_event::GlobalStateEventModelQuery,
    schema::global_state_events, utils::database::DbPoolConnection,
};

impl GlobalStateEventModelQuery {
    pub async fn get_latest(conn: &mut DbPoolConnection<'_>) -> QueryResult<Option<Self>> {
        global_state_events::table
            .select(global_state_events::all_columns)
            .order_by(global_state_events::registry_nonce.desc())
            .first::<Self>(conn)
            .await
            .optional()
    }
}
