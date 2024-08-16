use crate::{
    db::common::models::emojicoin_models::{enums::Period, json_types::EventWithMarket},
    schema::market_24h_rolling_1min_periods,
};
use bigdecimal::BigDecimal;
use field_count::FieldCount;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Deserialize, FieldCount, Identifiable, Insertable, Serialize)]
#[diesel(primary_key(market_id))]
#[diesel(table_name = market_24h_rolling_1min_periods)]
pub struct Market24HRolling1MinPeriodsModel {
    pub market_id: i64,
    pub market_nonces: Vec<i64>,
    pub period_volumes: Vec<BigDecimal>,
    pub start_times: Vec<i64>,
}

// Need a queryable version of the model to include the `inserted_at` field, since it's populated at insertion time.
// Unfortunately, this is a limitation with `diesel`'s `insertable` derive macro.
#[derive(Clone, Debug, Identifiable, Queryable)]
#[diesel(primary_key(market_id))]
#[diesel(table_name = market_24h_rolling_1min_periods)]
pub struct Market24HRolling1MinPeriodsQueryModel {
    pub market_id: i64,
    pub inserted_at: chrono::NaiveDateTime,
    pub market_nonces: Vec<i64>,
    pub period_volumes: Vec<BigDecimal>,
    pub start_times: Vec<i64>,
}

impl Market24HRolling1MinPeriodsModel {
    pub fn new(market_id: i64) -> Market24HRolling1MinPeriodsModel {
        Market24HRolling1MinPeriodsModel {
            market_id,
            market_nonces: vec![],
            period_volumes: vec![],
            start_times: vec![],
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct RecentOneMinutePeriodicStateEvent {
    pub market_id: i64,
    pub market_nonce: i64,
    pub period_volume: BigDecimal,
    pub start_time: i64,
}

// Recent being defined as within the time frame defined in the function logic below.
impl RecentOneMinutePeriodicStateEvent {
    pub fn try_from_event(event: EventWithMarket) -> Option<Self> {
        match event {
            EventWithMarket::PeriodicState(pse) => {
                let one_day_ago = chrono::Utc::now() - chrono::Duration::days(1);
                let one_day_ago_micros = one_day_ago.timestamp_millis() * 1000;
                let (period, start_time) = (
                    pse.periodic_state_metadata.period,
                    pse.periodic_state_metadata.start_time,
                );

                if period == Period::Period1M && start_time > one_day_ago_micros {
                    Some(RecentOneMinutePeriodicStateEvent {
                        market_id: pse.market_metadata.market_id,
                        market_nonce: pse.periodic_state_metadata.emit_market_nonce,
                        period_volume: pse.volume_quote.clone(),
                        start_time,
                    })
                } else {
                    None
                }
            },
            _ => None,
        }
    }
}

#[derive(QueryableByName, Debug)]
pub struct UpdateMarketRolling24hVolumeResult {
    #[diesel(sql_type = diesel::sql_types::BigInt)]
    pub market_id: i64,
    #[diesel(sql_type = diesel::sql_types::Array<diesel::sql_types::BigInt>)]
    pub nonces: Vec<i64>,
    #[diesel(sql_type = diesel::sql_types::Array<diesel::sql_types::Numeric>)]
    pub volumes: Vec<BigDecimal>,
    #[diesel(sql_type = diesel::sql_types::Array<diesel::sql_types::BigInt>)]
    pub times: Vec<i64>,
}
