//==================================================================
// trade/dsta/src/tests.rs - Integration tests for ttrs -> dsta conversions
//==================================================================
use std::collections::HashMap;
use std::fs;
use std::time::Instant;
use serde_json::Value;

    use super::*;

    fn setuplog() {
        let _ = env_logger::builder()
            .is_test(true)
            .try_init();
    }

    fn load_creds() -> Value {
        setuplog();
        let creds_str = include_str!("../../ttrs/tests/test-creds.json");
        let res: Value = serde_json::from_str(creds_str).expect("Invalid test_creds.json");
        log::info!("Loaded credentials!");

        if res["client_id"]
            .as_str()
            .map(|s| s.is_empty())
            .unwrap_or(true)
        {
            panic!("client_id is missing or empty in test-creds.json");
        }
        if res["client_secret"]
            .as_str()
            .map(|s| s.is_empty())
            .unwrap_or(true)
        {
            panic!("client_secret is missing or empty in test-creds.json");
        }
        if res["refresh_token"]
            .as_str()
            .map(|s| s.is_empty())
            .unwrap_or(true)
        {
            panic!("refresh_token is missing or empty in test-creds.json");
        }

        log::info!("All required credentials present");
        res
    }

    fn create_tests_dir() {
        let tests_dir = std::path::Path::new("tests");
        if !tests_dir.exists() {
            fs::create_dir(tests_dir).expect("Failed to create tests directory");
        }
    }

    fn write_timing_file(test_name: &str, duration_ms: u128) {
        return();
        
        create_tests_dir();
        let file_name = format!(
            "timing_for_test_{}_was_{}_milliseconds.txt",
            test_name.replace("::", "_"),
            duration_ms
        );
        let content = format!(
            "Test '{}' completed in {} milliseconds.",
            test_name, duration_ms
        );
        fs::write(format!("tests/{}", file_name), content)
            .expect("Failed to write timing file");
    }

fn stuffmap() ->  std::collections::HashMap<String,crate::FinAss>
{
  let expiry = chrono::Utc::now() + chrono::Duration::days(35);
  let mut stuff: std::collections::HashMap<String,crate::FinAss> = HashMap::new();
  let long_put = crate::Opt
  { ticker_name:".FAKE 95_P_35".to_string(),
    option_name:"FAKE 95_P_35".to_string(),
    option_rite:OptRite::Put,
    strike_spot:95.0,
    expire_date:expiry,
    option_type:OptStyle::Usa,
    tick_rulers:None,
    last_ticker:None,
  };
  let short_put = Opt
  { ticker_name:".FAKE 100_P_35".to_string(),
    option_name:"FAKE 100_P_35".to_string(),
    option_rite:OptRite::Put,
    strike_spot:100.0,
    expire_date:expiry,
    option_type:OptStyle::Usa,
    tick_rulers:None,
    last_ticker:None,
  };
  let stoke = crate::Stk
   { ticker_name:"FAKE".to_string(),
     ticker_info:format!(""),
    last_ticker:None,
    tick_rulers:None,
  };

  stuff.insert(long_put.option_name.clone(),FinAss::OptDeets(long_put));
  stuff.insert(short_put.option_name.clone(),FinAss::OptDeets(short_put));
  stuff.insert(stoke.ticker_name.clone(),FinAss::StkDeets(stoke));
  stuff
}

// ========================================================================
// Test 0: Futures Option Chain Diagnostic (/ESH6)
// ========================================================================
#[tokio::test]
async fn test_opt_es_option_chain_parsing() {
    let start = Instant::now();
    setuplog();
    log::debug!("ENTRY! test_es_option_chain_parsing()...");

    let creds = load_creds();
    let conn_info = ttrs::ConnectionInfo {
        base_url: "https://api.tastyworks.com".to_string(),
        oauth: (
            creds["client_id"].as_str().unwrap_or("").to_string(),
            creds["client_secret"].as_str().unwrap_or("").to_string(),
            creds["refresh_token"].as_str().unwrap_or("").to_string(),
        ),
    };

    let (error_tx, mut error_rx) = tokio::sync::mpsc::unbounded_channel();
    let (_shutdown_tx, shutdown_rx) = tokio::sync::mpsc::channel::<String>(1);
    let config = ttrs::CoreConfig { error_tx, shutdown_rx };

    let (thand, tfoot) = ttrs::make_core_api(100);
    let core_handle = tokio::spawn(ttrs::fn_run_core(tfoot, conn_info, config));

    // Wait for core to initialize
    tokio::time::sleep(std::time::Duration::from_millis(300)).await;

    log::trace!("\n=== Starting /ESH6 Option Chain Diagnostic Test ===\n");

    let symbol = "/ESH6";
    log::info!("Requesting option chain for: {}", symbol);

    match thand.tell_tastytrade_to_get_option_chain(symbol.to_string()).await {
        Ok(chain_data) => {
            log::info!("✓ Successfully fetched option chain data");
            log::info!("  Items count: {}", chain_data.items.len());

            if !chain_data.items.is_empty() {
                let first_item = &chain_data.items[0];
                log::info!("  First item root: {}", first_item.root_symbol);
                log::info!("  First item underlying: {}", first_item.underlying_symbol);
                log::info!("  Expirations: {}", first_item.expirations.len());
                log::info!("  Tick sizes: {}", first_item.tick_sizes.len());

                if !first_item.expirations.is_empty() {
                    let first_exp = &first_item.expirations[0];
                    log::info!(
                        "  First expiration: {} ({} DTE, {} strikes)",
                        first_exp.expiration_date,
                        first_exp.days_to_expiration,
                        first_exp.strikes.len()
                    );

                    if !first_exp.strikes.is_empty() {
                        let first_strike = &first_exp.strikes[0];
                        log::info!(
                            "    Strike: {} (call: {}, put: {})",
                            first_strike.strike_price,
                            first_strike.call,
                            first_strike.put
                        );
                    }
                }
            }

            // Now convert to derivatives
            log::trace!("\nConverting OptionChainData to Derivatives...");
            let derivatives = crate::convert::option_chain_data_to_derivatives(&chain_data);
            log::info!("✓ Converted to {} Derivatives objects", derivatives.len());

            if !derivatives.is_empty() {
                let first_deriv = &derivatives[0];
                log::info!(
                    "  First derivative: expires {}, puts: {}, calls: {}",
                    first_deriv.expire_time,
                    first_deriv.option_puts.len(),
                    first_deriv.option_calls.len()
                );
            }

            log::info!("\n✓ Full /ESH6 Option Chain Test PASSED");
        }
        Err(e) => {
            log::error!("✗ Failed to fetch option chain: {}", e);
            
            // Check if there were any core errors logged
            match error_rx.try_recv() {
                Ok(err) => log::error!("  Core error: {:#?}", err),
                Err(_) => log::info!("  No additional core errors"),
            }

            panic!("Option chain fetch failed: {}", e);
        }
    }

    drop(thand);
    core_handle.abort();

    let duration_ms = start.elapsed().as_millis();
    write_timing_file("test_es_option_chain_parsing", duration_ms);
    log::trace!("\nTest completed in {} ms\n", duration_ms);

    panic!("why its fine here?!");
}

// ========================================================================
// Test 1: Equity Option Chain Diagnostic (SPY)
// ========================================================================
#[tokio::test]
async fn test_opt_spy_option_chain_parsing() {
    let start = Instant::now();
    setuplog();
    log::debug!("ENTRY! test_spy_option_chain_parsing()...");
    
    let creds = load_creds();
    let conn_info = ttrs::ConnectionInfo {
        base_url: "https://api.tastyworks.com".to_string(),
        oauth: (
            creds["client_id"].as_str().unwrap_or("").to_string(),
            creds["client_secret"].as_str().unwrap_or("").to_string(),
            creds["refresh_token"].as_str().unwrap_or("").to_string(),
        ),
    };
    
    let (error_tx, mut error_rx) = tokio::sync::mpsc::unbounded_channel();
    let (_shutdown_tx, shutdown_rx) = tokio::sync::mpsc::channel::<String>(1);
    let config = ttrs::CoreConfig { error_tx, shutdown_rx };
    
    let (thand, tfoot) = ttrs::make_core_api(100);
    let core_handle = tokio::spawn(ttrs::fn_run_core(tfoot, conn_info, config));
    
    // Wait for core to initialize
    tokio::time::sleep(std::time::Duration::from_millis(300)).await;
    
    log::trace!("\n=== Starting SPY Option Chain Diagnostic Test ===\n");
    
    let symbol = "SPY".to_string();
    log::info!("Requesting option chain for: {}", symbol);
    
    match thand.tell_tastytrade_to_get_option_chain(symbol).await {
        Ok(chain_data) => {
            log::info!("✓ Successfully fetched option chain data");
            log::info!("Items count: {}", chain_data.items.len());
            
            if !chain_data.items.is_empty() {
                let first_item = &chain_data.items[0];
                log::info!("First item root: {}", first_item.root_symbol);
                log::info!("First item underlying: {}", first_item.underlying_symbol);
                log::info!("Expirations: {}", first_item.expirations.len());
                log::info!("Tick sizes: {}", first_item.tick_sizes.len());
                
                if !first_item.expirations.is_empty() {
                    let first_exp = &first_item.expirations[0];
                    log::info!(
                        "First expiration: {} ({} DTE, {} strikes)",
                        first_exp.expiration_date,
                        first_exp.days_to_expiration,
                        first_exp.strikes.len()
                    );
                    
                    if !first_exp.strikes.is_empty() {
                        let first_strike = &first_exp.strikes[0];
                        log::info!(
                            "Strike: {} (call: {}, put: {})",
                            first_strike.strike_price,
                            first_strike.call,
                            first_strike.put
                        );
                    }
                }
            }
            
            // Convert to derivatives (same as futures test)
            log::trace!("\nConverting OptionChainData to Derivatives...");
            let derivatives = crate::convert::option_chain_data_to_derivatives(&chain_data);
            log::info!("✓ Converted to {} Derivatives objects", derivatives.len());
            
            if !derivatives.is_empty() {
                let first_deriv = &derivatives[0];
                log::info!(
                    "First derivative: expires {}, puts: {}, calls: {}",
                    first_deriv.expire_time,
                    first_deriv.option_puts.len(),
                    first_deriv.option_calls.len()
                );
            }
            
            log::info!("\n✓ Full SPY Option Chain Test PASSED");
        }
        Err(e) => {
            log::error!("✗ Failed to fetch option chain: {}", e);
            
            // Check if there were any core errors
            match error_rx.try_recv() {
                Ok(err) => log::error!("Core error: {:#?}", err),
                Err(_) => log::info!("No additional core errors"),
            }
            panic!("Option chain fetch failed: {}", e);
        }
    }
    
    drop(thand);
    core_handle.abort();
    
    let duration_ms = start.elapsed().as_millis();
    write_timing_file("test_spy_option_chain_parsing", duration_ms);
    log::trace!("\nTest completed in {} ms\n", duration_ms);
    
    panic!("why its fine here?!");
}


    // ========================================================================
    // Test 1: Balance Conversion
    // ========================================================================
    #[tokio::test]
    async fn test_convert_balance_to_account_update() {
        let start = Instant::now();
        setuplog();
        log::debug!("ENTRY! test_convert_balance_to_account_update()...");

        let creds = load_creds();
        let conn_info = ttrs::ConnectionInfo {
            base_url: "https://api.tastyworks.com".to_string(),
            oauth: (
                creds["client_id"].as_str().unwrap_or("").to_string(),
                creds["client_secret"].as_str().unwrap_or("").to_string(),
                creds["refresh_token"].as_str().unwrap_or("").to_string(),
            ),
        };
        let acct_num = creds["account"].as_str().unwrap_or("").to_string();

        if acct_num.is_empty() {
            panic!("account number required in test-creds.json");
        }

        let (error_tx, _error_rx) = tokio::sync::mpsc::unbounded_channel();
        let (shutdown_tx, shutdown_rx) = tokio::sync::mpsc::channel::<String>(1);
        let config = ttrs::CoreConfig { error_tx, shutdown_rx };


        let (thand, tfoot) = ttrs::make_core_api(100);
        let core_handle = tokio::spawn(ttrs::fn_run_core(tfoot, conn_info, config));

        // Wait for core to initialize
        tokio::time::sleep(std::time::Duration::from_millis(200)).await;

        log::trace!("Fetching balance from ttrs...");
        let balance = thand
            .tell_tastytrade_to_get_balances(acct_num.clone())
            .await
            .expect("Failed to get balance");

        log::trace!("Converting balance to AccountUpdate...");
        let account_update = crate::convert::balance_to_account_update(&balance);

        // Assertions
        assert_eq!(
            account_update.ttai_uid, balance.account_number,
            "Account UID should match"
        );
        assert_eq!(
            account_update.long_usd, balance.cash_balance,
            "Cash balance should match"
        );
        assert_eq!(
            account_update.can_sell, balance.derivative_buying_power,
            "Derivative buying power should match"
        );
        assert_eq!(
            account_update.summings, balance.net_liquidating_value,
            "Net liquidating value should match"
        );

        log::trace!(
            "✓ Conversion successful: {} -> AccountUpdate",
            balance.account_number
        );
        log::trace!("  long_usd: ${:.2}", account_update.long_usd);
        log::trace!("  can_sell: ${:.2}", account_update.can_sell);
        log::trace!("  summings: ${:.2}", account_update.summings);

        drop(thand);
        core_handle.abort();

        let duration_ms = start.elapsed().as_millis();
        write_timing_file("test_convert_balance_to_account_update", duration_ms);
    }

    // ========================================================================
    // Test 2: Positions Conversion
    // ========================================================================
    #[tokio::test]
    async fn test_convert_positions_to_positions_list() {
        let start = Instant::now();
        setuplog();
        log::debug!("ENTRY! test_convert_positions_to_positions_list()...");

        let creds = load_creds();
        let conn_info = ttrs::ConnectionInfo {
            base_url: "https://api.tastyworks.com".to_string(),
            oauth: (
                creds["client_id"].as_str().unwrap_or("").to_string(),
                creds["client_secret"].as_str().unwrap_or("").to_string(),
                creds["refresh_token"].as_str().unwrap_or("").to_string(),
            ),
        };
        let acct_num = creds["account"].as_str().unwrap_or("").to_string();

        if acct_num.is_empty() {
            panic!("account number required in test-creds.json");
        }

        let (error_tx, _error_rx) = tokio::sync::mpsc::unbounded_channel();
        let (shutdown_tx, shutdown_rx) = tokio::sync::mpsc::channel::<String>(1);
        let config = ttrs::CoreConfig { error_tx, shutdown_rx };

        let (thand, tfoot) = ttrs::make_core_api(100);
        let core_handle = tokio::spawn(ttrs::fn_run_core(tfoot, conn_info, config));

        tokio::time::sleep(std::time::Duration::from_millis(200)).await;

        log::trace!("Fetching positions from ttrs...");
        let positions = thand
            .tell_tastytrade_to_get_positions(acct_num.clone())
            .await
            .expect("Failed to get positions");

        log::trace!("Converting {} positions to PositionsList...", positions.len());
        let positions_list = crate::convert::positions_to_positions_list(&positions);

        // Assertions
        assert_eq!(
            positions_list.positions.len(),
            positions.len(),
            "Position count should match"
        );

        for (i, pos) in positions.iter().enumerate() {
            let expected_key = &pos.symbol;
            assert!(
                positions_list.positions.contains_key(expected_key),
                "Position {} should exist in converted list",
                expected_key
            );

            let converted = &positions_list.positions[expected_key];
            let mut expected_qty = pos.quantity;
            if pos.quantity_direction.to_lowercase() == "short" && expected_qty > 0.0 {
                expected_qty = -expected_qty;
            }

            log::trace!(
                "  [{}] {} x{} @ ${:.2}",
                i + 1,
                converted.asset,
                converted.owned,
                converted.price
            );
        }

        log::trace!("✓ Conversion successful: {} positions", positions_list.positions.len());

        drop(thand);
        core_handle.abort();

        let duration_ms = start.elapsed().as_millis();
        write_timing_file("test_convert_positions_to_positions_list", duration_ms);
    }

    // ========================================================================
    // Test 3: Order Conversion
    // ========================================================================
    #[tokio::test]
    async fn test_convert_order_response_to_placed_order() {
        let start = Instant::now();
        setuplog();
        log::debug!("ENTRY! test_convert_order_response_to_placed_order()...");

        let creds = load_creds();
        let conn_info = ttrs::ConnectionInfo {
            base_url: "https://api.tastyworks.com".to_string(),
            oauth: (
                creds["client_id"].as_str().unwrap_or("").to_string(),
                creds["client_secret"].as_str().unwrap_or("").to_string(),
                creds["refresh_token"].as_str().unwrap_or("").to_string(),
            ),
        };
        let acct_num = creds["account"].as_str().unwrap_or("").to_string();

        if acct_num.is_empty() {
            panic!("account number required in test-creds.json");
        }

        let (error_tx, _error_rx) = tokio::sync::mpsc::unbounded_channel();
        let (shutdown_tx, shutdown_rx) = tokio::sync::mpsc::channel::<String>(1);
        let config = ttrs::CoreConfig { error_tx, shutdown_rx };

        let (thand, tfoot) = ttrs::make_core_api(100);
        let core_handle = tokio::spawn(ttrs::fn_run_core(tfoot, conn_info,config));

        tokio::time::sleep(std::time::Duration::from_millis(200)).await;

        log::trace!("Fetching live orders from ttrs...");
        let orders = thand
            .tell_tastytrade_to_get_live_orders(acct_num.clone())
            .await
            .unwrap_or_default();

        if orders.is_empty() {
            log::warn!("⚠ No live orders to test - skipping conversion test");
            log::trace!("(This is OK - just means no active orders)");
        } else {
            log::trace!("Converting {} orders to PlacedOrder...", orders.len());
            let placed_orders = crate::convert::order_responses_to_placed_orders(&orders);

            assert_eq!(
                placed_orders.len(),
                orders.len(),
                "Order count should match"
            );

            for (i, (orig, converted)) in orders.iter().zip(placed_orders.iter()).enumerate()
            {
                assert_eq!(
                    converted.order_hash, orig.id.unwrap(),
                    "Order hash should match ID"
                );
                assert_eq!(
                    converted.legs_fills.len(),
                    orig.legs.len(),
                    "Leg count should match"
                );

                log::trace!(
                    "  [{}] Order {} with {} legs, status: {}",
                    i + 1,
                    converted.order_hash,
                    converted.legs_fills.len(),
                    orig.status
                );
            }

            log::trace!(
                "✓ Conversion successful: {} orders converted",
                placed_orders.len()
            );
        }

        drop(thand);
        core_handle.abort();

        let duration_ms = start.elapsed().as_millis();
        write_timing_file("test_convert_order_response_to_placed_order", duration_ms);
    }

    // ========================================================================
    // Test 4: Ticker Creation
    // ========================================================================
    #[test]
    fn test_create_ticker() {
        let start = Instant::now();
        setuplog();
        log::debug!("ENTRY! test_create_ticker()...");

        let ticker = crate::convert::create_ticker("AAPL", 150.50, 150.75);

        assert_eq!(ticker.name, "AAPL", "Ticker name should match");
        assert_eq!(ticker.bid, 150.50, "Bid should match");
        assert_eq!(ticker.ask, 150.75, "Ask should match");
        assert!(
            (ticker.bid + ticker.ask) / 2.0 - 150.625 < 0.01,
            "Midpoint should be calculated correctly"
        );

        log::trace!("✓ Ticker created: {} @ ${:.2}", ticker.name, ticker.bid);

        let duration_ms = start.elapsed().as_millis();
        write_timing_file("test_create_ticker", duration_ms);
    }

    // ========================================================================
    // Test 5: FinAss Creation (Stock)
    // ========================================================================
    #[test]
    fn test_create_stock_fin_ass() {
        let start = Instant::now();
        setuplog();
        log::debug!("ENTRY! test_create_stock_fin_ass()...");

        let fin_ass = crate::convert::create_stock_fin_ass("TSLA", None);

        match fin_ass {
            crate::FinAss::StkDeets(stk) => {
                assert_eq!(stk.ticker_name, "TSLA", "Stock ticker name should match");
                log::trace!("✓ Stock FinAss created: {}", stk.ticker_name);
            }
            _ => panic!("Expected StkDeets variant"),
        }

        let duration_ms = start.elapsed().as_millis();
        write_timing_file("test_create_stock_fin_ass", duration_ms);
    }

    // ========================================================================
    // Test 6: FinAss Creation (Option)
    // ========================================================================
    #[test]
    fn test_create_option_fin_ass() {
        let start = Instant::now();
        setuplog();
        log::debug!("ENTRY! test_create_option_fin_ass()...");

        let expire = chrono::Utc::now() + chrono::Duration::days(30);
        let fin_ass =
            crate::convert::create_option_fin_ass("AAPL", "AAPL 230120C00150000", 150.0, expire, true);

        match fin_ass {
            crate::FinAss::OptDeets(opt) => {
                assert_eq!(opt.ticker_name, "AAPL", "Option ticker name should match");
                assert_eq!(opt.strike_spot, 150.0, "Strike price should match");
                assert_eq!(
                    opt.option_rite,
                    crate::OptRite::Call,
                    "Should be a call option"
                );
                log::trace!(
                    "✓ Option FinAss created: {} ${} Call",
                    opt.ticker_name,
                    opt.strike_spot
                );
            }
            _ => panic!("Expected OptDeets variant"),
        }

        let duration_ms = start.elapsed().as_millis();
        write_timing_file("test_create_option_fin_ass", duration_ms);
    }

    // ========================================================================
    // Test 7: Full Integration Test
    // ========================================================================
    #[tokio::test]
    async fn test_full_integration_ttrs_to_dsta() {
        let test_start = Instant::now();
        setuplog();
        log::debug!("ENTRY! test_full_integration_ttrs_to_dsta()...");

        let creds = load_creds();
        let conn_info = ttrs::ConnectionInfo {
            base_url: "https://api.tastyworks.com".to_string(),
            oauth: (
                creds["client_id"].as_str().unwrap_or("").to_string(),
                creds["client_secret"].as_str().unwrap_or("").to_string(),
                creds["refresh_token"].as_str().unwrap_or("").to_string(),
            ),
        };
        let acct_num = creds["account"].as_str().unwrap_or("").to_string();

        if acct_num.is_empty() {
            panic!("account number required in test-creds.json");
        }

        log::trace!("\n=== Starting full integration test ===");


        let (error_tx, _error_rx) = tokio::sync::mpsc::unbounded_channel();
        let (shutdown_tx, shutdown_rx) = tokio::sync::mpsc::channel::<String>(1);
        let config = ttrs::CoreConfig { error_tx, shutdown_rx };

        let (thand, tfoot) = ttrs::make_core_api(100);
        let core_handle = tokio::spawn(ttrs::fn_run_core(tfoot, conn_info,config));

        tokio::time::sleep(std::time::Duration::from_millis(300)).await;

        let mut timings = Vec::new();

        // Step 1: Balance
        log::trace!("\n[1/4] Converting Balance...");
        let start = Instant::now();
        let balance = thand
            .tell_tastytrade_to_get_balances(acct_num.clone())
            .await
            .expect("Failed to get balance");
        let account_update = crate::convert::balance_to_account_update(&balance);
        let elapsed = start.elapsed().as_millis();
        timings.push(format!("Balance conversion: {} ms", elapsed));
        log::trace!(" ✓ Account NLV: ${:.2} (took {} ms)", account_update.summings, elapsed);

        // Step 2: Positions
        log::trace!("\n[2/4] Converting Positions...");
        let start = Instant::now();
        let positions = thand
            .tell_tastytrade_to_get_positions(acct_num.clone())
            .await
            .expect("Failed to get positions");
        let positions_list = crate::convert::positions_to_positions_list(&positions);
        let elapsed = start.elapsed().as_millis();
        timings.push(format!("Positions conversion: {} ms", elapsed));
        log::trace!(
            " ✓ Converted {} positions (took {} ms)",
            positions_list.positions.len(),
            elapsed
        );

        // Step 3: Orders
        log::trace!("\n[3/4] Converting Orders...");
        let start = Instant::now();
        let orders = thand
            .tell_tastytrade_to_get_live_orders(acct_num.clone())
            .await
            .unwrap_or_default();
        let placed_orders = crate::convert::order_responses_to_placed_orders(&orders);
        let elapsed = start.elapsed().as_millis();
        timings.push(format!("Orders conversion: {} ms", elapsed));
        log::trace!(
            " ✓ Converted {} orders (took {} ms)",
            placed_orders.len(),
            elapsed
        );

        // Step 4: Create synthetic data structures
        log::trace!("\n[4/4] Creating synthetic structures...");
        let start = Instant::now();

        let ticker = crate::convert::create_ticker("AAPL", 150.50, 150.75);
        let stock_asset = crate::convert::create_stock_fin_ass("TSLA", None);
        let option_expire = chrono::Utc::now() + chrono::Duration::days(30);
        let option_asset =
            crate::convert::create_option_fin_ass("SPY", "SPY 230120C00400000", 400.0, option_expire, true);

        let owned_position = crate::OwnedPosition {
            asset: stock_asset,
            owned: 100.0,
            price: 250.0,
        };

        let elapsed = start.elapsed().as_millis();
        timings.push(format!("Synthetic structures: {} ms", elapsed));
        log::trace!(" ✓ Created sample structures (took {} ms)", elapsed);

        // Assertions
        assert_eq!(
            account_update.ttai_uid, balance.account_number,
            "Account number should match"
        );
        assert!(!positions_list.positions.is_empty() || positions.is_empty(),
            "Positions conversion should work"
        );

        log::trace!("\n=== Integration test completed successfully! ===\n");

        drop(thand);
        core_handle.abort();

        let total_duration_ms = test_start.elapsed().as_millis();
        create_tests_dir();
        let file_name = format!(
            "timing_for_test_full_integration_ttrs_to_dsta_was_{}_milliseconds.txt",
            total_duration_ms
        );
        let mut content = format!(
            "Test 'test_full_integration_ttrs_to_dsta' completed in {} milliseconds.\n\nPer-conversion timings:\n",
            total_duration_ms
        );
        for timing in timings {
            content.push_str(&format!("- {}\n", timing));
        }
        fs::write(format!("tests/{}", file_name), content)
            .expect("Failed to write timing file");
    }

    // ========================================================================
    // Test 8: Type Safety and Serialization
    // ========================================================================
    #[test]
    fn test_serde_roundtrip() {
        let start = Instant::now();
        setuplog();
        log::debug!("ENTRY! test_serde_roundtrip()...");

        // Test AccountUpdate serialization
        let orig_account = crate::AccountUpdate {
            ttai_uid: "TEST_ACCT".to_string(),
            long_usd: 10000.50,
            can_sell: 5000.25,
            long_stk: 25000.0,
            sell_stk: 0.0,
            long_opt: 500.0,
            sell_opt: 0.0,
            long_fut: 0.0,
            sell_fut: 0.0,
            long_fop: 0.0,
            sell_fop: 0.0,
            long_xxx: 0.0,
            sell_xxx: 0.0,
            long_bnd: 0.0,
            next_usd: 0.0,
            summings: 40500.75,
            cash_out: 10000.50,
        };

        let json_str = serde_json::to_string(&orig_account)
            .expect("Failed to serialize AccountUpdate");
        let restored: crate::AccountUpdate =
            serde_json::from_str(&json_str).expect("Failed to deserialize AccountUpdate");

        assert_eq!(
            orig_account.ttai_uid, restored.ttai_uid,
            "UID should match after roundtrip"
        );
        assert_eq!(
            orig_account.summings, restored.summings,
            "Summings should match after roundtrip"
        );

        log::trace!("✓ Serde roundtrip successful for AccountUpdate");

        

        let duration_ms = start.elapsed().as_millis();
        write_timing_file("test_serde_roundtrip", duration_ms);
    }


// ========================================================================
// Test 9: Convert dsta::Order → ttrs::OrderRequest (Stock Limit Order)
// ========================================================================
#[test]
fn test_order_to_order_request_stock_limit() {
    use crate::{BuyOrSell, MarketOrLimit, Order, OrderLeg};
    use crate::convert::order_to_order_request;
    use ttrs::OrderRequest;

    setuplog();

  let stuff = stuffmap();
    let order = Order {
        order_legs: vec![OrderLeg {
            buy_what: stuff.get("FAKE").unwrap().clone(),
            quantity: 1.0,
            remaining: 0.0,
            action: BuyOrSell::BuyToOpen,
            price: 42.0,
        }],
        order_type: MarketOrLimit::Limit,
    };

    log::info!("Input dsta Order:");
    log::info!("{:#?}", order);

    let request: OrderRequest = order_to_order_request(&order);

    log::info!("Converted ttrs OrderRequest:");
    log::info!("{:#?}", request);

    // Force print on test failure/panic so you can see the full output
    //panic!("Intentionally panicking to display the full converted OrderRequest above ↑");
}

// ========================================================================
// Test 10: Convert dsta::Order → ttrs::OrderRequest (Option Credit Spread Open)
// ========================================================================
#[test]
fn test_order_to_order_request_option_credit_spread_open() {
    use crate::{BuyOrSell, MarketOrLimit, Order, OrderLeg};
    use crate::convert::order_to_order_request;
    use ttrs::OrderRequest;

    setuplog();
  let stuff = stuffmap();

    let order = Order {
        order_legs: vec![
            OrderLeg {
                buy_what: stuff.get("FAKE 95_P_35").unwrap().clone(),
                quantity: 20.0,
                remaining: 20.0,
                action: BuyOrSell::BuyToOpen,
                price: 0.60,
            },
            OrderLeg {
                buy_what: stuff.get("FAKE 100_P_35").unwrap().clone(),
                quantity: 20.0,
                remaining: 20.0,
                action: BuyOrSell::SellToOpen,
                price: 1.90,
            },
        ],
        order_type: MarketOrLimit::Limit,
    };

    log::info!("Input dsta Order (Credit Spread Open):");
    log::info!("{:#?}", order);

    let request: OrderRequest = order_to_order_request(&order);

    log::info!("Converted ttrs OrderRequest:");
    log::info!("{:#?}", request);

    //panic!("Intentionally panicking to display the full converted OrderRequest above ↑");
}

// ========================================================================
// Test 11: Convert dsta::Order → ttrs::OrderRequest (Option Spread Close)
// ========================================================================
#[test]
fn test_order_to_order_request_option_spread_close() {
    use crate::{BuyOrSell, MarketOrLimit, Order, OrderLeg};
    use crate::convert::order_to_order_request;
    use ttrs::OrderRequest;

    setuplog();

  let stuff = stuffmap();

    let order = Order {
        order_legs: vec![
            OrderLeg {
                buy_what: stuff.get("FAKE 100_P_35").unwrap().clone(),
                quantity: 20.0,
                remaining: 20.0,
                action: BuyOrSell::SellToClose,
                price: 0.80,
            },
            OrderLeg {
                buy_what: stuff.get("FAKE 95_P_35").unwrap().clone(),
                quantity: 20.0,
                remaining: 20.0,
                action: BuyOrSell::BuyToClose,
                price: 3.10,
            },
        ],
        order_type: MarketOrLimit::Limit,
    };

    log::info!("Input dsta Order (Spread Close):");
    log::info!("{:#?}", order);

    let request: OrderRequest = order_to_order_request(&order);

    log::info!("Converted ttrs OrderRequest:");
    log::info!("{:#?}", request);

    //panic!("Intentionally panicking to display the full converted OrderRequest above ↑");
}