use crate::{
    schema::{self, market_1m_periods_in_last_day},
    utils::database::ArcDbPool,
};
use bigdecimal::BigDecimal;
use diesel::{result::Error, QueryResult};
use diesel_async::{scoped_futures::ScopedFutureExt, AsyncConnection};
use field_count::FieldCount;
use serde::{Deserialize, Serialize};

use super::market_24h_rolling_volume::RecentOneMinutePeriodicStateEvent;

#[derive(Clone, Debug, Deserialize, FieldCount, Identifiable, Insertable, Serialize)]
#[diesel(primary_key(market_id, nonce))]
#[diesel(table_name = market_1m_periods_in_last_day)]
pub struct MarketOneMinutePeriodsInLastDayModel {
    pub market_id: i64,
    pub nonce: i64,
    pub volume: BigDecimal,
    pub start_time: i64,
}

impl MarketOneMinutePeriodsInLastDayModel {
    pub fn new(
        market_id: i64,
        nonce: i64,
        volume: BigDecimal,
        start_time: i64,
    ) -> MarketOneMinutePeriodsInLastDayModel {
        MarketOneMinutePeriodsInLastDayModel {
            market_id,
            nonce,
            volume,
            start_time,
        }
    }
}

impl From<RecentOneMinutePeriodicStateEvent> for MarketOneMinutePeriodsInLastDayModel {
    fn from(event: RecentOneMinutePeriodicStateEvent) -> Self {
        MarketOneMinutePeriodsInLastDayModel {
            market_id: event.market_id,
            nonce: event.market_nonce,
            volume: event.period_volume,
            start_time: event.start_time,
        }
    }
}

impl MarketOneMinutePeriodsInLastDayModel {
    pub async fn insert_and_delete_periods(
        items: &[MarketOneMinutePeriodsInLastDayModel],
        pool: ArcDbPool,
    ) -> QueryResult<(usize, usize)> {
        use diesel::prelude::*;
        use schema::market_1m_periods_in_last_day::dsl::*;

        let conn = &mut pool.get().await.map_err(|e| {
            tracing::warn!("Error getting connection from pool: {:?}", e);
            diesel::result::Error::DatabaseError(
                diesel::result::DatabaseErrorKind::UnableToSendCommand,
                Box::new(e.to_string()),
            )
        })?;

        conn.transaction::<_, Error, _>(|conn| {
            async move {
                let inserted = diesel_async::RunQueryDsl::execute(
                    diesel::insert_into(schema::market_1m_periods_in_last_day::table)
                        .values(items)
                        .on_conflict((market_id, nonce))
                        .do_nothing(),
                    conn,
                )
                .await?;

                let one_day_ago_micros =
                    (chrono::Utc::now() - chrono::Duration::days(1)).timestamp_micros();

                let deleted = diesel_async::RunQueryDsl::execute(
                    diesel::delete(schema::market_1m_periods_in_last_day::table)
                        .filter(start_time.le(one_day_ago_micros)),
                    conn,
                )
                .await?;

                Ok((inserted, deleted))
            }
            .scope_boxed()
        })
        .await
    }
}
