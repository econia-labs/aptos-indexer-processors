use crate::db::common::models::emojicoin_models::enums;
use crate::schema::user_liquidity_pools;
use field_count::FieldCount;
use serde::{Deserialize, Serialize};

use super::liquidity_event::LiquidityEventModel;

#[derive(Clone, Debug, Deserialize, FieldCount, Identifiable, Insertable, Serialize)]
#[diesel(primary_key(provider, market_nonce))]
#[diesel(table_name = user_liquidity_pools)]
pub struct UserLiquidityPoolsModel {
    pub provider: String,
    pub transaction_version: i64,
    pub transaction_timestamp: chrono::NaiveDateTime,

    // Market and state metadata.
    pub market_id: i64,
    pub symbol_bytes: Vec<u8>,
    pub bump_time: chrono::NaiveDateTime,
    pub market_nonce: i64,
    pub trigger: enums::Trigger,

    pub base_amount: i64,
    pub quote_amount: i64,
    pub lp_coin_amount: i64,
    pub liquidity_provided: bool,
    pub pro_rata_base_donation_claim_amount: i64,
    pub pro_rata_quote_donation_claim_amount: i64,
}

#[derive(Clone, Debug, Deserialize, FieldCount, Identifiable, Insertable, Serialize)]
#[diesel(primary_key(provider, market_nonce))]
#[diesel(table_name = user_liquidity_pools)]
pub struct UserLiquidityPoolsQueryModel {
    pub provider: String,
    pub transaction_version: i64,
    pub transaction_timestamp: chrono::NaiveDateTime,

    // Market and state metadata.
    pub market_id: i64,
    pub symbol_bytes: Vec<u8>,
    pub bump_time: chrono::NaiveDateTime,
    pub market_nonce: i64,
    pub trigger: enums::Trigger,

    pub base_amount: i64,
    pub quote_amount: i64,
    pub lp_coin_amount: i64,
    pub liquidity_provided: bool,
    pub pro_rata_base_donation_claim_amount: i64,
    pub pro_rata_quote_donation_claim_amount: i64,
}

impl From<LiquidityEventModel> for UserLiquidityPoolsModel {
    fn from(evt: LiquidityEventModel) -> Self {
        UserLiquidityPoolsModel {
            provider: evt.provider.clone(),
            transaction_version: evt.transaction_version,
            transaction_timestamp: evt.transaction_timestamp,
            market_id: evt.market_id,
            symbol_bytes: evt.symbol_bytes,
            bump_time: evt.bump_time,
            market_nonce: evt.market_nonce,
            trigger: evt.trigger,
            base_amount: evt.base_amount,
            quote_amount: evt.quote_amount,
            lp_coin_amount: evt.lp_coin_amount,
            liquidity_provided: evt.liquidity_provided,
            pro_rata_base_donation_claim_amount: evt.pro_rata_base_donation_claim_amount,
            pro_rata_quote_donation_claim_amount: evt.pro_rata_quote_donation_claim_amount,
        }
    }
}
