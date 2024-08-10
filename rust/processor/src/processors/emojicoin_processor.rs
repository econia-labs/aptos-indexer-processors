// Copyright © Aptos Foundation
// SPDX-License-Identifier: Apache-2.0

use super::{DefaultProcessingResult, ProcessorName, ProcessorTrait};
use crate::{
    db::common::models::{
        emojicoin_models::event_types::{
            CumulativeStats, EmojicoinEventAndTxnInfo, InstantaneousStats, LastSwap, Reserves,
            StateMetadata,
        },
        events_models::events::EventModel,
    },
    gap_detectors::ProcessingResult,
    schema,
    utils::{
        counters::PROCESSOR_UNKNOWN_TYPE_COUNT,
        database::{execute_in_chunks, get_config_table_chunk_size, ArcDbPool},
    },
};
use ahash::AHashMap;
use anyhow::bail;
use aptos_protos::transaction::v1::Transaction;
use async_trait::async_trait;
use diesel::{
    pg::{upsert::excluded, Pg},
    query_builder::QueryFragment,
    ExpressionMethods,
};
use std::fmt::Debug;
use tracing::error;

pub struct EmojicoinProcessor {
    connection_pool: ArcDbPool,
    per_table_chunk_sizes: AHashMap<String, usize>,
}

impl EmojicoinProcessor {
    pub fn new(connection_pool: ArcDbPool, per_table_chunk_sizes: AHashMap<String, usize>) -> Self {
        Self {
            connection_pool,
            per_table_chunk_sizes,
        }
    }
}

impl Debug for EmojicoinProcessor {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let state = &self.connection_pool.state();
        write!(
            f,
            "EmojicoinProcessor {{ connections: {:?}  idle_connections: {:?} }}",
            state.connections, state.idle_connections
        )
    }
}

async fn insert_to_db(
    conn: ArcDbPool,
    name: &'static str,
    start_version: u64,
    end_version: u64,
    events: &[EventModel],
    per_table_chunk_sizes: &AHashMap<String, usize>,
) -> Result<(), diesel::result::Error> {
    tracing::trace!(
        name = name,
        start_version = start_version,
        end_version = end_version,
        "Inserting to db",
    );
    execute_in_chunks(
        conn,
        insert_events_query,
        events,
        get_config_table_chunk_size::<EventModel>("events", per_table_chunk_sizes),
    )
    .await?;
    Ok(())
}

fn insert_events_query(
    items_to_insert: Vec<EventModel>,
) -> (
    impl QueryFragment<Pg> + diesel::query_builder::QueryId + Send,
    Option<&'static str>,
) {
    use schema::events::dsl::*;
    (
        diesel::insert_into(schema::events::table)
            .values(items_to_insert)
            .on_conflict((transaction_version, event_index))
            .do_update()
            .set((
                inserted_at.eq(excluded(inserted_at)),
                indexed_type.eq(excluded(indexed_type)),
            )),
        None,
    )
}

#[async_trait]
impl ProcessorTrait for EmojicoinProcessor {
    fn name(&self) -> &'static str {
        ProcessorName::EmojicoinProcessor.into()
    }

    async fn process_transactions(
        &self,
        transactions: Vec<Transaction>,
        start_version: u64,
        end_version: u64,
        _: Option<u64>,
    ) -> anyhow::Result<ProcessingResult> {
        let json = r#"{
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
              "emoji_bytes": "0xf09faaa4",
              "market_address": "0xa66fb901175394d0883e28262c4c40cb8228e47a36e6a813d5117805c3c26a5c",
              "market_id": "328"
            },
            "state_metadata": {
              "bump_time": "1723246374791035",
              "market_nonce": "40278",
              "trigger": 4
            }
          }
          "#;

        let parsed: serde_json::Value = serde_json::from_str(json).unwrap();
        println!("{:?}", parsed);

        if let Some(state_metadata) = parsed.get("state_metadata") {
            let state_metadata: StateMetadata = serde_json::from_value(state_metadata.clone())?;
            println!("Parsed StateMetadata: {:?}", state_metadata);
        }
        if let Some(last_swap) = parsed.get("last_swap") {
            let last_swap: LastSwap = serde_json::from_value(last_swap.clone())?;
            println!("Parsed LastSwap: {:?}", last_swap);
        }

        if let Some(clamm_virtual_reserves) = parsed.get("clamm_virtual_reserves") {
            let clamm_virtual_reserves: Reserves =
                serde_json::from_value(clamm_virtual_reserves.clone())?;
            println!("Parsed Reserves: {:?}", clamm_virtual_reserves);
        }
        if let Some(cpamm_real_reserves) = parsed.get("cpamm_real_reserves") {
            let cpamm_real_reserves: Reserves =
                serde_json::from_value(cpamm_real_reserves.clone())?;
            println!("Parsed Reserves: {:?}", cpamm_real_reserves);
        }
        if let Some(cumulative_stats) = parsed.get("cumulative_stats") {
            let cumulative_stats: CumulativeStats =
                serde_json::from_value(cumulative_stats.clone())?;
            println!("Parsed CumulativeStats: {:?}", cumulative_stats);
        }
        if let Some(instantaneous_stats) = parsed.get("instantaneous_stats") {
            let instantaneous_stats: InstantaneousStats =
                serde_json::from_value(instantaneous_stats.clone())?;
            println!("Parsed InstantaneousStats: {:?}", instantaneous_stats);
        }

        let processing_start = std::time::Instant::now();
        let last_transaction_timestamp = transactions.last().unwrap().timestamp.clone();

        // let mut events = vec![];
        // let mut events = vec![];
        // let mut events = vec![];
        let events = vec![];
        // let mut events = vec![];
        // let mut events = vec![];
        // let mut events = vec![];
        for txn in &transactions {
            let txn_version = txn.version as i64;
            let txn_data = match txn.txn_data.as_ref() {
                Some(data) => data,
                None => {
                    tracing::warn!(
                        transaction_version = txn_version,
                        "Transaction data doesn't exist"
                    );
                    PROCESSOR_UNKNOWN_TYPE_COUNT
                        .with_label_values(&["EmojicoinProcessor"])
                        .inc();
                    continue;
                },
            };

            let emojicoin_events =
                EmojicoinEventAndTxnInfo::from_transaction(txn_version, txn_data);

            // let default = vec![];
            // let raw_events = match txn_data {
            //     TxnData::User(tx_inner) => &tx_inner.events,
            //     _ => &default,
            // };

            // let emojicoin_events = EventModel::from_events(raw_events, txn_version, block_height)
            //     .into_iter()
            //     .filter_map(|event| match EmojicoinEvent::from_event(&event) {
            //         Ok(Some(emojicoin_event)) => Some(emojicoin_event),
            //         _ => None,
            //     });

            for event in emojicoin_events {
                println!("{:?}", event);
            }
            // .collect::<Vec<EmojicoinEvent>>();

            // println!("{:?}", emojicoin_events);
            // println!("{:?}", emojicoin_events.collect::<Vec<EmojicoinEvent>>());

            // .into_iter()
            // .filter(|event| event.)
            // let emojicoin_events =
            // events.extend(EventModel::from_events(
            //     raw_events,
            //     txn_version,
            //     block_height,
            // ));
        }

        let processing_duration_in_secs = processing_start.elapsed().as_secs_f64();
        let db_insertion_start = std::time::Instant::now();

        let tx_result = insert_to_db(
            self.get_pool(),
            self.name(),
            start_version,
            end_version,
            &events,
            &self.per_table_chunk_sizes,
        )
        .await;

        let db_insertion_duration_in_secs = db_insertion_start.elapsed().as_secs_f64();
        match tx_result {
            Ok(_) => Ok(ProcessingResult::DefaultProcessingResult(
                DefaultProcessingResult {
                    start_version,
                    end_version,
                    processing_duration_in_secs,
                    db_insertion_duration_in_secs,
                    last_transaction_timestamp,
                },
            )),
            Err(e) => {
                error!(
                    start_version = start_version,
                    end_version = end_version,
                    processor_name = self.name(),
                    error = ?e,
                    "[Parser] Error inserting transactions to db",
                );
                bail!(e)
            },
        }
    }

    fn connection_pool(&self) -> &ArcDbPool {
        &self.connection_pool
    }
}
