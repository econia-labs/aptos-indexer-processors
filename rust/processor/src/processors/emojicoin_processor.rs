// Copyright © Aptos Foundation
// SPDX-License-Identifier: Apache-2.0

use super::{DefaultProcessingResult, ProcessorName, ProcessorTrait};
use crate::{
    db::common::models::{
        emojicoin_models::{
            event_utils::BumpGroupBuilder,
            json_types::{EmojicoinEvent, EventWithMarket, TxnInfo},
        },
        events_models::events::EventModel,
    },
    gap_detectors::ProcessingResult,
    schema,
    utils::{
        counters::PROCESSOR_UNKNOWN_TYPE_COUNT,
        database::{execute_in_chunks, get_config_table_chunk_size, ArcDbPool},
        util::{get_entry_function_from_user_request, standardize_address},
    },
};
use ahash::AHashMap;
use anyhow::bail;
use aptos_protos::transaction::v1::{transaction::TxnData, Transaction};
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
// For grouping all events in a single transaction into the various types:
// Each state event has a unique bump nonce, we can use that to group non-state events with state events.
// The following groupings are possible:
// -- Market-ID specific State Events
//    - ONE State Event
//    - ONE of the following:
//       - Market Registration
//       - Chat
//       - Swap
//       - Liquidity
//    - ZERO to SEVEN of the following:
//       - Periodic State Events
// Note that we have no (easy) way of knowing which state event triggered a GlobalStateEvent, because it doesn't emit
//  the market_id or bump_nonce. We can only know that it was triggered by a state event. This means we can't group
//  GlobalStateEvents with StateEvents in a BumpGroup.
//  This is totally fine, because we can just insert the GlobalStateEvent into the global_state_events table separately.
//  Note that there will only ever be one GlobalStateEvent per transaction, because it takes an entire day to emit
//   a GlobalStateEvent since the last one. Thus we just have an `Option<GlobalStateEvent>` for each transaction that
//   we possibly insert into the global_state_events table after event processing.

// Now we sort the vector by market_id first, then the bump_nonce. Note we don't need to even check the StateTrigger type because haveint the same market_id and bump_nonce means
// there will definitively *only* be one StateTrigger type.
// We can actually panic if we somehow don't fill the bump event by the end of the transaction event iteration.
//   It should literally never happen unless the processor was written incorrectly.
// So:
//    1. Create a vector of all events
//    2. Sort the vector by market_id, then bump_nonce
//    3. Iterate over the sorted vector. You MUST be able to place EVERY single event into a BumpGroup.
//    4. Use the BumpGroup to insert each event into its corresponding table:
//       - state_events
//       - periodic_state_events
//       - global_state_events
// Try to keep in mind that we will eventually query for the rolling 24-hour volume as well.
//   - This will be a query right before the insert. We will find the earliest row in `state_events` with a `last_swap` event time that was at least 24 hours ago.
//   - Then we use that to subtract the current total/cumulative volume from the total/cumulative volume at that time, which will give us the 24-hour volume.
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
        let processing_start = std::time::Instant::now();
        let last_transaction_timestamp = transactions.last().unwrap().timestamp.clone();

        // let mut events = vec![];
        let events = vec![];
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

            let mut non_global_events = vec![];
            let mut global_state_events = vec![];
            if let TxnData::User(user_txn) = txn_data {
                let user_request = user_txn
                    .request
                    .as_ref()
                    .expect("User request info is not present in the user transaction.");
                let entry_function = get_entry_function_from_user_request(user_request);
                let txn_info = TxnInfo {
                    version: txn_version,
                    sender: standardize_address(user_request.sender.as_ref()),
                    entry_function,
                };

                for event in user_txn.events.iter() {
                    let type_str = event.type_str.as_str();
                    let data = event.data.as_str();

                    if let Ok(Some(event_with_market)) =
                        EventWithMarket::from_event_type(type_str, data, txn_version)
                    {
                        non_global_events.push(event_with_market);
                    } else if let Ok(Some(global_event)) =
                        EmojicoinEvent::from_event_type(type_str, data, txn_version)
                    {
                        debug_assert!(matches!(global_event, EmojicoinEvent::GlobalState(_)));
                        global_state_events.push(global_event);
                    }
                }

                // Sort and group all events that have the same market ID together.
                // Note that we implemented the Ord trait for EventWithMarket with a specific order.
                non_global_events.sort();

                let mut bump_groups = vec![];
                // Build the bump group as long as the market ID and nonce are the same as the current group.
                // Once we encounter a new market ID or nonce, we build the current group and start a new one.
                // If the data is sorted incorrectly, the builder function will panic.
                let first = non_global_events.first().cloned();

                if let Some(first) = first {
                    let market_id = (EventWithMarket::from(first.clone())).get_market_id();
                    let nonce = (EventWithMarket::from(first.clone())).get_market_nonce();
                    let mut group = BumpGroupBuilder::new(market_id, nonce, txn_info.clone());
                    let mut building: bool = false;

                    group.add_event(first);

                    for evt in non_global_events.drain(1..) {
                        if evt.get_market_id() == market_id && evt.get_market_nonce() == nonce {
                            group.add_event(evt);
                            building = true;
                        } else {
                            bump_groups.push(group.build());
                            group = BumpGroupBuilder::new(
                                evt.get_market_id(),
                                evt.get_market_nonce(),
                                txn_info.clone(),
                            );
                            group.add_event(evt);
                            building = false;
                        }
                    }

                    // We need to push the last group if it was not pushed in the loop- this can happen
                    // if all events are for the same market and the get to the end of the non global events.
                    if building {
                        bump_groups.push(group.build());
                    }
                }
            }

            // REMEMBER! You can avoid querying when the last swap nonce is 0 or last swap time is > 24 h ago.
            // Because the 24h rolling volume is just the current state metadata's cumulative volume in both cases.
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
