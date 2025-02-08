use super::{
    arena_enter_event::ArenaEnterEventModel, arena_exit_event::ArenaExitEventModel,
    arena_swap_event::ArenaSwapEventModel, swap_event::SwapEventModel,
};
use crate::schema::arena_position;
use bigdecimal::BigDecimal;
use field_count::FieldCount;
use num::Zero;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Deserialize, FieldCount, Identifiable, Insertable, Serialize)]
#[diesel(primary_key(melee_id))]
#[diesel(table_name = arena_position)]
pub struct ArenaPositionModel {
    pub user: String,
    pub melee_id: BigDecimal,
    pub open: bool,
    pub emojicoin_0_balance: BigDecimal,
    pub emojicoin_1_balance: BigDecimal,
    pub withdrawals: BigDecimal,
    pub deposits: BigDecimal,
    pub match_amount: BigDecimal,
    pub last_exit_0: Option<bool>,
}

impl From<ArenaEnterEventModel> for ArenaPositionModel {
    fn from(arena_enter_event: ArenaEnterEventModel) -> ArenaPositionModel {
        ArenaPositionModel {
            user: arena_enter_event.user,
            melee_id: arena_enter_event.melee_id,
            open: true,
            emojicoin_0_balance: arena_enter_event.emojicoin_0_proceeds,
            emojicoin_1_balance: arena_enter_event.emojicoin_1_proceeds,
            withdrawals: BigDecimal::zero(),
            deposits: arena_enter_event.input_amount,
            match_amount: arena_enter_event.match_amount,
            last_exit_0: None,
        }
    }
}

impl ArenaPositionModel {
    fn from_swap(
        arena_swap_event: ArenaSwapEventModel,
        swaps: (SwapEventModel, SwapEventModel),
    ) -> ArenaPositionModel {
        ArenaPositionModel {
            user: arena_swap_event.user,
            melee_id: arena_swap_event.melee_id,
            open: true,
            emojicoin_0_balance: swaps.0.base_volume * if swaps.0.is_sell { -1 } else { 1 },
            emojicoin_1_balance: swaps.1.base_volume * if swaps.1.is_sell { -1 } else { 1 },
            withdrawals: BigDecimal::zero(),
            deposits: BigDecimal::zero(),
            match_amount: BigDecimal::zero(),
            last_exit_0: None,
        }
    }
}

impl From<ArenaExitEventModel> for ArenaPositionModel {
    fn from(arena_exit_event: ArenaExitEventModel) -> ArenaPositionModel {
        ArenaPositionModel {
            user: arena_exit_event.user,
            melee_id: arena_exit_event.melee_id,
            open: false,
            emojicoin_0_balance: -arena_exit_event.emojicoin_0_proceeds.clone(),
            emojicoin_1_balance: -arena_exit_event.emojicoin_1_proceeds.clone(),
            withdrawals: arena_exit_event.emojicoin_0_proceeds
                / arena_exit_event.emojicoin_0_exchange_rate_base
                * arena_exit_event.emojicoin_0_exchange_rate_quote
                + arena_exit_event.emojicoin_1_proceeds.clone()
                    / arena_exit_event.emojicoin_1_exchange_rate_base
                    * arena_exit_event.emojicoin_1_exchange_rate_quote,
            deposits: BigDecimal::zero(),
            match_amount: -arena_exit_event.tap_out_fee,
            last_exit_0: Some(arena_exit_event.emojicoin_1_proceeds.is_zero()),
        }
    }
}
