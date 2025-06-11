#[cfg(test)]
mod json_tests {
    use crate::db::common::models::{
        emojicoin_models::{
            enums::Trigger,
            json_types::EventWithMarket,
            utils::{to_lp_coin_type, to_lp_primary_store_address},
        },
        fungible_asset_models::v2_fungible_asset_balances::{
            get_paired_metadata_address, get_primary_fungible_store_address,
        },
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
            assert_eq!(e.market_metadata.market_id, 328.into());
            assert_eq!(e.state_metadata.trigger, Trigger::ProvideLiquidity);
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
            assert!(!e.starts_in_bonding_curve);
            assert_eq!(e.close_price_q64, 1128118906863219_u64.into());
            assert_eq!(e.periodic_state_metadata.trigger, Trigger::ProvideLiquidity);
        } else {
            panic!("Failed to parse periodic state event");
        }
    }

    #[test]
    fn test_fungible_store_address() {
        // All of these values are copied directly from the writeset changes in the explorer after
        // a real transaction for a liquidity event on localnet.
        let metadata_address = "0xf4c801d6592ecf9c24bfe60b505913576ff8e7b12937adb41b0e37e1b4a11a8d";
        assert_eq!(get_paired_metadata_address("0x58f40ecd236f430c28e30699bf8a7f478c6e4efe9c6d6a2227a86f41e1f0e44::coin_factory::EmojicoinLP"),
      metadata_address);
        let expected_fungible_store_address: &'static str =
            "0xc6e60ab1124a56340889861289be47b1cf6f62f5ce0e4ba6871d8400ef0b712e";
        let owner_address = "0x5048c88ba0ab78f78f4da8d2c3c3a35078315a79e28f8e223c1522761d0eec64";
        assert_eq!(
            get_primary_fungible_store_address(owner_address, metadata_address)
                .expect("Should be able to create a primary store address"),
            expected_fungible_store_address
        );
    }

    #[test]
    fn test_to_lp_coin_type() {
        let market_address = "0x58f40ecd236f430c28e30699bf8a7f478c6e4efe9c6d6a2227a86f41e1f0e44";
        let lp_coin_type = to_lp_coin_type(market_address);
        assert_eq!(lp_coin_type, "0x58f40ecd236f430c28e30699bf8a7f478c6e4efe9c6d6a2227a86f41e1f0e44::coin_factory::EmojicoinLP");
    }

    #[test]
    fn test_to_lp_primary_fungible_store_address() {
        let market_address = "0x58f40ecd236f430c28e30699bf8a7f478c6e4efe9c6d6a2227a86f41e1f0e44";
        let owner_address = "0x5048c88ba0ab78f78f4da8d2c3c3a35078315a79e28f8e223c1522761d0eec64";
        let expected_fungible_store_address =
            "0xc6e60ab1124a56340889861289be47b1cf6f62f5ce0e4ba6871d8400ef0b712e";
        let lp_coin_type = to_lp_coin_type(market_address).as_str();
        assert_eq!(
            to_lp_primary_store_address(lp_coin_type, owner_address),
            expected_fungible_store_address
        );
    }
}
