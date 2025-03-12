use crate::{
    db::common::models::emojicoin_models::{
        constants::CANDLESTICK_DECIMALS,
        enums::Period,
        json_types::{StateEvent, TxnInfo},
        parsers::emojis::parser::symbol_bytes_to_emojis,
    },
    schema::normal_candlesticks,
};
use bigdecimal::{BigDecimal, RoundingMode};
use chrono::{DurationRound, NaiveDateTime};
use field_count::FieldCount;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Clone)]
pub struct CandlestickDiffModelBuilder {
    pub market_id: BigDecimal,
    pub last_transaction_version: i64,

    pub period: Period,
    pub start_time: NaiveDateTime,

    pub open_price: BigDecimal,
    pub high_price: BigDecimal,
    pub low_price: BigDecimal,
    pub close_price: BigDecimal,

    pub open_timestamp: (i64, i64),
    pub close_timestamp: (i64, i64),

    pub symbol_emojis: Vec<String>,

    pub volume: BigDecimal,
}

impl CandlestickDiffModelBuilder {
    pub fn merge(sticks: Vec<Self>) -> Vec<Self> {
        let mut sticks_map: HashMap<(BigDecimal, Period, NaiveDateTime), Self> = HashMap::new();

        for stick in sticks {
            let stick_clone = stick.clone();
            sticks_map
                .entry((stick.market_id.clone(), stick.period, stick.start_time))
                .and_modify(|s| {
                    s.last_transaction_version = s
                        .last_transaction_version
                        .max(stick.last_transaction_version);
                    s.volume += stick.volume;
                    s.high_price = s.high_price.clone().max(stick.high_price);
                    s.low_price = s.low_price.clone().min(stick.low_price);
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
        txn_info: &TxnInfo,
        state: &StateEvent,
        swap_timestamp: (i64, i64),
    ) -> Vec<Self> {
        let periods = vec![
            Period::FifteenSeconds,
            Period::OneMinute,
            Period::FiveMinutes,
            Period::FifteenMinutes,
            Period::ThirtyMinutes,
            Period::OneHour,
            Period::FourHours,
            Period::OneDay,
        ];

        let mut candlesticks: Vec<Self> = vec![];

        for period in periods {
            let start_time = txn_info
                .timestamp
                .duration_trunc(period.to_time_delta())
                .unwrap();
            let symbol_emojis = symbol_bytes_to_emojis(&state.market_metadata.emoji_bytes);
            let price = state.curve_price();
            let x = Self {
                market_id: state.market_metadata.market_id.clone(),
                last_transaction_version: txn_info.version,
                period,
                start_time,
                open_price: price.clone(),
                high_price: price.clone(),
                low_price: price.clone(),
                close_price: price,
                close_timestamp: swap_timestamp,
                open_timestamp: swap_timestamp,
                symbol_emojis,
                volume: state.last_swap.quote_volume.clone(),
            };
            candlesticks.push(x);
        }
        candlesticks
    }
}
#[derive(Clone, Debug, Deserialize, FieldCount, Identifiable, Insertable, Serialize)]
#[diesel(primary_key(market_id, period, start_time))]
#[diesel(table_name = normal_candlesticks)]
pub struct CandlestickModel {
    pub market_id: BigDecimal,
    pub last_transaction_version: i64,

    pub period: Period,
    pub start_time: NaiveDateTime,

    pub open_price: BigDecimal,
    pub high_price: BigDecimal,
    pub low_price: BigDecimal,
    pub close_price: BigDecimal,

    pub symbol_emojis: Vec<String>,

    pub volume: BigDecimal,
}

impl CandlestickModel {
    fn truncate(value: BigDecimal) -> BigDecimal {
        value.with_precision_round(CANDLESTICK_DECIMALS, RoundingMode::HalfEven)
    }
}

impl From<CandlestickDiffModelBuilder> for CandlestickModel {
    fn from(value: CandlestickDiffModelBuilder) -> Self {
        Self {
            market_id: value.market_id,
            last_transaction_version: value.last_transaction_version,

            period: value.period,
            start_time: value.start_time,

            open_price: Self::truncate(value.open_price),
            high_price: Self::truncate(value.high_price),
            low_price: Self::truncate(value.low_price),
            close_price: Self::truncate(value.close_price),

            symbol_emojis: value.symbol_emojis,

            volume: value.volume,
        }
    }
}
