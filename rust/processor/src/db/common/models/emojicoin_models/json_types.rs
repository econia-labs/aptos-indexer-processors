use anyhow::{Context, Result};
use bigdecimal::BigDecimal;
use serde::{Deserialize, Deserializer, Serialize};

use crate::{
    db::common::models::emojicoin_models::enums::{
        deserialize_periodic_state_resolution, deserialize_state_trigger,
    },
    utils::util::{deserialize_from_string, hex_to_raw_bytes, standardize_address},
};

use super::enums::{EmojicoinTypeTag, PeriodicStateResolution, StateTrigger};

pub fn deserialize_bytes_from_hex_string<'de, D>(
    deserializer: D,
) -> core::result::Result<Vec<u8>, D::Error>
where
    D: Deserializer<'de>,
{
    use serde::de::Error;
    let s = <String>::deserialize(deserializer)?;
    hex_to_raw_bytes(&s).map_err(D::Error::custom)
}

pub fn deserialize_and_normalize_account_address<'de, D>(
    deserializer: D,
) -> core::result::Result<String, D::Error>
where
    D: Deserializer<'de>,
{
    let s = <String>::deserialize(deserializer)?;
    Ok(standardize_address(&s))
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct MarketMetadata {
    #[serde(deserialize_with = "deserialize_from_string")]
    pub market_id: i64,
    #[serde(deserialize_with = "deserialize_and_normalize_account_address")]
    pub market_address: String,
    #[serde(deserialize_with = "deserialize_bytes_from_hex_string")]
    pub emoji_bytes: Vec<u8>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Reserves {
    #[serde(deserialize_with = "deserialize_from_string")]
    pub base: i64,
    #[serde(deserialize_with = "deserialize_from_string")]
    pub quote: i64,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct PeriodicStateMetadata {
    #[serde(deserialize_with = "deserialize_from_string")]
    pub start_time: i64,
    #[serde(deserialize_with = "deserialize_periodic_state_resolution")]
    pub period: PeriodicStateResolution,
    #[serde(deserialize_with = "deserialize_from_string")]
    pub emit_time: i64,
    #[serde(deserialize_with = "deserialize_from_string")]
    pub emit_market_nonce: i64,
    #[serde(deserialize_with = "deserialize_state_trigger")]
    pub trigger: StateTrigger,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct StateMetadata {
    #[serde(deserialize_with = "deserialize_from_string")]
    pub market_nonce: i64,
    #[serde(deserialize_with = "deserialize_from_string")]
    pub bump_time: i64,
    #[serde(deserialize_with = "deserialize_state_trigger")]
    pub trigger: StateTrigger,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct CumulativeStats {
    #[serde(deserialize_with = "deserialize_from_string")]
    pub base_volume: BigDecimal,
    #[serde(deserialize_with = "deserialize_from_string")]
    pub quote_volume: BigDecimal,
    #[serde(deserialize_with = "deserialize_from_string")]
    pub integrator_fees: BigDecimal,
    #[serde(deserialize_with = "deserialize_from_string")]
    pub pool_fees_base: BigDecimal,
    #[serde(deserialize_with = "deserialize_from_string")]
    pub pool_fees_quote: BigDecimal,
    #[serde(deserialize_with = "deserialize_from_string")]
    pub n_swaps: i64,
    #[serde(deserialize_with = "deserialize_from_string")]
    pub n_chat_messages: i64,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct InstantaneousStats {
    #[serde(deserialize_with = "deserialize_from_string")]
    pub total_quote_locked: i64,
    #[serde(deserialize_with = "deserialize_from_string")]
    pub total_value_locked: BigDecimal,
    #[serde(deserialize_with = "deserialize_from_string")]
    pub market_cap: BigDecimal,
    #[serde(deserialize_with = "deserialize_from_string")]
    pub fully_diluted_value: BigDecimal,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct LastSwap {
    pub is_sell: bool,
    #[serde(deserialize_with = "deserialize_from_string")]
    pub avg_execution_price_q64: BigDecimal,
    #[serde(deserialize_with = "deserialize_from_string")]
    pub base_volume: i64,
    #[serde(deserialize_with = "deserialize_from_string")]
    pub quote_volume: i64,
    #[serde(deserialize_with = "deserialize_from_string")]
    pub nonce: i64,
    #[serde(deserialize_with = "deserialize_from_string")]
    pub time: i64,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct SwapEvent {
    #[serde(deserialize_with = "deserialize_from_string")]
    pub market_id: i64,
    #[serde(deserialize_with = "deserialize_from_string")]
    pub time: i64,
    #[serde(deserialize_with = "deserialize_from_string")]
    pub market_nonce: i64,
    #[serde(deserialize_with = "deserialize_and_normalize_account_address")]
    pub swapper: String,
    #[serde(deserialize_with = "deserialize_from_string")]
    pub input_amount: i64,
    pub is_sell: bool,
    #[serde(deserialize_with = "deserialize_and_normalize_account_address")]
    pub integrator: String,
    pub integrator_fee_rate_bps: i16,
    #[serde(deserialize_with = "deserialize_from_string")]
    pub net_proceeds: i64,
    #[serde(deserialize_with = "deserialize_from_string")]
    pub base_volume: i64,
    #[serde(deserialize_with = "deserialize_from_string")]
    pub quote_volume: i64,
    #[serde(deserialize_with = "deserialize_from_string")]
    pub avg_execution_price_q64: BigDecimal,
    #[serde(deserialize_with = "deserialize_from_string")]
    pub integrator_fee: i64,
    #[serde(deserialize_with = "deserialize_from_string")]
    pub pool_fee: i64,
    pub starts_in_bonding_curve: bool,
    pub results_in_state_transition: bool,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ChatEvent {
    pub market_metadata: MarketMetadata,
    #[serde(deserialize_with = "deserialize_from_string")]
    pub emit_time: i64,
    #[serde(deserialize_with = "deserialize_from_string")]
    pub emit_market_nonce: i64,
    #[serde(deserialize_with = "deserialize_and_normalize_account_address")]
    pub user: String,
    pub message: String,
    #[serde(deserialize_with = "deserialize_from_string")]
    pub user_emojicoin_balance: i64,
    #[serde(deserialize_with = "deserialize_from_string")]
    pub circulating_supply: i64,
    #[serde(deserialize_with = "deserialize_from_string")]
    pub balance_as_fraction_of_circulating_supply_q64: BigDecimal,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct MarketRegistrationEvent {
    pub market_metadata: MarketMetadata,
    #[serde(deserialize_with = "deserialize_from_string")]
    pub time: i64,
    #[serde(deserialize_with = "deserialize_and_normalize_account_address")]
    pub registrant: String,
    #[serde(deserialize_with = "deserialize_and_normalize_account_address")]
    pub integrator: String,
    #[serde(deserialize_with = "deserialize_from_string")]
    pub integrator_fee: i64,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct PeriodicStateEvent {
    pub market_metadata: MarketMetadata,
    pub periodic_state_metadata: PeriodicStateMetadata,
    #[serde(deserialize_with = "deserialize_from_string")]
    pub open_price_q64: BigDecimal,
    #[serde(deserialize_with = "deserialize_from_string")]
    pub high_price_q64: BigDecimal,
    #[serde(deserialize_with = "deserialize_from_string")]
    pub low_price_q64: BigDecimal,
    #[serde(deserialize_with = "deserialize_from_string")]
    pub close_price_q64: BigDecimal,
    #[serde(deserialize_with = "deserialize_from_string")]
    pub volume_base: BigDecimal,
    #[serde(deserialize_with = "deserialize_from_string")]
    pub volume_quote: BigDecimal,
    #[serde(deserialize_with = "deserialize_from_string")]
    pub integrator_fees: BigDecimal,
    #[serde(deserialize_with = "deserialize_from_string")]
    pub pool_fees_base: BigDecimal,
    #[serde(deserialize_with = "deserialize_from_string")]
    pub pool_fees_quote: BigDecimal,
    #[serde(deserialize_with = "deserialize_from_string")]
    pub n_swaps: i64,
    #[serde(deserialize_with = "deserialize_from_string")]
    pub n_chat_messages: i64,
    pub starts_in_bonding_curve: bool,
    pub ends_in_bonding_curve: bool,
    #[serde(deserialize_with = "deserialize_from_string")]
    pub tvl_per_lp_coin_growth_q64: BigDecimal,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct StateEvent {
    pub market_metadata: MarketMetadata,
    pub state_metadata: StateMetadata,
    pub clamm_virtual_reserves: Reserves,
    pub cpamm_real_reserves: Reserves,
    #[serde(deserialize_with = "deserialize_from_string")]
    pub lp_coin_supply: BigDecimal,
    pub cumulative_stats: CumulativeStats,
    pub instantaneous_stats: InstantaneousStats,
    pub last_swap: LastSwap,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct GlobalStateEvent {
    #[serde(deserialize_with = "deserialize_from_string")]
    pub emit_time: i64,
    #[serde(deserialize_with = "deserialize_from_string")]
    pub registry_nonce: i64,
    #[serde(deserialize_with = "deserialize_state_trigger")]
    pub trigger: StateTrigger,
    #[serde(deserialize_with = "deserialize_from_string")]
    pub cumulative_quote_volume: BigDecimal,
    #[serde(deserialize_with = "deserialize_from_string")]
    pub total_quote_locked: BigDecimal,
    #[serde(deserialize_with = "deserialize_from_string")]
    pub total_value_locked: BigDecimal,
    #[serde(deserialize_with = "deserialize_from_string")]
    pub market_cap: BigDecimal,
    #[serde(deserialize_with = "deserialize_from_string")]
    pub fully_diluted_value: BigDecimal,
    #[serde(deserialize_with = "deserialize_from_string")]
    pub cumulative_integrator_fees: BigDecimal,
    #[serde(deserialize_with = "deserialize_from_string")]
    pub cumulative_swaps: i64,
    #[serde(deserialize_with = "deserialize_from_string")]
    pub cumulative_chat_messages: i64,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct LiquidityEvent {
    #[serde(deserialize_with = "deserialize_from_string")]
    pub market_id: i64,
    #[serde(deserialize_with = "deserialize_from_string")]
    pub time: i64,
    #[serde(deserialize_with = "deserialize_from_string")]
    pub market_nonce: i64,
    #[serde(deserialize_with = "deserialize_and_normalize_account_address")]
    pub provider: String,
    #[serde(deserialize_with = "deserialize_from_string")]
    pub base_amount: i64,
    #[serde(deserialize_with = "deserialize_from_string")]
    pub quote_amount: i64,
    #[serde(deserialize_with = "deserialize_from_string")]
    pub lp_coin_amount: i64,
    pub liquidity_provided: bool,
    #[serde(deserialize_with = "deserialize_from_string")]
    pub pro_rata_base_donation_claim_amount: i64,
    #[serde(deserialize_with = "deserialize_from_string")]
    pub pro_rata_quote_donation_claim_amount: i64,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum EventWithMarket {
    PeriodicState(PeriodicStateEvent),
    State(StateEvent),
    Swap(SwapEvent),
    Chat(ChatEvent),
    Liquidity(LiquidityEvent),
    MarketRegistration(MarketRegistrationEvent),
}

impl From<PeriodicStateEvent> for EventWithMarket {
    fn from(event: PeriodicStateEvent) -> Self {
        EventWithMarket::PeriodicState(event)
    }
}

impl From<StateEvent> for EventWithMarket {
    fn from(event: StateEvent) -> Self {
        EventWithMarket::State(event)
    }
}

impl EventWithMarket {
    pub fn from_event_type(
        event_type: &str,
        data: &str,
        txn_version: i64,
    ) -> Result<Option<EventWithMarket>> {
        match EmojicoinTypeTag::from_str(event_type) {
            Some(EmojicoinTypeTag::PeriodicState) => {
                serde_json::from_str(data).map(|v| Some(EventWithMarket::PeriodicState(v)))
            },
            Some(EmojicoinTypeTag::State) => {
                serde_json::from_str(data).map(|v| Some(EventWithMarket::State(v)))
            },
            Some(EmojicoinTypeTag::Swap) => {
                serde_json::from_str(data).map(|v| Some(EventWithMarket::Swap(v)))
            },
            Some(EmojicoinTypeTag::Chat) => {
                serde_json::from_str(data).map(|v| Some(EventWithMarket::Chat(v)))
            },
            Some(EmojicoinTypeTag::MarketRegistration) => {
                serde_json::from_str(data).map(|v| Some(EventWithMarket::MarketRegistration(v)))
            },
            Some(EmojicoinTypeTag::Liquidity) => {
                serde_json::from_str(data).map(|v| Some(EventWithMarket::Liquidity(v)))
            },
            _ => Ok(None),
        }
        .context(format!(
            "version {} failed! Failed to parse type {}, with data: {:?}",
            txn_version, event_type, data,
        ))
    }
}

impl GlobalStateEvent {
    pub fn from_event_type(
        event_type: &str,
        data: &str,
        txn_version: i64,
    ) -> Result<Option<GlobalStateEvent>> {
        match EmojicoinTypeTag::from_str(event_type) {
            Some(EmojicoinTypeTag::GlobalState) => {
                serde_json::from_str::<GlobalStateEvent>(data).map(Some)
            },
            _ => Ok(None),
        }
        .context(format!(
            "version {} failed! Failed to parse type {}, with data: {:?}",
            txn_version, event_type, data,
        ))
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum BumpEvent {
    MarketRegistration(MarketRegistrationEvent),
    Swap(SwapEvent),
    Chat(ChatEvent),
    Liquidity(LiquidityEvent),
}
// A subset of the transaction info that comes in from the GRPC stream.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct TxnInfo {
    pub version: i64,
    pub sender: String,
    pub entry_function: Option<String>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct BumpGroup {
    pub market_id: i64,
    pub market_nonce: i64,
    pub bump_event: BumpEvent,
    pub state_event: StateEvent,
    pub periodic_state_events: Vec<PeriodicStateEvent>,
    pub txn_info: TxnInfo,
}

impl From<BumpEvent> for EventWithMarket {
    fn from(event: BumpEvent) -> Self {
        match event {
            BumpEvent::MarketRegistration(event) => EventWithMarket::MarketRegistration(event),
            BumpEvent::Swap(event) => EventWithMarket::Swap(event),
            BumpEvent::Chat(event) => EventWithMarket::Chat(event),
            BumpEvent::Liquidity(event) => EventWithMarket::Liquidity(event),
        }
    }
}
