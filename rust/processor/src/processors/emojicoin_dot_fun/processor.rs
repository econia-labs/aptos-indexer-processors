use crate::{
    db::common::models::emojicoin_models::{
        db_types::{
            global_state_events_model::GlobalStateEventModel,
            periodic_state_events_model::PeriodicStateEventModel,
            state_bumps_model::StateBumpModel,
        },
        event_utils::BumpGroupBuilder,
        json_types::{BumpGroup, EventWithMarket, GlobalStateEvent, TxnInfo},
    },
    gap_detectors::ProcessingResult,
    processors::{DefaultProcessingResult, ProcessorName, ProcessorTrait},
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
use diesel::{pg::Pg, query_builder::QueryFragment};
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
    global_state_events: &[GlobalStateEventModel],
    periodic_state_events: &[PeriodicStateEventModel],
    state_bumps: &[StateBumpModel],
    per_table_chunk_sizes: &AHashMap<String, usize>,
) -> Result<(), diesel::result::Error> {
    tracing::trace!(
        name = name,
        start_version = start_version,
        end_version = end_version,
        "Inserting to db",
    );
    let bump = execute_in_chunks(
        conn.clone(),
        insert_state_bumps_query,
        state_bumps,
        get_config_table_chunk_size::<StateBumpModel>("state_bumps", per_table_chunk_sizes),
    );
    let periodic = execute_in_chunks(
        conn.clone(),
        insert_periodic_state_events_query,
        periodic_state_events,
        get_config_table_chunk_size::<PeriodicStateEventModel>(
            "periodic_state_events",
            per_table_chunk_sizes,
        ),
    );
    let global = execute_in_chunks(
        conn.clone(),
        insert_global_events,
        global_state_events,
        get_config_table_chunk_size::<GlobalStateEventModel>(
            "global_state_events",
            per_table_chunk_sizes,
        ),
    );

    let (b, p, g) = tokio::join!(bump, periodic, global);
    for res in [b, p, g] {
        res?;
    }

    Ok(())
}

fn insert_state_bumps_query(
    items_to_insert: Vec<StateBumpModel>,
) -> (
    impl QueryFragment<Pg> + diesel::query_builder::QueryId + Send,
    Option<&'static str>,
) {
    use crate::schema::state_bumps;
    (
        diesel::insert_into(state_bumps::table)
            .values(items_to_insert)
            .on_conflict((state_bumps::market_id, state_bumps::market_nonce))
            .do_nothing(),
        None,
    )
}

fn insert_periodic_state_events_query(
    items_to_insert: Vec<PeriodicStateEventModel>,
) -> (
    impl QueryFragment<Pg> + diesel::query_builder::QueryId + Send,
    Option<&'static str>,
) {
    use crate::schema::periodic_state_events;
    (
        diesel::insert_into(periodic_state_events::table)
            .values(items_to_insert)
            .on_conflict((
                periodic_state_events::market_id,
                periodic_state_events::resolution,
                periodic_state_events::market_nonce,
            ))
            .do_nothing(),
        None,
    )
}

fn insert_global_events(
    items_to_insert: Vec<GlobalStateEventModel>,
) -> (
    impl QueryFragment<Pg> + diesel::query_builder::QueryId + Send,
    Option<&'static str>,
) {
    use crate::schema::global_state_events;
    (
        diesel::insert_into(global_state_events::table)
            .values(items_to_insert)
            .on_conflict(global_state_events::registry_nonce)
            .do_nothing(),
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
        let processing_start = std::time::Instant::now();
        let last_transaction_timestamp = transactions.last().unwrap().timestamp.clone();

        let mut state_bumps = vec![];
        let mut periodic_state_events = vec![];
        let mut global_state_events = vec![];
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

                // Push global events directly to the vector we use for insertion.
                let mut txn_non_global_state_events = vec![];

                for event in user_txn.events.iter() {
                    let type_str = event.type_str.as_str();
                    let data = event.data.as_str();

                    match EventWithMarket::from_event_type(type_str, data, txn_version)? {
                        Some(event_with_market) => {
                            txn_non_global_state_events.push(event_with_market);
                        },
                        _ => {
                            if let Some(global_event) =
                                GlobalStateEvent::from_event_type(type_str, data, txn_version)?
                            {
                                global_state_events.push(GlobalStateEventModel::new(
                                    global_event,
                                    txn_info.clone(),
                                ));
                            }
                        },
                    }
                }

                // Sort and group all events according to EventWithMarket's custom `Ord` implementation.
                // The builder function below groups events based on their order in the sorted vector.
                // See the `Ord` implementation in `EventWithMarket` for more details.
                txn_non_global_state_events.sort();

                let mut bump_groups = vec![];
                let mut iter = txn_non_global_state_events.into_iter();
                if let Some(first) = iter.next() {
                    let mut group = BumpGroupBuilder::new(first, txn_info.clone());

                    // Build upon the current group until the market ID or nonce changes.
                    for evt in iter {
                        let (curr_id, curr_nonce) = (evt.get_market_id(), evt.get_market_nonce());
                        if curr_id == group.market_id && curr_nonce == group.market_nonce {
                            group.add_event(evt);
                        } else {
                            bump_groups.push(group.build());
                            group = BumpGroupBuilder::new(evt, txn_info.clone());
                        }
                    }

                    // Since we only call `build` when a market ID or nonce changes, the last group
                    // will be fully formed but not built yet, so we call `build` here.
                    bump_groups.push(group.build());
                }

                for group in bump_groups {
                    let (bump, periodics) = BumpGroup::to_db_models(group);
                    state_bumps.push(bump);
                    periodic_state_events.extend(periodics);
                }
            }
        }

        let processing_duration_in_secs = processing_start.elapsed().as_secs_f64();
        let db_insertion_start = std::time::Instant::now();

        let tx_result = insert_to_db(
            self.get_pool(),
            self.name(),
            start_version,
            end_version,
            &global_state_events,
            &periodic_state_events,
            &state_bumps,
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
