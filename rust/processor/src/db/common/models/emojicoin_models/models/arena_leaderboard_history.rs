use crate::processors::emojicoin_dot_fun::processor::MeleeData;
use bigdecimal::BigDecimal;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct ArenaLeaderboardHistoryPartialModel {
    pub melee_id: BigDecimal,

    pub emojicoin_0_price: BigDecimal,
    pub emojicoin_1_price: BigDecimal,
}

impl ArenaLeaderboardHistoryPartialModel {
    pub fn new(melee_data: &MeleeData) -> ArenaLeaderboardHistoryPartialModel {
        ArenaLeaderboardHistoryPartialModel {
            melee_id: melee_data.melee_id.clone(),

            emojicoin_0_price: melee_data.price_0.clone(),
            emojicoin_1_price: melee_data.price_1.clone(),
        }
    }
}
