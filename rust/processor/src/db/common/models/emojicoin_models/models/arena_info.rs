use crate::schema::arena_info;
use bigdecimal::BigDecimal;
use field_count::FieldCount;
use num::Zero;
use serde::{Deserialize, Serialize};

use super::arena_melee_event::ArenaMeleeEventModel;

#[derive(Clone, Debug, Deserialize, FieldCount, Identifiable, Insertable, Serialize)]
#[diesel(primary_key(melee_id))]
#[diesel(table_name = arena_info)]
pub struct ArenaInfoModel {
    pub melee_id: BigDecimal,
    pub volume: BigDecimal,
    pub rewards_remaining: BigDecimal,
    pub apt_locked: BigDecimal,

    pub emojicoin_0_market_address: String,
    pub emojicoin_1_market_address: String,
    pub emojicoin_0_market_id: BigDecimal,
    pub emojicoin_1_market_id: BigDecimal,
    pub emojicoin_0_symbols: Vec<String>,
    pub emojicoin_1_symbols: Vec<String>,
    pub start_time: chrono::NaiveDateTime,
    pub duration: BigDecimal,
    pub max_match_percentage: BigDecimal,
    pub max_match_amount: BigDecimal,
}

pub struct ArenaInfoData {
    pub emojicoin_0_market_id: BigDecimal,
    pub emojicoin_1_market_id: BigDecimal,
    pub emojicoin_0_symbols: Vec<String>,
    pub emojicoin_1_symbols: Vec<String>,
}

impl ArenaInfoModel {
    pub fn new(arena_melee_event: ArenaMeleeEventModel, data: ArenaInfoData) -> ArenaInfoModel {
        ArenaInfoModel {
            melee_id: arena_melee_event.melee_id,
            volume: BigDecimal::zero(),
            rewards_remaining: arena_melee_event.available_rewards,
            apt_locked: BigDecimal::zero(),

            emojicoin_0_market_address: arena_melee_event.emojicoin_0_market_address,
            emojicoin_1_market_address: arena_melee_event.emojicoin_1_market_address,
            emojicoin_0_market_id: data.emojicoin_0_market_id,
            emojicoin_1_market_id: data.emojicoin_1_market_id,
            emojicoin_0_symbols: data.emojicoin_0_symbols,
            emojicoin_1_symbols: data.emojicoin_1_symbols,
            start_time: arena_melee_event.start_time,
            duration: arena_melee_event.duration,
            max_match_percentage: arena_melee_event.max_match_percentage,
            max_match_amount: arena_melee_event.max_match_amount,
        }
    }
}
