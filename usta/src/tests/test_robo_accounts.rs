// usta/src/tests/test_robo_accounts.rs
    use crate::*;
    use crate::core::*;
    use crate::dr_robo::*;
    use crate::tests::util::*;
    
    use dsta::*;
    use std::collections::HashMap;
use log::{LevelFilter, info, error};
use log4rs::{
    append::file::FileAppender,
    config::{Appender, Config, Root},
    encode::pattern::PatternEncoder,
    Handle,
};

fn setup() {let _ = setup_log4rs("default-log-name");}
fn setuplog() {let _ = setup_log4rs("default-log-name");}
fn _setup_old() {
        let _ = env_logger::builder()
        .is_test(true)
        .format_timestamp_millis()  // This gives millisecond precision in logs
        .try_init();
    }
fn _setuplog_old() {
    let _ = env_logger::builder()
        .is_test(true)
        .format_timestamp_millis()  // This gives millisecond precision in logs
        .format_module_path(false)
        .format_target(false)
        .try_init();
}

 
fn buy_stock(qty: f64, limit_price: f64) -> Order {
    let mut stk = dsta::Stk::default();
    stk.ticker_name = "STK".to_string();  // or whatever the actual ticker should be

    Order {
        order_legs: vec![OrderLeg {
            buy_what: dsta::FinAss::StkDeets(stk),
            quantity: qty,
            remaining: qty,
            action: BuyOrSell::BuyToOpen,
            price: limit_price,  // better to pass actual limit price here
        }],
        order_type: MarketOrLimit::Limit,
        //order_cost: Some(-limit_price * qty * 100.0),  // corrected: qty not qty as f64
    }
}

fn sell_short_put(strike: f64, days: f64, qty: f64, premium: f64) -> Order {
    let mut opt = dsta::Opt::default();
    opt.ticker_name = format!(".STK {} P {}", strike, days);
    opt.option_name = format!("STK {} P {}", strike, days);
    // You can set other fields like strike_spot, expire_date, option_rite, etc. if needed
    opt.strike_spot = strike;
    opt.option_rite = dsta::OptRite::Put;
    // opt.expire_date = ... (optional)

    Order {
        order_legs: vec![OrderLeg {
            buy_what: dsta::FinAss::OptDeets(opt),
            quantity: qty,
            remaining: qty,
            action: BuyOrSell::SellToOpen,
            price: premium,
        }],
        order_type: MarketOrLimit::Limit,
        //order_cost: Some(premium * qty * 100.0),  // credit received → positive cost
    }
}

fn sell_call_credit_spread(
    short_strike: f64,
    long_strike: f64,
    days: f64,
    qty: f64,
    credit: f64,
) -> Order {
    // Short leg (sell to open) - closer to ATM, higher premium
    let mut short_opt = dsta::Opt::default();
    short_opt.ticker_name = format!(".STK {} C {}", short_strike, days);
    short_opt.option_name = format!("STK {} C {}", short_strike, days);
    short_opt.strike_spot = short_strike;
    short_opt.option_rite = dsta::OptRite::Call;

    // Long leg (buy to open) - further OTM, protection
    let mut long_opt = dsta::Opt::default();
    long_opt.ticker_name = format!(".STK {} C {}", long_strike, days);
    long_opt.option_name = format!("STK {} C {}", long_strike, days);
    long_opt.strike_spot = long_strike;
    long_opt.option_rite = dsta::OptRite::Call;

    Order {
        order_legs: vec![
            OrderLeg {
                buy_what: dsta::FinAss::OptDeets(short_opt),
                quantity: qty,
                remaining: qty,
                action: BuyOrSell::SellToOpen,
                price: 1.0 + credit,  // or estimate premium if needed
            },
            OrderLeg {
                buy_what: dsta::FinAss::OptDeets(long_opt),
                quantity: qty,
                remaining: qty,
                action: BuyOrSell::BuyToOpen,
                price: 1.0,  // or estimate debit
            },
        ],
        order_type: MarketOrLimit::Limit,
        //order_cost: Some(credit * qty * 100.0),  // net credit received
    }
}

    #[test]
    fn legality_empty_account_buy_stock() {
        setup();
        let account = BotaAccount::new().with_cash(10000.0);
        let order = buy_stock(1.0, 50.0);
        let stuff = make_stuff_with_tickers(&["STK"]);

        assert!(fn_check_bota_legality(&account, &order, &stuff).is_ok());
    }

    #[test]
    fn legality_short_put_requires_cash() 
    {
      let _ = setup_log4rs("test_robo_accounts");
      info!("Starting sally_bot_submits_when_mid_price_reaches_target_buy test");


        setup();
        let account = BotaAccount::new().with_cash(4000.0);
        let order = sell_short_put(50.0, 30.0, 1.0, 2.0);
        let stuff = make_stuff_with_tickers(&["STK 50_P_30"]);

        // Legality passes, but margin check later will fail — this test only checks legality
        assert!(fn_check_bota_legality(&account, &order, &stuff).is_ok());
    }

    #[test]
    fn calculate_margin_naked_short_put() {
        setup();
        let account = BotaAccount::new()
            .with_cash(10000.0)
            .with_short_put(50.0, -1.0, 30.0, 2.0);

        let leins = calculate_option_margin(&account).unwrap();
        assert_eq!(leins.len(), 1);
        assert_eq!(leins[0].0, LeinType::ShortPut);
        assert_eq!(leins[0].1, 5000.0);
    }

    #[test]
    fn calculate_margin_call_credit_spread() {
        setup();
        let account = BotaAccount::new()
            .with_cash(10000.0)
            .with_short_call(55.0, -1.0, 30.0, 3.0)
            .with_long_call(60.0, 1.0, 30.0, 1.0);

        let leins = calculate_option_margin(&account).unwrap();
        assert_eq!(leins.len(), 1);
        assert_eq!(leins[0].0, LeinType::CallCreditSpread);
        assert_eq!(leins[0].1, 500.0);
    }

    #[test]
    fn calculate_margin_put_credit_spread() {
        setup();
        let account = BotaAccount::new()
            .with_cash(10000.0)
            .with_short_put(45.0, -1.0, 30.0, 2.5)
            .with_long_put(40.0, 1.0, 30.0, 0.8);

        let leins = calculate_option_margin(&account).unwrap();
        assert_eq!(leins.len(), 1);
        assert_eq!(leins[0].0, LeinType::PutCreditSpread);
        assert_eq!(leins[0].1, 500.0);
    }

    #[test]
    fn calculate_margin_iron_condor_max_width() {
        setup();
        let account = BotaAccount::new()
            .with_cash(20000.0)
            .with_short_call(55.0, -1.0, 30.0, 2.0)
            .with_long_call(60.0, 1.0, 30.0, 0.5)
            .with_short_put(45.0, -1.0, 30.0, 1.8)
            .with_long_put(40.0, 1.0, 30.0, 0.4);

        let leins = calculate_option_margin(&account).unwrap();
        assert_eq!(leins.len(), 1);
        assert_eq!(leins[0].0, LeinType::IronCondor);
        assert_eq!(leins[0].1, 500.0);
    }

    #[test]
    fn shenanigans_sell_call_credit_spread() {
        setup();
        let mut account = BotaAccount::new().with_cash(10000.0);
        let order = sell_call_credit_spread(55.0, 60.0, 30.0, 1.0, 1.5);
        let stuff = make_stuff_with_tickers(&["STK 55_C_30", "STK 60_C_30"]);

        let changes = fn_get_account_shennanagans(&order, &account, &stuff).unwrap();

        assert_eq!(changes.cash_gain_from_order, 150.0);
        assert_eq!(changes.cash_transfer_to_leins.len(), 1);
        assert_eq!(changes.cash_transfer_to_leins[0].0, LeinType::CallCreditSpread);
        assert_eq!(changes.cash_transfer_to_leins[0].1, 500.0);

        apply_account_cash_and_collateral_changes(&mut account, &changes);

        assert_eq!(account.m_cash, 9650.0);
        assert_eq!(account.m_call_credit_spread_lein, 500.0);
    }

    #[test]
    fn shenanigans_close_naked_short_put() {
        setup();
        let mut account = BotaAccount::new()
            .with_cash(10000.0)
            .with_short_put(50.0, -1.0, 30.0, 2.0);

        let initial_leins = calculate_option_margin(&account).unwrap();
        let initial_changes = AccountCashChanges {
            cash_gain_from_order: 200.0,
            cash_transfer_to_leins: initial_leins,
        };
        apply_account_cash_and_collateral_changes(&mut account, &initial_changes);

       let mut opt = dsta::Opt::default();
          opt.ticker_name = format!(".STK 50 P 30");
          opt.option_name = format!("STK 50 P 30");

        let close_order = Order {
            order_legs: vec![OrderLeg {
                buy_what: dsta::FinAss::OptDeets(opt),
                quantity: 1.0,
                remaining: 1.0,
                action: BuyOrSell::BuyToClose,
                price: 1.0,
            }],
            order_type: MarketOrLimit::Limit,
        };

        let stuff = make_stuff_with_tickers(&["STK 50_P_30"]);

        let changes = fn_get_account_shennanagans(&close_order, &account, &stuff).unwrap();

        assert_eq!(changes.cash_gain_from_order, -100.0);
        assert_eq!(changes.cash_transfer_to_leins.len(), 0);

        apply_account_cash_and_collateral_changes(&mut account, &changes);

        assert_eq!(account.m_short_put_lein, 0.0);
        assert_eq!(account.m_cash, 10100.0);
    }

    #[test]
    #[should_panic(expected = "calculate_option_margin detects uncovered short put")]
    fn shenanigans_insufficient_cash_for_short_put() {
        setup();
        let account = BotaAccount::new().with_cash(4000.0);
        let order = sell_short_put(50.0, 30.0, 1.0, 2.0);
        let stuff = make_stuff_with_tickers(&["STK 50_P_30"]);

        let _ = fn_get_account_shennanagans(&order, &account, &stuff).unwrap();
    }




  // Integration test to fetch real SPY option chains and inspect their structure
  #[tokio::test]
  #[ignore] // Add ignore so it doesn't run by default (remove to run with: cargo test inspect_real_spy_option_chain -- --nocapture --ignored)
  async fn inspect_real_spy_option_chain() {
      use tokio::sync::mpsc;
      use std::time::Duration as StdDuration;
      
      setuplog();
      log::info!("=== Starting SPY Option Chain Inspection Test ===");
      
      // Load credentials and setup API connection (using ttrs pattern)
      let creds = load_creds();
      let conn_info = ttrs::ConnectionInfo {
          base_url: "https://api.tastyworks.com".to_string(),
          oauth: (
              creds["client_id"].as_str().unwrap_or("").to_string(),
              creds["client_secret"].as_str().unwrap_or("").to_string(),
              creds["refresh_token"].as_str().unwrap_or("").to_string(),
          ),
      };
      
      log::info!("Creating API connection...");
      let (thand, tfoot) = ttrs::make_core_api(10);
      let (error_tx, mut error_rx) = mpsc::unbounded_channel();
      let (shutdown_tx, shutdown_rx) = mpsc::channel::<String>(1);
      let config = ttrs::CoreConfig { error_tx, shutdown_rx };
      let core_handle = tokio::spawn(ttrs::fn_run_core(tfoot, conn_info, config));
      
      // Give core time to initialize
      tokio::time::sleep(StdDuration::from_millis(500)).await;
      
      log::info!("Requesting SPY option chain from ttrs API...");
      let chain_result = thand
          .tell_tastytrade_to_get_option_chain("SPY".to_string())
          .await;
      
      match chain_result {
          Ok(ttrs_chain) => {
              log::info!("✓ Successfully retrieved SPY option chain from ttrs");
              log::info!("Number of roots: {}", ttrs_chain.items.len());
              
              // Convert to dsta format
              log::info!("\nConverting to dsta::Derivatives format...");
              let dsta_derivatives = dsta::convert::option_chain_data_to_derivatives(&ttrs_chain);
              
              log::info!("✓ Conversion complete");
              log::info!("Number of expiration dates: {}", dsta_derivatives.len());
              
              // Analyze the first expiration (nearest term, most liquid)
              if let Some(first_chain) = dsta_derivatives.first() {
                  log::info!("\n=== ANALYZING FIRST EXPIRATION ===");
                  log::info!("Expiry: {}", first_chain.expire_time);
                  log::info!("Number of puts: {}", first_chain.option_puts.len());
                  log::info!("Number of calls: {}", first_chain.option_calls.len());
                  
                  // Check if puts are sorted high-to-low (as expected)
                  let puts = &first_chain.option_puts;
                  if puts.len() >= 2 {
                      let sorted_desc = puts.windows(2).all(|w| w[0].strike_spot >= w[1].strike_spot);
                      log::info!("\nPuts sorted HIGH to LOW: {}", sorted_desc);
                      if sorted_desc {
                          log::info!("✓ Puts are sorted descending (higher strikes first)");
                      } else {
                          log::warn!("⚠ Puts are NOT sorted descending!");
                      }
                  }
                  
                  // Print first 15 puts to see the structure
                  log::info!("\n=== FIRST 15 PUTS (array indices) ===");
                  for (idx, put) in puts.iter().take(15).enumerate() {
                      log::info!("  [{}] Strike: ${:.2} - {}", 
                          idx, put.strike_spot, put.option_name);
                  }

                  // Check if calls are sorted low-to-high (as expected)
                  let calls = &first_chain.option_calls;
                  if calls.len() >= 2 {
                      let sorted_asc = calls.windows(2).all(|w| w[0].strike_spot <= w[1].strike_spot);
                      log::info!("\nCalls sorted LOW to HIGH: {}", sorted_asc);
                      if sorted_asc {
                          log::info!("✓ Calls are sorted ascending (lower strikes first)");
                      } else {
                          log::warn!("⚠ Calls are NOT sorted ascending!");
                      }
                  }
                  
                  // Print first 15 calls to see the structure
                  log::info!("\n=== FIRST 15 CALLS (array indices) ===");
                  for (idx, call) in calls.iter().take(15).enumerate() {
                      log::info!("  [{}] Strike: ${:.2} - {}", 
                          idx, call.strike_spot, call.option_name);
                  }
                  
                  // Simulate ATM calculation (assume SPY is around $600)
                  let assumed_spy_price = 600.0;
                  
                  // Find ATM indices for both puts and calls
                  let mut atm_put_index = 0;
                  let mut atm_call_index = 0;
                  let mut min_diff_puts = f64::MAX;
                  let mut min_diff_calls = f64::MAX;
                  
                  log::info!("\n=== FINDING ATM INDICES (assuming SPY ~${}) ===", assumed_spy_price);
                  for (idx, put) in puts.iter().enumerate() {
                      let diff = (put.strike_spot - assumed_spy_price).abs();
                      if diff < min_diff_puts {
                          min_diff_puts = diff;
                          atm_put_index = idx;
                      }
                  }
                  
                  for (idx, call) in calls.iter().enumerate() {
                      let diff = (call.strike_spot - assumed_spy_price).abs();
                      if diff < min_diff_calls {
                          min_diff_calls = diff;
                          atm_call_index = idx;
                      }
                  }
                  
                  log::info!("ATM put index: {} (Strike: ${:.2})", atm_put_index, puts[atm_put_index].strike_spot);
                  log::info!("ATM call index: {} (Strike: ${:.2})", atm_call_index, calls[atm_call_index].strike_spot);
                  
                  // ========== PUT CREDIT SPREAD LOGIC ==========
                  log::info!("\n=== STEALTH BOT PUT CREDIT SPREAD LOGIC ===");
                  log::info!("Config: option_bucket=1.0, spread_bucket=1");
                  log::info!("Direction: PUTS (MarketDirection::Corrects)");
                  
                  // For puts (sorted high-to-low): 
                  // - ATM is at some index
                  // - OTM puts are HIGHER indices (lower strike values)
                  // - To go further OTM, we ADD to the index
                  let put_long_index = atm_put_index + 1;
                  
                  log::info!("\nPUT SPREAD CONSTRUCTION:");
                  log::info!("  Long put (OTM leg) index: {} + {} = {}", atm_put_index, 1.0, put_long_index);
                  if put_long_index < puts.len() {
                      log::info!("  Long put strike: ${:.2} (OTM ✓)", puts[put_long_index].strike_spot);
                      
                      // PROPOSED FIX: For puts, subtract to move toward ATM (higher strike)
                      let put_short_index_fixed = put_long_index.saturating_sub(1);
                      log::info!("\n  PROPOSED FIX (subtract for puts):");
                      log::info!("  Short put (closer to ATM) index: {} - {} = {}", put_long_index, 1.0, put_short_index_fixed);
                      log::info!("  Short put strike: ${:.2}", puts[put_short_index_fixed].strike_spot);
                      
                      let spread_width = puts[put_short_index_fixed].strike_spot - puts[put_long_index].strike_spot;
                      log::info!("  Put credit spread: short ${:.2} / long ${:.2} (width: ${:.2})",
                          puts[put_short_index_fixed].strike_spot,
                          puts[put_long_index].strike_spot,
                          spread_width);
                      
                      if puts[put_short_index_fixed].strike_spot > puts[put_long_index].strike_spot {
                          log::info!("  ✓ CORRECT! Short strike > Long strike = credit spread");
                      } else {
                          log::error!("  ✗ WRONG! Short strike <= Long strike");
                      }
                  }
                  
                  // ========== CALL CREDIT SPREAD LOGIC ==========
                  log::info!("\n=== STEALTH BOT CALL CREDIT SPREAD LOGIC ===");
                  log::info!("Config: option_bucket=1.0, spread_bucket=1");
                  log::info!("Direction: CALLS (MarketDirection::Rallies)");
                  
                  // For calls (sorted low-to-high):
                  // - ATM is at some index
                  // - OTM calls are HIGHER indices (higher strike values)
                  // - To go further OTM, we ADD to the index
                  let call_short_index = atm_call_index + 1;
                  
                  log::info!("\nCALL SPREAD CONSTRUCTION:");
                  log::info!("  Short call (OTM leg) index: {} + {} = {}", atm_call_index, 1.0, call_short_index);
                  if call_short_index < calls.len() {
                      log::info!("  Short call strike: ${:.2} (OTM ✓)", calls[call_short_index].strike_spot);
                      
                      // For calls, ADD to move further OTM (higher strike)
                      let call_long_index = call_short_index + 1;
                      log::info!("  Long call (further OTM) index: {} + {} = {}", call_short_index, 1.0, call_long_index);
                      if call_long_index < calls.len() {
                          log::info!("  Long call strike: ${:.2}", calls[call_long_index].strike_spot);
                          
                          let spread_width = calls[call_long_index].strike_spot - calls[call_short_index].strike_spot;
                          log::info!("  Call credit spread: short ${:.2} / long ${:.2} (width: ${:.2})",
                              calls[call_short_index].strike_spot,
                              calls[call_long_index].strike_spot,
                              spread_width);
                          
                          if calls[call_long_index].strike_spot > calls[call_short_index].strike_spot {
                              log::info!("  ✓ CORRECT! Long strike > Short strike = credit spread");
                          } else {
                              log::error!("  ✗ WRONG! Long strike <= Short strike");
                          }
                      }
                  }
                  
                  // Print mock comparison
                  log::info!("\n=== MOCK TEST DATA COMPARISON ===");
                  log::info!("Mock test uses: strikes [105, 100, 95, 90]");
                  log::info!("Mock ATM: 100 (index 1)");
                  log::info!("Mock opt_1: 95 (index 2)");
                  log::info!("Mock opt_2 (current buggy): 90 (index 3) ✗");
                  log::info!("Mock opt_2 (should be): 100 (index 1) ✓");
                  
              } else {
                  log::error!("No expiration dates found in option chain!");
              }
              
              // Check for API errors
              while let Ok(err) = error_rx.try_recv() {
                  log::warn!("API error during test: {:?}", err);
              }
              
              // Intentionally fail to show output
              log::info!("\n=== TEST COMPLETE ===");
              log::info!("Review the logs above to understand the option chain structure");
              assert!(false, "Intentional failure to display full output - this is expected!");
          }
          Err(e) => {
              log::error!("Failed to get option chain: {:?}", e);
              assert!(false, "API request failed: {:?}", e);
          }
      }
      
      // Cleanup
      let _ = shutdown_tx.send("test shutdown".to_string()).await;
      drop(thand);
      let _ = core_handle.await;
  }
