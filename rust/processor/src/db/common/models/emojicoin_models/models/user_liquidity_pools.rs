use super::liquidity_event::LiquidityEventModel;
use crate::{
    db::common::models::{
        emojicoin_models::{enums, parsers::emojis::parser::symbol_bytes_to_emojis},
        fungible_asset_models::{
            v2_fungible_asset_balances::get_primary_fungible_store_address,
            v2_fungible_asset_utils::FungibleAssetStore,
        },
    },
    schema::user_liquidity_pools,
};
use aptos_protos::transaction::v1::{write_set_change::Change, Transaction};
use bigdecimal::BigDecimal;
use field_count::FieldCount;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Deserialize, FieldCount, Identifiable, Insertable, Serialize)]
#[diesel(primary_key(provider, market_nonce))]
#[diesel(table_name = user_liquidity_pools)]
pub struct UserLiquidityPoolsModel {
    pub provider: String,
    pub transaction_version: i64,
    pub transaction_timestamp: chrono::NaiveDateTime,

    // Market and state metadata.
    pub market_id: BigDecimal,
    pub symbol_bytes: Vec<u8>,
    pub symbol_emojis: Vec<String>,
    pub bump_time: chrono::NaiveDateTime,
    pub market_nonce: BigDecimal,
    pub trigger: enums::Trigger,
    pub market_address: String,

    pub base_amount: BigDecimal,
    pub quote_amount: BigDecimal,
    pub lp_coin_amount: BigDecimal,
    pub liquidity_provided: bool,
    pub base_donation_claim_amount: BigDecimal,
    pub quote_donation_claim_amount: BigDecimal,

    pub lp_coin_balance: BigDecimal,
}

impl UserLiquidityPoolsModel {
    pub fn from_event_and_writeset(txn: &Transaction, evt: LiquidityEventModel) -> Self {
        txn.info
            .as_ref()
            .expect("Transaction info should exist.")
            .changes
            .iter()
            .find_map(|wsc| {
                if let Change::WriteResource(write_resource) = &wsc.change.as_ref().unwrap() {
                    FungibleAssetStore::from_write_resource(write_resource, txn.version as i64)
                        .ok()
                        .flatten()
                        .and_then(|resource| {
                            let fungible_store_address = get_primary_fungible_store_address(
                                &evt.provider,
                                &resource.metadata.get_reference_address(),
                            ).expect("Should be able to create a primary fungible store address from the provider address");
                            if write_resource.address != fungible_store_address {
                                None
                            } else {
                                let lp_coin_balance = resource.balance;
                                Some(UserLiquidityPoolsModel {
                                    provider: evt.provider.clone(),
                                    transaction_version: evt.transaction_version,
                                    transaction_timestamp: evt.transaction_timestamp,
                                    market_id: evt.market_id.clone(),
                                    symbol_bytes: evt.symbol_bytes.clone(),
                                    symbol_emojis: symbol_bytes_to_emojis(&evt.symbol_bytes),
                                    bump_time: evt.bump_time,
                                    market_nonce: evt.market_nonce.clone(),
                                    trigger: evt.trigger,
                                    base_amount: evt.base_amount.clone(),
                                    quote_amount: evt.quote_amount.clone(),
                                    lp_coin_amount: evt.lp_coin_amount.clone(),
                                    liquidity_provided: evt.liquidity_provided,
                                    base_donation_claim_amount: evt
                                        .base_donation_claim_amount
                                        .clone(),
                                    quote_donation_claim_amount: evt
                                        .quote_donation_claim_amount
                                        .clone(),
                                    lp_coin_balance,
                                    market_address: evt.market_address.clone(),
                                })
                            }
                        })
                } else {
                    None
                }
            })
            .expect("LP fungible asset balance change should exist in the writeset.")
    }
}
