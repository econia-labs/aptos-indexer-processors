use crate::db::common::models::emojicoin_models::models::market_24h_rolling_volume::{
    OneMinutePeriodicStateEvent, UpdateMarketRolling24hVolumeResult,
};
use crate::utils::database::DbPoolConnection;
use ahash::AHashMap;
use bigdecimal::{BigDecimal, ToPrimitive};
use diesel::sql_query;
use diesel::QueryResult;
use diesel_async::RunQueryDsl;
use itertools::Itertools;

impl OneMinutePeriodicStateEvent {
    pub fn to_unzipped_period_data(
        events: Vec<OneMinutePeriodicStateEvent>,
    ) -> Vec<(i64, Vec<i64>, Vec<BigDecimal>, Vec<i64>)> {
        let mut models: AHashMap<i64, (i64, Vec<i64>, Vec<BigDecimal>, Vec<i64>)> = AHashMap::new();

        events.into_iter().for_each(|period_1m| {
            let market_id = period_1m.market_id;
            let entry = models
                .entry(market_id)
                .or_insert_with(|| (market_id, vec![], vec![], vec![]));
            entry.1.push(period_1m.market_nonce);
            entry.2.push(period_1m.period_volume);
            entry.3.push(period_1m.start_time);
        });

        models.into_iter().map(|(_, v)| v).collect()
    }
}

pub async fn update_volume_from_periodic_state_events(
    events: Vec<OneMinutePeriodicStateEvent>,
    conn: &mut DbPoolConnection<'_>,
) -> QueryResult<Vec<UpdateMarketRolling24hVolumeResult>> {
    let period_data = OneMinutePeriodicStateEvent::to_unzipped_period_data(events);
    sql_query(format_query(period_data).as_str())
        .load(conn)
        .await
}

pub fn format_query(unzipped_data: Vec<(i64, Vec<i64>, Vec<BigDecimal>, Vec<i64>)>) -> String {
    let mut rows = String::new();
    let length = unzipped_data.len();
    for (i, (market_id, market_nonces, period_volumes, start_times)) in
        unzipped_data.into_iter().enumerate()
    {
        rows.push_str(&format!(
            "ROW({}::BIGINT, ARRAY{:?}::BIGINT[], ARRAY{:?}::NUMERIC[], ARRAY{:?}::BIGINT[]){}",
            market_id,
            market_nonces,
            period_volumes
                .iter()
                .filter_map(BigDecimal::to_u128)
                .collect_vec(),
            start_times,
            if i != length - 1 { "," } else { "" }
        ));
    }

    let formatted_query = format!(
        "
        SELECT 
            market_id, 
            nonces,
            volumes,
            times,
            update_market_24h_rolling_1min_periods(market_id, nonces, volumes, times)
        FROM (
            SELECT * FROM UNNEST(
                ARRAY[
                    {rows}
                ]
            ) AS t(market_id BIGINT, nonces BIGINT[], volumes NUMERIC[], times BIGINT[])
        ) subquery;
        ",
        rows = rows
    );
    formatted_query
}
