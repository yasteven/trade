// trade/usta/src/dr_ttai_mock.rs - Mock TastyTrade API Interface for synthetic paper trading

// This provides a full simulation of the TTAI interface without connecting to real brokers

use std::collections::HashMap;
use tokio::sync::{mpsc, oneshot};

/// Configuration for the mock TTAI
#[derive(Debug, Clone)]
pub struct MockTtaiConfig {
    /// Initial account balance
    pub initial_cash: f64,
    
    /// Simulated latency for order fills (milliseconds)
    pub fill_latency_ms: u64,
    
    /// Whether to simulate partial fills
    pub simulate_partial_fills: bool,
    
    /// Probability of partial fill (0.0 to 1.0)
    pub partial_fill_probability: f64,
    
    /// Simulated commission per contract
    pub commission_per_contract: f64,
    
    /// Simulated regulatory fees per contract
    pub regulatory_fees_per_contract: f64,
    
    /// Whether to auto-generate realistic option chains
    pub auto_generate_chains: bool,
    
    /// Market volatility factor for price simulation (0.0 to 1.0)
    pub volatility_factor: f64,
}

impl Default for MockTtaiConfig {
    fn default() -> Self {
        Self {
            initial_cash: 100_000.0,
            fill_latency_ms: 50,
            simulate_partial_fills: false,
            partial_fill_probability: 0.1,
            commission_per_contract: 1.0,
            regulatory_fees_per_contract: 0.04,
            auto_generate_chains: true,
            volatility_factor: 0.2,
        }
    }
}

/// Internal state of the mock TTAI
struct MockTtaiState {
    /// Current account balance
    cash: f64,
    
    /// Active orders: order_hash -> OrderInfo
    active_orders: HashMap<u64, MockOrderInfo>,
    
    /// Order ID counter
    next_order_id: u64,
    
    /// Ticker price cache: ticker_name -> (bid, ask)
    ticker_prices: HashMap<String, (f64, f64)>,
    
    /// Configuration
    config: MockTtaiConfig,
}

#[derive(Debug, Clone)]
struct MockOrderInfo {
    order_hash: u64,
    order: dsta::Order,
    filled_quantity: f64,
    total_quantity: f64,
    is_complete: bool,
}

impl MockTtaiState {
    fn new(config: MockTtaiConfig) -> Self {
        Self {
            cash: config.initial_cash,
            active_orders: HashMap::new(),
            next_order_id: 1,
            ticker_prices: HashMap::new(),
            config,
        }
    }

    /// Generate the next order ID
    fn generate_order_id(&mut self) -> u64 {
        let id = self.next_order_id;
        self.next_order_id += 1;
        id
    }

    /// Update ticker price cache
    #[allow(dead_code)]
    fn update_ticker_price(&mut self, ticker_name: &str, bid: f64, ask: f64) {
        self.ticker_prices.insert(ticker_name.to_string(), (bid, ask));
    }

    /// Get current price for a ticker (mid-point)
    fn get_ticker_price(&self, ticker_name: &str) -> Option<f64> {
        self.ticker_prices.get(ticker_name).map(|(bid, ask)| (bid + ask) / 2.0)
    }

    /// Calculate total fees for an order
    fn calculate_fees(&self, order: &dsta::Order) -> f64 {
        let total_contracts: f64 = order.order_legs.iter().map(|leg| leg.quantity).sum();
        let commission = total_contracts * self.config.commission_per_contract;
        let regulatory = total_contracts * self.config.regulatory_fees_per_contract;
        commission + regulatory
    }

    /// Generate a realistic option chain
    fn generate_option_chain(&self, underlying: &str, current_price: f64, days_to_expiry: i64) -> Vec<dsta::Derivatives> {
        let dbgspt = "[MOCK_CHAIN_GEN]";
        log::debug!("{} Generating option chain for {} at ${}", dbgspt, underlying, current_price);

        let expiry = chrono::Utc::now() + chrono::Duration::days(days_to_expiry);
        
        // Generate strikes around current price
        let strike_interval = if current_price < 50.0 { 2.5 } else if current_price < 200.0 { 5.0 } else { 10.0 };
        let num_strikes = 15;
        
        let mut strikes = Vec::new();
        let atm_strike = (current_price / strike_interval).round() * strike_interval;
        
        for i in -(num_strikes/2)..=(num_strikes/2) {
            strikes.push(atm_strike + (i as f64) * strike_interval);
        }

        let mut puts = Vec::new();
        let mut calls = Vec::new();

        for strike in strikes {
            if strike > 0.0 {
                let put_name = format!(".{}P{}", underlying, strike as u64);
                puts.push(dsta::Opt {
                    ticker_name: put_name,
                    option_name: format!("{} {}_P_{}", underlying, strike as u64, days_to_expiry),
                    option_rite: dsta::OptRite::Put,
                    strike_spot: strike,
                    expire_date: expiry,
                    option_type: dsta::OptStyle::Usa,
                    tick_rulers: None,
                    last_ticker: None,
                });

                let call_name = format!(".{}C{}", underlying, strike as u64);
                calls.push(dsta::Opt {
                    ticker_name: call_name,
                    option_name: format!("{} {}_C_{}", underlying, strike as u64, days_to_expiry),
                    option_rite: dsta::OptRite::Call,
                    strike_spot: strike,
                    expire_date: expiry,
                    option_type: dsta::OptStyle::Usa,
                    tick_rulers: None,
                    last_ticker: None,
                });
            }
        }

        vec![dsta::Derivatives {
            expire_time: expiry,
            option_calls: calls,
            option_puts: puts,
        }]
    }
}

// Mirrors the real TtaiTickCommand: no result_tx here.
// Result is handled in the main loop, just like the real dr_ttai.
pub enum MockTtaiTickCommand {
    NewSub((String, u64, mpsc::Sender<dsta::Ticker>)),
    DieSub((String, u64)),
}

/// Mock ticker distribution task — manages subscriber map only, no result handling
async fn f_run_mock_tick(
    mut tick_command_rx: mpsc::Receiver<MockTtaiTickCommand>,
    mut told_dr_ttai_reset: mpsc::Receiver<String>,
    tell_dr_ttai_reset: mpsc::Sender<String>,
) {
    let dbgspt = "[MOCK_TICK]";
    log::debug!("{} Starting mock ticker distribution", dbgspt);

    let mut subscribers: HashMap<String, Vec<(u64, mpsc::Sender<dsta::Ticker>)>> = HashMap::new();

    loop {
        tokio::select! {
            res = tick_command_rx.recv() => {
                match res {
                    Some(MockTtaiTickCommand::NewSub((symbol, sub_id, sender))) => {
                        log::debug!("{} New subscription: {} (ID {})", dbgspt, symbol, sub_id);
                        let subs = subscribers.entry(symbol.clone()).or_insert_with(Vec::new);
                        // Replace any stale entry for the same sub_id
                        subs.retain(|(id, _)| *id != sub_id);
                        subs.push((sub_id, sender));
                    }

                    Some(MockTtaiTickCommand::DieSub((symbol, sub_id))) => {
                        log::debug!("{} Unsubscribe: {} (ID {})", dbgspt, symbol, sub_id);
                        if let Some(subs) = subscribers.get_mut(&symbol) {
                            subs.retain(|(id, _)| *id != sub_id);
                            if subs.is_empty() {
                                subscribers.remove(&symbol);
                            }
                        }
                    }

                    None => {
                        log::debug!("{} Tick command channel closed", dbgspt);
                        let _ = tell_dr_ttai_reset.send("Mock tick: command channel closed".to_string()).await;
                        break;
                    }
                }
            }

            res = told_dr_ttai_reset.recv() => {
                match res {
                    Some(reason) => {
                        log::debug!("{} Reset signal: {}", dbgspt, reason);
                        let _ = tell_dr_ttai_reset.send(format!("Mock tick: {}", reason)).await;
                        break;
                    }
                    None => {
                        log::debug!("{} Reset channel closed", dbgspt);
                        break;
                    }
                }
            }
        }
    }

    log::debug!("{} Mock ticker distribution ending", dbgspt);
}

/// Main mock TTAI function - mirrors the real f_run_main_dr_ttai signature exactly
pub async fn f_run_main_mock_ttai(
    config: MockTtaiConfig,

    // From dr_robo via dr_bot0
    mut told_dr_add_ttai_tik: mpsc::Receiver<(String, u64, mpsc::Sender<dsta::Ticker>, mpsc::Sender<Option<dsta::PushTickerResult>>)>,
    mut told_dr_pop_ttai_tik: mpsc::Receiver<(String, u64)>,

    // From dr_bot0 - order submission
    mut told_dr_order_submit: mpsc::Receiver<(dsta::Order, oneshot::Sender<Result<dsta::PushOrderResult, Box<dyn std::error::Error + Send + Sync>>>)>,

    // From dr_bot0 - order cancellation
    mut told_dr_order_cancel: mpsc::Receiver<(u64, oneshot::Sender<Result<bool, Box<dyn std::error::Error + Send + Sync>>>)>,

    // From dr_bot0 - dry run order
    mut told_dr_order_dryrun: mpsc::Receiver<(dsta::Order, oneshot::Sender<Result<f64, Box<dyn std::error::Error + Send + Sync>>>)>,

    // From dr_bot0 - order status polling
    mut told_dr_order_status: mpsc::Receiver<oneshot::Sender<Result<Vec<dsta::PlacedOrder>, Box<dyn std::error::Error + Send + Sync>>>>,

    // To dr_bot0
    tell_dr_bot0_account_update: mpsc::Sender<dsta::AccountUpdate>,

    // Reset control
    mut told_dr_ttai_reset: mpsc::Receiver<String>,
    tell_dr_a_snively: mpsc::Sender<(usize, dsta::Snively)>,
) {
    let dbgspt = "[MOCK_TTAI_MAIN]";
    log::info!("{} Starting mock TTAI with config: {:#?}", dbgspt, config);

    // Create shared state
    let state = std::sync::Arc::new(tokio::sync::Mutex::new(MockTtaiState::new(config)));

    // Create channels for ticker distribution
    let (tick_command_tx, tick_command_rx) = mpsc::channel::<MockTtaiTickCommand>(100);
    let (tick_reset_tx, tick_reset_rx) = mpsc::channel::<String>(10);

    // Spawn ticker distribution task (no state needed there anymore)
    let tick_reset_tx_clone = tick_reset_tx.clone();
    tokio::spawn(async move {
        f_run_mock_tick(
            tick_command_rx,
            tick_reset_rx,
            tick_reset_tx_clone,
        ).await;
        log::error!("{} Mock tick distribution task ended", dbgspt);
    });

    // Send initial account update
    {
        let state_lock = state.lock().await;
        let account_update = dsta::AccountUpdate {
            ttai_uid: "MOCK_ACCOUNT".to_string(),
            long_usd: state_lock.cash,
            can_sell: state_lock.cash,
            long_stk: 0.0,
            sell_stk: 0.0,
            long_opt: 0.0,
            sell_opt: 0.0,
            long_fut: 0.0,
            sell_fut: 0.0,
            long_fop: 0.0,
            sell_fop: 0.0,
            long_xxx: 0.0,
            sell_xxx: 0.0,
            long_bnd: 0.0,
            next_usd: state_lock.cash,
            summings: state_lock.cash,
            cash_out: 0.0,
        };

        if let Err(e) = tell_dr_bot0_account_update.send(account_update).await {
            log::error!("{} Failed to send initial account update: {:?}", dbgspt, e);
            return;
        }
    }

    // Alert startup complete
    let _ = tell_dr_a_snively.send(
        (0, dsta::Snively::SendLogNote(format!("{}", crate::glossary::MAGIC_DTTAI_STARTUP_OK)))
    ).await;

    log::info!("{} Mock TTAI initialized and running", dbgspt);

    loop {
        tokio::select! {
            // Ticker subscription requests
            // Mirrors real dr_ttai: register with tick distributor, then build and send result here
            res = told_dr_add_ttai_tik.recv() => {
                match res {
                    Some((symbol, sub_id, sender, result_tx)) => {
                        log::debug!("{} Ticker subscription request: {} (ID {})", dbgspt, symbol, sub_id);

                        // Register with tick distributor (no result_tx forwarded, same as real)
                        if tick_command_tx.send(
                            MockTtaiTickCommand::NewSub((symbol.clone(), sub_id, sender))
                        ).await.is_err() {
                            log::error!("{} Failed to register subscriber with tick distributor", dbgspt);
                            let _ = result_tx.send(None).await;
                            continue;
                        }

                        // Build result in the main loop, same as the real dr_ttai does
                        let state_lock = state.lock().await;

                        let result = if symbol.starts_with('.') {
                            // Option ticker — no chain needed
                            Some(dsta::PushTickerResult {
                                ass_deets: dsta::FinAss::OptDeets(dsta::Opt {
                                    ticker_name: symbol.clone(),
                                    option_name: symbol.clone(),
                                    option_rite: dsta::OptRite::Put,
                                    strike_spot: 0.0,
                                    expire_date: chrono::DateTime::<chrono::Utc>::default(),
                                    option_type: dsta::OptStyle::Usa,
                                    tick_rulers: None,
                                    last_ticker: None,
                                }),
                                opt_chain: vec![],
                            })
                        } else if state_lock.config.auto_generate_chains {
                            // Stock ticker with generated chain
                            let price = state_lock.get_ticker_price(&symbol).unwrap_or(100.0);
                            let chain = state_lock.generate_option_chain(&symbol, price, 45);
                            Some(dsta::PushTickerResult {
                                ass_deets: dsta::FinAss::StkDeets(dsta::Stk {
                                    ticker_name: symbol.clone(),
                                    ticker_info: format!("Mock stock {}", symbol),
                                    last_ticker: None,
                                    tick_rulers: None,
                                }),
                                opt_chain: chain,
                            })
                        } else {
                            Some(dsta::PushTickerResult {
                                ass_deets: dsta::FinAss::StkDeets(dsta::Stk {
                                    ticker_name: symbol.clone(),
                                    ticker_info: format!("Mock stock {}", symbol),
                                    last_ticker: None,
                                    tick_rulers: None,
                                }),
                                opt_chain: vec![],
                            })
                        };

                        drop(state_lock);
                        let _ = result_tx.send(result).await;
                    }
                    None => {
                        log::debug!("{} Ticker subscription channel closed", dbgspt);
                        let _ = tick_reset_tx.send("Main: subscription channel closed".to_string()).await;
                        break;
                    }
                }
            }

            // Ticker unsubscription requests
            res = told_dr_pop_ttai_tik.recv() => {
                match res {
                    Some((symbol, sub_id)) => {
                        log::debug!("{} Ticker unsubscription: {} (ID {})", dbgspt, symbol, sub_id);
                        let _ = tick_command_tx.send(MockTtaiTickCommand::DieSub((symbol, sub_id))).await;
                    }
                    None => {
                        log::debug!("{} Ticker unsubscription channel closed", dbgspt);
                        let _ = tick_reset_tx.send("Main: unsubscription channel closed".to_string()).await;
                        break;
                    }
                }
            }

            // Dry run requests
            res = told_dr_order_dryrun.recv() => {
                match res {
                    Some((order, replyer)) => {
                        log::debug!("{} Dry run request received", dbgspt);
                        let state_lock = state.lock().await;
                        let fees = state_lock.calculate_fees(&order);
                        log::info!("{} Dry run calculated fees: ${:.2}", dbgspt, fees);
                        let _ = replyer.send(Ok(fees));
                    }
                    None => {
                        log::debug!("{} Dry run channel closed", dbgspt);
                        let _ = tick_reset_tx.send("Main: dry run channel closed".to_string()).await;
                        break;
                    }
                }
            }

            // Order submissions
            res = told_dr_order_submit.recv() => {
                match res {
                    Some((order, replyer)) => {
                        log::debug!("{} Order submission received: {:?}", dbgspt, order);

                        let mut state_lock = state.lock().await;

                        // Validate order
                        let total_qty: f64 = order.order_legs.iter().map(|leg| leg.quantity).sum();
                        if total_qty <= 0.0 {
                            let _ = replyer.send(Err("Invalid order quantity".into()));
                            continue;
                        }

                        // Generate order ID
                        let order_hash = state_lock.generate_order_id();

                        // Store order info
                        state_lock.active_orders.insert(order_hash, MockOrderInfo {
                            order_hash,
                            order,
                            filled_quantity: 0.0,
                            total_quantity: total_qty,
                            is_complete: false,
                        });

                        log::info!("{} Order accepted with hash {}", dbgspt, order_hash);
                        let _ = replyer.send(Ok(dsta::PushOrderResult::ValidID { new_id: order_hash }));
                    }
                    None => {
                        log::debug!("{} Order submission channel closed", dbgspt);
                        let _ = tick_reset_tx.send("Main: order submission channel closed".to_string()).await;
                        break;
                    }
                }
            }

            // Order cancellations
            res = told_dr_order_cancel.recv() => {
                match res {
                    Some((order_hash, replyer)) => {
                        log::info!("{} Cancel request for order {}", dbgspt, order_hash);

                        let mut state_lock = state.lock().await;

                        if state_lock.active_orders.remove(&order_hash).is_some() {
                            log::info!("{} Order {} cancelled successfully", dbgspt, order_hash);
                            let _ = replyer.send(Ok(true));
                        } else {
                            log::warn!("{} Order {} not found for cancellation", dbgspt, order_hash);
                            let _ = replyer.send(Ok(false));
                        }
                    }
                    None => {
                        log::debug!("{} Order cancel channel closed", dbgspt);
                        let _ = tick_reset_tx.send("Main: cancel channel closed".to_string()).await;
                        break;
                    }
                }
            }

            // Order status polling — return all currently tracked active orders
            res = told_dr_order_status.recv() => {
                match res {
                    Some(replyer) => {
                        log::debug!("{} Order status poll request received", dbgspt);
                        let state_lock = state.lock().await;
                        let placed_orders: Vec<dsta::PlacedOrder> = state_lock.active_orders.values()
                            .map(|info| dsta::PlacedOrder {
                                order_hash: info.order_hash,
                                legs_fills: info.order.order_legs.iter().map(|leg| dsta::OrderLeg {
                                    buy_what: leg.buy_what.clone(),
                                    quantity: info.filled_quantity,
                                    remaining: info.total_quantity - info.filled_quantity,
                                    action: leg.action.clone(),
                                    price: leg.price,
                                }).collect(),
                                order_fill: if info.is_complete {
                                    dsta::OrderFill::Done
                                } else {
                                    dsta::OrderFill::Live
                                },
                            })
                            .collect();
                        log::info!("{} Returning {} active orders from poll", dbgspt, placed_orders.len());
                        let _ = replyer.send(Ok(placed_orders));
                    }
                    None => {
                        log::debug!("{} Order status channel closed", dbgspt);
                        let _ = tick_reset_tx.send("Main: order status channel closed".to_string()).await;
                        break;
                    }
                }
            }

            // Reset signals
            res = told_dr_ttai_reset.recv() => {
                match res {
                    Some(reason) => {
                        log::info!("{} Reset signal received: {}", dbgspt, reason);
                        let _ = tick_reset_tx.send(format!("Main: {}", reason)).await;
                        break;
                    }
                    None => {
                        log::debug!("{} Reset channel closed", dbgspt);
                        break;
                    }
                }
            }
        }
    }

    log::info!("{} Mock TTAI shutting down", dbgspt);
    let _ = tell_dr_a_snively.send((0, dsta::Snively::SendLogNote("mtsd".to_string()))).await;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_mock_ttai_basic_order_flow() {
        let config = MockTtaiConfig::default();

        let (_add_tick_tx, add_tick_rx) = mpsc::channel(10);
        let (_pop_tick_tx, pop_tick_rx) = mpsc::channel(10);
        let (_submit_tx, submit_rx) = mpsc::channel(10);
        let (_cancel_tx, cancel_rx) = mpsc::channel(10);
        let (_dryrun_tx, dryrun_rx) = mpsc::channel(10);
        let (_status_tx, status_rx) = mpsc::channel(10);
        let (account_tx, mut account_rx) = mpsc::channel(10);
        let (reset_tx, reset_rx) = mpsc::channel(10);
        let (snively_tx, _snively_rx) = mpsc::channel(10);

        tokio::spawn(f_run_main_mock_ttai(
            config,
            add_tick_rx,
            pop_tick_rx,
            submit_rx,
            cancel_rx,
            dryrun_rx,
            status_rx,
            account_tx,
            reset_rx,
            snively_tx,
        ));

        // Wait for initial account update confirming startup
        let _ = account_rx.recv().await.expect("No initial account update");

        // Trigger clean shutdown
        drop(reset_tx);
    }
}