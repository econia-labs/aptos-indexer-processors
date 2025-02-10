use crate::{
    db::common::models::emojicoin_models::{
        enums::{EmojicoinTypeTag, Trigger},
        event_utils::EventGroupBuilder,
        json_types::{
            ArenaEvent, BumpEvent, EventGroup, EventWithMarket, GlobalStateEvent,
            InstantaneousStats, MarketResource, SwapEvent, TxnInfo,
        },
        models::{
            arena_enter_event::ArenaEnterEventModel,
            arena_exit_event::ArenaExitEventModel,
            arena_info::{ArenaInfoData, ArenaInfoModel},
            arena_melee_event::ArenaMeleeEventModel,
            arena_position::ArenaPositionDiffModel,
            arena_swap_event::ArenaSwapEventModel,
            arena_vault_balance_update_event::ArenaVaultBalanceUpdateEventModel,
            chat_event::ChatEventModel,
            global_state_event::GlobalStateEventModel,
            liquidity_event::LiquidityEventModel,
            market_1m_periods_in_last_day::MarketOneMinutePeriodsInLastDayModel,
            market_24h_rolling_volume::RecentOneMinutePeriodicStateEvent,
            market_latest_state_event::MarketLatestStateEventModel,
            market_registration_event::MarketRegistrationEventModel,
            periodic_state_event::PeriodicStateEventModel,
            swap_event::SwapEventModel,
            user_liquidity_pools::UserLiquidityPoolsModel,
        },
        queries::insertion_queries::{
            delete_unregistered_markets_query, insert_arena_enter_events_query,
            insert_arena_exit_events_query, insert_arena_info_query,
            insert_arena_leaderboard_history_query, insert_arena_melee_events_query,
            insert_arena_position_query, insert_arena_swap_events_query,
            insert_arena_vault_balance_update_events_query, insert_chat_events_query,
            insert_global_events, insert_liquidity_events_query,
            insert_market_latest_state_event_query, insert_market_registration_events_query,
            insert_periodic_state_events_query, insert_swap_events_query,
            insert_user_liquidity_pools_query, update_arena_info_enter_query,
            update_arena_info_exit_query, update_arena_info_swap_query,
            ArenaLeaderboardHistoryParams,
        },
    },
    emojicoin_dot_fun::EmojicoinDbEvent,
    gap_detectors::ProcessingResult,
    processors::{DefaultProcessingResult, ProcessorName, ProcessorTrait},
    schema,
    utils::{
        counters::PROCESSOR_UNKNOWN_TYPE_COUNT,
        database::{execute_in_chunks, execute_single, get_config_table_chunk_size, ArcDbPool},
        util::{
            bigdecimal_to_u64, get_entry_function_from_user_request, parse_timestamp,
            standardize_address,
        },
    },
};
use ahash::AHashMap;
use anyhow::bail;
use aptos_protos::transaction::v1::{transaction::TxnData, Transaction};
use async_trait::async_trait;
use bigdecimal::BigDecimal;
use diesel::{ExpressionMethods as _, QueryDsl as _};
use diesel_async::RunQueryDsl;
use futures::{future::try_join_all, FutureExt};
use itertools::Itertools;
use num::Zero;
use std::fmt::Debug;
use tokio::sync::mpsc::UnboundedSender;
use tracing::error;

const Q64_DIVISOR: u128 = 2u128.pow(64u32);

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

struct InsertEvents<'a> {
    market_registration_events: &'a [MarketRegistrationEventModel],
    swap_events: &'a [SwapEventModel],
    chat_events: &'a [ChatEventModel],
    liquidity_events: &'a [LiquidityEventModel],
    periodic_state_events: &'a [PeriodicStateEventModel],
    global_state_events: &'a [GlobalStateEventModel],
    market_latest_state_events: &'a [MarketLatestStateEventModel],
    market_1m_periods: &'a [MarketOneMinutePeriodsInLastDayModel],
    user_pools: &'a [UserLiquidityPoolsModel],
    arena_melee_events: &'a [ArenaMeleeEventModel],
    arena_enter_events: &'a [ArenaEnterEventModel],
    arena_exit_events: &'a [ArenaExitEventModel],
    arena_swap_events: &'a [ArenaSwapEventModel],
    arena_vault_balance_update_events: &'a [ArenaVaultBalanceUpdateEventModel],
    arena_position: &'a [ArenaPositionDiffModel],
    arena_info: &'a [ArenaInfoModel],
    arena_leaderboard_history: &'a [ArenaLeaderboardHistoryParams],
}

async fn insert_to_db(
    conn: ArcDbPool,
    name: &'static str,
    start_version: u64,
    end_version: u64,
    insert_events: InsertEvents<'_>,
    per_table_chunk_sizes: &AHashMap<String, usize>,
) -> Result<(), diesel::result::Error> {
    tracing::trace!(
        name = name,
        start_version = start_version,
        end_version = end_version,
        "Inserting to db",
    );
    let InsertEvents {
        market_registration_events,
        swap_events,
        chat_events,
        liquidity_events,
        periodic_state_events,
        global_state_events,
        market_latest_state_events,
        market_1m_periods,
        user_pools,
        arena_melee_events,
        arena_enter_events,
        arena_exit_events,
        arena_swap_events,
        arena_vault_balance_update_events,
        arena_position,
        arena_info,
        arena_leaderboard_history,
    } = insert_events;

    let futures = vec![
        execute_in_chunks(
            conn.clone(),
            insert_market_registration_events_query,
            market_registration_events,
            get_config_table_chunk_size::<MarketRegistrationEventModel>(
                "market_registration_events",
                per_table_chunk_sizes,
            ),
        )
        .boxed(),
        execute_in_chunks(
            conn.clone(),
            delete_unregistered_markets_query,
            market_registration_events,
            get_config_table_chunk_size::<MarketRegistrationEventModel>(
                "unregistered_markets",
                per_table_chunk_sizes,
            ),
        )
        .boxed(),
        // Note that this is currently not chunked and could result in a query that deletes several
        // hundred rows at once.
        MarketOneMinutePeriodsInLastDayModel::insert_and_delete_periods(
            market_1m_periods,
            conn.clone(),
        )
        .boxed(),
        execute_in_chunks(
            conn.clone(),
            insert_swap_events_query,
            swap_events,
            get_config_table_chunk_size::<SwapEventModel>("swap_events", per_table_chunk_sizes),
        )
        .boxed(),
        execute_in_chunks(
            conn.clone(),
            insert_chat_events_query,
            chat_events,
            get_config_table_chunk_size::<ChatEventModel>("chat_events", per_table_chunk_sizes),
        )
        .boxed(),
        execute_in_chunks(
            conn.clone(),
            insert_liquidity_events_query,
            liquidity_events,
            get_config_table_chunk_size::<LiquidityEventModel>(
                "liquidity_events",
                per_table_chunk_sizes,
            ),
        )
        .boxed(),
        execute_in_chunks(
            conn.clone(),
            insert_periodic_state_events_query,
            periodic_state_events,
            get_config_table_chunk_size::<PeriodicStateEventModel>(
                "periodic_state_events",
                per_table_chunk_sizes,
            ),
        )
        .boxed(),
        execute_in_chunks(
            conn.clone(),
            insert_global_events,
            global_state_events,
            get_config_table_chunk_size::<GlobalStateEventModel>(
                "global_state_events",
                per_table_chunk_sizes,
            ),
        )
        .boxed(),
        execute_in_chunks(
            conn.clone(),
            insert_user_liquidity_pools_query,
            user_pools,
            get_config_table_chunk_size::<UserLiquidityPoolsModel>(
                "user_liquidity_pools",
                per_table_chunk_sizes,
            ),
        )
        .boxed(),
        execute_in_chunks(
            conn.clone(),
            insert_market_latest_state_event_query,
            market_latest_state_events,
            get_config_table_chunk_size::<MarketLatestStateEventModel>(
                "market_latest_state_events",
                per_table_chunk_sizes,
            ),
        )
        .boxed(),
        execute_in_chunks(
            conn.clone(),
            insert_arena_position_query,
            arena_position,
            get_config_table_chunk_size::<ArenaPositionDiffModel>(
                "arena_position",
                per_table_chunk_sizes,
            ),
        )
        .boxed(),
        execute_single(
            conn.clone(),
            insert_arena_leaderboard_history_query,
            arena_leaderboard_history,
        )
        .boxed(),
        execute_in_chunks(
            conn.clone(),
            insert_arena_info_query,
            arena_info,
            get_config_table_chunk_size::<ArenaPositionDiffModel>(
                "arena_info",
                per_table_chunk_sizes,
            ),
        )
        .boxed(),
        execute_single(
            conn.clone(),
            update_arena_info_enter_query,
            arena_enter_events,
        )
        .boxed(),
        execute_single(
            conn.clone(),
            update_arena_info_swap_query,
            arena_swap_events,
        )
        .boxed(),
        execute_single(
            conn.clone(),
            update_arena_info_exit_query,
            arena_exit_events,
        )
        .boxed(),
        execute_in_chunks(
            conn.clone(),
            insert_arena_enter_events_query,
            arena_enter_events,
            get_config_table_chunk_size::<ArenaEnterEventModel>(
                "arena_enter_events",
                per_table_chunk_sizes,
            ),
        )
        .boxed(),
        execute_in_chunks(
            conn.clone(),
            insert_arena_exit_events_query,
            arena_exit_events,
            get_config_table_chunk_size::<ArenaExitEventModel>(
                "arena_exit_events",
                per_table_chunk_sizes,
            ),
        )
        .boxed(),
        execute_in_chunks(
            conn.clone(),
            insert_arena_swap_events_query,
            arena_swap_events,
            get_config_table_chunk_size::<ArenaSwapEventModel>(
                "arena_swap_events",
                per_table_chunk_sizes,
            ),
        )
        .boxed(),
        execute_in_chunks(
            conn.clone(),
            insert_arena_vault_balance_update_events_query,
            arena_vault_balance_update_events,
            get_config_table_chunk_size::<ArenaVaultBalanceUpdateEventModel>(
                "arena_vault_balance_update_events",
                per_table_chunk_sizes,
            ),
        )
        .boxed(),
        execute_in_chunks(
            conn.clone(),
            insert_arena_melee_events_query,
            arena_melee_events,
            get_config_table_chunk_size::<ArenaMeleeEventModel>(
                "arena_melee_events",
                per_table_chunk_sizes,
            ),
        )
        .boxed(),
    ];

    try_join_all(futures).await?;

    Ok(())
}

struct MarketData {
    market_id: BigDecimal,
    price: BigDecimal,
    symbol_emojis: Vec<String>,
}

/// Get id, price and symbol emojis for a market.
///
/// If possible, the data will be extracted from the current batch of events.
/// If not, the database will be queried for historical data.
async fn get_market_data(
    market_address_str: &str,
    register_events_db: &Vec<MarketRegistrationEventModel>,
    swap_events_db: &Vec<SwapEventModel>,
    pool: &ArcDbPool,
) -> anyhow::Result<MarketData> {
    // Get market registration event for the market.
    //
    // This will return Some if the market was registered in the current batch of transactions.
    //
    // Although this is highly unlikely, it is possible for a market to be registered then selected
    // for a melee in the same transaction (or in two very close ones). Because of this, the market
    // latest state event could not be yet in the DB, and thus we have to check that it is not in
    // memory.
    let registration = register_events_db
        .iter()
        .find(|r| r.market_address == market_address_str);
    let data = if let Some(registration) = registration {
        // Get latest swap.
        //
        // If no swap is present, but the market was registered in this batch of transactions, we
        // know there are no swaps in the DB either and we can just say that the price is 0.
        let last_swap = swap_events_db
            .iter()
            .rev()
            .find(|s| s.market_id == registration.market_id);

        let price = if let Some(last_swap) = last_swap {
            last_swap.avg_execution_price_q64.clone() / BigDecimal::from(Q64_DIVISOR)
        } else {
            BigDecimal::zero()
        };

        MarketData {
            market_id: registration.market_id.clone(),
            price,
            symbol_emojis: registration.symbol_emojis.clone(),
        }
    } else {
        use schema::market_latest_state_event::dsl::*;
        let conn = &mut pool.get().await?;

        // Since the market was NOT registered in this batch of events, then a mlse MUST be present
        // in the DB. If no swap ever happened on the market, the price field is set to 0.
        let state = market_latest_state_event
            .filter(market_address.eq(market_address_str))
            .select((market_id, symbol_emojis, last_swap_avg_execution_price_q64))
            .first::<(BigDecimal, Vec<Option<String>>, BigDecimal)>(conn)
            .await?;

        // We try to get a swap from the current batch of transactions, in case there's a more
        // recent one.
        let last_swap = swap_events_db.iter().rev().find(|s| s.market_id == state.0);

        let price = if let Some(last_swap) = last_swap {
            last_swap.avg_execution_price_q64.clone()
        } else {
            state.2
        };

        MarketData {
            market_id: state.0,
            price: price / BigDecimal::from(Q64_DIVISOR),
            symbol_emojis: state.1.into_iter().map(|s| s.unwrap()).collect::<Vec<_>>(),
        }
    };
    Ok(data)
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
        let mut arena_melee_events_db = vec![];
        let mut arena_enter_events_db = vec![];
        let mut arena_exit_events_db = vec![];
        let mut arena_swap_events_db = vec![];
        let mut arena_vault_balance_update_events_db = vec![];
        let mut arena_position_db = vec![];
        let mut arena_info_db = vec![];
        let mut arena_leaderboard_history_db = vec![];
        // Store the writeset changes for each market in the transaction so we can lazily parse them later only for the
        // latest event for that market. We may get several writeset changes for the same market across all the transactions.
        let mut latest_market_resources: AHashMap<
            u64,
            (TxnInfo, MarketResource, Trigger, InstantaneousStats),
        > = AHashMap::new();
        let mut user_pools_db: AHashMap<(String, u64), UserLiquidityPoolsModel> = AHashMap::new();
        for txn in &transactions {
            let txn_version = txn.version as i64;
            let block_number = txn.block_height as i64;
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
                    block_number,
                    version: txn_version,
                    sender: standardize_address(user_request.sender.as_ref()),
                    entry_function,
                    timestamp: parse_timestamp(txn.timestamp.as_ref().unwrap(), txn_version),
                };

                // Group the market events in this transaction.
                let mut market_events = vec![];

                // Stock the two latest swap events.
                // When an arena swap event is encountered in the for loop, this variable will
                // contain the two corrensponding normal swap events.
                let mut last_swaps: (Option<SwapEvent>, Option<SwapEvent>) = (None, None);

                for (event_index, event) in user_txn.events.iter().enumerate() {
                    let type_str = event.type_str.as_str();
                    let data = event.data.as_str();

                    // Only parse events that match an `EmojicoinTypeTag`. This protects against
                    // parsing invalid or unexpected JSON data.
                    if EmojicoinTypeTag::from_type_str(type_str).is_some() {
                        // If it's an event with a market, parse it and add it to `market_events`
                        // and possibly the one minute periodic state events.
                        if let Some(evt) = EventWithMarket::from_event_type(
                            type_str,
                            data,
                            txn_version,
                            event_index as i64,
                        )? {
                            if let EventWithMarket::Swap(swap) = evt.clone() {
                                last_swaps = (last_swaps.1, Some(swap));
                            }
                            market_events.push(evt.clone());
                            if let Some(one_min_pse) =
                                RecentOneMinutePeriodicStateEvent::try_from_event(evt, txn_version)
                            {
                                period_data.push(one_min_pse);
                            }
                        // If it's an arena event, parse it and add it to the proper arena events vector.
                        } else if let Some(evt) = ArenaEvent::from_event_type(
                            type_str,
                            data,
                            txn_version,
                            event_index as i64,
                        )? {
                            match evt {
                                ArenaEvent::Melee(melee) => {
                                    arena_melee_events_db.push(ArenaMeleeEventModel::new(
                                        txn_info.clone(),
                                        melee.clone(),
                                    ));
                                },
                                ArenaEvent::Enter(enter) => {
                                    arena_position_db
                                        .push(ArenaPositionDiffModel::from(enter.clone()));
                                    arena_enter_events_db
                                        .push(ArenaEnterEventModel::new(txn_info.clone(), enter))
                                },
                                ArenaEvent::Exit(exit) => {
                                    arena_position_db
                                        .push(ArenaPositionDiffModel::from(exit.clone()));
                                    arena_exit_events_db
                                        .push(ArenaExitEventModel::new(txn_info.clone(), exit))
                                },
                                ArenaEvent::Swap(swap) => {
                                    let swaps = (last_swaps.0.unwrap(), last_swaps.1.unwrap());
                                    // This checks that the two previous swaps do indeed
                                    // correnspond to an arena swap. If stars align, two unrelated
                                    // swaps (not part of an arena swap) from the same transaction
                                    // could have net proceeds equal to input amount, and the
                                    // second swap could have net proceeds equal to the net
                                    // proceeds of the arena swap, but it is highly unlikely.
                                    // Moreover, this check is a "just to be sure" check: in
                                    // theory, last_swaps should always contain the correct swaps
                                    // due to the way events are emitted.
                                    if swaps.0.net_proceeds != swaps.1.input_amount
                                        && (swap.emojicoin_0_proceeds == swaps.1.net_proceeds
                                            || swap.emojicoin_0_proceeds == swaps.1.net_proceeds)
                                    {
                                        bail!("The two previous swaps to an arena swap are not related to the arena swap.");
                                    }
                                    // ArenaPositionModel::from_swap expects (swap_emojicoin_0,
                                    // swap_emojicoin_1).
                                    let swaps = if swap.emojicoin_0_proceeds > BigDecimal::zero() {
                                        (swaps.1, swaps.0)
                                    } else {
                                        swaps
                                    };
                                    arena_position_db.push(ArenaPositionDiffModel::from_swap(
                                        swap.clone(),
                                        swaps,
                                    ));
                                    last_swaps = (None, None);
                                    arena_swap_events_db
                                        .push(ArenaSwapEventModel::new(txn_info.clone(), swap))
                                },
                                ArenaEvent::VaultBalanceUpdate(vault_balance_update) => {
                                    arena_vault_balance_update_events_db.push(
                                        ArenaVaultBalanceUpdateEventModel::new(
                                            txn_info.clone(),
                                            vault_balance_update,
                                        ),
                                    )
                                },
                            }
                        // If it's a global state event, parse and add it to the global state events vector.
                        } else if let Some(global_event) =
                            GlobalStateEvent::from_event_type(type_str, data, txn_version)?
                        {
                            global_state_events_db
                                .push(GlobalStateEventModel::new(txn_info.clone(), global_event));
                        }
                    }
                }

                // Keep in mind that these are collecting events and changes within the context of a single transaction,
                // not all transactions.
                let mut builders: AHashMap<(u64, u64), EventGroupBuilder> = AHashMap::new();
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

                    // A market resource in a transaction changeset will *always* contain the latest
                    // market state for that transaction by virtue of the writeset reflecting the
                    // final state of the market at the end of the transaction.
                    //
                    // Thus, the boolean condition to enter the `and_modify` code block below must
                    // use `<=` to ensure that in the case where the event with a lower nonce is
                    // inserted into the hashmap with `or_insert_with` first, the `latest_trigger`
                    // and `latest_instant_stats` are still properly updated.
                    //
                    // These comparisons remove the need to parse the writeset for every single
                    // event and instead only parse it for events that are newer than what's
                    // currently in the hashamp for that market.
                    latest_market_resources
                        .entry(market_id)
                        .and_modify(
                            |(
                                txn_info_for_latest,
                                latest_resource,
                                latest_trigger,
                                latest_instant_stats,
                            )| {
                                if bigdecimal_to_u64(&latest_resource.sequence_info.nonce)
                                    <= market_nonce
                                {
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
                            let mkt_registration_model = MarketRegistrationEventModel::new(
                                txn_info.clone(),
                                event,
                                state_event,
                            );
                            register_events_db.push(mkt_registration_model);
                        },
                        BumpEvent::Chat(chat) => {
                            chat_events_db.push(ChatEventModel::new(
                                txn_info.clone(),
                                chat,
                                state_event,
                            ));
                        },
                        BumpEvent::Swap(swap) => {
                            let swap_model =
                                SwapEventModel::new(txn_info.clone(), swap, state_event);
                            swap_events_db.push(swap_model);
                        },
                        BumpEvent::Liquidity(event) => {
                            let market_addr = market_addr.clone();
                            let evt_model =
                                LiquidityEventModel::new(txn_info.clone(), event, state_event);
                            liquidity_events_db.push(evt_model.clone());

                            // Only insert the latest pool activity for a user in this transaction.
                            // That is, if a user interacts multiple times with one pool in one transaction,
                            // only the latest interaction is used to insert/update the user's row for that pool.
                            // Otherwise we'd needlessly overwrite the same row multiple times from one transaction.
                            let key = (
                                evt_model.provider.clone(),
                                bigdecimal_to_u64(&evt_model.market_id),
                            );
                            let new_pool: UserLiquidityPoolsModel =
                                UserLiquidityPoolsModel::from_event_and_writeset(
                                    txn,
                                    evt_model,
                                    &market_addr,
                                );
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

        let pool = self.get_pool();
        for melee in &arena_melee_events_db {
            let market_data_0 = get_market_data(
                &melee.emojicoin_0_market_address,
                &register_events_db,
                &swap_events_db,
                &pool,
            );
            let market_data_1 = get_market_data(
                &melee.emojicoin_1_market_address,
                &register_events_db,
                &swap_events_db,
                &pool,
            );
            let (market_data_0, market_data_1) = tokio::try_join!(market_data_0, market_data_1)?;
            arena_leaderboard_history_db.push(ArenaLeaderboardHistoryParams {
                melee_id_value: melee.melee_id.clone() - 1,
                emojicoin_0_price: market_data_0.price,
                emojicoin_1_price: market_data_1.price,
            });

            let arena_info_data = ArenaInfoData {
                emojicoin_0_market_id: market_data_0.market_id,
                emojicoin_1_market_id: market_data_1.market_id,
                emojicoin_0_symbols: market_data_0.symbol_emojis,
                emojicoin_1_symbols: market_data_1.symbol_emojis,
            };
            let arena_info = ArenaInfoModel::new(melee.clone(), arena_info_data);
            arena_info_db.push(arena_info);
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
            EmojicoinDbEvent::from_arena_melee(&arena_melee_events_db),
            EmojicoinDbEvent::from_arena_enter(&arena_enter_events_db),
            EmojicoinDbEvent::from_arena_exit(&arena_exit_events_db),
            EmojicoinDbEvent::from_arena_swap(&arena_swap_events_db),
            EmojicoinDbEvent::from_arena_vault_balance_update(
                &arena_vault_balance_update_events_db,
            ),
        ]
        .into_iter()
        .flatten()
        .collect_vec();

        self.publish_events(all_db_events);

        let tx_result = insert_to_db(
            pool,
            self.name(),
            start_version,
            end_version,
            InsertEvents {
                market_registration_events: &register_events_db,
                swap_events: &swap_events_db,
                chat_events: &chat_events_db,
                liquidity_events: &liquidity_events_db,
                periodic_state_events: &periodic_state_events_db,
                global_state_events: &global_state_events_db,
                market_latest_state_events: &market_latest_state_events,
                market_1m_periods: &market_1m_periods,
                user_pools: user_pools_db.into_values().collect_vec().as_slice(),
                arena_melee_events: &arena_melee_events_db,
                arena_enter_events: &arena_enter_events_db,
                arena_exit_events: &arena_exit_events_db,
                arena_swap_events: &arena_swap_events_db,
                arena_vault_balance_update_events: &arena_vault_balance_update_events_db,
                arena_position: &arena_position_db,
                arena_info: &arena_info_db,
                arena_leaderboard_history: &arena_leaderboard_history_db,
            },
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
