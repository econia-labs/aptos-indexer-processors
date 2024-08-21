use super::{constants::{
    CHAT_EVENT, GLOBAL_STATE_EVENT, LIQUIDITY_EVENT, MARKET_REGISTRATION_EVENT, MARKET_RESOURCE,
    PERIODIC_STATE_EVENT, STATE_EVENT, SWAP_EVENT,
}, json_types::{EventWithMarket, GlobalStateEvent}};
use serde::{Deserialize, Deserializer, Serialize, Serializer};

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

impl From<&Trigger> for i16 {
    fn from(i: &Trigger) -> Self {
        match i {
            Trigger::PackagePublication => 0,
            Trigger::MarketRegistration => 1,
            Trigger::SwapBuy => 2,
            Trigger::SwapSell => 3,
            Trigger::ProvideLiquidity => 4,
            Trigger::RemoveLiquidity => 5,
            Trigger::Chat => 6,
        }
    }
}

pub fn serialize_state_trigger<S>(element: &Trigger, s: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    s.serialize_i16(element.into())
}

pub fn deserialize_state_trigger<'de, D>(
    deserializer: D,
) -> core::result::Result<Trigger, D::Error>
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
pub enum Period {
    #[db_rename = "period_1m"]
    OneMinute,
    #[db_rename = "period_5m"]
    FiveMinutes,
    #[db_rename = "period_15m"]
    FifteenMinutes,
    #[db_rename = "period_30m"]
    ThirtyMinutes,
    #[db_rename = "period_1h"]
    OneHour,
    #[db_rename = "period_4h"]
    FourHours,
    #[db_rename = "period_1d"]
    OneDay,
}

pub fn serialize_state_period<S>(element: &Period, s: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    let r = match element {
        Period::OneMinute => "60000000",
        Period::FiveMinutes => "300000000",
        Period::FifteenMinutes => "900000000",
        Period::ThirtyMinutes => "1800000000",
        Period::OneHour => "3600000000",
        Period::FourHours => "14400000000",
        Period::OneDay => "86400000000",
    };
    s.serialize_str(r)
}

pub fn deserialize_state_period<'de, D>(deserializer: D) -> core::result::Result<Period, D::Error>
where
    D: Deserializer<'de>,
{
    use serde::de::Error;
    let period = <String>::deserialize(deserializer)?;
    match period.as_str() {
        "60000000" => Ok(Period::OneMinute),
        "300000000" => Ok(Period::FiveMinutes),
        "900000000" => Ok(Period::FifteenMinutes),
        "1800000000" => Ok(Period::ThirtyMinutes),
        "3600000000" => Ok(Period::OneHour),
        "14400000000" => Ok(Period::FourHours),
        "86400000000" => Ok(Period::OneDay),
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
    Market,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum EmojicoinEvent {
    EventWithMarket(EventWithMarket),
    EventWithoutMarket(GlobalStateEvent),
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
pub enum EmojicoinEventType {
    Swap,
    Chat,
    MarketRegistration,
    PeriodicState,
    State,
    GlobalState,
    Liquidity,
}

impl From<&EmojicoinEvent> for EmojicoinEventType {
    fn from(value: &EmojicoinEvent) -> Self {
        match value {
            EmojicoinEvent::EventWithMarket(e) => {
                match e {
                    EventWithMarket::PeriodicState(_) => EmojicoinEventType::PeriodicState,
                    EventWithMarket::State(_) => EmojicoinEventType::State,
                    EventWithMarket::Swap(_) => EmojicoinEventType::Swap,
                    EventWithMarket::Chat(_) => EmojicoinEventType::Chat,
                    EventWithMarket::Liquidity(_) => EmojicoinEventType::Liquidity,
                    EventWithMarket::MarketRegistration(_) => EmojicoinEventType::MarketRegistration,
                }
            },
            EmojicoinEvent::EventWithoutMarket(_) => EmojicoinEventType::GlobalState
        }
    }
}

impl EmojicoinTypeTag {
    pub fn from_type_str(type_str: &str) -> Option<Self> {
        match type_str {
            str if str == SWAP_EVENT.as_str() => Some(Self::Swap),
            str if str == CHAT_EVENT.as_str() => Some(Self::Chat),
            str if str == MARKET_REGISTRATION_EVENT.as_str() => Some(Self::MarketRegistration),
            str if str == PERIODIC_STATE_EVENT.as_str() => Some(Self::PeriodicState),
            str if str == STATE_EVENT.as_str() => Some(Self::State),
            str if str == GLOBAL_STATE_EVENT.as_str() => Some(Self::GlobalState),
            str if str == LIQUIDITY_EVENT.as_str() => Some(Self::Liquidity),
            str if str == MARKET_RESOURCE.as_str() => Some(Self::Market),
            _ => None,
        }
    }
}
