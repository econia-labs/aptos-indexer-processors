use crate::{
    db::common::models::emojicoin_models::{enums::Period, json_types::SwapEvent},
    schema::arena_candlestick,
};
use bigdecimal::{BigDecimal, Zero};
use chrono::{DurationRound, NaiveDateTime};
use field_count::FieldCount;
use num::FromPrimitive;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Clone)]
pub struct Prices {
    pub open_price: BigDecimal,
    pub high_price: BigDecimal,
    pub low_price: BigDecimal,
    pub close_price: BigDecimal,

    pub open_timestamp: (i64, i64),
    pub close_timestamp: (i64, i64),
}

#[derive(Clone)]
pub struct ArenaCandlestickDiffModelBuilder {
    pub melee_id: BigDecimal,

    pub period: Period,
    pub start_time: NaiveDateTime,

    pub prices: Option<Prices>,

    pub volume: BigDecimal,
    pub integrator_fees: BigDecimal,
    pub n_swaps: BigDecimal,
}

impl ArenaCandlestickDiffModelBuilder {
    pub fn merge(sticks: Vec<Self>) -> Vec<Self> {
        let mut sticks_map: HashMap<(BigDecimal, Period, NaiveDateTime), Self> = HashMap::new();

        for stick in sticks {
            let stick_clone = stick.clone();
            sticks_map
                .entry((
                    stick.melee_id.clone(),
                    stick.period.clone(),
                    stick.start_time.clone(),
                ))
                .and_modify(|s| {
                    s.volume += stick.volume;
                    s.integrator_fees += stick.integrator_fees;
                    s.n_swaps += stick.n_swaps;
                    if s.prices.is_none() {
                        s.prices = stick.prices;
                    } else if stick.prices.is_some() {
                        let s_prices = s.prices.as_mut().unwrap();
                        let stick_prices = stick.prices.unwrap();
                        s_prices.high_price =
                            BigDecimal::max(s_prices.high_price.clone(), stick_prices.high_price);
                        s_prices.low_price =
                            BigDecimal::min(s_prices.low_price.clone(), stick_prices.low_price);
                        if s_prices.open_timestamp > stick_prices.open_timestamp {
                            s_prices.open_price = stick_prices.open_price;
                            s_prices.open_timestamp = stick_prices.open_timestamp;
                        }
                        if s_prices.close_timestamp < stick_prices.close_timestamp {
                            s_prices.close_price = stick_prices.close_price;
                            s_prices.close_timestamp = stick_prices.close_timestamp;
                        }
                    }
                })
                .or_insert(stick_clone);
        }

        sticks_map.into_values().collect()
    }

    pub fn from_swap_event(
        melee_id: BigDecimal,
        swap: SwapEvent,
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
                .duration_round(period.clone().to_time_delta())
                .unwrap();
            let prices = if !price_0.is_zero() && !price_1.is_zero() {
                let price = price_0.clone() / price_1.clone();
                Some(Prices {
                    open_price: price.clone(),
                    high_price: price.clone(),
                    low_price: price.clone(),
                    close_price: price,
                    close_timestamp: swap_timestamp,
                    open_timestamp: swap_timestamp,
                })
            } else {
                None
            };
            let x = Self {
                melee_id: melee_id.clone(),
                period,
                start_time,
                prices,
                volume: swap.quote_volume.clone(),
                integrator_fees: swap.integrator_fee.clone(),
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

    pub open_price: Option<BigDecimal>,
    pub high_price: Option<BigDecimal>,
    pub low_price: Option<BigDecimal>,
    pub close_price: Option<BigDecimal>,
    pub volume: BigDecimal,
    pub integrator_fees: BigDecimal,
    pub n_swaps: BigDecimal,
}

impl From<ArenaCandlestickDiffModelBuilder> for ArenaCandlestickDiffModel {
    fn from(value: ArenaCandlestickDiffModelBuilder) -> Self {
        Self {
            melee_id: value.melee_id,

            period: value.period,
            start_time: value.start_time,

            open_price: value.prices.as_ref().map(|p| p.open_price.clone()),
            high_price: value.prices.as_ref().map(|p| p.high_price.clone()),
            low_price: value.prices.as_ref().map(|p| p.low_price.clone()),
            close_price: value.prices.as_ref().map(|p| p.close_price.clone()),
            volume: value.volume,
            integrator_fees: value.integrator_fees,
            n_swaps: value.n_swaps,
        }
    }
}
