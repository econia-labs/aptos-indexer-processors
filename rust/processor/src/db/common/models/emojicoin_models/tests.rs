#[cfg(test)]
mod json_tests {
    use crate::db::common::models::{
        default_models::transactions::Transaction as JSONTransaction,
        emojicoin_models::{
            enums::{EmojicoinTypeTag, Trigger},
            json_types::{EventWithMarket, GlobalStateEvent},
            models::prelude::UserLiquidityPoolsModel,
        },
    };
    use aptos_protos::transaction::v1::{
        transaction::TxnData, Transaction, TransactionInfo, UserTransaction,
    };
    use tracing::{debug, field::debug};

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
    fn test_liquidity_event_json() {
        let liquidity_json = r#"
          {
            "base_amount": "1639206334780",
            "liquidity_provided": true,
            "lp_coin_amount": "4272180527",
            "market_id": "328",
            "market_nonce": "40278",
            "base_donation_claim_amount": "0",
            "quote_donation_claim_amount": "0",
            "provider": "0x000006d68589500aa64d92f4f0e14d2f9d8075d003b8adf1e90ae6037f100000",
            "quote_amount": "100000000",
            "time": "1723246374791035",
            "event_index": "1"
          }
        "#;

        let liquidity_event = serde_json::from_str(liquidity_json)
            .map(|e| Some(EventWithMarket::Liquidity(e)))
            .unwrap();
        if let Some(EventWithMarket::Liquidity(e)) = liquidity_event {
            assert_eq!(e.market_nonce, 40278.into());
            assert!(e.liquidity_provided);
            assert_eq!(e.lp_coin_amount, 4272180527_u64.into());
            assert_eq!(e.base_amount, 1639206334780_u64.into());
            assert_eq!(e.quote_amount, 100000000.into());
            assert_eq!(e.base_donation_claim_amount, 0.into());
            assert_eq!(e.quote_donation_claim_amount, 0.into());
            assert_eq!(e.market_id, 328.into());
            assert_eq!(e.time, 1723246374791035_u64.into());
            assert_eq!(
                e.provider,
                "0x000006d68589500aa64d92f4f0e14d2f9d8075d003b8adf1e90ae6037f100000"
            );
            assert_eq!(e.event_index, 1);
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
            "balance_as_fraction_of_circulating_supply_before_q64": "0",
            "balance_as_fraction_of_circulating_supply_after_q64": "1",
            "swapper": "0xbad225596d685895aa64d92f4f0e14d2f9d8075d3b8adf1e90ae6037f1fcbabe",
            "time": "1723253663706846",
            "event_index": "1"
          }
        "#;

        let swap_event = serde_json::from_str(swap_json)
            .map(|e| Some(EventWithMarket::Swap(e)))
            .unwrap();
        if let Some(EventWithMarket::Swap(e)) = swap_event {
            assert_eq!(e.avg_execution_price_q64, 150622935860149_u64.into());
            assert_eq!(e.base_volume, 12124499186451_u64.into());
            assert_eq!(e.integrator_fee, 1000000.into());
            assert_eq!(e.input_amount, 100000000.into());
            assert!(!e.is_sell);
            assert_eq!(e.integrator_fee_rate_bps, 100);
            assert!(!e.results_in_state_transition);
            assert!(e.starts_in_bonding_curve);
            assert_eq!(e.market_id, 3523452345_u64.into());
            assert_eq!(e.market_nonce, 2.into());
            assert_eq!(e.time, 1723253663706846_u64.into());
            assert_eq!(
                e.balance_as_fraction_of_circulating_supply_before_q64,
                0.into()
            );
            assert_eq!(
                e.balance_as_fraction_of_circulating_supply_after_q64,
                1.into()
            );
            assert_eq!(e.event_index, 1);
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
            assert_eq!(e.integrator_fee, 100000000.into());
            assert_eq!(e.market_metadata.emoji_bytes, [
                240, 159, 152, 141, 240, 159, 152, 156
            ]);
            assert_eq!(
                e.market_metadata.market_address,
                "0xd3cbef2c5d489228ae5304f39d94bd794847b5c0e9d7968ab0391999926d3679"
            );
            assert_eq!(e.market_metadata.market_id, 2304.into());
            assert_eq!(
                e.registrant,
                "0xbad225596d685895aa64d92f4f0e14d2f9d8075d3b8adf1e90ae6037f1fcbabe"
            );
            assert_eq!(e.time, 1723253654764692_u64.into());
        } else {
            panic!("Failed to parse periodic state event");
        }
    }

    #[test]
    fn test_global_state_json() {
        let global_state_json = r#"
          {
            "cumulative_chat_messages": {
              "value": "16891"
            },
            "cumulative_integrator_fees": {
              "value": "249444000000"
            },
            "cumulative_quote_volume": {
              "value": "200576291031"
            },
            "cumulative_swaps": {
              "value": "14209"
            },
            "emit_time": "1723350357240102",
            "fully_diluted_value": {
              "value": "912838434139348"
            },
            "market_cap": {
              "value": "213923864245"
            },
            "registry_nonce": {
              "value": "33586"
            },
            "total_quote_locked": {
              "value": "165704422193"
            },
            "total_value_locked": {
              "value": "5075928984264"
            },
            "trigger": 1
          }
        "#;
        match serde_json::from_str::<GlobalStateEvent>(global_state_json) {
            Ok(global_state_event) => {
                assert_eq!(global_state_event.cumulative_chat_messages, 16891.into());
                assert_eq!(
                    global_state_event.cumulative_integrator_fees,
                    249444000000_u64.into()
                );
                assert_eq!(
                    global_state_event.cumulative_quote_volume,
                    200576291031_u64.into()
                );
                assert_eq!(global_state_event.cumulative_swaps, 14209.into());
                assert_eq!(global_state_event.emit_time, 1723350357240102_u64.into());
                assert_eq!(
                    global_state_event.fully_diluted_value,
                    912838434139348_u64.into()
                );
                assert_eq!(global_state_event.market_cap, 213923864245_u64.into());
                assert_eq!(global_state_event.registry_nonce, 33586.into());
                assert_eq!(
                    global_state_event.total_quote_locked,
                    165704422193_u64.into()
                );
                assert_eq!(
                    global_state_event.total_value_locked,
                    5075928984264_u64.into()
                );
                assert_eq!(global_state_event.trigger, Trigger::MarketRegistration);
            },
            Err(e) => {
                panic!("Failed to parse global state event: {:?}", e);
            },
        }
    }

    #[test]
    fn test_pools_txn_json() {
        let pool_txn_json: &'static str = r#"
        {
          "version": "448",
          "hash": "0xcf3cb8933497f20d066e5ad431da3fd2894c727c27418c593b530bb33d677991",
          "state_change_hash": "0xb594565db7bc4bd8f31f01bbe1d4b3ea72b32d43d8d161662c00545d283e8830",
          "event_root_hash": "0x06e041179e212aa257df16ba18265d19779c65e6e2e18351bfd7e3070b8fd59a",
          "state_checkpoint_hash": null,
          "gas_used": "46",
          "success": true,
          "vm_status": "Executed successfully",
          "accumulator_root_hash": "0xe59aaa8e869759ae912f09fe3d25f92e2c1d398bd140384f6eaa218759c4bbb1",
          "changes": [
            {
              "address": "0xa",
              "state_key_hash": "0x1db5441d8fa4229c5844f73fd66da4ad8176cb8793d8b3a7f6ca858722030043",
              "data": {
                "type": "0x1::coin::PairedCoinType",
                "data": {
                  "type": {
                    "account_address": "0x1",
                    "module_name": "0x6170746f735f636f696e",
                    "struct_name": "0x4170746f73436f696e"
                  }
                }
              },
              "type": "write_resource"
            },
            {
              "address": "0xa",
              "state_key_hash": "0x1db5441d8fa4229c5844f73fd66da4ad8176cb8793d8b3a7f6ca858722030043",
              "data": {
                "type": "0x1::coin::PairedFungibleAssetRefs",
                "data": {
                  "burn_ref_opt": {
                    "vec": []
                  },
                  "mint_ref_opt": {
                    "vec": [
                      {
                        "metadata": {
                          "inner": "0xa"
                        }
                      }
                    ]
                  },
                  "transfer_ref_opt": {
                    "vec": [
                      {
                        "metadata": {
                          "inner": "0xa"
                        }
                      }
                    ]
                  }
                }
              },
              "type": "write_resource"
            },
            {
              "address": "0xa",
              "state_key_hash": "0x1db5441d8fa4229c5844f73fd66da4ad8176cb8793d8b3a7f6ca858722030043",
              "data": {
                "type": "0x1::fungible_asset::ConcurrentSupply",
                "data": {
                  "current": {
                    "max_value": "340282366920938463463374607431768211455",
                    "value": "18467744073501967495"
                  }
                }
              },
              "type": "write_resource"
            },
            {
              "address": "0xa",
              "state_key_hash": "0x1db5441d8fa4229c5844f73fd66da4ad8176cb8793d8b3a7f6ca858722030043",
              "data": {
                "type": "0x1::fungible_asset::Metadata",
                "data": {
                  "decimals": 8,
                  "icon_uri": "",
                  "name": "Aptos Coin",
                  "project_uri": "",
                  "symbol": "APT"
                }
              },
              "type": "write_resource"
            },
            {
              "address": "0xa",
              "state_key_hash": "0x1db5441d8fa4229c5844f73fd66da4ad8176cb8793d8b3a7f6ca858722030043",
              "data": {
                "type": "0x1::object::ObjectCore",
                "data": {
                  "allow_ungated_transfer": true,
                  "guid_creation_num": "1125899906842625",
                  "owner": "0x1",
                  "transfer_events": {
                    "counter": "0",
                    "guid": {
                      "id": {
                        "addr": "0xa",
                        "creation_num": "1125899906842624"
                      }
                    }
                  }
                }
              },
              "type": "write_resource"
            },
            {
              "address": "0xa",
              "state_key_hash": "0x1db5441d8fa4229c5844f73fd66da4ad8176cb8793d8b3a7f6ca858722030043",
              "data": {
                "type": "0x1::primary_fungible_store::DeriveRefPod",
                "data": {
                  "metadata_derive_ref": {
                    "self": "0xa"
                  }
                }
              },
              "type": "write_resource"
            },
            {
              "address": "0xd14c168e2b66a2b7955575b2ebae2c452adf45d34290919db9856753ead4443",
              "state_key_hash": "0x8a6d19a087afb2a3233840c6e70da2f1d72e6f368b550fd9e6fc51decc3c612c",
              "data": {
                "type": "0xf000d910b99722d201c6cf88eb7d1112b43475b9765b118f289b5d65d919000d::emojicoin_dot_fun::Registry",
                "data": {
                  "coin_symbol_emojis": {
                    "handle": "0x461507797f7720a83b29ffe65577cd6e3d70eb430c6826e4602a5b290e946f2"
                  },
                  "extend_ref": {
                    "self": "0xd14c168e2b66a2b7955575b2ebae2c452adf45d34290919db9856753ead4443"
                  },
                  "global_stats": {
                    "cumulative_chat_messages": {
                      "max_value": "18446744073709551615",
                      "value": "0"
                    },
                    "cumulative_integrator_fees": {
                      "max_value": "340282366920938463463374607431768211455",
                      "value": "5755500000"
                    },
                    "cumulative_quote_volume": {
                      "max_value": "340282366920938463463374607431768211455",
                      "value": "1105544500000"
                    },
                    "cumulative_swaps": {
                      "max_value": "18446744073709551615",
                      "value": "1"
                    },
                    "fully_diluted_value": {
                      "max_value": "340282366920938463463374607431768211455",
                      "value": "53725032401736"
                    },
                    "market_cap": {
                      "max_value": "340282366920938463463374607431768211455",
                      "value": "52545818513982"
                    },
                    "total_quote_locked": {
                      "max_value": "340282366920938463463374607431768211455",
                      "value": "1105744500000"
                    },
                    "total_value_locked": {
                      "max_value": "340282366920938463463374607431768211455",
                      "value": "2211489000000"
                    }
                  },
                  "markets_by_emoji_bytes": {
                    "buckets": {
                      "inner": {
                        "handle": "0x1f5fa701721ef3be77f5e5377adfffdc09882cd8825d7e6f7199581cc85c68c5"
                      },
                      "length": "2"
                    },
                    "level": 1,
                    "num_buckets": "2",
                    "size": "3",
                    "split_load_threshold": 75,
                    "target_bucket_size": "22"
                  },
                  "markets_by_market_id": {
                    "buckets": {
                      "inner": {
                        "handle": "0xc4b884b6449d6345160f67c760f34fcd6ebbc09d229bc6cb2d5f29875b031029"
                      },
                      "length": "2"
                    },
                    "level": 1,
                    "num_buckets": "2",
                    "size": "3",
                    "split_load_threshold": 75,
                    "target_bucket_size": "21"
                  },
                  "registry_address": "0xd14c168e2b66a2b7955575b2ebae2c452adf45d34290919db9856753ead4443",
                  "sequence_info": {
                    "last_bump_time": "1749513600000000",
                    "nonce": {
                      "max_value": "18446744073709551615",
                      "value": "7"
                    }
                  },
                  "supplemental_chat_emojis": {
                    "handle": "0x9f0a0d97900ac008b5c21d26dc54a4629e3eee7e525c6bbba9b1ced307bdcdce"
                  }
                }
              },
              "type": "write_resource"
            },
            {
              "address": "0x1356651cd783ff9af7ad9840f8b657cb5f8b13efffad05f53001256946db1d18",
              "state_key_hash": "0xc0c8e35ce074f046dc66e4d5d3cb32cb14402160f44a68daba00de6ffd689a74",
              "data": {
                "type": "0x1::coin::CoinInfo<0x1356651cd783ff9af7ad9840f8b657cb5f8b13efffad05f53001256946db1d18::coin_factory::Emojicoin>",
                "data": {
                  "decimals": 8,
                  "name": "🎃 emojicoin",
                  "supply": {
                    "vec": [
                      {
                        "aggregator": {
                          "vec": []
                        },
                        "integer": {
                          "vec": [
                            {
                              "limit": "340282366920938463463374607431768211455",
                              "value": "0"
                            }
                          ]
                        }
                      }
                    ]
                  },
                  "symbol": "🎃"
                }
              },
              "type": "write_resource"
            },
            {
              "address": "0x1356651cd783ff9af7ad9840f8b657cb5f8b13efffad05f53001256946db1d18",
              "state_key_hash": "0x760bd7bc23f21a7a381bd48507e0cfc80f97db90134139eb84569a98e29710fc",
              "data": {
                "type": "0x1::coin::CoinInfo<0x1356651cd783ff9af7ad9840f8b657cb5f8b13efffad05f53001256946db1d18::coin_factory::EmojicoinLP>",
                "data": {
                  "decimals": 8,
                  "name": "🎃 emojicoin LP",
                  "supply": {
                    "vec": [
                      {
                        "aggregator": {
                          "vec": []
                        },
                        "integer": {
                          "vec": [
                            {
                              "limit": "340282366920938463463374607431768211455",
                              "value": "0"
                            }
                          ]
                        }
                      }
                    ]
                  },
                  "symbol": "LP-3"
                }
              },
              "type": "write_resource"
            },
            {
              "address": "0x1356651cd783ff9af7ad9840f8b657cb5f8b13efffad05f53001256946db1d18",
              "state_key_hash": "0x122b119bd8cafd2c58944a2c94f2dce6f45fa029ca464e420a26fdcddd0b347f",
              "data": {
                "type": "0xf000d910b99722d201c6cf88eb7d1112b43475b9765b118f289b5d65d919000d::emojicoin_dot_fun::Market",
                "data": {
                  "clamm_virtual_reserves": {
                    "base": "0",
                    "quote": "0"
                  },
                  "cpamm_real_reserves": {
                    "base": "92743807830970",
                    "quote": "1105744500000"
                  },
                  "cumulative_stats": {
                    "base_volume": "4407272967076404",
                    "integrator_fees": "5655500000",
                    "n_chat_messages": "0",
                    "n_swaps": "1",
                    "pool_fees_base": "2273867085404",
                    "pool_fees_quote": "0",
                    "quote_volume": "1105544500000"
                  },
                  "extend_ref": {
                    "self": "0x1356651cd783ff9af7ad9840f8b657cb5f8b13efffad05f53001256946db1d18"
                  },
                  "last_swap": {
                    "avg_execution_price_q64": "4627282359396380",
                    "base_volume": "4407272967076404",
                    "is_sell": false,
                    "nonce": "2",
                    "quote_volume": "1105544500000",
                    "time": "1749589716980950"
                  },
                  "lp_coin_supply": "10001809063316",
                  "metadata": {
                    "emoji_bytes": "0xf09f8e83",
                    "market_address": "0x1356651cd783ff9af7ad9840f8b657cb5f8b13efffad05f53001256946db1d18",
                    "market_id": "3"
                  },
                  "periodic_state_trackers": [
                    {
                      "close_price_q64": "0",
                      "ends_in_bonding_curve": false,
                      "high_price_q64": "0",
                      "integrator_fees": "0",
                      "low_price_q64": "0",
                      "n_chat_messages": "0",
                      "n_swaps": "0",
                      "open_price_q64": "0",
                      "period": "60000000",
                      "pool_fees_base": "0",
                      "pool_fees_quote": "0",
                      "start_time": "1749590100000000",
                      "starts_in_bonding_curve": false,
                      "tvl_to_lp_coin_ratio_end": {
                        "lp_coins": "10001809063316",
                        "tvl": "2211489000000"
                      },
                      "tvl_to_lp_coin_ratio_start": {
                        "lp_coins": "10000904531658",
                        "tvl": "2211289000000"
                      },
                      "volume_base": "0",
                      "volume_quote": "0"
                    },
                    {
                      "close_price_q64": "0",
                      "ends_in_bonding_curve": false,
                      "high_price_q64": "0",
                      "integrator_fees": "0",
                      "low_price_q64": "0",
                      "n_chat_messages": "0",
                      "n_swaps": "0",
                      "open_price_q64": "0",
                      "period": "300000000",
                      "pool_fees_base": "0",
                      "pool_fees_quote": "0",
                      "start_time": "1749590100000000",
                      "starts_in_bonding_curve": false,
                      "tvl_to_lp_coin_ratio_end": {
                        "lp_coins": "10001809063316",
                        "tvl": "2211489000000"
                      },
                      "tvl_to_lp_coin_ratio_start": {
                        "lp_coins": "10000904531658",
                        "tvl": "2211289000000"
                      },
                      "volume_base": "0",
                      "volume_quote": "0"
                    },
                    {
                      "close_price_q64": "0",
                      "ends_in_bonding_curve": false,
                      "high_price_q64": "0",
                      "integrator_fees": "0",
                      "low_price_q64": "0",
                      "n_chat_messages": "0",
                      "n_swaps": "0",
                      "open_price_q64": "0",
                      "period": "900000000",
                      "pool_fees_base": "0",
                      "pool_fees_quote": "0",
                      "start_time": "1749590100000000",
                      "starts_in_bonding_curve": false,
                      "tvl_to_lp_coin_ratio_end": {
                        "lp_coins": "10001809063316",
                        "tvl": "2211489000000"
                      },
                      "tvl_to_lp_coin_ratio_start": {
                        "lp_coins": "10000904531658",
                        "tvl": "2211289000000"
                      },
                      "volume_base": "0",
                      "volume_quote": "0"
                    },
                    {
                      "close_price_q64": "4627282359396380",
                      "ends_in_bonding_curve": false,
                      "high_price_q64": "4627282359396380",
                      "integrator_fees": "5655500000",
                      "low_price_q64": "4627282359396380",
                      "n_chat_messages": "0",
                      "n_swaps": "1",
                      "open_price_q64": "4627282359396380",
                      "period": "1800000000",
                      "pool_fees_base": "2273867085404",
                      "pool_fees_quote": "0",
                      "start_time": "1749589200000000",
                      "starts_in_bonding_curve": true,
                      "tvl_to_lp_coin_ratio_end": {
                        "lp_coins": "10001809063316",
                        "tvl": "2211489000000"
                      },
                      "tvl_to_lp_coin_ratio_start": {
                        "lp_coins": "0",
                        "tvl": "0"
                      },
                      "volume_base": "4407272967076404",
                      "volume_quote": "1105544500000"
                    },
                    {
                      "close_price_q64": "4627282359396380",
                      "ends_in_bonding_curve": false,
                      "high_price_q64": "4627282359396380",
                      "integrator_fees": "5655500000",
                      "low_price_q64": "4627282359396380",
                      "n_chat_messages": "0",
                      "n_swaps": "1",
                      "open_price_q64": "4627282359396380",
                      "period": "3600000000",
                      "pool_fees_base": "2273867085404",
                      "pool_fees_quote": "0",
                      "start_time": "1749589200000000",
                      "starts_in_bonding_curve": true,
                      "tvl_to_lp_coin_ratio_end": {
                        "lp_coins": "10001809063316",
                        "tvl": "2211489000000"
                      },
                      "tvl_to_lp_coin_ratio_start": {
                        "lp_coins": "0",
                        "tvl": "0"
                      },
                      "volume_base": "4407272967076404",
                      "volume_quote": "1105544500000"
                    },
                    {
                      "close_price_q64": "4627282359396380",
                      "ends_in_bonding_curve": false,
                      "high_price_q64": "4627282359396380",
                      "integrator_fees": "5655500000",
                      "low_price_q64": "4627282359396380",
                      "n_chat_messages": "0",
                      "n_swaps": "1",
                      "open_price_q64": "4627282359396380",
                      "period": "14400000000",
                      "pool_fees_base": "2273867085404",
                      "pool_fees_quote": "0",
                      "start_time": "1749585600000000",
                      "starts_in_bonding_curve": true,
                      "tvl_to_lp_coin_ratio_end": {
                        "lp_coins": "10001809063316",
                        "tvl": "2211489000000"
                      },
                      "tvl_to_lp_coin_ratio_start": {
                        "lp_coins": "0",
                        "tvl": "0"
                      },
                      "volume_base": "4407272967076404",
                      "volume_quote": "1105544500000"
                    },
                    {
                      "close_price_q64": "4627282359396380",
                      "ends_in_bonding_curve": false,
                      "high_price_q64": "4627282359396380",
                      "integrator_fees": "5655500000",
                      "low_price_q64": "4627282359396380",
                      "n_chat_messages": "0",
                      "n_swaps": "1",
                      "open_price_q64": "4627282359396380",
                      "period": "86400000000",
                      "pool_fees_base": "2273867085404",
                      "pool_fees_quote": "0",
                      "start_time": "1749513600000000",
                      "starts_in_bonding_curve": true,
                      "tvl_to_lp_coin_ratio_end": {
                        "lp_coins": "10001809063316",
                        "tvl": "2211489000000"
                      },
                      "tvl_to_lp_coin_ratio_start": {
                        "lp_coins": "0",
                        "tvl": "0"
                      },
                      "volume_base": "4407272967076404",
                      "volume_quote": "1105544500000"
                    }
                  ],
                  "sequence_info": {
                    "last_bump_time": "1749590115188762",
                    "nonce": "4"
                  }
                }
              },
              "type": "write_resource"
            },
            {
              "address": "0x282415b709466709d27126d3c19f4b00993c4b371ecd595c80ecf3ef86d2c96d",
              "state_key_hash": "0x4f8df6bc733b248b34ab4eeb40de450c0de7519bf11874e3b7855867e6c0d12a",
              "data": {
                "type": "0x1::fungible_asset::FungibleStore",
                "data": {
                  "balance": "4407256192169030",
                  "frozen": false,
                  "metadata": {
                    "inner": "0xb002bbb96c8b4a1ea4d79d4b59a6c608ae4873668c3203b4142eb8e454dade72"
                  }
                }
              },
              "type": "write_resource"
            },
            {
              "address": "0x282415b709466709d27126d3c19f4b00993c4b371ecd595c80ecf3ef86d2c96d",
              "state_key_hash": "0x4f8df6bc733b248b34ab4eeb40de450c0de7519bf11874e3b7855867e6c0d12a",
              "data": {
                "type": "0x1::object::ObjectCore",
                "data": {
                  "allow_ungated_transfer": false,
                  "guid_creation_num": "1125899906842625",
                  "owner": "0x5048c88ba0ab78f78f4da8d2c3c3a35078315a79e28f8e223c1522761d0eec64",
                  "transfer_events": {
                    "counter": "0",
                    "guid": {
                      "id": {
                        "addr": "0x282415b709466709d27126d3c19f4b00993c4b371ecd595c80ecf3ef86d2c96d",
                        "creation_num": "1125899906842624"
                      }
                    }
                  }
                }
              },
              "type": "write_resource"
            },
            {
              "address": "0x41535865d434b1bbe46f3821f5fa2e89e46547b667b38ee46bf23986e287634a",
              "state_key_hash": "0xa49506bd9342c17ca27e6d189b805f660569b77fa2a96570354d63275ebcfaab",
              "data": {
                "type": "0x1::coin::PairedCoinType",
                "data": {
                  "type": {
                    "account_address": "0x1356651cd783ff9af7ad9840f8b657cb5f8b13efffad05f53001256946db1d18",
                    "module_name": "0x636f696e5f666163746f7279",
                    "struct_name": "0x456d6f6a69636f696e4c50"
                  }
                }
              },
              "type": "write_resource"
            },
            {
              "address": "0x41535865d434b1bbe46f3821f5fa2e89e46547b667b38ee46bf23986e287634a",
              "state_key_hash": "0xa49506bd9342c17ca27e6d189b805f660569b77fa2a96570354d63275ebcfaab",
              "data": {
                "type": "0x1::coin::PairedFungibleAssetRefs",
                "data": {
                  "burn_ref_opt": {
                    "vec": [
                      {
                        "metadata": {
                          "inner": "0x41535865d434b1bbe46f3821f5fa2e89e46547b667b38ee46bf23986e287634a"
                        }
                      }
                    ]
                  },
                  "mint_ref_opt": {
                    "vec": [
                      {
                        "metadata": {
                          "inner": "0x41535865d434b1bbe46f3821f5fa2e89e46547b667b38ee46bf23986e287634a"
                        }
                      }
                    ]
                  },
                  "transfer_ref_opt": {
                    "vec": [
                      {
                        "metadata": {
                          "inner": "0x41535865d434b1bbe46f3821f5fa2e89e46547b667b38ee46bf23986e287634a"
                        }
                      }
                    ]
                  }
                }
              },
              "type": "write_resource"
            },
            {
              "address": "0x41535865d434b1bbe46f3821f5fa2e89e46547b667b38ee46bf23986e287634a",
              "state_key_hash": "0xa49506bd9342c17ca27e6d189b805f660569b77fa2a96570354d63275ebcfaab",
              "data": {
                "type": "0x1::fungible_asset::ConcurrentSupply",
                "data": {
                  "current": {
                    "max_value": "340282366920938463463374607431768211455",
                    "value": "10001809063316"
                  }
                }
              },
              "type": "write_resource"
            },
            {
              "address": "0x41535865d434b1bbe46f3821f5fa2e89e46547b667b38ee46bf23986e287634a",
              "state_key_hash": "0xa49506bd9342c17ca27e6d189b805f660569b77fa2a96570354d63275ebcfaab",
              "data": {
                "type": "0x1::fungible_asset::Metadata",
                "data": {
                  "decimals": 8,
                  "icon_uri": "",
                  "name": "🎃 emojicoin LP",
                  "project_uri": "",
                  "symbol": "LP-3"
                }
              },
              "type": "write_resource"
            },
            {
              "address": "0x41535865d434b1bbe46f3821f5fa2e89e46547b667b38ee46bf23986e287634a",
              "state_key_hash": "0xa49506bd9342c17ca27e6d189b805f660569b77fa2a96570354d63275ebcfaab",
              "data": {
                "type": "0x1::object::ObjectCore",
                "data": {
                  "allow_ungated_transfer": true,
                  "guid_creation_num": "1125899906842625",
                  "owner": "0xa",
                  "transfer_events": {
                    "counter": "0",
                    "guid": {
                      "id": {
                        "addr": "0x41535865d434b1bbe46f3821f5fa2e89e46547b667b38ee46bf23986e287634a",
                        "creation_num": "1125899906842624"
                      }
                    }
                  }
                }
              },
              "type": "write_resource"
            },
            {
              "address": "0x41535865d434b1bbe46f3821f5fa2e89e46547b667b38ee46bf23986e287634a",
              "state_key_hash": "0xa49506bd9342c17ca27e6d189b805f660569b77fa2a96570354d63275ebcfaab",
              "data": {
                "type": "0x1::primary_fungible_store::DeriveRefPod",
                "data": {
                  "metadata_derive_ref": {
                    "self": "0x41535865d434b1bbe46f3821f5fa2e89e46547b667b38ee46bf23986e287634a"
                  }
                }
              },
              "type": "write_resource"
            },
            {
              "address": "0x5048c88ba0ab78f78f4da8d2c3c3a35078315a79e28f8e223c1522761d0eec64",
              "state_key_hash": "0x7b3d46a2e8eee09aeb455e2c41c0ae0ac7e8a952c668e5e3f74bb379d9742382",
              "data": {
                "type": "0x1::account::Account",
                "data": {
                  "authentication_key": "0x5048c88ba0ab78f78f4da8d2c3c3a35078315a79e28f8e223c1522761d0eec64",
                  "coin_register_events": {
                    "counter": "0",
                    "guid": {
                      "id": {
                        "addr": "0x5048c88ba0ab78f78f4da8d2c3c3a35078315a79e28f8e223c1522761d0eec64",
                        "creation_num": "0"
                      }
                    }
                  },
                  "guid_creation_num": "2",
                  "key_rotation_events": {
                    "counter": "0",
                    "guid": {
                      "id": {
                        "addr": "0x5048c88ba0ab78f78f4da8d2c3c3a35078315a79e28f8e223c1522761d0eec64",
                        "creation_num": "1"
                      }
                    }
                  },
                  "rotation_capability_offer": {
                    "for": {
                      "vec": []
                    }
                  },
                  "sequence_number": "4",
                  "signer_capability_offer": {
                    "for": {
                      "vec": []
                    }
                  }
                }
              },
              "type": "write_resource"
            },
            {
              "address": "0xb002bbb96c8b4a1ea4d79d4b59a6c608ae4873668c3203b4142eb8e454dade72",
              "state_key_hash": "0xceb81aeb14d25fb2ca1c0d3711ac769e9f6d1d429648d33997edf2aa386b0ebf",
              "data": {
                "type": "0x1::coin::PairedCoinType",
                "data": {
                  "type": {
                    "account_address": "0x1356651cd783ff9af7ad9840f8b657cb5f8b13efffad05f53001256946db1d18",
                    "module_name": "0x636f696e5f666163746f7279",
                    "struct_name": "0x456d6f6a69636f696e"
                  }
                }
              },
              "type": "write_resource"
            },
            {
              "address": "0xb002bbb96c8b4a1ea4d79d4b59a6c608ae4873668c3203b4142eb8e454dade72",
              "state_key_hash": "0xceb81aeb14d25fb2ca1c0d3711ac769e9f6d1d429648d33997edf2aa386b0ebf",
              "data": {
                "type": "0x1::coin::PairedFungibleAssetRefs",
                "data": {
                  "burn_ref_opt": {
                    "vec": [
                      {
                        "metadata": {
                          "inner": "0xb002bbb96c8b4a1ea4d79d4b59a6c608ae4873668c3203b4142eb8e454dade72"
                        }
                      }
                    ]
                  },
                  "mint_ref_opt": {
                    "vec": [
                      {
                        "metadata": {
                          "inner": "0xb002bbb96c8b4a1ea4d79d4b59a6c608ae4873668c3203b4142eb8e454dade72"
                        }
                      }
                    ]
                  },
                  "transfer_ref_opt": {
                    "vec": [
                      {
                        "metadata": {
                          "inner": "0xb002bbb96c8b4a1ea4d79d4b59a6c608ae4873668c3203b4142eb8e454dade72"
                        }
                      }
                    ]
                  }
                }
              },
              "type": "write_resource"
            },
            {
              "address": "0xb002bbb96c8b4a1ea4d79d4b59a6c608ae4873668c3203b4142eb8e454dade72",
              "state_key_hash": "0xceb81aeb14d25fb2ca1c0d3711ac769e9f6d1d429648d33997edf2aa386b0ebf",
              "data": {
                "type": "0x1::fungible_asset::ConcurrentSupply",
                "data": {
                  "current": {
                    "max_value": "340282366920938463463374607431768211455",
                    "value": "4500000000000000"
                  }
                }
              },
              "type": "write_resource"
            },
            {
              "address": "0xb002bbb96c8b4a1ea4d79d4b59a6c608ae4873668c3203b4142eb8e454dade72",
              "state_key_hash": "0xceb81aeb14d25fb2ca1c0d3711ac769e9f6d1d429648d33997edf2aa386b0ebf",
              "data": {
                "type": "0x1::fungible_asset::Metadata",
                "data": {
                  "decimals": 8,
                  "icon_uri": "",
                  "name": "🎃 emojicoin",
                  "project_uri": "",
                  "symbol": "🎃"
                }
              },
              "type": "write_resource"
            },
            {
              "address": "0xb002bbb96c8b4a1ea4d79d4b59a6c608ae4873668c3203b4142eb8e454dade72",
              "state_key_hash": "0xceb81aeb14d25fb2ca1c0d3711ac769e9f6d1d429648d33997edf2aa386b0ebf",
              "data": {
                "type": "0x1::object::ObjectCore",
                "data": {
                  "allow_ungated_transfer": true,
                  "guid_creation_num": "1125899906842625",
                  "owner": "0xa",
                  "transfer_events": {
                    "counter": "0",
                    "guid": {
                      "id": {
                        "addr": "0xb002bbb96c8b4a1ea4d79d4b59a6c608ae4873668c3203b4142eb8e454dade72",
                        "creation_num": "1125899906842624"
                      }
                    }
                  }
                }
              },
              "type": "write_resource"
            },
            {
              "address": "0xb002bbb96c8b4a1ea4d79d4b59a6c608ae4873668c3203b4142eb8e454dade72",
              "state_key_hash": "0xceb81aeb14d25fb2ca1c0d3711ac769e9f6d1d429648d33997edf2aa386b0ebf",
              "data": {
                "type": "0x1::primary_fungible_store::DeriveRefPod",
                "data": {
                  "metadata_derive_ref": {
                    "self": "0xb002bbb96c8b4a1ea4d79d4b59a6c608ae4873668c3203b4142eb8e454dade72"
                  }
                }
              },
              "type": "write_resource"
            },
            {
              "address": "0xe4458914726dd8f66887f904f90281a7f042707ed480ddb513ca1176f4305b6f",
              "state_key_hash": "0x14e4c50b7c5ddbf4915fe72debf9a07f5da1d722646cd3fc95a64db4c32c2ad6",
              "data": {
                "type": "0x1::fungible_asset::FungibleStore",
                "data": {
                  "balance": "1105744500000",
                  "frozen": false,
                  "metadata": {
                    "inner": "0xa"
                  }
                }
              },
              "type": "write_resource"
            },
            {
              "address": "0xe4458914726dd8f66887f904f90281a7f042707ed480ddb513ca1176f4305b6f",
              "state_key_hash": "0x14e4c50b7c5ddbf4915fe72debf9a07f5da1d722646cd3fc95a64db4c32c2ad6",
              "data": {
                "type": "0x1::object::ObjectCore",
                "data": {
                  "allow_ungated_transfer": false,
                  "guid_creation_num": "1125899906842625",
                  "owner": "0x1356651cd783ff9af7ad9840f8b657cb5f8b13efffad05f53001256946db1d18",
                  "transfer_events": {
                    "counter": "0",
                    "guid": {
                      "id": {
                        "addr": "0xe4458914726dd8f66887f904f90281a7f042707ed480ddb513ca1176f4305b6f",
                        "creation_num": "1125899906842624"
                      }
                    }
                  }
                }
              },
              "type": "write_resource"
            },
            {
              "address": "0xe777e1d3a5e9446cffe96c48ddbfbda615abfc374fafba904905579d4aaa0783",
              "state_key_hash": "0xfe9f68f3a676971d7e3e662b8364ee5779245e33876ea50f38039ebcc19311b6",
              "data": {
                "type": "0x1::fungible_asset::FungibleStore",
                "data": {
                  "balance": "1809063316",
                  "frozen": false,
                  "metadata": {
                    "inner": "0x41535865d434b1bbe46f3821f5fa2e89e46547b667b38ee46bf23986e287634a"
                  }
                }
              },
              "type": "write_resource"
            },
            {
              "address": "0xe777e1d3a5e9446cffe96c48ddbfbda615abfc374fafba904905579d4aaa0783",
              "state_key_hash": "0xfe9f68f3a676971d7e3e662b8364ee5779245e33876ea50f38039ebcc19311b6",
              "data": {
                "type": "0x1::object::ObjectCore",
                "data": {
                  "allow_ungated_transfer": false,
                  "guid_creation_num": "1125899906842625",
                  "owner": "0x5048c88ba0ab78f78f4da8d2c3c3a35078315a79e28f8e223c1522761d0eec64",
                  "transfer_events": {
                    "counter": "0",
                    "guid": {
                      "id": {
                        "addr": "0xe777e1d3a5e9446cffe96c48ddbfbda615abfc374fafba904905579d4aaa0783",
                        "creation_num": "1125899906842624"
                      }
                    }
                  }
                }
              },
              "type": "write_resource"
            },
            {
              "address": "0xfc36e3b6494512eb6fb15e69ff2cde14e569ded975f77286b4f964a930ad6c09",
              "state_key_hash": "0x669ee5cd4d8e3bc9d7dcaed2b279dbb367591de6eb89372b1ead34857853c8bc",
              "data": {
                "type": "0x1::fungible_asset::FungibleStore",
                "data": {
                  "balance": "998888598955580",
                  "frozen": false,
                  "metadata": {
                    "inner": "0xa"
                  }
                }
              },
              "type": "write_resource"
            },
            {
              "address": "0xfc36e3b6494512eb6fb15e69ff2cde14e569ded975f77286b4f964a930ad6c09",
              "state_key_hash": "0x669ee5cd4d8e3bc9d7dcaed2b279dbb367591de6eb89372b1ead34857853c8bc",
              "data": {
                "type": "0x1::object::ObjectCore",
                "data": {
                  "allow_ungated_transfer": false,
                  "guid_creation_num": "1125899906842625",
                  "owner": "0x5048c88ba0ab78f78f4da8d2c3c3a35078315a79e28f8e223c1522761d0eec64",
                  "transfer_events": {
                    "counter": "0",
                    "guid": {
                      "id": {
                        "addr": "0xfc36e3b6494512eb6fb15e69ff2cde14e569ded975f77286b4f964a930ad6c09",
                        "creation_num": "1125899906842624"
                      }
                    }
                  }
                }
              },
              "type": "write_resource"
            },
            {
              "address": "0xff17dda15a10063cb24bcf1c99abe59cbe863fe3be800bfcae9988fe402841c7",
              "state_key_hash": "0xa784bd29e9e796c5956cc545f984f5cef23e957ddbf802115101c753244fa249",
              "data": {
                "type": "0x1::fungible_asset::FungibleStore",
                "data": {
                  "balance": "92743807830970",
                  "frozen": false,
                  "metadata": {
                    "inner": "0xb002bbb96c8b4a1ea4d79d4b59a6c608ae4873668c3203b4142eb8e454dade72"
                  }
                }
              },
              "type": "write_resource"
            },
            {
              "address": "0xff17dda15a10063cb24bcf1c99abe59cbe863fe3be800bfcae9988fe402841c7",
              "state_key_hash": "0xa784bd29e9e796c5956cc545f984f5cef23e957ddbf802115101c753244fa249",
              "data": {
                "type": "0x1::object::ObjectCore",
                "data": {
                  "allow_ungated_transfer": false,
                  "guid_creation_num": "1125899906842625",
                  "owner": "0x1356651cd783ff9af7ad9840f8b657cb5f8b13efffad05f53001256946db1d18",
                  "transfer_events": {
                    "counter": "0",
                    "guid": {
                      "id": {
                        "addr": "0xff17dda15a10063cb24bcf1c99abe59cbe863fe3be800bfcae9988fe402841c7",
                        "creation_num": "1125899906842624"
                      }
                    }
                  }
                }
              },
              "type": "write_resource"
            },
            {
              "state_key_hash": "0x6e4b28d40f98a106a65163530924c0dcb40c1349d3aa915d108b4d6cfc1ddb19",
              "handle": "0x1b854694ae746cdbd8d44186ca4929b2b337df21d1c74633be19b2710552fdca",
              "key": "0x0619dc29a0aac8fa146714058e8dd6d2d0f3bdf5f6331907bf91f3acd81e6935",
              "value": "0x0ac2eb0b000000000000000000000000",
              "data": {
                "key": "0x619dc29a0aac8fa146714058e8dd6d2d0f3bdf5f6331907bf91f3acd81e6935",
                "key_type": "address",
                "value": "200000010",
                "value_type": "u128"
              },
              "type": "write_table_item"
            }
          ],
          "sender": "0x5048c88ba0ab78f78f4da8d2c3c3a35078315a79e28f8e223c1522761d0eec64",
          "sequence_number": "3",
          "max_gas_amount": "92",
          "gas_unit_price": "100",
          "expiration_timestamp_secs": "1749590205",
          "payload": {
            "function": "0xf000d910b99722d201c6cf88eb7d1112b43475b9765b118f289b5d65d919000d::emojicoin_dot_fun::provide_liquidity",
            "type_arguments": [
              "0x1356651cd783ff9af7ad9840f8b657cb5f8b13efffad05f53001256946db1d18::coin_factory::Emojicoin",
              "0x1356651cd783ff9af7ad9840f8b657cb5f8b13efffad05f53001256946db1d18::coin_factory::EmojicoinLP"
            ],
            "arguments": [
              "0x1356651cd783ff9af7ad9840f8b657cb5f8b13efffad05f53001256946db1d18",
              "100000000",
              "1"
            ],
            "type": "entry_function_payload"
          },
          "signature": {
            "public_key": "0xa75c7b1d71d12c127d36323c47ff2422abe60174cded0bd05ed8d0d4ed54caaf",
            "signature": "0x5846727a2caf12f340f60850e8b59089fd70264e7d6eca32caf806b4e1000dd3fb9fcc77763720341f2ef203fe5fcf331027c5affa80eff111a2dda102839201",
            "type": "ed25519_signature"
          },
          "replay_protection_nonce": null,
          "events": [
            {
              "guid": {
                "creation_number": "0",
                "account_address": "0x0"
              },
              "sequence_number": "0",
              "type": "0xf000d910b99722d201c6cf88eb7d1112b43475b9765b118f289b5d65d919000d::emojicoin_dot_fun::PeriodicState",
              "data": {
                "close_price_q64": "4627282359396380",
                "ends_in_bonding_curve": false,
                "high_price_q64": "4627282359396380",
                "integrator_fees": "5655500000",
                "low_price_q64": "4627282359396380",
                "market_metadata": {
                  "emoji_bytes": "0xf09f8e83",
                  "market_address": "0x1356651cd783ff9af7ad9840f8b657cb5f8b13efffad05f53001256946db1d18",
                  "market_id": "3"
                },
                "n_chat_messages": "0",
                "n_swaps": "1",
                "open_price_q64": "4627282359396380",
                "periodic_state_metadata": {
                  "emit_market_nonce": "4",
                  "emit_time": "1749590115188762",
                  "period": "60000000",
                  "start_time": "1749589680000000",
                  "trigger": 4
                },
                "pool_fees_base": "2273867085404",
                "pool_fees_quote": "0",
                "starts_in_bonding_curve": true,
                "tvl_per_lp_coin_growth_q64": "0",
                "volume_base": "4407272967076404",
                "volume_quote": "1105544500000"
              }
            },
            {
              "guid": {
                "creation_number": "0",
                "account_address": "0x0"
              },
              "sequence_number": "0",
              "type": "0xf000d910b99722d201c6cf88eb7d1112b43475b9765b118f289b5d65d919000d::emojicoin_dot_fun::PeriodicState",
              "data": {
                "close_price_q64": "4627282359396380",
                "ends_in_bonding_curve": false,
                "high_price_q64": "4627282359396380",
                "integrator_fees": "5655500000",
                "low_price_q64": "4627282359396380",
                "market_metadata": {
                  "emoji_bytes": "0xf09f8e83",
                  "market_address": "0x1356651cd783ff9af7ad9840f8b657cb5f8b13efffad05f53001256946db1d18",
                  "market_id": "3"
                },
                "n_chat_messages": "0",
                "n_swaps": "1",
                "open_price_q64": "4627282359396380",
                "periodic_state_metadata": {
                  "emit_market_nonce": "4",
                  "emit_time": "1749590115188762",
                  "period": "300000000",
                  "start_time": "1749589500000000",
                  "trigger": 4
                },
                "pool_fees_base": "2273867085404",
                "pool_fees_quote": "0",
                "starts_in_bonding_curve": true,
                "tvl_per_lp_coin_growth_q64": "0",
                "volume_base": "4407272967076404",
                "volume_quote": "1105544500000"
              }
            },
            {
              "guid": {
                "creation_number": "0",
                "account_address": "0x0"
              },
              "sequence_number": "0",
              "type": "0xf000d910b99722d201c6cf88eb7d1112b43475b9765b118f289b5d65d919000d::emojicoin_dot_fun::PeriodicState",
              "data": {
                "close_price_q64": "4627282359396380",
                "ends_in_bonding_curve": false,
                "high_price_q64": "4627282359396380",
                "integrator_fees": "5655500000",
                "low_price_q64": "4627282359396380",
                "market_metadata": {
                  "emoji_bytes": "0xf09f8e83",
                  "market_address": "0x1356651cd783ff9af7ad9840f8b657cb5f8b13efffad05f53001256946db1d18",
                  "market_id": "3"
                },
                "n_chat_messages": "0",
                "n_swaps": "1",
                "open_price_q64": "4627282359396380",
                "periodic_state_metadata": {
                  "emit_market_nonce": "4",
                  "emit_time": "1749590115188762",
                  "period": "900000000",
                  "start_time": "1749589200000000",
                  "trigger": 4
                },
                "pool_fees_base": "2273867085404",
                "pool_fees_quote": "0",
                "starts_in_bonding_curve": true,
                "tvl_per_lp_coin_growth_q64": "0",
                "volume_base": "4407272967076404",
                "volume_quote": "1105544500000"
              }
            },
            {
              "guid": {
                "creation_number": "0",
                "account_address": "0x0"
              },
              "sequence_number": "0",
              "type": "0x1::fungible_asset::Withdraw",
              "data": {
                "amount": "8387453687",
                "store": "0x282415b709466709d27126d3c19f4b00993c4b371ecd595c80ecf3ef86d2c96d"
              }
            },
            {
              "guid": {
                "creation_number": "0",
                "account_address": "0x0"
              },
              "sequence_number": "0",
              "type": "0x1::fungible_asset::Deposit",
              "data": {
                "amount": "8387453687",
                "store": "0xff17dda15a10063cb24bcf1c99abe59cbe863fe3be800bfcae9988fe402841c7"
              }
            },
            {
              "guid": {
                "creation_number": "0",
                "account_address": "0x0"
              },
              "sequence_number": "0",
              "type": "0x1::fungible_asset::Withdraw",
              "data": {
                "amount": "100000000",
                "store": "0xfc36e3b6494512eb6fb15e69ff2cde14e569ded975f77286b4f964a930ad6c09"
              }
            },
            {
              "guid": {
                "creation_number": "0",
                "account_address": "0x0"
              },
              "sequence_number": "0",
              "type": "0x1::fungible_asset::Deposit",
              "data": {
                "amount": "100000000",
                "store": "0xe4458914726dd8f66887f904f90281a7f042707ed480ddb513ca1176f4305b6f"
              }
            },
            {
              "guid": {
                "creation_number": "0",
                "account_address": "0x0"
              },
              "sequence_number": "0",
              "type": "0x1::fungible_asset::Deposit",
              "data": {
                "amount": "904531658",
                "store": "0xe777e1d3a5e9446cffe96c48ddbfbda615abfc374fafba904905579d4aaa0783"
              }
            },
            {
              "guid": {
                "creation_number": "0",
                "account_address": "0x0"
              },
              "sequence_number": "0",
              "type": "0xf000d910b99722d201c6cf88eb7d1112b43475b9765b118f289b5d65d919000d::emojicoin_dot_fun::Liquidity",
              "data": {
                "base_amount": "8387453687",
                "base_donation_claim_amount": "0",
                "liquidity_provided": true,
                "lp_coin_amount": "904531658",
                "market_id": "3",
                "market_nonce": "4",
                "provider": "0x5048c88ba0ab78f78f4da8d2c3c3a35078315a79e28f8e223c1522761d0eec64",
                "quote_amount": "100000000",
                "quote_donation_claim_amount": "0",
                "time": "1749590115188762"
              }
            },
            {
              "guid": {
                "creation_number": "0",
                "account_address": "0x0"
              },
              "sequence_number": "0",
              "type": "0xf000d910b99722d201c6cf88eb7d1112b43475b9765b118f289b5d65d919000d::emojicoin_dot_fun::State",
              "data": {
                "clamm_virtual_reserves": {
                  "base": "0",
                  "quote": "0"
                },
                "cpamm_real_reserves": {
                  "base": "92743807830970",
                  "quote": "1105744500000"
                },
                "cumulative_stats": {
                  "base_volume": "4407272967076404",
                  "integrator_fees": "5655500000",
                  "n_chat_messages": "0",
                  "n_swaps": "1",
                  "pool_fees_base": "2273867085404",
                  "pool_fees_quote": "0",
                  "quote_volume": "1105544500000"
                },
                "instantaneous_stats": {
                  "fully_diluted_value": "53651563013982",
                  "market_cap": "52545818513982",
                  "total_quote_locked": "1105744500000",
                  "total_value_locked": "2211489000000"
                },
                "last_swap": {
                  "avg_execution_price_q64": "4627282359396380",
                  "base_volume": "4407272967076404",
                  "is_sell": false,
                  "nonce": "2",
                  "quote_volume": "1105544500000",
                  "time": "1749589716980950"
                },
                "lp_coin_supply": "10001809063316",
                "market_metadata": {
                  "emoji_bytes": "0xf09f8e83",
                  "market_address": "0x1356651cd783ff9af7ad9840f8b657cb5f8b13efffad05f53001256946db1d18",
                  "market_id": "3"
                },
                "state_metadata": {
                  "bump_time": "1749590115188762",
                  "market_nonce": "4",
                  "trigger": 4
                }
              }
            },
            {
              "guid": {
                "creation_number": "0",
                "account_address": "0x0"
              },
              "sequence_number": "0",
              "type": "0x1::transaction_fee::FeeStatement",
              "data": {
                "execution_gas_units": "17",
                "io_gas_units": "30",
                "storage_fee_octas": "0",
                "storage_fee_refund_octas": "0",
                "total_charge_gas_units": "46"
              }
            }
          ],
          "timestamp": "1749590115188762",
          "type": "user_transaction"
      }
      "#;
        match serde_json::from_str::<TransactionInfo>(pool_txn_json) {
            Ok(txn_info) => {
                let txn: JSONTransaction = txn_info.into();
                let ttxn: Transaction = txn.into();
                let txn_data = txn
                    .txn_data
                    .as_ref()
                    .expect("Transaction data should exist");
                let events = match txn_data {
                    TxnData::User(user_txn) => &user_txn.events,
                    _ => panic!("Expected TxnData::User"),
                };

                let liquidity_event = events
                    .iter()
                    .enumerate()
                    .find_map(|(event_index, event)| {
                        let type_str = event.type_str.as_str();
                        let data = event.data.as_str();
                        EmojicoinTypeTag::from_type_str(type_str).and_then(|_| {
                            EventWithMarket::from_event_type(
                                type_str,
                                data,
                                txn.version as i64,
                                event_index as i64,
                            )
                            .ok()
                            .and_then(|evt| match &evt {
                                Some(EventWithMarket::Liquidity(liq)) => Some(liq.clone()),
                                _ => None,
                            })
                        })
                    })
                    .expect("Should find liquidity event.");
                // UserLiquidityPoolsModel::from_event_and_writeset(&txn, asdf);
                //
                println!("{:?}", liquidity_event);
            },
            Err(e) => {
                panic!("Failed to parse transaction: {:?}", e);
            },
        }
    }
}
