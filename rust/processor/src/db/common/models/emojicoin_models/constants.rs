use std::env;

pub const SWAP_EVENT: &'static str = concat!(
    env!("EMOJICOIN_MODULE_ADDRESS"),
    "::emojicoin_dot_fun::Swap"
);
pub const CHAT_EVENT: &'static str = concat!(
    env!("EMOJICOIN_MODULE_ADDRESS"),
    "::emojicoin_dot_fun::Chat"
);
pub const MARKET_REGISTRATION_EVENT: &'static str = concat!(
    env!("EMOJICOIN_MODULE_ADDRESS"),
    "::emojicoin_dot_fun::MarketRegistration"
);
pub const PERIODIC_STATE_EVENT: &'static str = concat!(
    env!("EMOJICOIN_MODULE_ADDRESS"),
    "::emojicoin_dot_fun::PeriodicState"
);
pub const STATE_EVENT: &'static str = concat!(
    env!("EMOJICOIN_MODULE_ADDRESS"),
    "::emojicoin_dot_fun::State"
);
pub const GLOBAL_STATE_EVENT: &'static str = concat!(
    env!("EMOJICOIN_MODULE_ADDRESS"),
    "::emojicoin_dot_fun::GlobalState"
);
pub const LIQUIDITY_EVENT: &'static str = concat!(
    env!("EMOJICOIN_MODULE_ADDRESS"),
    "::emojicoin_dot_fun::Liquidity"
);
pub const MARKET_RESOURCE: &'static str = concat!(
    env!("EMOJICOIN_MODULE_ADDRESS"),
    "::emojicoin_dot_fun::Market"
);
// When a market is first registered, the market_nonce field is emitted in the resulting events as 1.
pub const INITIAL_MARKET_NONCE: i64 = 1;

#[cfg(test)]
mod tests {
    use crate::utils::util::standardize_address;

    #[test]
    fn ensure_contract_address_is_standardized() {
        if standardize_address(env!("EMOJICOIN_MODULE_ADDRESS")) != env!("EMOJICOIN_MODULE_ADDRESS")
        {
            panic!(
                "The non-standardized contract address: {} is invalid because it doesn't match the standardized address: {}",
                env!("EMOJICOIN_MODULE_ADDRESS"),
                standardize_address(env!("EMOJICOIN_MODULE_ADDRESS"))
            );
        }
    }
}
