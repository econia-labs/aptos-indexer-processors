use crate::{
    db::common::models::emojicoin_models::{
        enums::Trigger,
        event_utils::EventGroupBuilder,
        json_types::{
            BumpEvent, EventGroup, EventWithMarket, GlobalStateEvent, InstantaneousStats,
            MarketResource, TxnInfo,
        },
        models::{
            chat_event::ChatEventModel, global_state_event::GlobalStateEventModel,
            liquidity_event::LiquidityEventModel,
            market_1m_periods_in_last_day::MarketOneMinutePeriodsInLastDayModel,
            market_24h_rolling_volume::RecentOneMinutePeriodicStateEvent,
            market_latest_state_event::MarketLatestStateEventModel,
            market_registration_event::MarketRegistrationEventModel,
            periodic_state_event::PeriodicStateEventModel, swap_event::SwapEventModel,
            user_liquidity_pools::UserLiquidityPoolsModel,
        },
        queries::insertion_queries::{
            insert_chat_events_query, insert_global_events, insert_liquidity_events_query,
            insert_market_latest_state_event_query, insert_market_registration_events_query,
            insert_periodic_state_events_query, insert_swap_events_query,
            insert_user_liquidity_pools_query,
        },
    },
    emojicoin_dot_fun::EmojicoinDbEvent,
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
use itertools::Itertools;
use std::fmt::Debug;
use tokio::sync::mpsc::UnboundedSender;
use tracing::error;

pub struct EmojicoinProcessor {
    connection_pool: ArcDbPool,
    per_table_chunk_sizes: AHashMap<String, usize>,
    notif_sender: UnboundedSender<EmojicoinDbEvent>,
}

impl EmojicoinProcessor {
    pub fn new(
        connection_pool: ArcDbPool,
        per_table_chunk_sizes: AHashMap<String, usize>,
        notif_sender: UnboundedSender<EmojicoinDbEvent>,
    ) -> Self {
        Self {
            connection_pool,
            per_table_chunk_sizes,
            notif_sender,
        }
    }

    pub fn publish_events(&self, events: Vec<EmojicoinDbEvent>) {
        for event in events {
            if let Err(e) = self.notif_sender.send(event) {
                tracing::error!("Could not send events to websocket server: {e}")
            }
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
    market_latest_state_events: &[MarketLatestStateEventModel],
    market_1m_periods: &[MarketOneMinutePeriodsInLastDayModel],
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

    // Note that this is currently not chunked and could result in a query that deletes several hundred rows at once.
    let update_one_min_periods = MarketOneMinutePeriodsInLastDayModel::insert_and_delete_periods(
        market_1m_periods,
        conn.clone(),
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
    let latest_state_events = execute_in_chunks(
        conn.clone(),
        insert_market_latest_state_event_query,
        market_latest_state_events,
        get_config_table_chunk_size::<MarketLatestStateEventModel>(
            "market_latest_state_events",
            per_table_chunk_sizes,
        ),
    );

    let (m, s, c, l, per, g, pools, lse, update_1mins) = tokio::join!(
        market_registration,
        swap,
        chat,
        liquidity,
        periodic,
        global,
        lp_pools,
        latest_state_events,
        update_one_min_periods,
    );

    for res in [m, s, c, l, per, g, pools, lse] {
        res?;
    }

    update_1mins?;

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

        let mut register_events_db = vec![];
        let mut swap_events_db = vec![];
        let mut chat_events_db = vec![];
        let mut liquidity_events_db = vec![];
        let mut periodic_state_events_db = vec![];
        let mut global_state_events_db = vec![];
        let mut period_data = vec![];
        // Store the writeset changes for each market in the transaction so we can lazily parse them later only for the
        // latest event for that market. We may get several writeset changes for the same market across all the transactions.
        let mut latest_market_resources: AHashMap<
            i64,
            (TxnInfo, MarketResource, Trigger, InstantaneousStats),
        > = AHashMap::new();
        let mut user_pools_db: AHashMap<(String, i64), UserLiquidityPoolsModel> = AHashMap::new();
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
                            market_events.push(evt.clone());
                            if let Some(one_min_pse) =
                                RecentOneMinutePeriodicStateEvent::try_from_event(evt, txn_version)
                            {
                                period_data.push(one_min_pse);
                            }
                        },
                        _ => {
                            if let Some(global_event) =
                                GlobalStateEvent::from_event_type(type_str, data, txn_version)?
                            {
                                global_state_events_db.push(GlobalStateEventModel::new(
                                    txn_info.clone(),
                                    global_event,
                                ));
                            }
                        },
                    }
                }

                // Keep in mind that these are collecting events and changes within the context of a single transaction,
                // not all transactions.
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
                        market_id,
                        market_nonce,
                        bump_event,
                        state_event,
                        periodic_state_events: periodic_events,
                        txn_info,
                    } = builder.build();

                    periodic_state_events_db.extend(PeriodicStateEventModel::from_periodic_events(
                        txn_info.clone(),
                        periodic_events,
                        state_event.last_swap.clone(),
                    ));

                    let market_addr = &state_event.market_metadata.market_address;

                    latest_market_resources
                        .entry(market_id)
                        .and_modify(
                            |(
                                txn_info_for_latest,
                                latest_resource,
                                latest_trigger,
                                latest_instant_stats,
                            )| {
                                if latest_resource.sequence_info.nonce < market_nonce {
                                    // Writeset changes reflect the final state changes from the transaction; same version == same changes.
                                    if txn_info_for_latest.version != txn_version {
                                        *latest_resource = MarketResource::from_write_set_changes(
                                            txn,
                                            market_addr,
                                        );
                                        *txn_info_for_latest = txn_info.clone();
                                    }
                                    *latest_trigger = state_event.state_metadata.trigger;
                                    *latest_instant_stats = state_event.instantaneous_stats.clone();
                                }
                            },
                        )
                        .or_insert_with(|| {
                            (
                                txn_info.clone(),
                                MarketResource::from_write_set_changes(txn, market_addr),
                                state_event.state_metadata.trigger,
                                state_event.instantaneous_stats.clone(),
                            )
                        });

                    match bump_event {
                        BumpEvent::MarketRegistration(event) => {
                            let mkt_registration_model =
                                MarketRegistrationEventModel::new(txn_info, event, state_event);
                            register_events_db.push(mkt_registration_model);
                        },
                        BumpEvent::Chat(chat) => {
                            chat_events_db.push(ChatEventModel::new(txn_info, chat, state_event));
                        },
                        BumpEvent::Swap(swap) => {
                            let swap_model = SwapEventModel::new(txn_info, swap, state_event);
                            swap_events_db.push(swap_model);
                        },
                        BumpEvent::Liquidity(event) => {
                            let market_addr = market_addr.clone();
                            let evt_model = LiquidityEventModel::new(txn_info, event, state_event);
                            liquidity_events_db.push(evt_model.clone());

                            // Only insert the latest pool activity for a user in this transaction.
                            // That is, if a user interacts multiple times with one pool in one transaction,
                            // only the latest interaction is used to insert/update the user's row for that pool.
                            // Otherwise we'd needlessly overwrite the same row multiple times from one transaction.
                            let key = (evt_model.provider.clone(), evt_model.market_id);
                            let new_pool: UserLiquidityPoolsModel = UserLiquidityPoolsModel::from_event_and_writeset(&txn, evt_model, &market_addr);
                            user_pools_db
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

        let market_latest_state_events = latest_market_resources
            .into_values()
            .map(|(txn_info, market, trigger, instant_stats)| {
                MarketLatestStateEventModel::from_txn_and_market_resource(
                    txn_info,
                    market,
                    trigger,
                    instant_stats,
                )
            })
            .collect_vec();

        let market_1m_periods: Vec<MarketOneMinutePeriodsInLastDayModel> = period_data
            .clone()
            .into_iter()
            .map(|p| p.into())
            .collect_vec();

        let processing_duration_in_secs = processing_start.elapsed().as_secs_f64();
        let db_insertion_start = std::time::Instant::now();

        let all_db_events = vec![
            EmojicoinDbEvent::from_market_registration_events(&register_events_db),
            EmojicoinDbEvent::from_swap_events(&swap_events_db),
            EmojicoinDbEvent::from_chat_events(&chat_events_db),
            EmojicoinDbEvent::from_liquidity_events(&liquidity_events_db),
            EmojicoinDbEvent::from_periodic_state_events(&periodic_state_events_db),
            EmojicoinDbEvent::from_global_state_events(&global_state_events_db),
            EmojicoinDbEvent::from_market_latest_state_events(&market_latest_state_events),
        ]
        .into_iter()
        .flatten()
        .collect_vec();

        self.publish_events(all_db_events);

        let tx_result = insert_to_db(
            self.get_pool(),
            self.name(),
            start_version,
            end_version,
            &register_events_db,
            &swap_events_db,
            &chat_events_db,
            &liquidity_events_db,
            &periodic_state_events_db,
            &global_state_events_db,
            &market_latest_state_events,
            &market_1m_periods,
            user_pools_db.into_values().collect_vec().as_slice(),
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
