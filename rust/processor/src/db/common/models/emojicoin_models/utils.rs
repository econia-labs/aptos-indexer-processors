use crate::db::common::models::fungible_asset_models::v2_fungible_asset_balances::{
    get_paired_metadata_address, get_primary_fungible_store_address,
};
use bigdecimal::BigDecimal;
use chrono::{DateTime, NaiveDateTime};
use num::ToPrimitive;

pub fn micros_to_naive_datetime(microseconds: &BigDecimal) -> NaiveDateTime {
    // There should be no truncation issues for almost ~300,000 years.
    let micros_i64 = BigDecimal::to_i64(microseconds)
        .expect("Microsecond values should always be representable as an i64.");
    DateTime::from_timestamp_micros(micros_i64)
        .expect("Should be able to convert microseconds to a DateTime and then to a NaiveDateTime.")
        .naive_utc()
}

pub fn within_past_day(time: NaiveDateTime) -> bool {
    let one_day_ago = chrono::Utc::now() - chrono::Duration::hours(24);

    time.and_utc() > one_day_ago
}

// Expects that the `market_address` has already been standardized.
pub fn to_lp_coin_type(market_address: &str) -> String {
    format!("{market_address}::coin_factory::EmojicoinLP")
}

// Expects that both inputs have already been standardized.
pub fn to_lp_primary_store_address(lp_coin_type: &str, owner_address: &str) -> String {
    let metadata_address = get_paired_metadata_address(lp_coin_type);
    get_primary_fungible_store_address(owner_address, metadata_address.as_str())
        .expect("Should be able to get the primary fungible store address")
}
