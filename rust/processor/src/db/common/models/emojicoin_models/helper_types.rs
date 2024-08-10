use super::enums::StateTrigger;
use bigdecimal::BigDecimal;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SharedMetadata {
    pub transaction_version: i64,
    pub sender: String,
    pub entry_function: Option<String>,
    pub market_id: i64,
    pub symbol_bytes: Vec<u8>,
    pub bump_and_emit_time: chrono::NaiveDateTime,
    pub market_nonce: i64,
    pub trigger: StateTrigger,
}

#[derive(Clone, Debug, Eq, PartialEq)]

pub struct FlattenedLastSwap {
    pub last_swap_is_sell: bool,
    pub last_swap_avg_execution_price_q64: BigDecimal,
    pub last_swap_base_volume: i64,
    pub last_swap_quote_volume: i64,
    pub last_swap_nonce: i64,
    pub last_swap_time: chrono::NaiveDateTime,
}
