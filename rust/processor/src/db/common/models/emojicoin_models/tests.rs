// Triggers for state event bumps.
// const TRIGGER_PACKAGE_PUBLICATION: u8 = 0;
// const TRIGGER_MARKET_REGISTRATION: u8 = 1;
// const TRIGGER_SWAP_BUY: u8 = 2;
// const TRIGGER_SWAP_SELL: u8 = 3;
// const TRIGGER_PROVIDE_LIQUIDITY: u8 = 4;
// const TRIGGER_REMOVE_LIQUIDITY: u8 = 5;
// const TRIGGER_CHAT: u8 = 6;

#[cfg(test)]
mod tests {
    use crate::db::common::models::emojicoin_models::{
        enums::StateTrigger, event_types::EventWithMarket,
    };

    #[test]
    fn test_state_event_json() {
        let state_json = r#"
          {
            "clamm_virtual_reserves": {
              "base": "0",
              "quote": "0"
            },
            "cpamm_real_reserves": {
              "base": "38384115850650366",
              "quote": "2341628081606"
            },
            "cumulative_stats": {
              "base_volume": "53352238440663367910",
              "integrator_fees": "143651433",
              "n_chat_messages": "306",
              "n_swaps": "39931",
              "pool_fees_base": "36234321200920750",
              "pool_fees_quote": "1012916465349",
              "quote_volume": "1143635821587662"
            },
            "instantaneous_stats": {
              "fully_diluted_value": "2745230972162",
              "market_cap": "403602890556",
              "total_quote_locked": "2341628081606",
              "total_value_locked": "4683256163212"
            },
            "last_swap": {
              "avg_execution_price_q64": "1128118906863219",
              "base_volume": "1618825508718",
              "is_sell": false,
              "nonce": "40277",
              "quote_volume": "99000000",
              "time": "1722900364541025"
            },
            "lp_coin_supply": "100038578918103",
            "market_metadata": {
              "emoji_bytes": "0xf09f9fa5",
              "market_address": "0x066fb901175394d0883e28262c4c40cb8228e47a36e6a813d5117805c3c26a5c",
              "market_id": "328"
            },
            "state_metadata": {
              "bump_time": "1723246374791035",
              "market_nonce": "40278",
              "trigger": 4
            }
          }
      "#;

        let state_event = serde_json::from_str(state_json)
            .map(|e| Some(EventWithMarket::State(e)))
            .unwrap();
        if let Some(EventWithMarket::State(e)) = state_event {
            assert_eq!(
                e.market_metadata.market_address,
                "0x066fb901175394d0883e28262c4c40cb8228e47a36e6a813d5117805c3c26a5c"
            );
            assert_eq!(e.market_metadata.market_id, 328);
            assert_eq!(e.state_metadata.trigger, StateTrigger::ProvideLiquidity);
            assert_eq!(e.market_metadata.emoji_bytes, vec![240, 159, 159, 165])
        } else {
            panic!("Failed to parse state event");
        }
    }

    #[test]
    fn test_periodic_state_event_json() {
        let periodic_state_json = r#"
          {
            "close_price_q64": "1128118906863219",
            "ends_in_bonding_curve": false,
            "high_price_q64": "1128118906863219",
            "integrator_fees": "1000000",
            "low_price_q64": "1128118906863219",
            "market_metadata": {
              "emoji_bytes": "0xf09f9fa5",
              "market_address": "0x175394d0883e28262c4c40cb8228e47a36e6a813d5117805c3c26a5c",
              "market_id": "328"
            },
            "n_chat_messages": "0",
            "n_swaps": "1",
            "open_price_q64": "1128118906863219",
            "periodic_state_metadata": {
              "emit_market_nonce": "40278",
              "emit_time": "1723246374791035",
              "period": "60000000",
              "start_time": "1722900360000000",
              "trigger": 4
            },
            "pool_fees_base": "4057206788",
            "pool_fees_quote": "0",
            "starts_in_bonding_curve": false,
            "tvl_per_lp_coin_growth_q64": "18447524036544063189",
            "volume_base": "1618825508718",
            "volume_quote": "99000000"
          }
        "#;

        let periodic_state_event = serde_json::from_str(periodic_state_json)
            .map(|e| Some(EventWithMarket::PeriodicState(e)))
            .unwrap();
        if let Some(EventWithMarket::PeriodicState(e)) = periodic_state_event {
            assert_eq!(
                e.market_metadata.market_address,
                "0x00000000175394d0883e28262c4c40cb8228e47a36e6a813d5117805c3c26a5c"
            );
            assert_eq!(e.starts_in_bonding_curve, false);
            assert_eq!(e.close_price_q64, (1128118906863219 as i64).into());
            assert_eq!(
                e.periodic_state_metadata.trigger,
                StateTrigger::ProvideLiquidity
            );
        } else {
            panic!("Failed to parse periodic state event");
        }
    }

    #[test]
    fn test_liquidity_event_json() {
        let liquidity_json = r#"
          {
            "base_amount": "1639206334780",
            "liquidity_provided": true,
            "lp_coin_amount": "4272180527",
            "market_id": "328",
            "market_nonce": "40278",
            "pro_rata_base_donation_claim_amount": "0",
            "pro_rata_quote_donation_claim_amount": "0",
            "provider": "0x000006d68589500aa64d92f4f0e14d2f9d8075d003b8adf1e90ae6037f100000",
            "quote_amount": "100000000",
            "time": "1723246374791035"
          }
        "#;

        let liquidity_event = serde_json::from_str(liquidity_json)
            .map(|e| Some(EventWithMarket::Liquidity(e)))
            .unwrap();
        if let Some(EventWithMarket::Liquidity(e)) = liquidity_event {
            assert_eq!(e.market_nonce, 40278);
            assert_eq!(e.liquidity_provided, true);
            assert_eq!(e.lp_coin_amount, 4272180527);
            assert_eq!(e.base_amount, 1639206334780);
            assert_eq!(e.quote_amount, 100000000);
            assert_eq!(e.pro_rata_base_donation_claim_amount, 0);
            assert_eq!(e.pro_rata_quote_donation_claim_amount, 0);
            assert_eq!(e.market_id, 328);
            assert_eq!(e.time, 1723246374791035);
            assert_eq!(
                e.provider,
                "0x000006d68589500aa64d92f4f0e14d2f9d8075d003b8adf1e90ae6037f100000"
            );
        } else {
            panic!("Failed to parse periodic state event");
        }
    }

    #[test]
    fn test_swap_json() {
        let swap_json = r#"
          {
            "avg_execution_price_q64": "150622935860149",
            "base_volume": "12124499186451",
            "input_amount": "100000000",
            "integrator": "0x76044a237dcc3f71af75fb314f016e8032633587f7d70df4e70777f2b0221e75",
            "integrator_fee": "1000000",
            "integrator_fee_rate_bps": 100,
            "is_sell": false,
            "market_id": "3523452345",
            "market_nonce": "2",
            "net_proceeds": "12124499186451",
            "pool_fee": "0",
            "quote_volume": "99000000",
            "results_in_state_transition": false,
            "starts_in_bonding_curve": true,
            "swapper": "0xbad225596d685895aa64d92f4f0e14d2f9d8075d3b8adf1e90ae6037f1fcbabe",
            "time": "1723253663706846"
          }
        "#;

        let swap_event = serde_json::from_str(swap_json)
            .map(|e| Some(EventWithMarket::Swap(e)))
            .unwrap();
        if let Some(EventWithMarket::Swap(e)) = swap_event {
            assert_eq!(e.avg_execution_price_q64, (150622935860149 as i64).into());
            assert_eq!(e.base_volume, 12124499186451);
            assert_eq!(e.integrator_fee, 1000000);
            assert_eq!(e.input_amount, 100000000);
            assert_eq!(e.is_sell, false);
            assert_eq!(e.integrator_fee_rate_bps, 100);
            assert_eq!(e.results_in_state_transition, false);
            assert_eq!(e.starts_in_bonding_curve, true);
            assert_eq!(e.market_id, 3523452345);
            assert_eq!(e.market_nonce, 2);
            assert_eq!(e.time, 1723253663706846);
        } else {
            panic!("Failed to parse periodic state event");
        }
    }

    #[test]
    fn test_market_registration_json() {
        let market_registration_json = r#"
          {
            "integrator": "d00db145c047cd3619ecba69e45b4ad77f43737d309d8113d6c1c35f7a8dd00d",
            "integrator_fee": "100000000",
            "market_metadata": {
              "emoji_bytes": "0xf09f988df09f989c",
              "market_address": "0xd3cbef2c5d489228ae5304f39d94bd794847b5c0e9d7968ab0391999926d3679",
              "market_id": "2304"
            },
            "registrant": "0xbad225596d685895aa64d92f4f0e14d2f9d8075d3b8adf1e90ae6037f1fcbabe",
            "time": "1723253654764692"
          }
        "#;

        let market_registration_event = serde_json::from_str(market_registration_json)
            .map(|e| Some(EventWithMarket::MarketRegistration(e)))
            .unwrap();
        if let Some(EventWithMarket::MarketRegistration(e)) = market_registration_event {
            assert_eq!(
                e.integrator,
                "0xd00db145c047cd3619ecba69e45b4ad77f43737d309d8113d6c1c35f7a8dd00d"
            );
            assert_eq!(e.integrator_fee, 100000000);
            assert_eq!(
                e.market_metadata.emoji_bytes,
                [240, 159, 152, 141, 240, 159, 152, 156]
            );
            println!("{:?}", e.market_metadata.emoji_bytes);
            assert_eq!(
                e.market_metadata.market_address,
                "0xd3cbef2c5d489228ae5304f39d94bd794847b5c0e9d7968ab0391999926d3679"
            );
            assert_eq!(e.market_metadata.market_id, 2304);
            assert_eq!(
                e.registrant,
                "0xbad225596d685895aa64d92f4f0e14d2f9d8075d3b8adf1e90ae6037f1fcbabe"
            );
            assert_eq!(e.time, 1723253654764692);
        } else {
            panic!("Failed to parse periodic state event");
        }
    }
}
