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
