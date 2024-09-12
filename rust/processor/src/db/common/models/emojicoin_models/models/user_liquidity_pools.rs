use super::liquidity_event::LiquidityEventModel;
use crate::{
    db::common::models::emojicoin_models::enums, schema::user_liquidity_pools,
    utils::util::standardize_address,
};
use aptos_protos::transaction::v1::{write_set_change::Change, Transaction};
use field_count::FieldCount;
use once_cell::sync::Lazy;
use regex::Regex;
use serde::{Deserialize, Serialize};

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
    pub base_donation_claim_amount: i64,
    pub quote_donation_claim_amount: i64,

    pub lp_coin_balance: i64,
}

impl UserLiquidityPoolsModel {
    pub fn from_event_and_writeset(
        txn: &Transaction,
        evt: LiquidityEventModel,
        market_address: &str,
    ) -> Self {
        static ADDRESSES_REGEX: Lazy<Regex> = Lazy::new(|| {
            Regex::new("^0x0*1::coin::CoinStore<(0x[^:]*)::coin_factory::EmojicoinLP>$").unwrap()
        });
        txn.info
            .as_ref()
            .expect("Transaction info should exist.")
            .changes
            .iter()
            .find_map(|wsc| {
                if let Change::WriteResource(write) = &wsc.change.as_ref().unwrap() {
                    if !ADDRESSES_REGEX.is_match(&write.type_str) {
                        return None;
                    }
                    let Some(caps) = ADDRESSES_REGEX.captures(&write.type_str) else {
                        return None;
                    };
                    if standardize_address(&caps[1]) == standardize_address(market_address) {
                        let Ok(data) = serde_json::from_str::<serde_json::Value>(&write.data) else {
                            return None;
                        };
                        let Some(amount) = data["coin"]["value"].as_str() else { return None };
                        Some(UserLiquidityPoolsModel {
                            provider: evt.provider.clone(),
                            transaction_version: evt.transaction_version,
                            transaction_timestamp: evt.transaction_timestamp,
                            market_id: evt.market_id,
                            symbol_bytes: evt.symbol_bytes.clone(),
                            bump_time: evt.bump_time,
                            market_nonce: evt.market_nonce,
                            trigger: evt.trigger,
                            base_amount: evt.base_amount,
                            quote_amount: evt.quote_amount,
                            lp_coin_amount: evt.lp_coin_amount,
                            liquidity_provided: evt.liquidity_provided,
                            base_donation_claim_amount: evt.base_donation_claim_amount,
                            quote_donation_claim_amount: evt.quote_donation_claim_amount,
                            lp_coin_balance: amount.parse().unwrap(),
                        })
                    } else {
                        None
                    }
                } else {
                    None
                }
            })
            .expect("LP coin change should exist.")
    }
}
