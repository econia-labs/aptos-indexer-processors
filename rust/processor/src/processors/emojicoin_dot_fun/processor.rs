use super::insertion_queries::{
    insert_chat_events_query, insert_global_events, insert_liquidity_events_query,
    insert_market_registration_events_query, insert_periodic_state_events_query,
    insert_swap_events_query,
};
use crate::{
    db::common::models::emojicoin_models::{
        event_utils::EventGroupBuilder,
        json_types::{BumpEvent, EventGroup, EventWithMarket, GlobalStateEvent, TxnInfo},
        models::{
            chat_event::ChatEventModel, global_state_event::GlobalStateEventModel,
            liquidity_event::LiquidityEventModel,
            market_registration_event::MarketRegistrationEventModel,
            periodic_state_event::PeriodicStateEventModel, swap_event::SwapEventModel,
            user_liquidity_pools::UserLiquidityPoolsModel,
        },
    },
    gap_detectors::ProcessingResult,
    processors::{
        emojicoin_dot_fun::insertion_queries::insert_user_liquidity_pools_query,
        DefaultProcessingResult, ProcessorName, ProcessorTrait,
    },
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
use itertools::Itertools;
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
    user_pools: &[UserLiquidityPoolsModel],
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
    let lp_pools = execute_in_chunks(
        conn.clone(),
        insert_user_liquidity_pools_query,
        user_pools,
        get_config_table_chunk_size::<UserLiquidityPoolsModel>(
            "user_liquidity_pools",
            per_table_chunk_sizes,
        ),
    );

    let (m, s, c, l, per, g, pools) = tokio::join!(
        market_registration,
        chat,
        swap,
        liquidity,
        periodic,
        global,
        lp_pools
    );
    for res in [m, s, c, l, per, g, pools] {
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

        let mut registration_events = vec![];
        let mut swap_events = vec![];
        let mut chat_events = vec![];
        let mut liquidity_events = vec![];
        let mut periodic_state_events = vec![];
        let mut global_state_events = vec![];
        let mut user_pools: AHashMap<(String, i64), UserLiquidityPoolsModel> = AHashMap::new();
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

                // Group the market events in this transaction.
                let mut market_events = vec![];
                for event in user_txn.events.iter() {
                    let type_str = event.type_str.as_str();
                    let data = event.data.as_str();

                    match EventWithMarket::from_event_type(type_str, data, txn_version)? {
                        Some(evt) => {
                            market_events.push(evt);
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

                let mut builders: AHashMap<(i64, i64), EventGroupBuilder> = AHashMap::new();
                for evt in market_events.into_iter() {
                    let (market_id, market_nonce) = (evt.get_market_id(), evt.get_market_nonce());
                    match builders.get_mut(&(market_id, market_nonce)) {
                        Some(group) => {
                            group.add_event(evt);
                        },
                        None => {
                            builders.insert(
                                (market_id, market_nonce),
                                EventGroupBuilder::new(evt, txn_info.clone()),
                            );
                        },
                    };
                }

                for builder in builders.into_values() {
                    let EventGroup {
                        bump_event,
                        state_event: state_evt,
                        periodic_state_events: periodic_events,
                        txn_info,
                        ..
                    } = builder.build();

                    periodic_state_events.extend(PeriodicStateEventModel::from_periodic_events(
                        txn_info.clone(),
                        periodic_events,
                        state_evt.last_swap.clone(),
                    ));

                    match bump_event {
                        BumpEvent::MarketRegistration(bump) => {
                            registration_events
                                .push(MarketRegistrationEventModel::new(txn_info, bump, state_evt));
                        },
                        BumpEvent::Chat(bump) => {
                            chat_events.push(ChatEventModel::new(txn_info, bump, state_evt));
                        },
                        BumpEvent::Swap(bump) => {
                            swap_events.push(SwapEventModel::new(txn_info, bump, state_evt));
                        },
                        BumpEvent::Liquidity(bump) => {
                            let ev = LiquidityEventModel::new(txn_info, bump, state_evt);
                            liquidity_events.push(ev.clone());

                            let key = (ev.provider.clone(), ev.market_id);
                            let new_pool: UserLiquidityPoolsModel = ev.into();
                            user_pools
                                .entry(key)
                                .and_modify(|pool| {
                                    if pool.market_nonce < new_pool.market_nonce {
                                        *pool = new_pool.clone();
                                    }
                                })
                                .or_insert(new_pool);
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
            &registration_events,
            &swap_events,
            &chat_events,
            &liquidity_events,
            &periodic_state_events,
            &global_state_events,
            user_pools.into_values().collect_vec().as_slice(),
            &self.per_table_chunk_sizes,
        )
        .await;

        let db_insertion_duration_in_secs = db_insertion_start.elapsed().as_secs_f64();
        match tx_result {
            Ok(_) => {
                let res = ProcessingResult::DefaultProcessingResult(DefaultProcessingResult {
                    start_version,
                    end_version,
                    processing_duration_in_secs,
                    db_insertion_duration_in_secs,
                    last_transaction_timestamp: last_transaction_timestamp.clone(),
                });
                println!(
                    "::: EmojicoinProcessor: {:?}",
                    (
                        start_version,
                        end_version,
                        processing_duration_in_secs,
                        db_insertion_duration_in_secs,
                        last_transaction_timestamp,
                    )
                );
                Ok(res)
            },
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
