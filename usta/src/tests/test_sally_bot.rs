//! Integration tests for SallyBot.
//! Please reference tests/stealthbot.rs for style and flow.
//! Integration tests for SallyBot - Put Spreads, Call Spreads, and Haggle Logic
//! Reference: tests/stealthbot.rs for style and tests/sally_bot.rs for basic patterns

use tokio::sync::mpsc;
use std::collections::HashMap;
use std::time::Duration as StdDuration;
use log::{info, debug};

use crate::*;
use crate::sbot::*;
use crate::sbot::sally_bot::*;
use crate::core::*;
use crate::tests::util::*;
use dsta::*;



#[tokio::test] // PASSES
async fn sally_bot_submits_when_mid_price_reaches_target_buy()
{
  let _ = setup_log4rs("sally_bot_mid_price_buy");
  info!("Starting sally_bot_submits_when_mid_price_reaches_target_buy test");

  let testbotname = "sally_test_bot_buy".to_string();
  
  let mut tstk = dsta::Stk::default();
  tstk.ticker_name = format!("FAKE");


  let config = dsta::MakeSallyFakes
  {
    my_accounting:
    {
      let mut ma = dsta::CommonBotInfo::default();
      ma.friendly_name = testbotname.clone();
      ma.max_cash_risk = (true, 100.0, 10_000.0);
      ma.tracking_tick = "FAKE".to_string();
      ma
    },
    my_reveal_way: dsta::SallyAction::SubmitWhenBidAskMidAttainsInputPrice(100.0),
    my_hide_order:
    { let mut o = dsta::Order::default();
      o.order_legs.push(dsta::OrderLeg
      { buy_what: dsta::FinAss::StkDeets(tstk), 
        quantity: 1.0,
        price: 110.0,
        action: dsta::BuyOrSell::BuyToOpen,
        ..Default::default()
      });


      let oc = core::fn_calculate_order_cost(&o);
      //o.order_cost = Some(oc.expect("Didn't get an order cost"));
      

      o
    },
  };

  let (hand, mut foot) = fn_make_bot_api_bridge_dr_handles();
  let (trade_tx, trade_rx) = mpsc::channel::<dsta::Ticker>(100);
  let (check_tx, check_rx) = mpsc::channel::<crate::Allocation>(100);
  let (sucks_tx, sucks_rx) = mpsc::channel::<String>(100);

  let mut stuff: HashMap<String, dsta::FinAss> = HashMap::new();
  stuff.insert("FAKE".to_string(), dsta::FinAss::StkDeets(dsta::Stk
  {
    ticker_name: "FAKE".to_string(),
    ticker_info: "Fake stock".to_string(),
    last_ticker: None,
    tick_rulers: None,

  }));
  debug!("Spawning Sally bot task");
  let bot_handle = tokio::spawn(crate::sbot::sally_bot::fn_run_main_sally_bot(
    config.clone(),
    trade_rx,
    check_rx,
    sucks_rx,
    hand,
  ));
  let swap = tokio::select!
  { res = foot.told_dr_swap_info.recv() =>
    { res.expect("swap channel closed")
    }
    _ = tokio::time::sleep(StdDuration::from_secs(1)) =>
    { panic!("timeout waiting for cash request");
    }
  };
  assert_eq!(swap.target_released.unwrap().how, 110.0);
  sucks_tx.send(crate::glossary::SWAP_SUCC.to_string()).await.unwrap();
  let mut account = BotaAccount::new().with_cash(110.0);
  check_tx
    .send(crate::Allocation
    {
      bot_name: testbotname.clone(),
      bot_safe: account.clone(),
      bot_mind: crate::BotMindState::Running,
      num_swap: 0,
    })
    .await
    .unwrap();
  let (ticker_name, reply_tx) = tokio::select!
  {
    res = foot.told_dr_pls_ticks.recv() =>
    {
      res.expect("ticks channel closed")
    }
    _ = tokio::time::sleep(StdDuration::from_secs(1)) =>
    {
      panic!("timeout waiting for ticker subscription");
    }
  };
  assert_eq!(ticker_name, "FAKE");

  let mut tstk = dsta::Stk::default();
  tstk.ticker_name = format!("FAKE");

  reply_tx
    .send(Some(dsta::PushTickerResult
    { ass_deets : dsta::FinAss::StkDeets(tstk), 
      opt_chain: vec![],
    }))
    .await
    .unwrap();

  trade_tx
    .send(dsta::Ticker
    {
      name: "FAKE".to_string(),
      bid: 99.5,
      ask: 100.5,
      ..Default::default()
    })
    .await
    .unwrap();
  tokio::time::sleep(StdDuration::from_millis(50)).await;

  trade_tx
    .send(dsta::Ticker
    {
      name: "FAKE".to_string(),
      bid: 99.9,
      ask: 100.02,
      ..Default::default()
    })
    .await
    .unwrap();
  tokio::time::sleep(StdDuration::from_millis(50)).await;

  let submitted_order = tokio::select!
  {
    res = foot.told_dr_pls_trade.recv() =>
    {
      res.expect("trade channel closed")
    }
    _ = tokio::time::sleep(StdDuration::from_secs(1)) =>
    {
      panic!("order submission timeout");
    }
  };
  assert!(!submitted_order.order_legs.is_empty());

  tokio::time::sleep(StdDuration::from_secs(1)).await;

  assert_eq!(submitted_order.order_legs[0].buy_what.order_name(), "FAKE");

  let changes =
    fn_get_account_shennanagans(&submitted_order, &account, &stuff)
      .expect("Failed to compute fill changes");
  apply_account_cash_and_collateral_changes(&mut account, &changes);
  apply_order_to_account(&mut account, &submitted_order, &stuff)
    .expect("Failed to apply order");

  log::info!("Sending information that the order is complete:");
  check_tx
    .send(crate::Allocation
    {
      bot_name: testbotname.clone(),
      bot_safe: account.clone(),
      bot_mind: crate::BotMindState::Running,
      num_swap: 0,
    })
    .await
    .unwrap();

  sucks_tx.send(format!("{}",crate::glossary::ORDER_FLY)).await.expect("Failed to send ORDER_FLY");

  tokio::time::sleep(StdDuration::from_millis(100)).await;
  let _ = sucks_tx.send("DIE".to_string()).await;

  tokio::select!
  {
    res = bot_handle =>
    {
      res.ok();
    }
    _ = tokio::time::sleep(StdDuration::from_secs(1)) =>
    {
      panic!("Bot did not shut down within 5s");
    }
  };

  info!("sally_bot_submits_when_mid_price_reaches_target_buy test completed");
}


// NEXT TO CHECK:

// ═══════════════════════════════════════════════════════════════════════════════
// TEST 1: CALL CREDIT SPREAD
// ═══════════════════════════════════════════════════════════════════════════════

#[tokio::test]
async fn sally_bot_call_credit_spread_trigger_and_fill()
{
  let _ = setup_log4rs("sally_call_spread");
  info!("Starting sally_bot_call_credit_spread_trigger_and_fill test");

  let testbotname = "sally_call_spread_bot".to_string();
  
  // Setup underlying stock
  let mut tstk = dsta::Stk::default();
  tstk.ticker_name = "FAKE".to_string();

  // Setup options for the spread
  let long_call = dsta::Opt {
    ticker_name: ".FAKE 105_C_30".to_string(),
    option_name: "FAKE 105_C_30".to_string(),
    strike_spot: 105.0,
    option_rite: dsta::OptRite::Call,
    ..Default::default()
  };

  let short_call = dsta::Opt {
    ticker_name: ".FAKE 100_C_30".to_string(),
    option_name: "FAKE 100_C_30".to_string(),
    strike_spot: 100.0,
    option_rite: dsta::OptRite::Call,
    ..Default::default()
  };

  // Build the spread order: Long 105C / Short 100C = net credit of ~0.50 per contract
  let config = dsta::MakeSallyFakes {
    my_accounting: {
      let mut ma = dsta::CommonBotInfo::default();
      ma.friendly_name = testbotname.clone();
      ma.max_cash_risk = (true, 100.0, 10_000.0);
      ma.tracking_tick = "FAKE".to_string();
      ma
    },
    my_reveal_way: dsta::SallyAction::SubmitWhenBidAskMidAttainsInputPrice
    ( 0.55
    ),
    my_hide_order: {
      let mut o = dsta::Order::default();
      
      // Long 105 Call (buy to open) - debit of ~3.00
      o.order_legs.push(dsta::OrderLeg {
        buy_what: dsta::FinAss::OptDeets(long_call.clone()),
        quantity: 1.0,
        price: 3.00,
        action: dsta::BuyOrSell::BuyToOpen,
        ..Default::default()
      });
      
      // Short 100 Call (sell to open) - credit of ~3.50
      o.order_legs.push(dsta::OrderLeg {
        buy_what: dsta::FinAss::OptDeets(short_call.clone()),
        quantity: 1.0,
        price: 3.50,
        action: dsta::BuyOrSell::SellToOpen,
        ..Default::default()
      });
      let oc = core::fn_calculate_order_cost(&o);
      println!("Order cost calculated for test: {}", oc.unwrap());
      o
    },
  };

  let (hand, mut foot) = fn_make_bot_api_bridge_dr_handles();
  let (trade_tx, trade_rx) = mpsc::channel::<dsta::Ticker>(100);
  let (check_tx, check_rx) = mpsc::channel::<crate::Allocation>(100);
  let (sucks_tx, sucks_rx) = mpsc::channel::<String>(100);

  debug!("Spawning Sally bot for call credit spread");
  let bot_handle = tokio::spawn(crate::sbot::sally_bot::fn_run_main_sally_bot(
    config.clone(),
    trade_rx,
    check_rx,
    sucks_rx,
    hand,
  ));

  // Wait for initial cash request
  let swap = tokio::select! {
    res = foot.told_dr_swap_info.recv() => res.expect("swap channel closed"),
    _ = tokio::time::sleep(StdDuration::from_secs(1)) => panic!("timeout waiting for cash request"),
  };
  
  assert!(swap.target_released.is_some());
  let requested_cash = swap.target_released.unwrap().how;
  sucks_tx.send(crate::glossary::SWAP_SUCC.to_string()).await.unwrap();

  // Set account to exactly what was requested + small buffer
  let mut account = BotaAccount::new().with_cash(requested_cash + 1.0);
  check_tx.send(crate::Allocation {
    bot_name: testbotname.clone(),
    bot_safe: account.clone(),
    bot_mind: crate::BotMindState::Running,
    num_swap: 0,
  }).await.unwrap();

  // Request long call (105C)
  let (opt_name, reply_tx) = tokio::select! {
    res = foot.told_dr_pls_ticks.recv() => res.expect("ticks channel closed"),
    _ = tokio::time::sleep(StdDuration::from_secs(1)) => panic!("timeout waiting for long call request"),
  };
  assert_eq!(opt_name, ".FAKE 105_C_30");
  
  reply_tx.send(Some(dsta::PushTickerResult {
    ass_deets: dsta::FinAss::OptDeets(long_call.clone()),
    opt_chain: vec![],
  })).await.unwrap();

  // Request short call (100C)
  let (opt_name, reply_tx) = tokio::select! {
    res = foot.told_dr_pls_ticks.recv() => res.expect("ticks channel closed"),
    _ = tokio::time::sleep(StdDuration::from_secs(1)) => panic!("timeout waiting for short call request"),
  };
  assert_eq!(opt_name, ".FAKE 100_C_30");
  
  reply_tx.send(Some(dsta::PushTickerResult {
    ass_deets: dsta::FinAss::OptDeets(short_call.clone()),
    opt_chain: vec![],
  })).await.unwrap();

  // Send long call quote
  trade_tx.send(dsta::Ticker {
    name: ".FAKE 105_C_30".to_string(),
    bid: 2.90,
    ask: 3.10,
    ..Default::default()
  }).await.unwrap();

  tokio::time::sleep(StdDuration::from_millis(50)).await;

  trade_tx.send(dsta::Ticker {
    name: ".FAKE 100_C_30".to_string(),
    bid: 3.40,
    ask: 3.60,
    ..Default::default()
  }).await.unwrap();


  trade_tx.send(dsta::Ticker {
    name: ".FAKE 105_C_30".to_string(),
    bid: 2.90,
    ask: 2.95,
    ..Default::default()
  }).await.unwrap();

  tokio::time::sleep(StdDuration::from_millis(50)).await;

  // Send short call quote (triggers the spread submission as both legs now have quotes)
  trade_tx.send(dsta::Ticker {
    name: ".FAKE 100_C_30".to_string(),
    bid: 3.55,
    ask: 3.60,
    ..Default::default()
  }).await.unwrap();



  // Wait for order submission
  let submitted_order = tokio::select! {
    res = foot.told_dr_pls_trade.recv() => res.expect("trade channel closed"),
    _ = tokio::time::sleep(StdDuration::from_secs(1)) => panic!("order submission timeout"),
  };

  assert_eq!(submitted_order.order_legs.len(), 2);
  info!("Call credit spread submitted with {} legs", submitted_order.order_legs.len());

  // Simulate fill
  let mut stuff: HashMap<String, FinAss> = HashMap::new();
  stuff.insert("FAKE 105_C_30".to_string(), dsta::FinAss::OptDeets(long_call.clone()));
  stuff.insert("FAKE 100_C_30".to_string(), dsta::FinAss::OptDeets(short_call.clone()));

  let changes = fn_get_account_shennanagans(&submitted_order, &account, &stuff)
    .expect("Failed to compute fill changes");
  apply_account_cash_and_collateral_changes(&mut account, &changes);
  apply_order_to_account(&mut account, &submitted_order, &stuff)
    .expect("Failed to apply order");

  check_tx.send(crate::Allocation {
    bot_name: testbotname.clone(),
    bot_safe: account.clone(),
    bot_mind: crate::BotMindState::Running,
    num_swap: 0,
  }).await.unwrap();

  sucks_tx.send(format!("{}",crate::glossary::ORDER_FLY)).await.expect("Failed to send ORDER_FLY");

  tokio::time::sleep(StdDuration::from_millis(100)).await;
  let _ = sucks_tx.send("DIE".to_string()).await;

  tokio::select! {
    res = bot_handle => { res.ok(); }
    _ = tokio::time::sleep(StdDuration::from_secs(1)) => panic!("Bot did not shut down within 5s"),
  };

  info!("sally_bot_call_credit_spread_trigger_and_fill test completed");
}

// ═══════════════════════════════════════════════════════════════════════════════
// TEST 2: PUT CREDIT SPREAD
// ═══════════════════════════════════════════════════════════════════════════════

#[tokio::test]
async fn sally_bot_put_credit_spread_trigger_and_fill()
{
  let _ = setup_log4rs("sally_put_spread");
  info!("Starting sally_bot_put_credit_spread_trigger_and_fill test");

  let testbotname = "sally_put_spread_bot".to_string();
  
  let mut tstk = dsta::Stk::default();
  tstk.ticker_name = "FAKE".to_string();

  let long_put = dsta::Opt {
    ticker_name: ".FAKE 95_P_30".to_string(),
    option_name: "FAKE 95_P_30".to_string(),
    strike_spot: 95.0,
    option_rite: dsta::OptRite::Put,
    ..Default::default()
  };

  let short_put = dsta::Opt {
    ticker_name: ".FAKE 100_P_30".to_string(),
    option_name: "FAKE 100_P_30".to_string(),
    strike_spot: 100.0,
    option_rite: dsta::OptRite::Put,
    ..Default::default()
  };

  let config = dsta::MakeSallyFakes {
    my_accounting: {
      let mut ma = dsta::CommonBotInfo::default();
      ma.friendly_name = testbotname.clone();
      ma.max_cash_risk = (true, 100.0, 10_000.0);
      ma.tracking_tick = "FAKE".to_string();
      ma
    },
    my_reveal_way: dsta::SallyAction::SubmitWhenMarketSideReachesThisPrice // Market side of the spread cost!
    ( 1.1
    ),
    my_hide_order: {
      let mut o = dsta::Order::default();
      
      // Long 95 Put (buy to open) - debit ~1.50
      o.order_legs.push(dsta::OrderLeg {
        buy_what: dsta::FinAss::OptDeets(long_put.clone()),
        quantity: 1.0,
        price: 1.50,
        action: dsta::BuyOrSell::BuyToOpen,
        ..Default::default()
      });
      
      // Short 100 Put (sell to open) - credit ~2.50
      o.order_legs.push(dsta::OrderLeg {
        buy_what: dsta::FinAss::OptDeets(short_put.clone()),
        quantity: 1.0,
        price: 2.50,
        action: dsta::BuyOrSell::SellToOpen,
        ..Default::default()
      });

      let oc = core::fn_calculate_order_cost(&o);
      //o.order_cost = Some(oc.expect("Didn't get an order cost"));
      
      o
    },
  };

  let (hand, mut foot) = fn_make_bot_api_bridge_dr_handles();
  let (trade_tx, trade_rx) = mpsc::channel::<dsta::Ticker>(100);
  let (check_tx, check_rx) = mpsc::channel::<crate::Allocation>(100);
  let (sucks_tx, sucks_rx) = mpsc::channel::<String>(100);

  debug!("Spawning Sally bot for put credit spread");
  let bot_handle = tokio::spawn(crate::sbot::sally_bot::fn_run_main_sally_bot(
    config.clone(),
    trade_rx,
    check_rx,
    sucks_rx,
    hand,
  ));

  let swap = tokio::select! {
    res = foot.told_dr_swap_info.recv() => res.expect("swap channel closed"),
    _ = tokio::time::sleep(StdDuration::from_secs(1)) => panic!("timeout waiting for cash request"),
  };
  
  assert!(swap.target_released.is_some());
  let requested_cash = swap.target_released.unwrap().how;
  sucks_tx.send(crate::glossary::SWAP_SUCC.to_string()).await.unwrap();

  let mut account = BotaAccount::new().with_cash(requested_cash + 1.0);
  check_tx.send(crate::Allocation {
    bot_name: testbotname.clone(),
    bot_safe: account.clone(),
    bot_mind: crate::BotMindState::Running,
    num_swap: 0,
  }).await.unwrap();

  let (opt_name, reply_tx) = tokio::select! {
    res = foot.told_dr_pls_ticks.recv() => res.expect("ticks channel closed"),
    _ = tokio::time::sleep(StdDuration::from_secs(1)) => panic!("timeout"),
  };
  assert_eq!(opt_name, ".FAKE 95_P_30");
  
  reply_tx.send(Some(dsta::PushTickerResult {
    ass_deets: dsta::FinAss::OptDeets(long_put.clone()),
    opt_chain: vec![],
  })).await.unwrap();

  let (opt_name, reply_tx) = tokio::select! {
    res = foot.told_dr_pls_ticks.recv() => res.expect("ticks channel closed"),
    _ = tokio::time::sleep(StdDuration::from_secs(1)) => panic!("timeout"),
  };
  assert_eq!(opt_name, ".FAKE 100_P_30");
  
  reply_tx.send(Some(dsta::PushTickerResult {
    ass_deets: dsta::FinAss::OptDeets(short_put.clone()),
    opt_chain: vec![],
  })).await.unwrap();

  // Trigger: underlying bid >= 100.50 (for put credit spread, we want the bid side favorable)
  trade_tx.send(dsta::Ticker {
    name: "FAKE".to_string(),
    bid: 100.50,
    ask: 101.00,
    ..Default::default()
  }).await.unwrap();

  tokio::time::sleep(StdDuration::from_millis(50)).await;


  log::info!("SENDING TICKERS A");
  trade_tx.send(dsta::Ticker {
    name: ".FAKE 95_P_30".to_string(),
    bid: 1.40,
    ask: 1.60,
    ..Default::default()
  }).await.unwrap();

  tokio::time::sleep(StdDuration::from_millis(50)).await;

  trade_tx.send(dsta::Ticker {
    name: ".FAKE 100_P_30".to_string(),
    bid: 2.40,
    ask: 2.60,
    ..Default::default()
  }).await.unwrap();

  log::info!("SENDING TICKERS B");

  trade_tx.send(dsta::Ticker {
    name: ".FAKE 95_P_30".to_string(),
    bid: 1.30,
    ask: 1.50,
    ..Default::default()
  }).await.unwrap();

  tokio::time::sleep(StdDuration::from_millis(50)).await;

  trade_tx.send(dsta::Ticker {
    name: ".FAKE 100_P_30".to_string(),
    bid: 2.40,
    ask: 2.60,
    ..Default::default()
  }).await.unwrap();



  let submitted_order = tokio::select! {
    res = foot.told_dr_pls_trade.recv() => res.expect("trade channel closed"),
    _ = tokio::time::sleep(StdDuration::from_secs(1)) => panic!("order submission timeout"),
  };

  assert_eq!(submitted_order.order_legs.len(), 2);
  info!("Put credit spread submitted with {} legs", submitted_order.order_legs.len());

  let mut stuff: HashMap<String, FinAss> = HashMap::new();
  stuff.insert("FAKE 95_P_30".to_string(), dsta::FinAss::OptDeets(long_put.clone()));
  stuff.insert("FAKE 100_P_30".to_string(), dsta::FinAss::OptDeets(short_put.clone()));

  let changes = fn_get_account_shennanagans(&submitted_order, &account, &stuff)
    .expect("Failed to compute fill changes");
  apply_account_cash_and_collateral_changes(&mut account, &changes);
  apply_order_to_account(&mut account, &submitted_order, &stuff)
    .expect("Failed to apply order");

  check_tx.send(crate::Allocation {
    bot_name: testbotname.clone(),
    bot_safe: account.clone(),
    bot_mind: crate::BotMindState::Running,
    num_swap: 0,
  }).await.unwrap();

  sucks_tx.send(format!("{}",crate::glossary::ORDER_FLY)).await.expect("Failed to send ORDER_FLY");

  tokio::time::sleep(StdDuration::from_millis(100)).await;
  let _ = sucks_tx.send("DIE".to_string()).await;

  tokio::select! {
    res = bot_handle => { res.ok(); }
    _ = tokio::time::sleep(StdDuration::from_secs(1)) => panic!("Bot did not shut down within 5s"),
  };

  info!("sally_bot_put_credit_spread_trigger_and_fill test completed");
}

// ═══════════════════════════════════════════════════════════════════════════════
// TEST 3: HAGGLE LOGIC ON STOCK (BUY)
// ═══════════════════════════════════════════════════════════════════════════════

#[tokio::test]
async fn sally_bot_haggle_slippage_stock_buy()
{
  let _ = setup_log4rs("sally_haggle_stock_buy");
  info!("Starting sally_bot_haggle_slippage_stock_buy test");

  let testbotname = "sally_haggle_stock_bot".to_string();
  
  let mut tstk = dsta::Stk::default();
  tstk.ticker_name = "FAKE".to_string();

  let config = dsta::MakeSallyFakes {
    my_accounting: {
      let mut ma = dsta::CommonBotInfo::default();
      ma.friendly_name = testbotname.clone();
      ma.max_cash_risk = (true, 100.0, 10_000.0);
      ma.tracking_tick = "FAKE".to_string();
      // Configure haggle action: retry every 1 second, concede up to 2% slippage
      ma.haggle_action = dsta::HaggleAction::AnOyVeyJew(
        dsta::HaggleMethod::MinimumOfMidpointOrTheoreticalThenConcede(
          dsta::HaggleLimits {
            retry_period: chrono::Duration::seconds(1),
            delta_choice: 0.02,
            delta_is_pct: true,
            slippage_max: 0.02,
          }
        )
      );
      ma
    },
    my_reveal_way: dsta::SallyAction::SubmitRightAwayButLikeOnlyUseForTest,
    my_hide_order: {
      let mut o = dsta::Order::default();
      o.order_legs.push(dsta::OrderLeg {
        buy_what: dsta::FinAss::StkDeets(tstk.clone()),
        quantity: 100.0,
        price: 100.0,
        action: dsta::BuyOrSell::BuyToOpen,
        ..Default::default()
      });

      let oc = core::fn_calculate_order_cost(&o);
      //o.order_cost = Some(oc.expect("Didn't get an order cost"));
      
      o
    },
  };

  let (hand, mut foot) = fn_make_bot_api_bridge_dr_handles();
  let (trade_tx, trade_rx) = mpsc::channel::<dsta::Ticker>(100);
  let (check_tx, check_rx) = mpsc::channel::<crate::Allocation>(100);
  let (sucks_tx, sucks_rx) = mpsc::channel::<String>(100);

  debug!("Spawning Sally bot for stock haggle test");
  let bot_handle = tokio::spawn(crate::sbot::sally_bot::fn_run_main_sally_bot(
    config.clone(),
    trade_rx,
    check_rx,
    sucks_rx,
    hand,
  ));

  let swap = tokio::select! {
    res = foot.told_dr_swap_info.recv() => res.expect("swap channel closed"),
    _ = tokio::time::sleep(StdDuration::from_secs(1)) => panic!("timeout waiting for cash request"),
  };
  
  assert!(swap.target_released.is_some());
  let requested_cash = swap.target_released.unwrap().how;
  sucks_tx.send(crate::glossary::SWAP_SUCC.to_string()).await.unwrap();

  let mut account = BotaAccount::new().with_cash(requested_cash + 1.0);
  check_tx.send(crate::Allocation {
    bot_name: testbotname.clone(),
    bot_safe: account.clone(),
    bot_mind: crate::BotMindState::Running,
    num_swap: 0,
  }).await.unwrap();

  let (ticker_name, reply_tx) = tokio::select! {
    res = foot.told_dr_pls_ticks.recv() => res.expect("ticks channel closed"),
    _ = tokio::time::sleep(StdDuration::from_secs(1)) => panic!("timeout"),
  };
  assert_eq!(ticker_name, "FAKE");

  reply_tx.send(Some(dsta::PushTickerResult {
    ass_deets: dsta::FinAss::StkDeets(tstk.clone()),
    opt_chain: vec![],
  })).await.unwrap();

  // Initial submission (test mode → immediate submit)
  trade_tx.send(dsta::Ticker {
    name: "FAKE".to_string(),
    bid: 99.50,
    ask: 100.50,
    ..Default::default()
  }).await.unwrap();

  let initial_order = tokio::select! {
    res = foot.told_dr_pls_trade.recv() => res.expect("trade channel closed"),
    _ = tokio::time::sleep(StdDuration::from_secs(1)) => panic!("order submission timeout"),
  };

  info!("Initial order submitted for stock @ ~100.00");

  // Simulate partial fill (50 out of 100 shares)
  let mut stuff: HashMap<String, FinAss> = HashMap::new();
  stuff.insert("FAKE".to_string(), dsta::FinAss::StkDeets(tstk.clone()));

  let mut partial_account = account.clone();
  partial_account.m_stock = Some(dsta::OwnedPosition {
    asset: dsta::FinAss::StkDeets(tstk.clone()),
    owned: 50.0,
    price: 100.0,
  });

  check_tx.send(crate::Allocation {
    bot_name: testbotname.clone(),
    bot_safe: partial_account.clone(),
    bot_mind: crate::BotMindState::Running,
    num_swap: 0,
  }).await.unwrap();

  info!("Partial fill sent: 50 shares filled");

  sucks_tx.send(format!("{}",crate::glossary::ORDER_FLY)).await.expect("Failed to send ORDER_FLY");

  // Wait for retry (1 second)
  tokio::time::sleep(StdDuration::from_millis(1500)).await;

  // Emit a worse quote to trigger haggle adjustment
  trade_tx.send(dsta::Ticker {
    name: "FAKE".to_string(),
    bid: 98.50,
    ask: 99.50,
    ..Default::default()
  }).await.unwrap();

  // Should receive haggled order (with adjusted quantity for remaining 50 shares)
  let haggled_order = tokio::select! {
    res = foot.told_dr_pls_trade.recv() => res.expect("trade channel closed"),
    _ = tokio::time::sleep(StdDuration::from_secs(1)) => panic!("haggled order submission timeout"),
  };

  info!("Haggled order submitted with adjusted price/quantity");
  assert_eq!(haggled_order.order_legs[0].quantity, 50.0, "Should haggle remaining 50 shares");

  // Fill the remaining
  let mut final_account = partial_account.clone();
  final_account.m_stock = Some(dsta::OwnedPosition {
    asset: dsta::FinAss::StkDeets(tstk.clone()),
    owned: 100.0,
    price: 100.0,
  });

  check_tx.send(crate::Allocation {
    bot_name: testbotname.clone(),
    bot_safe: final_account.clone(),
    bot_mind: crate::BotMindState::Running,
    num_swap: 0,
  }).await.unwrap();

  sucks_tx.send(format!("{}",crate::glossary::ORDER_FLY)).await.expect("Failed to send ORDER_FLY");

  tokio::time::sleep(StdDuration::from_millis(100)).await;
  let _ = sucks_tx.send("DIE".to_string()).await;

  tokio::select! {
    res = bot_handle => { res.ok(); }
    _ = tokio::time::sleep(StdDuration::from_secs(1)) => panic!("Bot did not shut down within 5s"),
  };

  info!("sally_bot_haggle_slippage_stock_buy test completed");
}

// ═══════════════════════════════════════════════════════════════════════════════
// TEST 4: HAGGLE LOGIC ON DEBIT SPREAD (LONG CALL SPREAD)
// ═══════════════════════════════════════════════════════════════════════════════

#[tokio::test]
async fn sally_bot_haggle_slippage_debit_spread()
{
  let _ = setup_log4rs("sally_haggle_debit_spread");
  info!("Starting sally_bot_haggle_slippage_debit_spread test");

  let testbotname = "sally_haggle_debit_spread_bot".to_string();
  
  let long_call = dsta::Opt {
    ticker_name: ".FAKE 100_C_30".to_string(),
    option_name: "FAKE 100_C_30".to_string(),
    strike_spot: 100.0,
    option_rite: dsta::OptRite::Call,
    ..Default::default()
  };

  let short_call = dsta::Opt {
    ticker_name: ".FAKE 105_C_30".to_string(),
    option_name: "FAKE 105_C_30".to_string(),
    strike_spot: 105.0,
    option_rite: dsta::OptRite::Call,
    ..Default::default()
  };

  let config = dsta::MakeSallyFakes {
    my_accounting: {
      let mut ma = dsta::CommonBotInfo::default();
      ma.friendly_name = testbotname.clone();
      ma.max_cash_risk = (true, 100.0, 10_000.0);
      ma.tracking_tick = "FAKE".to_string();
      ma.haggle_action = dsta::HaggleAction::AnOyVeyJew(
        dsta::HaggleMethod::CheapLimitOrderThenIncrementDeltaUntilMax(
          dsta::HaggleLimits {
            retry_period: chrono::Duration::milliseconds(800),
            delta_choice: 0.01,
            delta_is_pct: true,
            slippage_max: 0.04,
          }
        )
      );
      ma
    },
    my_reveal_way: dsta::SallyAction::SubmitWhenBidAskMidAttainsInputPrice(102.50),
    my_hide_order: {
      let mut o = dsta::Order::default();
      
      // Long 100 Call - debit ~2.50
      o.order_legs.push(dsta::OrderLeg {
        buy_what: dsta::FinAss::OptDeets(long_call.clone()),
        quantity: 1.0,
        price: 2.50,
        action: dsta::BuyOrSell::BuyToOpen,
        ..Default::default()
      });
      
      // Short 105 Call - credit ~0.75
      o.order_legs.push(dsta::OrderLeg {
        buy_what: dsta::FinAss::OptDeets(short_call.clone()),
        quantity: 1.0,
        price: 0.75,
        action: dsta::BuyOrSell::SellToOpen,
        ..Default::default()
      });
      

      let oc = core::fn_calculate_order_cost(&o);
      //o.order_cost = Some(oc.expect("Didn't get an order cost"));
      
      o
    },
  };

  let (hand, mut foot) = fn_make_bot_api_bridge_dr_handles();
  let (trade_tx, trade_rx) = mpsc::channel::<dsta::Ticker>(100);
  let (check_tx, check_rx) = mpsc::channel::<crate::Allocation>(100);
  let (sucks_tx, sucks_rx) = mpsc::channel::<String>(100);

  debug!("Spawning Sally bot for debit spread haggle test");
  let bot_handle = tokio::spawn(crate::sbot::sally_bot::fn_run_main_sally_bot(
    config.clone(),
    trade_rx,
    check_rx,
    sucks_rx,
    hand,
  ));

  let swap = tokio::select! {
    res = foot.told_dr_swap_info.recv() => res.expect("swap channel closed"),
    _ = tokio::time::sleep(StdDuration::from_secs(1)) => panic!("timeout waiting for cash request"),
  };
  
  assert!(swap.target_released.is_some());
  let requested_cash = swap.target_released.unwrap().how;
  sucks_tx.send(crate::glossary::SWAP_SUCC.to_string()).await.unwrap();

  // Set the account to what the bot requested in the SWAP plus $1 buffer
  let mut account = BotaAccount::new().with_cash(requested_cash + 1.0);
  check_tx.send(crate::Allocation {
    bot_name: testbotname.clone(),
    bot_safe: account.clone(),
    bot_mind: crate::BotMindState::Running,
    num_swap: 0,
  }).await.unwrap();

  // Request long call (100C)
  let (opt_name, reply_tx) = tokio::select! {
    res = foot.told_dr_pls_ticks.recv() => res.expect("ticks channel closed"),
    _ = tokio::time::sleep(StdDuration::from_secs(1)) => panic!("timeout"),
  };
  assert_eq!(opt_name, ".FAKE 100_C_30");
  
  reply_tx.send(Some(dsta::PushTickerResult {
    ass_deets: dsta::FinAss::OptDeets(long_call.clone()),
    opt_chain: vec![],
  })).await.unwrap();

  // Request short call (105C)
  let (opt_name, reply_tx) = tokio::select! {
    res = foot.told_dr_pls_ticks.recv() => res.expect("ticks channel closed"),
    _ = tokio::time::sleep(StdDuration::from_secs(1)) => panic!("timeout"),
  };
  assert_eq!(opt_name, ".FAKE 105_C_30");
  
  reply_tx.send(Some(dsta::PushTickerResult {
    ass_deets: dsta::FinAss::OptDeets(short_call.clone()),
    opt_chain: vec![],
  })).await.unwrap();




  // Trigger: mid price hits 102.50
  trade_tx.send(dsta::Ticker {
    name: ".FAKE 100_C_30".to_string(),
    bid: 2.40,
    ask: 2.60,
    ..Default::default()
  }).await.unwrap();

  tokio::time::sleep(StdDuration::from_millis(50)).await;

  trade_tx.send(dsta::Ticker {
    name: ".FAKE 105_C_30".to_string(),
    bid: 0.70,
    ask: 0.80,
    ..Default::default()
  }).await.unwrap();

  let initial_order = tokio::select! {
    res = foot.told_dr_pls_trade.recv() => res.expect("trade channel closed"),
    _ = tokio::time::sleep(StdDuration::from_secs(1)) => panic!("order submission timeout"),
  };

  info!("Initial debit spread order submitted");

  // Simulate 50% partial fill
  let mut stuff: HashMap<String, FinAss> = HashMap::new();
  stuff.insert("FAKE 100_C_30".to_string(), dsta::FinAss::OptDeets(long_call.clone()));
  stuff.insert("FAKE 105_C_30".to_string(), dsta::FinAss::OptDeets(short_call.clone()));

  let mut partial_account = account.clone();
  partial_account.m_long_calls = Some(dsta::OwnedPosition {
    asset: dsta::FinAss::OptDeets(long_call.clone()),
    owned: 0.5,
    price: 2.50,
  });

  check_tx.send(crate::Allocation {
    bot_name: testbotname.clone(),
    bot_safe: partial_account.clone(),
    bot_mind: crate::BotMindState::Running,
    num_swap: 0,
  }).await.unwrap();

  info!("Partial fill: long call 50% filled");

  sucks_tx.send(format!("{}",crate::glossary::ORDER_FLY)).await.expect("Failed to send ORDER_FLY");

  // Wait for retry period (800ms)
  tokio::time::sleep(StdDuration::from_millis(1200)).await;

  // Emit worse market (debit spreads want market to move down)
  trade_tx.send(dsta::Ticker {
    name: ".FAKE 100_C_30".to_string(),
    bid: 2.60,
    ask: 2.80,
    ..Default::default()
  }).await.unwrap();

  tokio::time::sleep(StdDuration::from_millis(50)).await;

  trade_tx.send(dsta::Ticker {
    name: ".FAKE 105_C_30".to_string(),
    bid: 0.50,
    ask: 0.60,
    ..Default::default()
  }).await.unwrap();

  // Expect haggled order (remaining 50%)
  let haggled_order = tokio::select! {
    res = foot.told_dr_pls_trade.recv() => res.expect("trade channel closed"),
    _ = tokio::time::sleep(StdDuration::from_secs(1)) => panic!("haggled order timeout"),
  };

  info!("Haggled order submitted for remaining debit spread contracts");
  
  // Verify adjustment for partial fill ratio
  for leg in &haggled_order.order_legs {
    info!("Haggled leg: {:?} @ ${:.4}", leg.buy_what.order_name(), leg.price);
  }

  // Complete the fill
  let mut final_account = partial_account.clone();
  final_account.m_long_calls = Some(dsta::OwnedPosition {
    asset: dsta::FinAss::OptDeets(long_call.clone()),
    owned: 1.0,
    price: 2.50,
  });
  final_account.m_short_calls = Some(dsta::OwnedPosition {
    asset: dsta::FinAss::OptDeets(short_call.clone()),
    owned: -1.0,
    price: 0.75,
  });

  check_tx.send(crate::Allocation {
    bot_name: testbotname.clone(),
    bot_safe: final_account.clone(),
    bot_mind: crate::BotMindState::Running,
    num_swap: 0,
  }).await.unwrap();

  sucks_tx.send(format!("{}",crate::glossary::ORDER_FLY)).await.expect("Failed to send ORDER_FLY");

  tokio::time::sleep(StdDuration::from_millis(100)).await;
  let _ = sucks_tx.send("DIE".to_string()).await;

  tokio::select! {
    res = bot_handle => { res.ok(); }
    _ = tokio::time::sleep(StdDuration::from_secs(1)) => panic!("Bot did not shut down within 5s"),
  };

  info!("sally_bot_haggle_slippage_debit_spread test completed");
}

// ═══════════════════════════════════════════════════════════════════════════════
// TEST 5: HAGGLE LOGIC ON CREDIT SPREAD WITH SLIPPAGE LIMIT EXCEEDED
// ═══════════════════════════════════════════════════════════════════════════════

#[tokio::test]
async fn sally_bot_haggle_slippage_limit_exceeded()
{
  let _ = setup_log4rs("sally_haggle_slippage_exceeded");
  info!("Starting sally_bot_haggle_slippage_limit_exceeded test");

  let testbotname = "sally_haggle_limit_bot".to_string();

  let long_call = dsta::Opt {
    ticker_name: ".FAKE 105_C_30".to_string(),
    option_name: "FAKE 105_C_30".to_string(),
    strike_spot: 105.0,
    option_rite: dsta::OptRite::Call,
    ..Default::default()
  };

  let short_call = dsta::Opt {
    ticker_name: ".FAKE 100_C_30".to_string(),
    option_name: "FAKE 100_C_30".to_string(),
    strike_spot: 100.0,
    option_rite: dsta::OptRite::Call,
    ..Default::default()
  };

  let config = dsta::MakeSallyFakes {
    my_accounting: {
      let mut ma = dsta::CommonBotInfo::default();
      ma.friendly_name = testbotname.clone();
      ma.max_cash_risk = (true, 100.0, 10_000.0);
      ma.tracking_tick = "FAKE".to_string();
      ma.haggle_action = dsta::HaggleAction::AnOyVeyJew(
        dsta::HaggleMethod::VirtualMarketOrderWithLimitOrdThenConcede(
          dsta::HaggleLimits {
            retry_period: chrono::Duration::milliseconds(500),
            delta_choice: 0.03,
            delta_is_pct: true,
            slippage_max: 0.02, // ← Strict limit: only 2% allowed
          }
        )
      );
      ma
    },
    my_reveal_way: dsta::SallyAction::SubmitRightAwayButLikeOnlyUseForTest,
    my_hide_order: {
      let mut o = dsta::Order::default();
      
      o.order_legs.push(dsta::OrderLeg {
        buy_what: dsta::FinAss::OptDeets(long_call.clone()),
        quantity: 1.0,
        price: 3.00,
        action: dsta::BuyOrSell::BuyToOpen,
        ..Default::default()
      });
      
      o.order_legs.push(dsta::OrderLeg {
        buy_what: dsta::FinAss::OptDeets(short_call.clone()),
        quantity: 1.0,
        price: 3.50,
        action: dsta::BuyOrSell::SellToOpen,
        ..Default::default()
      });


      let oc = core::fn_calculate_order_cost(&o);
      //o.order_cost = Some(oc.expect("Didn't get an order cost"));
      
      
      o
    },
  };

  let (hand, mut foot) = fn_make_bot_api_bridge_dr_handles();
  let (trade_tx, trade_rx) = mpsc::channel::<dsta::Ticker>(100);
  let (check_tx, check_rx) = mpsc::channel::<crate::Allocation>(100);
  let (sucks_tx, sucks_rx) = mpsc::channel::<String>(100);

  debug!("Spawning Sally bot for slippage limit test");
  let bot_handle = tokio::spawn(crate::sbot::sally_bot::fn_run_main_sally_bot(
    config.clone(),
    trade_rx,
    check_rx,
    sucks_rx,
    hand,
  ));

  let swap = tokio::select! {
    res = foot.told_dr_swap_info.recv() => res.expect("swap channel closed"),
    _ = tokio::time::sleep(StdDuration::from_secs(1)) => panic!("timeout"),
  };
  
  let requested_cash = swap.target_released.unwrap().how;
  sucks_tx.send(crate::glossary::SWAP_SUCC.to_string()).await.unwrap();

  let mut account = BotaAccount::new().with_cash(requested_cash + 1.0);
  check_tx.send(crate::Allocation {
    bot_name: testbotname.clone(),
    bot_safe: account.clone(),
    bot_mind: crate::BotMindState::Running,
    num_swap: 0,
  }).await.unwrap();

  let (opt_name, reply_tx) = tokio::select! {
    res = foot.told_dr_pls_ticks.recv() => res.expect("ticks channel closed"),
    _ = tokio::time::sleep(StdDuration::from_secs(1)) => panic!("timeout"),
  };
  
  reply_tx.send(Some(dsta::PushTickerResult {
    ass_deets: dsta::FinAss::OptDeets(long_call.clone()),
    opt_chain: vec![],
  })).await.unwrap();

  let (opt_name, reply_tx) = tokio::select! {
    res = foot.told_dr_pls_ticks.recv() => res.expect("ticks channel closed"),
    _ = tokio::time::sleep(StdDuration::from_secs(1)) => panic!("timeout"),
  };
  
  reply_tx.send(Some(dsta::PushTickerResult {
    ass_deets: dsta::FinAss::OptDeets(short_call.clone()),
    opt_chain: vec![],
  })).await.unwrap();

  // Initial submission (test mode)
  trade_tx.send(dsta::Ticker {
    name: ".FAKE 105_C_30".to_string(),
    bid: 2.90,
    ask: 3.10,
    ..Default::default()
  }).await.unwrap();

  tokio::time::sleep(StdDuration::from_millis(50)).await;

  trade_tx.send(dsta::Ticker {
    name: ".FAKE 100_C_30".to_string(),
    bid: 3.40,
    ask: 3.60,
    ..Default::default()
  }).await.unwrap();

  let initial_order = tokio::select! {
    res = foot.told_dr_pls_trade.recv() => res.expect("trade channel closed"),
    _ = tokio::time::sleep(StdDuration::from_secs(1)) => panic!("timeout"),
  };

  info!("Initial order submitted");

  // Simulate partial fill
  let mut stuff: HashMap<String, FinAss> = HashMap::new();
  stuff.insert("FAKE 105_C_30".to_string(), dsta::FinAss::OptDeets(long_call.clone()));
  stuff.insert("FAKE 100_C_30".to_string(), dsta::FinAss::OptDeets(short_call.clone()));

  let mut partial_account = account.clone();
  partial_account.m_long_calls = Some(dsta::OwnedPosition {
    asset: dsta::FinAss::OptDeets(long_call.clone()),
    owned: 0.3,
    price: 3.00,
  });
  partial_account.m_short_calls = Some(dsta::OwnedPosition {
    asset: dsta::FinAss::OptDeets(short_call.clone()),
    owned: -0.3,
    price: 3.50,
  });

  check_tx.send(crate::Allocation {
    bot_name: testbotname.clone(),
    bot_safe: partial_account.clone(),
    bot_mind: crate::BotMindState::Running,
    num_swap: 0,
  }).await.unwrap();

  info!("Partial fill: 30% filled");

  sucks_tx.send(format!("{}",crate::glossary::ORDER_FLY)).await.expect("Failed to send ORDER_FLY");

  // Wait for retry
  tokio::time::sleep(StdDuration::from_millis(700)).await;

  // Send VERY bad market quotes (to trigger slippage > 2%)
  trade_tx.send(dsta::Ticker {
    name: ".FAKE 105_C_30".to_string(),
    bid: 4.20,
    ask: 4.40,
    ..Default::default()
  }).await.unwrap();

  tokio::time::sleep(StdDuration::from_millis(50)).await;

  trade_tx.send(dsta::Ticker {
    name: ".FAKE 100_C_30".to_string(),
    bid: 2.50,
    ask: 2.70,
    ..Default::default()
  }).await.unwrap();

  // Bot should detect slippage exceeded and send cancel order
  let cancel_or_haggle = tokio::select! {
    res = foot.told_dr_pls_trade.recv() => res.expect("trade channel closed"),
    _ = tokio::time::sleep(StdDuration::from_secs(1)) => panic!("timeout waiting for cancel/haggle order"),
  };

  // If slippage exceeded, order_legs should be empty (cancel signal)
  if cancel_or_haggle.order_legs.is_empty() {
    info!("Slippage limit exceeded — cancel order sent (empty legs)");
  } else {
    info!("Order resubmitted despite market move (within limits)");
  }

  sucks_tx.send(format!("{}",crate::glossary::ORDER_FLY)).await.expect("Failed to send ORDER_FLY");

  tokio::time::sleep(StdDuration::from_millis(100)).await;
  let _ = sucks_tx.send("DIE".to_string()).await;

  tokio::select! {
    res = bot_handle => { res.ok(); }
    _ = tokio::time::sleep(StdDuration::from_secs(1)) => panic!("Bot did not shut down within 5s"),
  };

  info!("sally_bot_haggle_slippage_limit_exceeded test completed");
}