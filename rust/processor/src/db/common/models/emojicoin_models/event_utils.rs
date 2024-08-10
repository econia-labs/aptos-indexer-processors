use super::db_types::{
    periodic_state_events_model::PeriodicStateEventModel, state_bumps_model::StateBumpModel,
};
use super::json_types::{
    BumpEvent, BumpGroup, EventWithMarket, PeriodicStateEvent, StateEvent, TxnInfo,
};
use std::cmp::Ordering;

impl EventWithMarket {
    pub fn get_rank_by_type(&self) -> i64 {
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

// We implement the sorting here to prioritize the following values:
// 1. Market ID, asc, so we can group events by market.
// 2. Market nonce, asc, so we can group events in chronological order.
// 3. The type of event.
impl Ord for EventWithMarket {
    fn cmp(&self, other: &Self) -> Ordering {
        self.get_market_id()
            .cmp(&other.get_market_id())
            .then(self.get_market_nonce().cmp(&other.get_market_nonce()))
            .then(self.get_rank_by_type().cmp(&other.get_rank_by_type()))
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
    pub fn new(market_id: i64, market_nonce: i64, txn_info: TxnInfo) -> Self {
        Self {
            market_id,
            market_nonce,
            bump_event: None,
            state_event: None,
            periodic_state_events: vec![],
            txn_info,
        }
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
    pub fn to_db_rows(&self) -> (StateBumpModel, Vec<PeriodicStateEventModel>) {
        // Market registration & Swap data.
        // let integrator = self.state_event.state_metadata.integrator.clone();

        let bump_event = self.bump_event.to_db_row();
        let state_event = self.state_event.to_db_row();
        (bump_event, state_event)
    }
}
