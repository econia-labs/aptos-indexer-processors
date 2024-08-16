use aptos_protos::transaction::v1::{write_set_change::Change as WriteSetChangeEnum, Transaction};

use crate::{
    db::common::models::emojicoin_models::json_types::MarketResource,
    utils::util::standardize_address,
};

impl MarketResource {
    pub fn from_wsc(txn: &Transaction, market_address: &String) -> Self {
        txn.info
            .as_ref()
            .expect("Transaction info should exist.")
            .changes
            .iter()
            .find_map(|wsc| {
                if let WriteSetChangeEnum::WriteResource(resource) = &wsc.change.as_ref().unwrap() {
                    if standardize_address(resource.address.as_str()) == market_address.as_str() {
                        if let Ok(Some(market)) = Self::from_write_resource(resource) {
                            return Some(market);
                        }
                    }
                }
                return None;
            })
            .expect(
                format!(
                    "Market resource should exist. Version: {} Market address: {}",
                    txn.version, market_address
                )
                .as_str(),
            )
    }
}