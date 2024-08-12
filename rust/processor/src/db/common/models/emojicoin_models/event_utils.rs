use super::json_types::{
    BumpEvent, BumpGroup, EventWithMarket, PeriodicStateEvent, StateEvent, TxnInfo,
};
use super::models::{bump_event::BumpEventModel, periodic_state_event::PeriodicStateEventModel};
use std::cmp::Ordering;

impl EventWithMarket {
    pub fn get_sort_rank(&self) -> i64 {
        match self {
            EventWithMarket::State(_) => 0,
            EventWithMarket::MarketRegistration(_) => 1,
            EventWithMarket::Chat(_) => 2,
            EventWithMarket::Swap(_) => 3,
            EventWithMarket::PeriodicState(_) => 4,
            EventWithMarket::Liquidity(_) => 5,
        }
    }

    pub fn get_market_id(&self) -> i64 {
        match self {
            EventWithMarket::Chat(event) => event.market_metadata.market_id,
            EventWithMarket::Swap(event) => event.market_id,
            EventWithMarket::State(event) => event.market_metadata.market_id,
            EventWithMarket::Liquidity(event) => event.market_id,
            EventWithMarket::MarketRegistration(event) => event.market_metadata.market_id,
            EventWithMarket::PeriodicState(event) => event.market_metadata.market_id,
        }
    }

    pub fn get_market_nonce(&self) -> i64 {
        match self {
            EventWithMarket::MarketRegistration(_) => 1,
            EventWithMarket::Chat(event) => event.emit_market_nonce,
            EventWithMarket::Swap(event) => event.market_nonce,
            EventWithMarket::State(event) => event.state_metadata.market_nonce,
            EventWithMarket::Liquidity(event) => event.market_nonce,
            EventWithMarket::PeriodicState(event) => {
                event.periodic_state_metadata.emit_market_nonce
            },
        }
    }
}

// For grouping all events in a single transaction into the various types:
// Each state event has a unique bump nonce, we can use that to group bump events (events that trigger state bumps)
// with their respective StateEvent.
// The following groupings are possible:
// -- ONE market ID and ONE market nonce.
//    - ONE State Event
//    - ONE Bump Event; i.e., one of the following:
//       - Market Registration Event
//       - Chat Event
//       - Swap Event
//       - Liquidity Event
//    - ZERO to SEVEN of the following:
//       - Periodic State Events (1m, 5m, 15m, 30m, 1h, 4h, 1d)
// Note that we have no easy way of knowing for sure which state event triggered a GlobalStateEvent, because it doesn't emit
// the market_id or bump_nonce. This means we can't group GlobalStateEvents with StateEvents in a BumpGroup.
//
/// We implement the sorting here to prioritize the following values:
/// 1. Market ID, asc, so we can group events by market.
/// 2. Market nonce, asc, so we can group events in chronological order.
/// 3. The type of event.
impl Ord for EventWithMarket {
    fn cmp(&self, other: &Self) -> Ordering {
        self.get_market_id()
            .cmp(&other.get_market_id())
            .then(self.get_market_nonce().cmp(&other.get_market_nonce()))
            .then(self.get_sort_rank().cmp(&other.get_sort_rank()))
    }
}

impl PartialOrd for EventWithMarket {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Eq for EventWithMarket {}

impl PartialEq for EventWithMarket {
    fn eq(&self, other: &Self) -> bool {
        self.cmp(other) == Ordering::Equal
    }
}

#[derive(Debug)]
pub struct BumpGroupBuilder {
    pub market_id: i64,
    pub market_nonce: i64,
    pub bump_event: Option<BumpEvent>,
    pub state_event: Option<StateEvent>,
    pub periodic_state_events: Vec<PeriodicStateEvent>,
    pub txn_info: TxnInfo,
}
impl BumpGroupBuilder {
    pub fn new(event: EventWithMarket, txn_info: TxnInfo) -> Self {
        let mut builder = Self {
            market_id: event.get_market_id(),
            market_nonce: event.get_market_nonce(),
            bump_event: None,
            state_event: None,
            periodic_state_events: vec![],
            txn_info,
        };

        builder.add_event(event);

        builder
    }

    pub fn add_event(&mut self, event: EventWithMarket) {
        debug_assert!(event.get_market_id() == self.market_id);
        debug_assert!(event.get_market_nonce() == self.market_nonce);
        match event {
            EventWithMarket::MarketRegistration(e) => {
                self.add_bump(BumpEvent::MarketRegistration(e))
            },
            EventWithMarket::Chat(e) => self.add_bump(BumpEvent::Chat(e)),
            EventWithMarket::Swap(e) => self.add_bump(BumpEvent::Swap(e)),
            EventWithMarket::Liquidity(e) => self.add_bump(BumpEvent::Liquidity(e)),
            EventWithMarket::State(e) => self.add_state(e.clone()),
            EventWithMarket::PeriodicState(e) => self.add_periodic_state(e.clone()),
        }
    }

    pub fn add_bump(&mut self, bump_event: BumpEvent) {
        debug_assert!(self.bump_event.is_none());
        self.bump_event = Some(bump_event);
    }

    pub fn add_state(&mut self, state_event: StateEvent) {
        debug_assert!(self.state_event.is_none());
        self.state_event = Some(state_event);
    }

    pub fn add_periodic_state(&mut self, periodic_state_event: PeriodicStateEvent) {
        debug_assert!(self.periodic_state_events.len() < 7);
        self.periodic_state_events.push(periodic_state_event);
    }

    pub fn build(self) -> BumpGroup {
        let bump_event = self.bump_event.expect("BumpGroups must have a BumpEvent.");
        let state_event = self
            .state_event
            .expect("BumpGroups must have a StateEvent.");

        BumpGroup {
            market_id: self.market_id,
            market_nonce: self.market_nonce,
            bump_event,
            state_event,
            periodic_state_events: self.periodic_state_events,
            txn_info: self.txn_info,
        }
    }
}

impl BumpGroup {
    pub fn to_db_models(self) -> (BumpEventModel, Vec<PeriodicStateEventModel>) {
        let BumpGroup {
            bump_event,
            state_event,
            periodic_state_events,
            txn_info,
            ..
        } = self;

        let periodic_events_model = PeriodicStateEventModel::from_periodic_events(
            txn_info.clone(),
            periodic_state_events,
            state_event.last_swap.clone(),
        );

        let state_bump_model =
            BumpEventModel::from_bump_and_state_event(txn_info, bump_event, state_event);

        (state_bump_model, periodic_events_model)
    }
}
