// usta/src/tests/test_buzz_bomber.rs

use tokio::sync::mpsc;
use chrono::{Utc, Duration, Weekday};
use std::time::Duration as StdDuration;
use log::{info, debug, trace};
use crate::*;
use crate::core::*;
use crate::dr_robo::*;
use crate::tests::util::*;
use dsta::*;
use std::collections::HashMap;

// ============================================================================
// HAPPY PATH TESTS
// ============================================================================
// ============================================================================
// HAPPY PATH TESTS - REFACTORED TO MATCH STEALTH BOT PATTERN
// ============================================================================
#[tokio::test] // PASSES
async fn buzz_bomber_happy_path_put_credit_spread()
{
  // Do not change my formatting, spacing, or comments.
  // WORKING - buzz bomber put credit spread entry → exit cycle
  use tokio::sync::mpsc;
  use chrono::{Utc, Duration, Datelike, TimeZone};
  use std::time::Duration as StdDuration;
  use log::{info, debug, trace};
  use crate::*;
  use crate::tests::util::*;
  use crate::dr_robo::*;
  use dsta::*;
  use std::collections::HashMap;
  
  let _ = setup_log4rs("buzz_bomber_happy_path_put_credit_spread");
  info!("Starting buzz_bomber_happy_path_put_credit_spread test");

  let bot_name = "buzz_test_bot_puts".to_string();

  trace!
  ( r#"
  // ───────────────────────────────────────────────────────────────
  // Phase 1: Fresh start → entry → exit cycle
  // ───────────────────────────────────────────────────────────────
  "#
  );

  trace!
  ( r#"
  // ───────────────────────────────────────────────────────────────
  // Phase 1.1: Initialize channels and start bot
  // ───────────────────────────────────────────────────────────────
  "#
  );

  let reference = Utc::now(); 
  let lwd = reference.weekday();
  let dow = lwd.number_from_sunday() as u16;

  log::debug!("wrong time_to_order lwd = {:#?}, day_of_week = {} ",lwd, dow);

  let ny = reference.with_timezone(&chrono_tz::America::New_York);
  //let today_eastern = ny.date_naive();

  // Are we even on the correct day of week?
  let nyw = ny.weekday();
  let dow = nyw.num_days_from_sunday() as u16;

  log::debug!("est time_to_order nyw = {:#?}, day_of_week = {} ",nyw, dow);

  let config = dsta::MakeBuzzBomber {
      my_accounting: {
          let mut ma = dsta::CommonBotInfo::default();
          ma.friendly_name = bot_name.clone();
          ma.max_cash_risk = (true, 100.0, 10_000.0);
          ma.tracking_tick = "FAKE".to_string();
          ma
      },
      my_cash_alloc: 10_000.0,
      stonk_feeling: dsta::MarketDirection::GetStonk,           // put credit spread
      option_expire: 30,
      target_spread: 0.20,                                       // 0.20 per contract => $20 per contract
      time_to_order: dsta::BotActionTime {
          relative_days: dow,  // TODAY's day number
          my_entry_time: dsta::UsaMarketTimes::OpenChaos,
          my_entry_wait: chrono::Duration::hours(-24),            // 24 hours BEFORE open (guarantees we're in window)
      },
      follow_a_exit: vec![
          // Exit when we can buy back the spread for ≤ $0.05 (50% profit on 0.05 credit)
          dsta::BuzzBotDoneNiceStyle::SpreadValueGain(0.05),
          // Optional: hard stop loss
          // dsta::BuzzBotDoneNiceStyle::SpreadValueLoss(0.15),
          // Optional: time-based exit (example: 5 days before expiry at power hour)
          /*
          dsta::BuzzBotDoneNiceStyle::SpreadValueTime(
              dsta::BotActionTime {
                  relative_days: 5,                              // 5 = number of days BEFORE expiry
                  my_entry_time: dsta::UsaMarketTimes::PowerHour,
                  my_entry_wait: chrono::Duration::zero(),
              }
          ),
          */
      ],
      algo_cooldown: chrono::Duration::milliseconds(40),               // was 1.0 → now correct type
      bombs_forever: false,
  };

  let (trade_tx, trade_rx) = mpsc::channel(100);
  let (check_tx, check_rx) = mpsc::channel(100);
  let (sucks_tx, mut sucks_rx) = mpsc::channel(100);
  let (pls_ticks_tx, mut pls_ticks_rx) = mpsc::channel(10);
  let (pls_trade_tx, mut pls_trade_rx) = mpsc::channel(10);
  let (swap_info_tx, mut swap_info_rx) = mpsc::channel(10);
  let (new_brain_tx, _) = mpsc::channel(10);
  let (snivtx, mut snivrx) = mpsc::channel(100);

  let bridge = crate::BridgeDrHand
  { tell_dr_pls_ticks: pls_ticks_tx.clone(),
    tell_dr_pls_trade: pls_trade_tx.clone(),
    tell_dr_swap_info: swap_info_tx.clone(),
    tell_dr_new_brain: new_brain_tx.clone(),
    tell_dr_pls_check: mpsc::channel(1).0,
    tell_dr_stop_info: mpsc::channel(1).0,
    tell_dr_stop_tick: mpsc::channel(1).0,
    tell_dr_a_snively: snivtx.clone(),
  };

  let bot_handle = tokio::spawn(crate::sbot::buzz_bomber::fn_run_main_buzz_bot(
    config.clone(),
    trade_rx,
    check_rx,
    sucks_rx,
    bridge,
  ));

  let expiry = Utc::now() + Duration::days(35);

  trace!
  ( r#"
  // ───────────────────────────────────────────────────────────────
  // Phase 1.2: Bot asks for underlying → send chain
  // ───────────────────────────────────────────────────────────────
  "#
  );

  // Create a fuller option chain with ~30 strikes centered around $100
  let mut put_chain = vec![];
  for i in 0..30 {
    let strike = 85.0 + (i as f64 * 2.5);  // strikes from $85 to $157.50
    put_chain.push(dsta::Opt {
      ticker_name: format!(".FAKE{:03.0}P35", strike),
      strike_spot: strike,
      option_name: format!("FAKE {:03.0}_P_35", strike),
      expire_date: expiry,
      option_rite: dsta::OptRite::Put,
      option_type: dsta::OptStyle::Usa,
      ..Default::default()
    });
  }

  let fake_chain = dsta::PushTickerResult
  { ass_deets: dsta::FinAss::StkDeets(dsta::Stk { 
      ticker_name: "FAKE".into(), 
      ..Default::default() 
    }),
    opt_chain: vec![dsta::Derivatives
    { expire_time: expiry,
      option_calls: vec![],
      option_puts: put_chain,
    }],
  };

  let (req_ticker, reply_tx) = pls_ticks_rx.recv().await.expect("No initial ticker request");
  assert_eq!(req_ticker, "FAKE");
  reply_tx.send(Some(fake_chain.clone())).await.unwrap();

  trace!
  ( r#"
  // ───────────────────────────────────────────────────────────────
  // Phase 1.3: Cash swap request
  // ───────────────────────────────────────────────────────────────
  "#
  );

  let _swap = swap_info_rx.recv().await.expect("No cash swap request");
  sucks_tx.send(crate::glossary::SWAP_SUCC.to_string()).await.unwrap();

  trace!
  ( r#"
  // ───────────────────────────────────────────────────────────────
  // Phase 1.4: Send initial allocation
  // ───────────────────────────────────────────────────────────────
  "#
  );

  let mut account = BotaAccount::new().with_cash(10_000.0);
  check_tx.send(crate::Allocation {
    bot_name: bot_name.clone(),
    bot_safe: account.clone(),
    bot_mind: BotMindState::Running,
    num_swap: 0,
  }).await.expect("Failed to send initial cash allocation");
  tokio::time::sleep(StdDuration::from_millis(200)).await;

  trace!
  ( r#"
  // ───────────────────────────────────────────────────────────────
  // Phase 1.5a: Underlying quote → triggers exit of startup 
  // ───────────────────────────────────────────────────────────────
  "#
  );

  trade_tx.send(dsta::Ticker {
    name: "FAKE".into(),
    bid: 99.95,
    ask: 100.05,
    ..Default::default()
  }).await.unwrap();
  tokio::time::sleep(StdDuration::from_millis(120)).await;

  trace!
  ( r#"
  // ───────────────────────────────────────────────────────────────
  // Phase 1.5b: Underlying quote → triggers execution of logic0
  // ───────────────────────────────────────────────────────────────
  "#
  );

  trade_tx.send(dsta::Ticker {
    name: "FAKE".into(),
    bid: 99.95,
    ask: 100.05,
    ..Default::default()
  }).await.unwrap();
  tokio::time::sleep(StdDuration::from_millis(120)).await;


  trace!
  ( r#"
  // ───────────────────────────────────────────────────────────────
  // Phase 1.6: Bot re-requests chain in Logic0 → send full chain
  // ───────────────────────────────────────────────────────────────
  "#
  );

  let (req_ticker2, reply_tx2) = pls_ticks_rx.recv().await.expect("No chain refresh request in Logic0");
  assert_eq!(req_ticker2, "FAKE");
  reply_tx2.send(Some(fake_chain.clone())).await.unwrap();
  
  tokio::time::sleep(StdDuration::from_millis(200)).await;

  trace!(
    r#"
    // ───────────────────────────────────────────────────────────────
    // Phase 1.7: Bot requests tickers for candidate options → reply immediately using original chain
    //            Exit loop after ~1 second of no activity
    // ───────────────────────────────────────────────────────────────
    "#
  );



  let timeout_duration = StdDuration::from_secs(1);
  let mut last_activity = tokio::time::Instant::now();

  loop {
      tokio::select! {
          // Wait for next ticker request
          maybe_req = pls_ticks_rx.recv() => {
              match maybe_req {
                  Some((ticker_name, reply_tx)) => {
                      last_activity = tokio::time::Instant::now();  // reset inactivity timer

                      if !ticker_name.starts_with(".FAKE") {
                          trace!("Ignoring non-option ticker request: {}", ticker_name);
                          continue;
                      }

                      // Find matching Opt from the chain we sent earlier
                      let matching_opt = fake_chain.opt_chain.iter()
                          .flat_map(|deriv| &deriv.option_puts)
                          .find(|opt| opt.ticker_name == ticker_name)
                          .cloned();

                      let response = matching_opt.map(|opt| dsta::PushTickerResult {
                          ass_deets: dsta::FinAss::OptDeets(opt),
                          opt_chain: vec![],
                      });

                      // Send reply right away
                      let _ = reply_tx.send(response).await;
                      trace!("Replied to option request: {}", ticker_name);
                  }
                  None => {
                      trace!("Option ticker channel closed unexpectedly");
                      break;
                  }
              }
          }

          // Inactivity timeout: exit if no request for 1 second
          _ = tokio::time::sleep_until(last_activity + timeout_duration) => {
              trace!("No more option ticker requests for 1 second → moving to next phase");
              break;
          }
      }
  }


  tokio::time::sleep(StdDuration::from_millis(200)).await;

  trace!
  ( r#"
  // ───────────────────────────────────────────────────────────────
  // Phase 1.8: Send quotes for candidate options → trigger entry
  // ───────────────────────────────────────────────────────────────
  "#
  );

  // Send quotes for options around the $95-$100 strike range
  // This should create a qualifying put credit spread
  for strike in [92.5, 95.0, 97.5, 100.0, 102.5, 105.0].iter() {
    let bid = if *strike < 100.0 { 
      0.30 + (100.0 - strike) * 0.05  // OTM puts worth less
    } else { 
      0.50 + (strike - 100.0) * 0.08  // ITM puts worth more
    };
    let ask = bid + 0.10;
    
    trade_tx.send(dsta::Ticker {
      name: format!(".FAKE{:03.0}P35", strike),
      bid,
      ask,
      ..Default::default()
    }).await.unwrap();
  }

  tokio::time::sleep(StdDuration::from_millis(300)).await;

  trace!
  ( r#"
  // ───────────────────────────────────────────────────────────────
  // Phase 1.9: Expect entry order for put credit spread
  // ───────────────────────────────────────────────────────────────
  "#
  );

  let entry_order = pls_trade_rx.recv().await.expect("No entry order");
  assert_eq!(entry_order.order_legs.len(), 2);
  assert!(entry_order.order_legs[0].action == dsta::BuyOrSell::BuyToOpen || entry_order.order_legs[1].action == dsta::BuyOrSell::BuyToOpen);
  assert!(entry_order.order_legs[0].action == dsta::BuyOrSell::SellToOpen || entry_order.order_legs[1].action == dsta::BuyOrSell::SellToOpen);

  info!("Entry order received: {:?}", entry_order);

  trace!
  ( r#"
  // ───────────────────────────────────────────────────────────────
  // Phase 1.9b: Law 2 — confirm ORDER_FLY for entry order
  //             Bot is blocking in wait_for_buzz_order_confirmation
  // ───────────────────────────────────────────────────────────────
  "#
  );

  // Law 2: bot is now blocked waiting for ORDER_FLY — send it before doing anything else
  sucks_tx.send(crate::glossary::ORDER_FLY.to_string()).await.expect("Failed to send ORDER_FLY for entry");

  trace!
  ( r#"
  // ───────────────────────────────────────────────────────────────
  // Phase 1.10: Apply entry fill to account
  // ───────────────────────────────────────────────────────────────
  "#
  );

  // Build assets map from the actual order legs
  let mut assets: HashMap<String, FinAss> = HashMap::new();
  for leg in &entry_order.order_legs {
    if let FinAss::OptDeets(ref opt) = leg.buy_what {
      assets.insert(opt.option_name.clone(), FinAss::OptDeets(opt.clone()));
    }
  }

  let changes = fn_get_account_shennanagans(&entry_order, &account, &assets).unwrap();
  apply_account_cash_and_collateral_changes(&mut account, &changes);
  apply_order_to_account(&mut account, &entry_order, &assets).unwrap();

  check_tx.send(crate::Allocation {
    bot_name: bot_name.clone(),
    bot_safe: account.clone(),
    bot_mind: BotMindState::Running,
    num_swap: 0,
  }).await.unwrap();
  tokio::time::sleep(StdDuration::from_millis(150)).await;

  trace!
  ( r#"
  // ───────────────────────────────────────────────────────────────
  // Phase 1.11: Wait for cooldown to expire
  // ───────────────────────────────────────────────────────────────
  "#
  );

  tokio::time::sleep(StdDuration::from_secs(2)).await;

  trace!
  ( r#"
  // ───────────────────────────────────────────────────────────────
  // Phase 2: Favorable market move → exit triggered
  // ───────────────────────────────────────────────────────────────
  "#
  );

  trace!
  ( r#"
  // ───────────────────────────────────────────────────────────────
  // Phase 2.1: Send quotes showing profit condition met
  //            (spread value has decreased enough to exit)
  // ───────────────────────────────────────────────────────────────
  "#
  );

  // Determine which legs were actually used from the entry order
  let mut short_ticker = String::new();
  let mut long_ticker = String::new();
  
  for leg in &entry_order.order_legs {
    if let FinAss::OptDeets(ref opt) = leg.buy_what {
      match leg.action {
        dsta::BuyOrSell::SellToOpen => short_ticker = opt.ticker_name.clone(),
        dsta::BuyOrSell::BuyToOpen => long_ticker = opt.ticker_name.clone(),
        _ => {}
      }
    }
  }

  // Send favorable quotes (underlying moved up, puts now cheaper)
  trade_tx.send(dsta::Ticker {
    name: long_ticker.clone(),
    bid: 0.02,
    ask: 0.04,
    ..Default::default()
  }).await.unwrap();

  trade_tx.send(dsta::Ticker {
    name: short_ticker.clone(),
    bid: 0.03,
    ask: 0.07,
    ..Default::default()
  }).await.unwrap();

  trade_tx.send(dsta::Ticker {
    name: "FAKE".into(),
    bid: 104.90,
    ask: 105.10,
    ..Default::default()
  }).await.unwrap();

  tokio::time::sleep(StdDuration::from_millis(200)).await;

  trace!
  ( r#"
  // ───────────────────────────────────────────────────────────────
  // Phase 2.2: Expect exit/close order
  // ───────────────────────────────────────────────────────────────
  "#
  );

  let close_order = pls_trade_rx.recv().await.expect("No close order after profit trigger");
  assert_eq!(close_order.order_legs.len(), 2);
  assert!(close_order.order_legs.iter().any(|l| l.action == dsta::BuyOrSell::SellToClose));
  assert!(close_order.order_legs.iter().any(|l| l.action == dsta::BuyOrSell::BuyToClose));

  trace!
  ( r#"
  // ───────────────────────────────────────────────────────────────
  // Phase 2.2b: Law 2 — confirm ORDER_FLY for exit order
  //             Bot is blocking in wait_for_buzz_order_confirmation
  // ───────────────────────────────────────────────────────────────
  "#
  );

  // Law 2: bot is now blocked waiting for ORDER_FLY on the exit order
  sucks_tx.send(crate::glossary::ORDER_FLY.to_string()).await.expect("Failed to send ORDER_FLY for exit");

  trace!
  ( r#"
  // ───────────────────────────────────────────────────────────────
  // Phase 2.3: Apply close fill
  // ───────────────────────────────────────────────────────────────
  "#
  );

  let changes = fn_get_account_shennanagans(&close_order, &account, &assets).unwrap();
  apply_account_cash_and_collateral_changes(&mut account, &changes);
  apply_order_to_account(&mut account, &close_order, &assets).unwrap();

  check_tx.send(crate::Allocation {
    bot_name: bot_name.clone(),
    bot_safe: account.clone(),
    bot_mind: BotMindState::Running,
    num_swap: 0,
  }).await.unwrap();
  tokio::time::sleep(StdDuration::from_millis(150)).await;

  trace!
  ( r#"
  // ───────────────────────────────────────────────────────────────
  // Phase 3: Cleanup
  // ───────────────────────────────────────────────────────────────
  "#
  );

  let _ = sucks_tx.send("DIE".to_string()).await;
  let _ = bot_handle.await.expect("Bot didn't shut down cleanly");

  drop(trade_tx);
  drop(check_tx);

  tokio::time::sleep(StdDuration::from_millis(200)).await;

  info!("Test completed - put credit spread entry → exit cycle validated");
}

#[tokio::test] 
async fn buzz_bomber_happy_path_call_credit_spread()
{
  // Do not change my formatting, spacing, or comments.
  // WORKING - buzz bomber call credit spread entry → exit cycle
  use tokio::sync::mpsc;
  use chrono::{Utc, Duration, Datelike, TimeZone};
  use std::time::Duration as StdDuration;
  use log::{info, debug, trace};
  use crate::*;
  use crate::tests::util::*;
  use crate::dr_robo::*;
  use dsta::*;
  use std::collections::HashMap;
  
  let _ = setup_log4rs("buzz_bomber_happy_path_call_credit_spread");
  info!("Starting buzz_bomber_happy_path_call_credit_spread test");

  let bot_name = "buzz_test_bot_calls".to_string();

  trace!
  ( r#"
  // ───────────────────────────────────────────────────────────────
  // Phase 1: Fresh start → entry → exit cycle
  // ───────────────────────────────────────────────────────────────
  "#
  );

  trace!
  ( r#"
  // ───────────────────────────────────────────────────────────────
  // Phase 1.1: Initialize channels and start bot
  // ───────────────────────────────────────────────────────────────
  "#
  );

  let reference = Utc::now(); 
  let lwd = reference.weekday();
  let dow = lwd.number_from_sunday() as u16;

  log::debug!("wrong time_to_order lwd = {:#?}, day_of_week = {} ",lwd, dow);

  let ny = reference.with_timezone(&chrono_tz::America::New_York);
  //let today_eastern = ny.date_naive();

  // Are we even on the correct day of week?
  let nyw = ny.weekday();
  let dow = nyw.num_days_from_sunday() as u16;

  log::debug!("est time_to_order nyw = {:#?}, day_of_week = {} ",nyw, dow);

  let config = dsta::MakeBuzzBomber {
      my_accounting: {
          let mut ma = dsta::CommonBotInfo::default();
          ma.friendly_name = bot_name.clone();
          ma.max_cash_risk = (true, 100.0, 10_000.0);
          ma.tracking_tick = "FAKE".to_string();
          ma
      },
      my_cash_alloc: 10_000.0,
      stonk_feeling: dsta::MarketDirection::Corrects,           // call credit spread
      option_expire: 30,
      target_spread: 0.20,                                       // 0.20 per contract => $20 per contract
      time_to_order: dsta::BotActionTime {
          relative_days: dow,  // TODAY's day number
          my_entry_time: dsta::UsaMarketTimes::OpenChaos,
          my_entry_wait: chrono::Duration::hours(-24),            // 24 hours BEFORE open (guarantees we're in window)
      },
      follow_a_exit: vec![
          // Exit when we can buy back the spread for ≤ $0.05 (50% profit on 0.05 credit)
          dsta::BuzzBotDoneNiceStyle::SpreadValueGain(0.05),
          // Optional: hard stop loss
          // dsta::BuzzBotDoneNiceStyle::SpreadValueLoss(0.15),
          // Optional: time-based exit (example: 5 days before expiry at power hour)
          /*
          dsta::BuzzBotDoneNiceStyle::SpreadValueTime(
              dsta::BotActionTime {
                  relative_days: 5,                              // 5 = number of days BEFORE expiry
                  my_entry_time: dsta::UsaMarketTimes::PowerHour,
                  my_entry_wait: chrono::Duration::zero(),
              }
          ),
          */
      ],
      algo_cooldown: chrono::Duration::milliseconds(40),               // was 1.0 → now correct type
      bombs_forever: false,
  };

  let (trade_tx, trade_rx) = mpsc::channel(100);
  let (check_tx, check_rx) = mpsc::channel(100);
  let (sucks_tx, mut sucks_rx) = mpsc::channel(100);
  let (pls_ticks_tx, mut pls_ticks_rx) = mpsc::channel(10);
  let (pls_trade_tx, mut pls_trade_rx) = mpsc::channel(10);
  let (swap_info_tx, mut swap_info_rx) = mpsc::channel(10);
  let (new_brain_tx, _) = mpsc::channel(10);
  let (snivtx, mut snivrx) = mpsc::channel(100);

  let bridge = crate::BridgeDrHand
  { tell_dr_pls_ticks: pls_ticks_tx.clone(),
    tell_dr_pls_trade: pls_trade_tx.clone(),
    tell_dr_swap_info: swap_info_tx.clone(),
    tell_dr_new_brain: new_brain_tx.clone(),
    tell_dr_pls_check: mpsc::channel(1).0,
    tell_dr_stop_info: mpsc::channel(1).0,
    tell_dr_stop_tick: mpsc::channel(1).0,
    tell_dr_a_snively: snivtx.clone(),
  };

  let bot_handle = tokio::spawn(crate::sbot::buzz_bomber::fn_run_main_buzz_bot(
    config.clone(),
    trade_rx,
    check_rx,
    sucks_rx,
    bridge,
  ));

  let expiry = Utc::now() + Duration::days(35);

  trace!
  ( r#"
  // ───────────────────────────────────────────────────────────────
  // Phase 1.2: Bot asks for underlying → send chain
  // ───────────────────────────────────────────────────────────────
  "#
  );

  // Create a fuller option chain with ~30 strikes centered around $100
  let mut call_chain = vec![];
  for i in 0..30 {
    let strike = 85.0 + (i as f64 * 2.5);  // strikes from $85 to $157.50
    call_chain.push(dsta::Opt {
      ticker_name: format!(".FAKE{:03.0}C35", strike),
      strike_spot: strike,
      option_name: format!("FAKE {:03.0}_C_35", strike),
      expire_date: expiry,
      option_rite: dsta::OptRite::Call,
      option_type: dsta::OptStyle::Usa,
      ..Default::default()
    });
  }

  let fake_chain = dsta::PushTickerResult
  { ass_deets: dsta::FinAss::StkDeets(dsta::Stk { 
      ticker_name: "FAKE".into(), 
      ..Default::default() 
    }),
    opt_chain: vec![dsta::Derivatives
    { expire_time: expiry,
      option_calls: call_chain,
      option_puts: vec![],
    }],
  };

  let (req_ticker, reply_tx) = pls_ticks_rx.recv().await.expect("No initial ticker request");
  assert_eq!(req_ticker, "FAKE");
  reply_tx.send(Some(fake_chain.clone())).await.unwrap();

  trace!
  ( r#"
  // ───────────────────────────────────────────────────────────────
  // Phase 1.3: Cash swap request
  // ───────────────────────────────────────────────────────────────
  "#
  );

  let _swap = swap_info_rx.recv().await.expect("No cash swap request");
  sucks_tx.send(crate::glossary::SWAP_SUCC.to_string()).await.unwrap();

  trace!
  ( r#"
  // ───────────────────────────────────────────────────────────────
  // Phase 1.4: Send initial allocation
  // ───────────────────────────────────────────────────────────────
  "#
  );

  let mut account = BotaAccount::new().with_cash(10_000.0);
  check_tx.send(crate::Allocation {
    bot_name: bot_name.clone(),
    bot_safe: account.clone(),
    bot_mind: BotMindState::Running,
    num_swap: 0,
  }).await.expect("Failed to send initial cash allocation");
  tokio::time::sleep(StdDuration::from_millis(200)).await;

  trace!
  ( r#"
  // ───────────────────────────────────────────────────────────────
  // Phase 1.5a: Underlying quote → triggers exit of startup 
  // ───────────────────────────────────────────────────────────────
  "#
  );

  trade_tx.send(dsta::Ticker {
    name: "FAKE".into(),
    bid: 99.95,
    ask: 100.05,
    ..Default::default()
  }).await.unwrap();
  tokio::time::sleep(StdDuration::from_millis(120)).await;

  trace!
  ( r#"
  // ───────────────────────────────────────────────────────────────
  // Phase 1.5b: Underlying quote → triggers execution of logic0
  // ───────────────────────────────────────────────────────────────
  "#
  );

  trade_tx.send(dsta::Ticker {
    name: "FAKE".into(),
    bid: 99.95,
    ask: 100.05,
    ..Default::default()
  }).await.unwrap();
  tokio::time::sleep(StdDuration::from_millis(120)).await;


  trace!
  ( r#"
  // ───────────────────────────────────────────────────────────────
  // Phase 1.6: Bot re-requests chain in Logic0 → send full chain
  // ───────────────────────────────────────────────────────────────
  "#
  );

  let (req_ticker2, reply_tx2) = pls_ticks_rx.recv().await.expect("No chain refresh request in Logic0");
  assert_eq!(req_ticker2, "FAKE");
  reply_tx2.send(Some(fake_chain.clone())).await.unwrap();
  
  tokio::time::sleep(StdDuration::from_millis(200)).await;

  trace!(
    r#"
    // ───────────────────────────────────────────────────────────────
    // Phase 1.7: Bot requests tickers for candidate options → reply immediately using original chain
    //            Exit loop after ~1 second of no activity
    // ───────────────────────────────────────────────────────────────
    "#
  );


  let timeout_duration = StdDuration::from_secs(1);
  let mut last_activity = tokio::time::Instant::now();

  loop {
      tokio::select! {
          // Wait for next ticker request
          maybe_req = pls_ticks_rx.recv() => {
              match maybe_req {
                  Some((ticker_name, reply_tx)) => {
                      last_activity = tokio::time::Instant::now();  // reset inactivity timer

                      if !ticker_name.starts_with(".FAKE") {
                          trace!("Ignoring non-option ticker request: {}", ticker_name);
                          continue;
                      }

                      // Find matching Opt from the chain we sent earlier
                      let matching_opt = fake_chain.opt_chain.iter()
                          .flat_map(|deriv| &deriv.option_calls)
                          .find(|opt| opt.ticker_name == ticker_name)
                          .cloned();

                      let response = matching_opt.map(|opt| dsta::PushTickerResult {
                          ass_deets: dsta::FinAss::OptDeets(opt),
                          opt_chain: vec![],
                      });

                      // Send reply right away
                      let _ = reply_tx.send(response).await;
                      trace!("Replied to option request: {}", ticker_name);
                  }
                  None => {
                      trace!("Option ticker channel closed unexpectedly");
                      break;
                  }
              }
          }

          // Inactivity timeout: exit if no request for 1 second
          _ = tokio::time::sleep_until(last_activity + timeout_duration) => {
              trace!("No more option ticker requests for 1 second → moving to next phase");
              break;
          }
      }
  }


  tokio::time::sleep(StdDuration::from_millis(200)).await;

  trace!
  ( r#"
  // ───────────────────────────────────────────────────────────────
  // Phase 1.8: Send quotes for candidate options → trigger entry
  // ───────────────────────────────────────────────────────────────
  "#
  );

  // Send quotes for options around the $95-$100 strike range
  // This should create a qualifying call credit spread
  for strike in [92.5, 95.0, 97.5, 100.0, 102.5, 105.0].iter() {
    let bid = if *strike < 100.0 { 
      0.50 + (100.0 - strike) * 0.08  // OTM calls worth less
    } else { 
      0.30 + (strike - 100.0) * 0.05  // ITM callss worth more
    };
    let ask = bid + 0.10;
    
    trade_tx.send(dsta::Ticker {
      name: format!(".FAKE{:03.0}C35", strike),
      bid,
      ask,
      ..Default::default()
    }).await.unwrap();
  }

  tokio::time::sleep(StdDuration::from_millis(300)).await;

  trace!
  ( r#"
  // ───────────────────────────────────────────────────────────────
  // Phase 1.9: Expect entry order for call credit spread
  // ───────────────────────────────────────────────────────────────
  "#
  );

  let entry_order = pls_trade_rx.recv().await.expect("No entry order");
  assert_eq!(entry_order.order_legs.len(), 2);
  assert!(entry_order.order_legs[0].action == dsta::BuyOrSell::BuyToOpen || entry_order.order_legs[1].action == dsta::BuyOrSell::BuyToOpen);
  assert!(entry_order.order_legs[0].action == dsta::BuyOrSell::SellToOpen || entry_order.order_legs[1].action == dsta::BuyOrSell::SellToOpen);

  info!("Entry order received: {:?}", entry_order);

  trace!
  ( r#"
  // ───────────────────────────────────────────────────────────────
  // Phase 1.9b: Law 2 — confirm ORDER_FLY for entry order
  //             Bot is blocking in wait_for_buzz_order_confirmation
  // ───────────────────────────────────────────────────────────────
  "#
  );

  // Law 2: bot is now blocked waiting for ORDER_FLY — send it before doing anything else
  sucks_tx.send(crate::glossary::ORDER_FLY.to_string()).await.expect("Failed to send ORDER_FLY for entry");

  trace!
  ( r#"
  // ───────────────────────────────────────────────────────────────
  // Phase 1.10: Apply entry fill to account
  // ───────────────────────────────────────────────────────────────
  "#
  );

  // Build assets map from the actual order legs
  let mut assets: HashMap<String, FinAss> = HashMap::new();
  for leg in &entry_order.order_legs {
    if let FinAss::OptDeets(ref opt) = leg.buy_what {
      assets.insert(opt.option_name.clone(), FinAss::OptDeets(opt.clone()));
    }
  }

  let changes = fn_get_account_shennanagans(&entry_order, &account, &assets).unwrap();
  apply_account_cash_and_collateral_changes(&mut account, &changes);
  apply_order_to_account(&mut account, &entry_order, &assets).unwrap();

  check_tx.send(crate::Allocation {
    bot_name: bot_name.clone(),
    bot_safe: account.clone(),
    bot_mind: BotMindState::Running,
    num_swap: 0,
  }).await.unwrap();
  tokio::time::sleep(StdDuration::from_millis(150)).await;

  trace!
  ( r#"
  // ───────────────────────────────────────────────────────────────
  // Phase 1.11: Wait for cooldown to expire
  // ───────────────────────────────────────────────────────────────
  "#
  );

  tokio::time::sleep(StdDuration::from_secs(2)).await;

  trace!
  ( r#"
  // ───────────────────────────────────────────────────────────────
  // Phase 2: Favorable market move → exit triggered
  // ───────────────────────────────────────────────────────────────
  "#
  );

  trace!
  ( r#"
  // ───────────────────────────────────────────────────────────────
  // Phase 2.1: Send quotes showing profit condition met
  //            (spread value has decreased enough to exit)
  // ───────────────────────────────────────────────────────────────
  "#
  );

  // Determine which legs were actually used from the entry order
  let mut short_ticker = String::new();
  let mut long_ticker = String::new();
  
  for leg in &entry_order.order_legs {
    if let FinAss::OptDeets(ref opt) = leg.buy_what {
      match leg.action {
        dsta::BuyOrSell::SellToOpen => short_ticker = opt.ticker_name.clone(),
        dsta::BuyOrSell::BuyToOpen => long_ticker = opt.ticker_name.clone(),
        _ => {}
      }
    }
  }

  // Send favorable quotes (underlying moved up, calls now cheaper)
  trade_tx.send(dsta::Ticker {
    name: long_ticker.clone(),
    bid: 0.02,
    ask: 0.04,
    ..Default::default()
  }).await.unwrap();

  trade_tx.send(dsta::Ticker {
    name: short_ticker.clone(),
    bid: 0.03,
    ask: 0.07,
    ..Default::default()
  }).await.unwrap();

  trade_tx.send(dsta::Ticker {
    name: "FAKE".into(),
    bid: 104.90,
    ask: 105.10,
    ..Default::default()
  }).await.unwrap();

  tokio::time::sleep(StdDuration::from_millis(200)).await;

  trace!
  ( r#"
  // ───────────────────────────────────────────────────────────────
  // Phase 2.2: Expect exit/close order
  // ───────────────────────────────────────────────────────────────
  "#
  );

  let close_order = pls_trade_rx.recv().await.expect("No close order after profit trigger");
  assert_eq!(close_order.order_legs.len(), 2);
  assert!(close_order.order_legs.iter().any(|l| l.action == dsta::BuyOrSell::SellToClose));
  assert!(close_order.order_legs.iter().any(|l| l.action == dsta::BuyOrSell::BuyToClose));

  trace!
  ( r#"
  // ───────────────────────────────────────────────────────────────
  // Phase 2.2b: Law 2 — confirm ORDER_FLY for exit order
  //             Bot is blocking in wait_for_buzz_order_confirmation
  // ───────────────────────────────────────────────────────────────
  "#
  );

  // Law 2: bot is now blocked waiting for ORDER_FLY on the exit order
  sucks_tx.send(crate::glossary::ORDER_FLY.to_string()).await.expect("Failed to send ORDER_FLY for exit");

  trace!
  ( r#"
  // ───────────────────────────────────────────────────────────────
  // Phase 2.3: Apply close fill
  // ───────────────────────────────────────────────────────────────
  "#
  );

  let changes = fn_get_account_shennanagans(&close_order, &account, &assets).unwrap();
  apply_account_cash_and_collateral_changes(&mut account, &changes);
  apply_order_to_account(&mut account, &close_order, &assets).unwrap();

  check_tx.send(crate::Allocation {
    bot_name: bot_name.clone(),
    bot_safe: account.clone(),
    bot_mind: BotMindState::Running,
    num_swap: 0,
  }).await.unwrap();
  tokio::time::sleep(StdDuration::from_millis(150)).await;

  trace!
  ( r#"
  // ───────────────────────────────────────────────────────────────
  // Phase 3: Cleanup
  // ───────────────────────────────────────────────────────────────
  "#
  );

  let _ = sucks_tx.send("DIE".to_string()).await;
  let _ = bot_handle.await.expect("Bot didn't shut down cleanly");

  drop(trade_tx);
  drop(check_tx);

  tokio::time::sleep(StdDuration::from_millis(200)).await;

  info!("Test completed - call credit spread entry → exit cycle validated");
}


#[tokio::test] // UPDATED - now with stop/save/restart/resume cycle
async fn buzz_bomber_happy_path_put_credit_spread_with_restart()
{
  // Do not change my formatting, spacing, or comments.
  // WORKING - buzz bomber put credit spread entry → partial profit → stop/save → restart → resume → final exit
  use tokio::sync::mpsc;
  use chrono::{Utc, Duration, Datelike, TimeZone};
  use std::time::Duration as StdDuration;
  use log::{info, debug, trace};
  use crate::*;
  use crate::tests::util::*;
  use crate::dr_robo::*;
  use dsta::*;
  use std::collections::HashMap;
  use std::path::Path;

  let _ = setup_log4rs("buzz_bomber_happy_path_put_credit_spread_with_restart");
  info!("Starting buzz_bomber_happy_path_put_credit_spread_with_restart test");

  let bot_name = "buzz_test_bot_puts".to_string();
  let friendly_path = format!("./dat/{}.json", bot_name);

  // Clean up any old save file from previous test runs
  if Path::new(&friendly_path).exists() {
      let _ = tokio::fs::remove_file(&friendly_path).await;
      info!("Removed old save file before test start");
  }

  trace!
  ( r#"
  // ───────────────────────────────────────────────────────────────
  // Phase 1: Fresh start → entry → partial profit
  // ───────────────────────────────────────────────────────────────
  "# );

  trace!
  ( r#"
  // ───────────────────────────────────────────────────────────────
  // Phase 1.1: Initialize channels and start bot
  // ───────────────────────────────────────────────────────────────
  "# );

  let reference = Utc::now();
  let ny = reference.with_timezone(&chrono_tz::America::New_York);
  let nyw = ny.weekday();
  let dow = nyw.num_days_from_sunday() as u16;
  log::debug!("est time_to_order nyw = {:#?}, day_of_week = {} ",nyw, dow);

  let config = dsta::MakeBuzzBomber {
      my_accounting: {
          let mut ma = dsta::CommonBotInfo::default();
          ma.friendly_name = bot_name.clone();
          ma.max_cash_risk = (true, 100.0, 10_000.0);
          ma.tracking_tick = "FAKE".to_string();
          ma
      },
      my_cash_alloc: 10_000.0,
      stonk_feeling: dsta::MarketDirection::GetStonk, // put credit spread
      option_expire: 30,
      target_spread: 0.20,
      time_to_order: dsta::BotActionTime {
          relative_days: dow,
          my_entry_time: dsta::UsaMarketTimes::OpenChaos,
          my_entry_wait: chrono::Duration::hours(-24),
      },
      follow_a_exit: vec![
          dsta::BuzzBotDoneNiceStyle::SpreadValueGain(0.05),
          // dsta::BuzzBotDoneNiceStyle::SpreadValueLoss(0.15),
      ],
      algo_cooldown: chrono::Duration::milliseconds(40),
      bombs_forever: false,
  };

  let (trade_tx, trade_rx)     = mpsc::channel(100);
  let (check_tx, check_rx)     = mpsc::channel(100);
  let (sucks_tx, mut sucks_rx) = mpsc::channel(100);
  let (pls_ticks_tx, mut pls_ticks_rx) = mpsc::channel(10);
  let (pls_trade_tx, mut pls_trade_rx) = mpsc::channel(10);
  let (swap_info_tx, mut swap_info_rx) = mpsc::channel(10);
  let (new_brain_tx, _)        = mpsc::channel(10);
  let (snivtx, mut snivrx)     = mpsc::channel(100);

  let bridge = crate::BridgeDrHand {
      tell_dr_pls_ticks: pls_ticks_tx.clone(),
      tell_dr_pls_trade: pls_trade_tx.clone(),
      tell_dr_swap_info: swap_info_tx.clone(),
      tell_dr_new_brain: new_brain_tx.clone(),
      tell_dr_pls_check: mpsc::channel(1).0,
      tell_dr_stop_info: mpsc::channel(1).0,
      tell_dr_stop_tick: mpsc::channel(1).0,
      tell_dr_a_snively: snivtx.clone(),
  };

  let bot_handle = tokio::spawn(crate::sbot::buzz_bomber::fn_run_main_buzz_bot(
      config.clone(),
      trade_rx,
      check_rx,
      sucks_rx,
      bridge,
  ));

  let expiry = Utc::now() + Duration::days(35);

  trace!
  ( r#"
  // ───────────────────────────────────────────────────────────────
  // Phase 1.2: Bot asks for underlying → send chain
  // ───────────────────────────────────────────────────────────────
  "# );

  let mut put_chain = vec![];
  for i in 0..30 {
      let strike = 85.0 + (i as f64 * 2.5);
      put_chain.push(dsta::Opt {
          ticker_name: format!(".FAKE{:03.0}P35", strike),
          strike_spot: strike,
          option_name: format!("FAKE {:03.0}_P_35", strike),
          expire_date: expiry,
          option_rite: dsta::OptRite::Put,
          option_type: dsta::OptStyle::Usa,
          ..Default::default()
      });
  }

  let fake_chain = dsta::PushTickerResult {
      ass_deets: dsta::FinAss::StkDeets(dsta::Stk {
          ticker_name: "FAKE".into(),
          ..Default::default()
      }),
      opt_chain: vec![dsta::Derivatives {
          expire_time: expiry,
          option_calls: vec![],
          option_puts: put_chain,
      }],
  };

  let (req_ticker, reply_tx) = pls_ticks_rx.recv().await.expect("No initial ticker request");
  assert_eq!(req_ticker, "FAKE");
  reply_tx.send(Some(fake_chain.clone())).await.unwrap();

  trace!
  ( r#"
  // ───────────────────────────────────────────────────────────────
  // Phase 1.3: Cash swap request
  // ───────────────────────────────────────────────────────────────
  "# );

  let _swap = swap_info_rx.recv().await.expect("No cash swap request");
  sucks_tx.send(crate::glossary::SWAP_SUCC.to_string()).await.unwrap();

  trace!
  ( r#"
  // ───────────────────────────────────────────────────────────────
  // Phase 1.4: Send initial allocation
  // ───────────────────────────────────────────────────────────────
  "# );

  let mut account = BotaAccount::new().with_cash(10_000.0);
  check_tx.send(crate::Allocation {
      bot_name: bot_name.clone(),
      bot_safe: account.clone(),
      bot_mind: BotMindState::Running,
      num_swap: 0,
  }).await.expect("Failed to send initial cash allocation");

  tokio::time::sleep(StdDuration::from_millis(200)).await;

  trace!
  ( r#"
  // ───────────────────────────────────────────────────────────────
  // Phase 1.5: Underlying quotes → trigger logic
  // ───────────────────────────────────────────────────────────────
  "# );

  trade_tx.send(dsta::Ticker { name: "FAKE".into(), bid: 99.95, ask: 100.05, ..Default::default() }).await.unwrap();
  tokio::time::sleep(StdDuration::from_millis(120)).await;
  trade_tx.send(dsta::Ticker { name: "FAKE".into(), bid: 99.95, ask: 100.05, ..Default::default() }).await.unwrap();
  tokio::time::sleep(StdDuration::from_millis(120)).await;

  trace!
  ( r#"
  // ───────────────────────────────────────────────────────────────
  // Phase 1.6: Bot re-requests chain → reply
  // ───────────────────────────────────────────────────────────────
  "# );

  let (req_ticker2, reply_tx2) = pls_ticks_rx.recv().await.expect("No chain refresh request");
  assert_eq!(req_ticker2, "FAKE");
  reply_tx2.send(Some(fake_chain.clone())).await.unwrap();

  tokio::time::sleep(StdDuration::from_millis(200)).await;

  // ───────────────────────────────────────────────────────────────
  // Phase 1.7: Option ticker requests loop (short version)
  // ───────────────────────────────────────────────────────────────
  let timeout_duration = StdDuration::from_secs(1);
  let mut last_activity = tokio::time::Instant::now();

  loop {
      tokio::select! {
          maybe_req = pls_ticks_rx.recv() => {
              match maybe_req {
                  Some((ticker_name, reply_tx)) => {
                      last_activity = tokio::time::Instant::now();
                      if !ticker_name.starts_with(".FAKE") { continue; }

                      let matching_opt = fake_chain.opt_chain.iter()
                          .flat_map(|d| &d.option_puts)
                          .find(|o| o.ticker_name == ticker_name)
                          .cloned();

                      let response = matching_opt.map(|opt| dsta::PushTickerResult {
                          ass_deets: dsta::FinAss::OptDeets(opt),
                          opt_chain: vec![],
                      });

                      let _ = reply_tx.send(response).await;
                  }
                  None => break,
              }
          }
          _ = tokio::time::sleep_until(last_activity + timeout_duration) => {
              trace!("No more option requests → continuing");
              break;
          }
      }
  }

  tokio::time::sleep(StdDuration::from_millis(200)).await;

  trace!
  ( r#"
  // ───────────────────────────────────────────────────────────────
  // Phase 1.8: Send option quotes → trigger entry
  // ───────────────────────────────────────────────────────────────
  "# );

  for strike in [92.5, 95.0, 97.5, 100.0, 102.5, 105.0] {
      let bid = if strike < 100.0 { 0.30 + (100.0 - strike) * 0.05 } else { 0.50 + (strike - 100.0) * 0.08 };
      let ask = bid + 0.10;

      trade_tx.send(dsta::Ticker {
          name: format!(".FAKE{:03.0}P35", strike),
          bid, ask,
          ..Default::default()
      }).await.unwrap();
  }

  tokio::time::sleep(StdDuration::from_millis(300)).await;

  let entry_order = pls_trade_rx.recv().await.expect("No entry order");
  assert_eq!(entry_order.order_legs.len(), 2);
  info!("Entry order received: {:?}", entry_order);

  trace!
  ( r#"
  // ───────────────────────────────────────────────────────────────
  // Phase 1.8b: Law 2 — confirm ORDER_FLY for entry order
  //             Bot is blocking in wait_for_buzz_order_confirmation
  // ───────────────────────────────────────────────────────────────
  "# );

  // Law 2: bot is now blocked waiting for ORDER_FLY — send it before applying fill
  sucks_tx.send(crate::glossary::ORDER_FLY.to_string()).await.expect("Failed to send ORDER_FLY for entry");

  // Build assets from actual legs
  let mut assets: HashMap<String, FinAss> = HashMap::new();
  for leg in &entry_order.order_legs {
      if let FinAss::OptDeets(ref opt) = leg.buy_what {
          assets.insert(opt.option_name.clone(), FinAss::OptDeets(opt.clone()));
      }
  }

  let changes = fn_get_account_shennanagans(&entry_order, &account, &assets).unwrap();
  apply_account_cash_and_collateral_changes(&mut account, &changes);
  apply_order_to_account(&mut account, &entry_order, &assets).unwrap();

  check_tx.send(crate::Allocation {
      bot_name: bot_name.clone(),
      bot_safe: account.clone(),
      bot_mind: BotMindState::Running,
      num_swap: 0,
  }).await.unwrap();

  tokio::time::sleep(StdDuration::from_millis(150)).await;
  tokio::time::sleep(StdDuration::from_secs(2)).await; // cooldown

  trace!
  ( r#"
  // ───────────────────────────────────────────────────────────────
  // Phase 1.9: Partial favorable move (profit but not full exit yet)
  // ───────────────────────────────────────────────────────────────
  "# );

  let mut short_ticker = String::new();
  let mut long_ticker  = String::new();

  for leg in &entry_order.order_legs {
      if let FinAss::OptDeets(ref opt) = leg.buy_what {
          match leg.action {
              dsta::BuyOrSell::SellToOpen => short_ticker = opt.ticker_name.clone(),
              dsta::BuyOrSell::BuyToOpen  => long_ticker  = opt.ticker_name.clone(),
              _ => {}
          }
      }
  }

  // Partial profit — spread narrowed but not yet ≤ 0.05 debit
  trade_tx.send(dsta::Ticker { name: long_ticker.clone(),  bid: 0.08, ask: 0.12, ..Default::default() }).await.unwrap();
  trade_tx.send(dsta::Ticker { name: short_ticker.clone(), bid: 0.18, ask: 0.24, ..Default::default() }).await.unwrap();
  trade_tx.send(dsta::Ticker { name: "FAKE".into(), bid: 102.80, ask: 103.20, ..Default::default() }).await.unwrap();

  tokio::time::sleep(StdDuration::from_millis(400)).await;

  // ───────────────────────────────────────────────────────────────
  // Phase 2: Graceful stop → save state
  // ───────────────────────────────────────────────────────────────

  info!("=== Telling bot to stop (transition to Stopped) ===");
  check_tx.send(crate::Allocation {
      bot_name: bot_name.clone(),
      bot_safe: account.clone(),
      bot_mind: BotMindState::Stopped,
      num_swap: 0,
  }).await.expect("Failed to send Stopped allocation");

  tokio::time::sleep(StdDuration::from_millis(600)).await; // give time to save

  assert!(Path::new(&friendly_path).exists(), "Save file not created after Stopped");

  trace!
  ( r#"
  // ───────────────────────────────────────────────────────────────
  // Phase 2.1: Kill bot
  // ───────────────────────────────────────────────────────────────
  "# );

  let _ = sucks_tx.send("DIE".to_string()).await;
  let _ = bot_handle.await.expect("Bot didn't shut down cleanly");

  drop(trade_tx);
  drop(check_tx);
  drop(sucks_tx);

  tokio::time::sleep(StdDuration::from_millis(200)).await;

  // ───────────────────────────────────────────────────────────────
  // Phase 3: Restart bot → should load state
  // ───────────────────────────────────────────────────────────────

  trace!
  ( r#"
  // ───────────────────────────────────────────────────────────────
  // Phase 3.1: Recreate channels
  // ───────────────────────────────────────────────────────────────
  "# );

  let (trade_tx, trade_rx)     = mpsc::channel(100);
  let (check_tx, check_rx)     = mpsc::channel(100);
  let (sucks_tx, mut sucks_rx) = mpsc::channel(100);

  let bridge2 = crate::BridgeDrHand {
      tell_dr_pls_ticks: pls_ticks_tx.clone(),
      tell_dr_pls_trade: pls_trade_tx.clone(),
      tell_dr_swap_info: swap_info_tx.clone(),
      tell_dr_new_brain: new_brain_tx.clone(),
      tell_dr_pls_check: mpsc::channel(1).0,
      tell_dr_stop_info: mpsc::channel(1).0,
      tell_dr_stop_tick: mpsc::channel(1).0,
      tell_dr_a_snively: snivtx.clone(),
  };

  trace!
  ( r#"
  // ───────────────────────────────────────────────────────────────
  // Phase 3.2: Spawn restarted bot
  // ───────────────────────────────────────────────────────────────
  "# );

  let bot_handle2 = tokio::spawn(crate::sbot::buzz_bomber::fn_run_main_buzz_bot(
      config.clone(),
      trade_rx,
      check_rx,
      sucks_rx,
      bridge2,
  ));

  tokio::time::sleep(StdDuration::from_millis(300)).await;

  // Should re-request underlying after load
  let (req_after_load, reply_after) = pls_ticks_rx.recv().await.expect("No ticker request after restart");
  assert_eq!(req_after_load, "FAKE");
  reply_after.send(Some(fake_chain)).await.unwrap();

  trace!
  ( r#"
  // ───────────────────────────────────────────────────────────────
  // Phase 3.3: Resume → transition back to Running
  //            Law 9: this Stopped → Running transition also triggers a save
  // ───────────────────────────────────────────────────────────────
  "# );

  info!("=== Resuming bot: transition to Running after load ===");
  check_tx.send(crate::Allocation {
      bot_name: bot_name.clone(),
      bot_safe: account.clone(),
      bot_mind: BotMindState::Running,
      num_swap: 0,
  }).await.unwrap();

  tokio::time::sleep(StdDuration::from_millis(400)).await;

  trace!
  ( r#"
  // ───────────────────────────────────────────────────────────────
  // Phase 4: Now trigger full profit condition → expect exit
  // ───────────────────────────────────────────────────────────────
  "# );

  trade_tx.send(dsta::Ticker { name: long_ticker.clone(),  bid: 0.01, ask: 0.03, ..Default::default() }).await.unwrap();
  trade_tx.send(dsta::Ticker { name: short_ticker.clone(), bid: 0.02, ask: 0.06, ..Default::default() }).await.unwrap();
  trade_tx.send(dsta::Ticker { name: "FAKE".into(), bid: 105.50, ask: 105.90, ..Default::default() }).await.unwrap();

  tokio::time::sleep(StdDuration::from_millis(300)).await;

  let close_order = pls_trade_rx.recv().await.expect("No exit order after resume + profit trigger");
  assert_eq!(close_order.order_legs.len(), 2);
  assert!(close_order.order_legs.iter().any(|l| l.action == dsta::BuyOrSell::SellToClose));
  assert!(close_order.order_legs.iter().any(|l| l.action == dsta::BuyOrSell::BuyToClose));

  info!("Exit order received after restart/resume cycle");

  trace!
  ( r#"
  // ───────────────────────────────────────────────────────────────
  // Phase 4.1: Law 2 — confirm ORDER_FLY for exit order after restart
  //             Bot is blocking in wait_for_buzz_order_confirmation
  // ───────────────────────────────────────────────────────────────
  "# );

  // Law 2: bot is now blocked waiting for ORDER_FLY on exit
  sucks_tx.send(crate::glossary::ORDER_FLY.to_string()).await.expect("Failed to send ORDER_FLY for exit after restart");

  // Optional: apply close fill (not strictly needed for this test)
  let changes_close = fn_get_account_shennanagans(&close_order, &account, &assets).unwrap();
  apply_account_cash_and_collateral_changes(&mut account, &changes_close);
  apply_order_to_account(&mut account, &close_order, &assets).unwrap();

  // ───────────────────────────────────────────────────────────────
  // Phase 5: Cleanup
  // ───────────────────────────────────────────────────────────────

  let _ = sucks_tx.send("DIE".to_string()).await;
  let _ = bot_handle2.await.expect("Restarted bot didn't shut down cleanly");

  let _ = tokio::fs::remove_file(&friendly_path).await;

  drop(trade_tx);
  drop(check_tx);

  info!("Test completed - put credit spread with stop/save/restart/resume/exit cycle validated");
}


// ============================================================================
// TDD STUBS - NON-HAPPY PATH SCENARIOS
// ============================================================================

#[tokio::test]
#[ignore]
async fn buzz_bomber_insufficient_cash() {
    // TODO: Test behavior when cash < $105
    // 
    // Scenario:
    // 1. Start with $500
    // 2. Verify bot enters NoMoney state
    // 3. Increase cash to $1000
    // 4. Verify bot returns to Startup
    panic!("Test not yet implemented - TDD stub");
}

#[tokio::test]
#[ignore]
async fn buzz_bomber_no_valid_expiration() {
    // TODO: Test when no options meet minimum expiry requirement
    // 
    // Scenario:
    // 1. option_expire = 30 days
    // 2. All chains expire in < 30 days
    // 3. Verify error handling and Seppuku
    panic!("Test not yet implemented - TDD stub");
}

#[tokio::test]
#[ignore]
async fn buzz_bomber_loss_exit() {
    // TODO: Test exit when loss limit is hit
    // 
    // Scenario:
    // 1. Enter spread with credit of $2.00
    // 2. Spread widens to debit of $4.00 (-100% loss)
    // 3. Verify bot exits at loss limit (-80%)
    panic!("Test not yet implemented - TDD stub");
}

#[tokio::test]
#[ignore]
async fn buzz_bomber_time_based_exit() {
    // TODO: Test exit based on close_day_num and my_exits_time
    // 
    // Scenario:
    // 1. Enter spread on Monday
    // 2. Fast-forward to Friday 3:00 PM
    // 3. Verify bot exits regardless of P&L
    panic!("Test not yet implemented - TDD stub");
}

#[tokio::test]
#[ignore]
async fn buzz_bomber_wrong_entry_day() {
    // TODO: Test that bot waits for correct entry day
    // 
    // Scenario:
    // 1. relative_days = 3 (Wednesday)
    // 2. Current day is Monday
    // 3. Verify bot stays in Wait state
    // 4. Advance to Wednesday
    // 5. Verify bot proceeds to Entry
    panic!("Test not yet implemented - TDD stub");
}

#[tokio::test]
#[ignore]
async fn buzz_bomber_wrong_entry_time() {
    // TODO: Test that bot waits for correct entry time
    // 
    // Scenario:
    // 1. my_entry_time = PowerHour (3:00 PM)
    // 2. Current time is 10:00 AM
    // 3. Verify bot stays in Entry state
    // 4. Advance to 3:00 PM
    // 5. Verify bot places order
    panic!("Test not yet implemented - TDD stub");
}

#[tokio::test]
#[ignore]
async fn buzz_bomber_cooling_period() {
    // TODO: Test that cooling period is respected
    // 
    // Scenario:
    // 1. algo_cooldown = 5 minutes
    // 2. Spread fills at 10:00 AM
    // 3. Verify bot stays in Cooling until 10:05 AM
    // 4. Verify bot moves to GoExits at 10:05 AM
    panic!("Test not yet implemented - TDD stub");
}

#[tokio::test]
#[ignore]
async fn buzz_bomber_partial_fill_entry() {
    // TODO: Test handling of partial fills on entry
    // 
    // Scenario:
    // 1. Order for 10 contracts
    // 2. Only 6 contracts fill
    // 3. Verify bot adjusts quantity tracking
    // 4. Or cancels and retries
    panic!("Test not yet implemented - TDD stub");
}

#[tokio::test]
#[ignore]
async fn buzz_bomber_spread_not_meeting_target() {
    // TODO: Test when no spread meets target_spread requirement
    // 
    // Scenario:
    // 1. target_spread = $5.00 per $100
    // 2. All available spreads yield < $5.00
    // 3. Verify bot logs error and retries or waits
    panic!("Test not yet implemented - TDD stub");
}

#[tokio::test]
#[ignore]
async fn buzz_bomber_channel_closures() {
    // TODO: Test graceful handling of channel closures
    // 
    // Scenario:
    // 1. Bot in various states
    // 2. Close told_to_trade_api unexpectedly
    // 3. Verify bot logs error and exits cleanly
    panic!("Test not yet implemented - TDD stub");
}

#[tokio::test]
#[ignore]
async fn buzz_bomber_multiple_cycles() {
    // TODO: Test bot completing multiple trade cycles
    // 
    // Scenario:
    // 1. Complete first trade cycle (entry -> exit)
    // 2. Verify return to Startup
    // 3. Complete second trade cycle
    // 4. Verify accounting updates correctly
    panic!("Test not yet implemented - TDD stub");
}

#[tokio::test]
#[ignore]
async fn buzz_bomber_max_risk_enforcement() {
    // TODO: Test that max_cash_risk is respected
    // 
    // Scenario:
    // 1. max_cash_risk = $500
    // 2. Account has $10,000
    // 3. Verify order uses max $500 (5 contracts @ $100)
    panic!("Test not yet implemented - TDD stub");
}

#[tokio::test]
#[ignore]
async fn buzz_bomber_market_closed() {
    // TODO: Test behavior when attempting entry during market closure
    // 
    // Scenario:
    // 1. Entry time = 3:00 PM
    // 2. Current time = 5:00 PM (after close)
    // 3. Verify bot waits until next trading day
    panic!("Test not yet implemented - TDD stub");
}

#[tokio::test]
#[ignore]
async fn buzz_bomber_option_chain_empty() {
    // TODO: Test when option chain request returns empty
    // 
    // Scenario:
    // 1. Request option chain for ticker
    // 2. Receive empty chain
    // 3. Verify proper error handling and Seppuku
    panic!("Test not yet implemented - TDD stub");
}

#[tokio::test]
#[ignore]
async fn buzz_bomber_rapid_price_changes() {
    // TODO: Test bot with rapidly changing option prices
    // 
    // Scenario:
    // 1. Send 100+ quote updates in quick succession
    // 2. Verify bot processes them correctly
    // 3. Verify exit logic responds to latest prices
    panic!("Test not yet implemented - TDD stub");
}

// ============================================================================
// UNIT TESTS - SPREAD PRICING & SELECTION LOGIC
// ============================================================================

#[cfg(test)]
mod spread_pricing_tests {
    use super::*;
    use chrono::Utc;
    use dsta::{MarketDirection, Opt, OptRite, OptStyle, Ticker};

    // Helper to create a fake Opt with last_ticker
    fn fake_opt(strike: f64, rite: OptRite, mid: f64, spread: f64) -> Opt {
        let bid = mid - spread / 2.0;
        let ask = mid + spread / 2.0;
        Opt {
            ticker_name: format!(".FAKE{:05.1}{}35", strike, if rite == OptRite::Put { 'P' } else { 'C' }),
            option_name: format!("FAKE {:05.1}_{}_35", strike, if rite == OptRite::Put { 'P' } else { 'C' }),
            option_rite: rite,
            strike_spot: strike,
            expire_date: Utc::now() + chrono::Duration::days(35),
            option_type: OptStyle::Usa,
            last_ticker: Some(Ticker {
                name: "".to_string(),
                bid,
                ask,
                ..Default::default()
            }),
            ..Default::default()
        }
    }


#[test] // PASS
fn test_put_credit_spread_calculation() {
    let opts = vec![
        fake_opt(95.0, OptRite::Put, 1.20, 0.10),  // lower
        fake_opt(100.0, OptRite::Put, 3.40, 0.10), // higher → short this
        fake_opt(105.0, OptRite::Put, 6.80, 0.10),
    ];

    let target = 0.50;

    let result = crate::sbot::buzz_bomber::select_best_spread(&opts, MarketDirection::GetStonk, target);

    assert!(result.is_some(), "Should have found a qualifying spread");
    let (credit, short, long) = result.unwrap();

    assert_eq!(short.strike_spot, 100.0, "Short leg should be higher strike in selected pair (100)");
    assert_eq!(long.strike_spot, 95.0,  "Long leg should be lower strike (95)");
    assert!((credit - (3.40 - 1.20)).abs() < 0.001, "Credit should be 2.20 (tightest qualifying)");
    assert!(credit >= target, "Credit should meet minimum target");
}

#[test] // PASS
fn test_prefers_tightest_qualifying_put_credit_spread() {
    let opts = vec![
        fake_opt(90.0, OptRite::Put, 0.80, 0.05),   // pair1: credit 1.00
        fake_opt(95.0, OptRite::Put, 1.80, 0.05),
        fake_opt(100.0, OptRite::Put, 4.10, 0.05),  // pair2: credit 2.30 (higher but wider risk)
        fake_opt(105.0, OptRite::Put, 6.40, 0.05),
    ];

    let result = crate::sbot::buzz_bomber::select_best_spread(&opts, MarketDirection::GetStonk, 0.90);

    assert!(result.is_some());
    let (credit, short, long) = result.unwrap();

    assert_eq!(short.strike_spot, 95.0, "Should pick tightest (90/95) over wider but higher credit");
    assert_eq!(long.strike_spot, 90.0);
    assert!((credit - 1.00).abs() < 0.001);
}

#[test] // FAIL
fn test_call_credit_spread_calculation() {
    setup_logenv();
    let opts = vec![
        fake_opt(95.0, OptRite::Call, 6.80, 0.10), // lower strike → sell this (higher premium)
        fake_opt(100.0, OptRite::Call, 3.40, 0.10),// middle
        fake_opt(105.0, OptRite::Call, 1.20, 0.10),// higher strike
    ];

    let target = 0.40;

    let result = crate::sbot::buzz_bomber::select_best_spread(&opts, MarketDirection::Corrects, target);

    assert!(result.is_some());
    let (credit, short, long) = result.unwrap();

    // For call credit: sell lower strike, buy higher strike
    // Tightest qualifying = 95/100 (credit 3.40), not 100/105 (credit 2.20)
    assert_eq!(short.strike_spot, 100.0, "Short leg should be lower strike (sell premium)");
    assert_eq!(long.strike_spot, 105.0, "Long leg should be higher strike");
    assert!((credit - (3.40 - 1.20)).abs() < 0.001, "Credit should be 2.20 (tightest qualifying)");
    assert!(credit >= target);
}

    #[test]
    fn test_no_valid_quote_skips_pair() {
        let mut opts = vec![
            fake_opt(95.0, OptRite::Put, 1.20, 0.10),
            fake_opt(100.0, OptRite::Put, 3.40, 0.10),
        ];
        // Remove quote from higher strike → should skip
        opts[1].last_ticker = None;

        let result = crate::sbot::buzz_bomber::select_best_spread(&opts, MarketDirection::GetStonk, 0.50);
        assert!(result.is_none(), "Pair with missing quote should be skipped");
    }

    #[test]
    fn test_negative_or_zero_credit_ignored() {
        let opts = vec![
            fake_opt(95.0, OptRite::Put, 2.00, 0.05),
            fake_opt(100.0, OptRite::Put, 1.80, 0.05), // lower premium → negative credit
        ];

        let result = crate::sbot::buzz_bomber::select_best_spread(&opts, MarketDirection::GetStonk, 0.10);
        assert!(result.is_none(), "Negative credit should not qualify");
    }

    #[test]
    fn test_selects_best_among_multiple_qualifying_pairs() {
        let opts = vec![
            fake_opt(90.0, OptRite::Put, 0.40, 0.05),  // pair1: credit ~0.60 (tightest)
            fake_opt(95.0, OptRite::Put, 1.00, 0.05),
            fake_opt(100.0, OptRite::Put, 2.80, 0.05), // pair2: credit ~1.80 (higher but wider)
            fake_opt(105.0, OptRite::Put, 4.60, 0.05),
        ];

        let result = crate::sbot::buzz_bomber::select_best_spread(&opts, MarketDirection::GetStonk, 0.50);

        assert!(result.is_some());
        let (credit, short, long) = result.unwrap();

        // With tightest logic: picks 90/95 (credit 0.60) over 100/105 (1.80)
        assert_eq!(short.strike_spot, 95.0, "Should pick tightest qualifying spread (short 95, long 90)");
        assert_eq!(long.strike_spot, 90.0);
        assert!((credit - (1.00 - 0.40)).abs() < 0.001, "Credit should be 0.60 (tightest)");
    }

    #[test]
    fn test_too_few_strikes_returns_none() {
        let opts = vec![fake_opt(100.0, OptRite::Put, 2.0, 0.1)]; // only 1
        let result = crate::sbot::buzz_bomber::select_best_spread(&opts, MarketDirection::GetStonk, 0.1);
        assert!(result.is_none());
    }
}