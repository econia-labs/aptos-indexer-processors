use crate::{
    db::common::models::emojicoin_models::{enums::Period, json_types::StateEvent},
    schema::arena_candlestick,
};
use bigdecimal::BigDecimal;
use chrono::{DurationRound, NaiveDateTime};
use field_count::FieldCount;
use num::FromPrimitive;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Clone)]
pub struct ArenaCandlestickDiffModelBuilder {
    pub melee_id: BigDecimal,

    pub period: Period,
    pub start_time: NaiveDateTime,

    pub open_price: BigDecimal,
    pub high_price: BigDecimal,
    pub low_price: BigDecimal,
    pub close_price: BigDecimal,

    pub open_timestamp: (i64, i64),
    pub close_timestamp: (i64, i64),

    pub volume: BigDecimal,
    pub n_swaps: BigDecimal,
}

impl ArenaCandlestickDiffModelBuilder {
    pub fn merge(sticks: Vec<Self>) -> Vec<Self> {
        let mut sticks_map: HashMap<(BigDecimal, Period, NaiveDateTime), Self> = HashMap::new();

        for stick in sticks {
            let stick_clone = stick.clone();
            sticks_map
                .entry((stick.melee_id.clone(), stick.period, stick.start_time))
                .and_modify(|s| {
                    s.volume += stick.volume;
                    s.n_swaps += stick.n_swaps;
                    s.high_price = BigDecimal::max(s.high_price.clone(), stick.high_price);
                    s.low_price = BigDecimal::min(s.low_price.clone(), stick.low_price);
                    if s.open_timestamp > stick.open_timestamp {
                        s.open_price = stick.open_price;
                        s.open_timestamp = stick.open_timestamp;
                    }
                    if s.close_timestamp < stick.close_timestamp {
                        s.close_price = stick.close_price;
                        s.close_timestamp = stick.close_timestamp;
                    }
                })
                .or_insert(stick_clone);
        }

        sticks_map.into_values().collect()
    }

    pub fn from_state_event(
        melee_id: BigDecimal,
        state: StateEvent,
        transaction_timestamp: NaiveDateTime,
        swap_timestamp: (i64, i64),
        price_0: BigDecimal,
        price_1: BigDecimal,
    ) -> Vec<Self> {
        let periods = vec![
            Period::FifteenSeconds,
            Period::OneMinute,
            Period::FiveMinutes,
            Period::FifteenMinutes,
            Period::ThirtyMinutes,
            Period::OneHour,
        ];

        let mut candlesticks: Vec<Self> = vec![];

        for period in periods {
            let start_time = transaction_timestamp
                .duration_trunc(period.to_time_delta())
                .unwrap();
            let price = price_0.clone() / price_1.clone();
            let x = Self {
                melee_id: melee_id.clone(),
                period,
                start_time,
                open_price: price.clone(),
                high_price: price.clone(),
                low_price: price.clone(),
                close_price: price,
                close_timestamp: swap_timestamp,
                open_timestamp: swap_timestamp,
                volume: state.last_swap.quote_volume.clone(),
                n_swaps: BigDecimal::from_u8(1).unwrap(),
            };
            candlesticks.push(x);
        }
        candlesticks
    }
}
#[derive(Clone, Debug, Deserialize, FieldCount, Identifiable, Insertable, Serialize)]
#[diesel(primary_key(melee_id, period, start_time))]
#[diesel(table_name = arena_candlestick)]
pub struct ArenaCandlestickDiffModel {
    pub melee_id: BigDecimal,

    pub period: Period,
    pub start_time: NaiveDateTime,

    pub open_price: BigDecimal,
    pub high_price: BigDecimal,
    pub low_price: BigDecimal,
    pub close_price: BigDecimal,
    pub volume: BigDecimal,
    pub n_swaps: BigDecimal,
}

impl From<ArenaCandlestickDiffModelBuilder> for ArenaCandlestickDiffModel {
    fn from(value: ArenaCandlestickDiffModelBuilder) -> Self {
        Self {
            melee_id: value.melee_id,

            period: value.period,
            start_time: value.start_time,

            open_price: value.open_price,
            high_price: value.high_price,
            low_price: value.low_price,
            close_price: value.close_price,

            volume: value.volume,
            n_swaps: value.n_swaps,
        }
    }
}
