// impl StateTrigger {
//     pub fn from_str(s: &str) -> Option<Self> {
//         match s {
//             "PackagePublication" => Some(Self::PackagePublication),
//             "MarketRegistration" => Some(Self::MarketRegistration),
//             "SwapBuy" => Some(Self::SwapBuy),
//             "SwapSell" => Some(Self::SwapSell),
//             "ProvideLiquidity" => Some(Self::ProvideLiquidity),
//             "RemoveLiquidity" => Some(Self::RemoveLiquidity),
//             "Chat" => Some(Self::Chat),
//             _ => None,
//         }
//     }
// }

// use crate::db::common::models::events_models::events::EventModel;

// pub struct MarketRegistrationEvent {
//     pub market_metadata:
// }

// pub enum StateEvents {
//     PackagePublicationEvent(PackagePublicationEvent),
//     MarketRegistrationEvent(MarketRegistrationEvent),
//     SwapBuyEvent(SwapBuyEvent),
//     SwapSellEvent(SwapSellEvent),
//     ProvideLiquidityEvent(ProvideLiquidityEvent),
//     RemoveLiquidityEvent(RemoveLiquidityEvent),
//     ChatEvent(ChatEvent),
// }

// impl MarketRegistrationEvent {
//     pub fn from_event(event: &EventModel) -> Option<Self> {
//         match event {
//             EventModel::MarketRegistrationEvent(event) => Some(event.clone()),
//             _ => None,
//         }
//     }
// }
