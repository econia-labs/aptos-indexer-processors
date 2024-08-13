use super::insertion_queries::{
    insert_chat_events_query, insert_global_events, insert_liquidity_events_query,
    insert_market_registration_events_query, insert_periodic_state_events_query,
    insert_swap_events_query,
};
use crate::{
    db::common::models::emojicoin_models::{
        event_utils::BumpGroupBuilder,
        json_types::{BumpEvent, BumpGroup, EventWithMarket, GlobalStateEvent, TxnInfo},
        models::{
            chat_event::ChatEventModel, global_state_event::GlobalStateEventModel,
            liquidity_event::LiquidityEventModel,
            market_registration_event::MarketRegistrationEventModel,
            periodic_state_event::PeriodicStateEventModel, swap_event::SwapEventModel,
        },
    },
    gap_detectors::ProcessingResult,
    processors::{DefaultProcessingResult, ProcessorName, ProcessorTrait},
    utils::{
        counters::PROCESSOR_UNKNOWN_TYPE_COUNT,
        database::{execute_in_chunks, get_config_table_chunk_size, ArcDbPool},
        util::{get_entry_function_from_user_request, parse_timestamp, standardize_address},
    },
};
use ahash::AHashMap;
use anyhow::bail;
use aptos_protos::transaction::v1::{transaction::TxnData, Transaction};
use async_trait::async_trait;
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
    market_registration_events: &[MarketRegistrationEventModel],
    swap_events: &[SwapEventModel],
    chat_events: &[ChatEventModel],
    liquidity_events: &[LiquidityEventModel],
    periodic_state_events: &[PeriodicStateEventModel],
    global_state_events: &[GlobalStateEventModel],
    per_table_chunk_sizes: &AHashMap<String, usize>,
) -> Result<(), diesel::result::Error> {
    tracing::trace!(
        name = name,
        start_version = start_version,
        end_version = end_version,
        "Inserting to db",
    );
    let market_registration = execute_in_chunks(
        conn.clone(),
        insert_market_registration_events_query,
        market_registration_events,
        get_config_table_chunk_size::<MarketRegistrationEventModel>(
            "market_registration_events",
            per_table_chunk_sizes,
        ),
    );
    let swap = execute_in_chunks(
        conn.clone(),
        insert_swap_events_query,
        swap_events,
        get_config_table_chunk_size::<SwapEventModel>("swap_events", per_table_chunk_sizes),
    );
    let chat = execute_in_chunks(
        conn.clone(),
        insert_chat_events_query,
        chat_events,
        get_config_table_chunk_size::<ChatEventModel>("chat_events", per_table_chunk_sizes),
    );
    let liquidity = execute_in_chunks(
        conn.clone(),
        insert_liquidity_events_query,
        liquidity_events,
        get_config_table_chunk_size::<LiquidityEventModel>(
            "liquidity_events",
            per_table_chunk_sizes,
        ),
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

    let (m, s, c, l, p, g) =
        tokio::join!(market_registration, chat, swap, liquidity, periodic, global);
    for res in [m, s, c, l, p, g] {
        res?;
    }

    Ok(())
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

        let mut market_registration_events = vec![];
        let mut swap_events = vec![];
        let mut chat_events = vec![];
        let mut liquidity_events = vec![];
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
                    timestamp: parse_timestamp(txn.timestamp.as_ref().unwrap(), txn_version),
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
                                    txn_info.clone(),
                                    global_event,
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
                    let BumpGroup {
                        bump_event,
                        state_event: state_ev,
                        periodic_state_events: periodic_events,
                        txn_info,
                        ..
                    } = group;

                    periodic_state_events.extend(PeriodicStateEventModel::from_periodic_events(
                        txn_info.clone(),
                        periodic_events,
                        state_ev.last_swap.clone(),
                    ));

                    match bump_event {
                        BumpEvent::MarketRegistration(ev) => market_registration_events
                            .push(MarketRegistrationEventModel::new(txn_info, ev, state_ev)),
                        BumpEvent::Chat(ev) => {
                            chat_events.push(ChatEventModel::new(txn_info, ev, state_ev))
                        },
                        BumpEvent::Swap(ev) => {
                            swap_events.push(SwapEventModel::new(txn_info, ev, state_ev))
                        },
                        BumpEvent::Liquidity(ev) => {
                            liquidity_events.push(LiquidityEventModel::new(txn_info, ev, state_ev))
                        },
                    }
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
            &market_registration_events,
            &swap_events,
            &chat_events,
            &liquidity_events,
            &periodic_state_events,
            &global_state_events,
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
