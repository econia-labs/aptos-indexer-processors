use crate::db::common::models::emojicoin_models::{enums::Period, json_types::EventWithMarket};
use bigdecimal::BigDecimal;
use serde::{Deserialize, Serialize};

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

                if period == Period::OneMinute && start_time > one_day_ago_micros {
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
