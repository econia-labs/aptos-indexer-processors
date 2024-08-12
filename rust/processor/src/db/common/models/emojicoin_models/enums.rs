use super::constants::{
    CHAT_EVENT, GLOBAL_STATE_EVENT, LIQUIDITY_EVENT, MARKET_REGISTRATION_EVENT,
    PERIODIC_STATE_EVENT, STATE_EVENT, SWAP_EVENT,
};
use serde::{Deserialize, Deserializer, Serialize};

#[derive(
    Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, diesel_derive_enum::DbEnum,
)]
#[ExistingTypePath = "crate::schema::sql_types::TriggerType"]
pub enum Trigger {
    PackagePublication,
    MarketRegistration,
    SwapBuy,
    SwapSell,
    ProvideLiquidity,
    RemoveLiquidity,
    Chat,
}

impl Trigger {
    pub fn from_i16(i: i16) -> Option<Self> {
        match i {
            0 => Some(Self::PackagePublication),
            1 => Some(Self::MarketRegistration),
            2 => Some(Self::SwapBuy),
            3 => Some(Self::SwapSell),
            4 => Some(Self::ProvideLiquidity),
            5 => Some(Self::RemoveLiquidity),
            6 => Some(Self::Chat),
            _ => None,
        }
    }
}

pub fn deserialize_state_trigger<'de, D>(deserializer: D) -> core::result::Result<Trigger, D::Error>
where
    D: Deserializer<'de>,
{
    use serde::de::Error;
    let trigger = <i16>::deserialize(deserializer)?;
    match Trigger::from_i16(trigger) {
        Some(trigger) => Ok(trigger),
        None => Err(D::Error::custom(format!(
            "Failed to deserialize Trigger from i16: {}",
            trigger
        ))),
    }
}

#[derive(
    Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, diesel_derive_enum::DbEnum,
)]
#[ExistingTypePath = "crate::schema::sql_types::PeriodType"]
pub enum PeriodType {
    #[db_rename = "1m"]
    Period1M,
    #[db_rename = "5m"]
    Period5M,
    #[db_rename = "15m"]
    Period15M,
    #[db_rename = "30m"]
    Period30M,
    #[db_rename = "1h"]
    Period1H,
    #[db_rename = "4h"]
    Period4H,
    #[db_rename = "1d"]
    Period1D,
}

pub fn deserialize_state_period<'de, D>(
    deserializer: D,
) -> core::result::Result<PeriodType, D::Error>
where
    D: Deserializer<'de>,
{
    use serde::de::Error;
    let period = <String>::deserialize(deserializer)?;
    match period.as_str() {
        "60000000" => Ok(PeriodType::Period1M),
        "300000000" => Ok(PeriodType::Period5M),
        "900000000" => Ok(PeriodType::Period15M),
        "1800000000" => Ok(PeriodType::Period30M),
        "3600000000" => Ok(PeriodType::Period1H),
        "14400000000" => Ok(PeriodType::Period4H),
        "86400000000" => Ok(PeriodType::Period1D),
        _ => Err(D::Error::custom(format!(
            "Failed to deserialize PeriodType from string: {}",
            period
        ))),
    }
}

pub enum EmojicoinTypeTag {
    Swap,
    Chat,
    MarketRegistration,
    PeriodicState,
    State,
    GlobalState,
    Liquidity,
}

impl EmojicoinTypeTag {
    pub fn from_type_str(type_str: &str) -> Option<Self> {
        match type_str {
            SWAP_EVENT => Some(Self::Swap),
            CHAT_EVENT => Some(Self::Chat),
            MARKET_REGISTRATION_EVENT => Some(Self::MarketRegistration),
            PERIODIC_STATE_EVENT => Some(Self::PeriodicState),
            STATE_EVENT => Some(Self::State),
            GLOBAL_STATE_EVENT => Some(Self::GlobalState),
            LIQUIDITY_EVENT => Some(Self::Liquidity),
            _ => None,
        }
    }
}
