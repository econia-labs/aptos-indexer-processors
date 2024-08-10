use anyhow::{Context, Result};
use aptos_protos::transaction::v1::{transaction::TxnData, Event};
use bigdecimal::BigDecimal;
use serde::{Deserialize, Deserializer, Serialize};

use crate::{
    db::common::models::emojicoin_models::enums::{
        deserialize_periodic_state_resolution, deserialize_state_trigger,
    },
    utils::util::{
        deserialize_from_string, get_entry_function_from_user_request, hex_to_raw_bytes,
        standardize_address,
    },
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
    pub n_chat_messages: u64,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct InstantaneousStats {
    #[serde(deserialize_with = "deserialize_from_string")]
    pub total_quote_locked: i64,
    #[serde(deserialize_with = "deserialize_from_string")]
    pub total_value_locked: i64,
    #[serde(deserialize_with = "deserialize_from_string")]
    pub market_cap: i64,
    #[serde(deserialize_with = "deserialize_from_string")]
    pub fully_diluted_value: i64,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct LastSwap {
    is_sell: bool,
    #[serde(deserialize_with = "deserialize_from_string")]
    avg_execution_price_q64: BigDecimal,
    #[serde(deserialize_with = "deserialize_from_string")]
    base_volume: i64,
    #[serde(deserialize_with = "deserialize_from_string")]
    quote_volume: i64,
    #[serde(deserialize_with = "deserialize_from_string")]
    nonce: i64,
    #[serde(deserialize_with = "deserialize_from_string")]
    time: i64,
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
pub enum EmojicoinEvent {
    EventWithMarket(EventWithMarket),
    GlobalState(GlobalStateEvent),
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

impl From<EventWithMarket> for EmojicoinEvent {
    fn from(event: EventWithMarket) -> Self {
        EmojicoinEvent::EventWithMarket(event)
    }
}

impl EmojicoinEvent {
    pub fn from_event_type(
        event_type: &str,
        data: &str,
        txn_version: i64,
    ) -> Result<Option<EmojicoinEvent>> {
        match EmojicoinTypeTag::from_str(event_type) {
            Some(EmojicoinTypeTag::PeriodicState) => serde_json::from_str(data)
                .map(|v| Some(EventWithMarket::PeriodicState(v)))
                .map(|opt| opt.map(EmojicoinEvent::from)),
            Some(EmojicoinTypeTag::State) => serde_json::from_str(data)
                .map(|v| Some(EventWithMarket::State(v)))
                .map(|opt| opt.map(EmojicoinEvent::from)),
            Some(EmojicoinTypeTag::Swap) => serde_json::from_str(data)
                .map(|v| Some(EventWithMarket::Swap(v)))
                .map(|opt| opt.map(EmojicoinEvent::from)),
            Some(EmojicoinTypeTag::Chat) => serde_json::from_str(data)
                .map(|v| Some(EventWithMarket::Chat(v)))
                .map(|opt| opt.map(EmojicoinEvent::from)),
            Some(EmojicoinTypeTag::MarketRegistration) => serde_json::from_str(data)
                .map(|v| Some(EventWithMarket::MarketRegistration(v)))
                .map(|opt| opt.map(EmojicoinEvent::from)),
            Some(EmojicoinTypeTag::Liquidity) => serde_json::from_str(data)
                .map(|v| Some(EventWithMarket::Liquidity(v)))
                .map(|opt| opt.map(EmojicoinEvent::from)),
            Some(EmojicoinTypeTag::GlobalState) => {
                serde_json::from_str(data).map(|v| Some(EmojicoinEvent::GlobalState(v)))
            },
            _ => Ok(None),
        }
        .context(format!(
            "version {} failed! Failed to parse type {}, with data: {:?}",
            txn_version, event_type, data,
        ))
    }

    pub fn maybe_from_event(event: &Event, txn_version: i64) -> Result<Option<EmojicoinEvent>> {
        Self::from_event_type(
            &event.type_str.as_str(),
            &event.data.to_string(),
            txn_version,
        )
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum BumpEvent {
    MarketRegistration(MarketRegistrationEvent),
    Swap(SwapEvent),
    Chat(ChatEvent),
    Liquidity(LiquidityEvent),
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct EmojicoinEventAndTxnInfo {
    event: EmojicoinEvent,
    txn_info: TxnInfo,
}

impl EmojicoinEventAndTxnInfo {
    pub fn maybe_from_event(event: &Event, txn_info: TxnInfo) -> Option<Self> {
        EmojicoinEvent::maybe_from_event(event, txn_info.version)
            .ok()?
            .map(|event| Self { event, txn_info })
    }

    pub fn from_transaction(txn_version: i64, txn_data: &TxnData) -> Vec<Self> {
        let mut emojicoin_events = vec![];
        if let TxnData::User(user_txn) = txn_data {
            let user_request = user_txn
                .request
                .as_ref()
                .expect("User request info is not present in the user transaction.");
            let entry_function = get_entry_function_from_user_request(user_request);
            let txn_info = TxnInfo {
                version: txn_version,
                sender: standardize_address(user_request.sender.as_ref()),
                entry_function,
            };

            for event in user_txn.events.iter() {
                if let Some(emojicoin_event) = Self::maybe_from_event(event, txn_info.clone()) {
                    emojicoin_events.push(emojicoin_event);
                }
            }
        }
        emojicoin_events
    }
}

// A subset of the transaction info that comes in from the GRPC stream.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct TxnInfo {
    version: i64,
    sender: String,
    entry_function: Option<String>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct BumpGroup {
    market_id: i64,
    market_nonce: i64,
    bump_event: BumpEvent,
    state_event: StateEvent,
    periodic_state_events: Vec<PeriodicStateEvent>,
    txn_info: TxnInfo,
}

// For grouping all events in a single transaction into the various types:
// Each state event has a unique bump nonce, we can use that to group non-state events with state events.
// The following groupings are possible:
// -- Market-ID specific State Events
//    - ONE State Event
//    - ONE of the following:
//       - Market Registration
//       - Chat
//       - Swap
//       - Liquidity
//    - ZERO to SIX of the following:
//       - Periodic State Events
// Note that we have no (easy) way of knowing which state event triggered a GlobalStateEvent, because it doesn't emit
//  the market_id or bump_nonce. We can only know that it was triggered by a state event. This means we can't group
//  GlobalStateEvents with StateEvents in a BumpGroup.
//  This is totally fine, because we can just insert the GlobalStateEvent into the global_state_events table separately.
//  Note that there will only ever be one GlobalStateEvent per transaction, because it takes an entire day to emit
//   a GlobalStateEvent since the last one. Thus we just have an `Option<GlobalStateEvent>` for each transaction that
//   we possibly insert into the global_state_events table after event processing.

// We can collect all events into a big vector of events.
// Sort the vector by market_id first, then the bump_nonce. Note we don't need to even check the StateTrigger type because haveint the same market_id and bump_nonce means
// there will definitively *only* be one StateTrigger type.
// We can actually panic if we somehow don't fill the bump event by the end of the transaction event iteration.
//   It should literally never happen unless the processor was written incorrectly.
// So:
//    1. Create a vector of all events
//    2. Sort the vector by market_id, then bump_nonce
//    3. Iterate over the sorted vector. You MUST be able to place EVERY single event into a BumpGroup.
//    4. Use the BumpGroup to insert each event into its corresponding table:
//       - state_events
//       - periodic_state_events
//       - global_state_events
// Try to keep in mind that we will eventually query for the rolling 24-hour volume as well.
//   - This will be a query right before the insert. We will find the earliest row in `state_events` with a `last_swap` event time that was at least 24 hours ago.
//   - Then we use that to subtract the current total/cumulative volume from the total/cumulative volume at that time, which will give us the 24-hour volume.
