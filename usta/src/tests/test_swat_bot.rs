// usta/src/tests/test_swat_bot.rs

use super::*;
use tokio::sync::mpsc;
use std::time::Duration as StdDuration;
use crate::*;
use crate::core::*;
use crate::dr_robo::*;
use crate::sbot::swat_bot::*;
use crate::tests::util::*;
use dsta::*;
use chrono::{Utc, Duration};
use std::collections::HashMap;
use log::{warn, trace, error, info, debug};


#[tokio::test] // PASSES
async fn swat_bot_seppuku_exit_long_stock()
{
  let _ = setup_log4rs("swat_bot_seppuku_exit_long_stock");
  info!("Starting swat_bot_seppuku_exit_long_stock test");

  let bot_name = "USER_AAPL";
  let tracking_tick = "AAPL";

  let (trade_tx, trade_rx) = mpsc::channel(100);
  let (check_tx, check_rx) = mpsc::channel(100);
  let (sucks_tx, sucks_rx) = mpsc::channel(10);
  let (ticks_tx, tickss_rx) = mpsc::channel(10);

  let (hand, mut foot) = fn_make_bot_api_bridge_dr_handles();

  let config = dsta::MakeSwatBotsGo
  {
    my_accounting: dsta::CommonBotInfo
    {
      friendly_name: format!("{}", bot_name),
      tracking_tick: format!("{}", tracking_tick),
      ..Default::default()
    },
  };

  debug!("Spawning SWAT bot task");
  let bot_handle = tokio::spawn(fn_run_main_swat(
    config,
    trade_rx,
    check_rx,
    sucks_rx,
    hand,
    tokio::sync::mpsc::channel(1).0,
    None,
    ticks_tx,
  ));

  tokio::time::sleep(StdDuration::from_millis(50)).await;

  log::trace!("Waiting for tracking ticker subscription request...");
  let (ticker_name, reply_tx) = tokio::select!
  {
    res = foot.told_dr_pls_ticks.recv() =>
    {
      res.expect("ticks channel closed")
    }
    _ = tokio::time::sleep(StdDuration::from_secs(2)) =>
    {
      panic!("timeout waiting for ticker subscription");
    }
  };
  assert_eq!(ticker_name, tracking_tick);

  let mut stk = dsta::Stk::default();
  stk.ticker_name = format!("{}",tracking_tick);

  reply_tx
    .send(Some(dsta::PushTickerResult
    { ass_deets: dsta::FinAss::StkDeets(stk.clone()),
      opt_chain: vec![],
    }))
    .await
    .unwrap();


  // Swat bot then requests account finass tickers
  log::trace!("Waiting for tracking ticker subscription request...");
  let (ticker_name, reply_tx) = tokio::select!
  {
    res = foot.told_dr_pls_ticks.recv() =>
    {
      res.expect("ticks channel closed")
    }
    _ = tokio::time::sleep(StdDuration::from_secs(2)) =>
    {
      panic!("timeout waiting for ticker subscription");
    }
  };
  assert_eq!(ticker_name, tracking_tick);
  reply_tx
    .send(Some(dsta::PushTickerResult
    { ass_deets: dsta::FinAss::StkDeets(stk.clone()),
      opt_chain: vec![],
    }))
    .await
    .unwrap();

  let mut alloc = crate::Allocation
  {
    bot_name: format!("{}", bot_name),
    bot_safe: crate::BotaAccount::default(),
    bot_mind: crate::BotMindState::Running,
    num_swap: 0,
  };
  alloc.bot_safe.m_cash = 5000.0;
  alloc.bot_safe.m_stock = Some(dsta::OwnedPosition
  {
    owned: 10.0,
    price: 150.0,
    asset: dsta::FinAss::StkDeets(dsta::Stk
    {
      ticker_name: format!("{}", tracking_tick),
      ..Default::default()
    }),
  });

  check_tx.send(alloc.clone()).await.unwrap();
  tokio::time::sleep(StdDuration::from_millis(50)).await;

  trade_tx.send(dsta::Ticker
  {
    name: format!("{}", tracking_tick),
    bid: 150.5,
    ask: 150.6,
    bid_size: 100.0,
    ask_size: 100.0,
    time: chrono::Utc::now(),
    theo_price: None,
  }).await.unwrap();
  tokio::time::sleep(StdDuration::from_millis(50)).await;

  let mut seppuku_alloc = alloc.clone();
  seppuku_alloc.bot_mind = crate::BotMindState::Seppuku;
  check_tx.send(seppuku_alloc).await.unwrap();
  tokio::time::sleep(StdDuration::from_millis(100)).await;

  let exit_order = tokio::select!
  {
    res = foot.told_dr_pls_trade.recv() =>
    {
      res.expect("trade channel closed")
    }
    _ = tokio::time::sleep(StdDuration::from_secs(2)) =>
    {
      panic!("timeout waiting for exit order");
    }
  };

  assert_eq!(exit_order.order_legs.len(), 1);
  let leg = &exit_order.order_legs[0];
  assert_eq!(leg.buy_what.order_name(), tracking_tick);
  assert_eq!(leg.action, dsta::BuyOrSell::SellToClose);


  let epsilon = 1e-10; 
  assert!( (leg.quantity - 10.0).abs() < epsilon);
  assert!(leg.price > 0.0);

  info!("[TEST] ✓ Exit order validated: {} {} @ {:.2}", leg.quantity, leg.action as i32, leg.price);
  sucks_tx.send(format!("{}",crate::glossary::ORDER_FLY)).await.expect("Failed to send SWAP_SUCC");

  sucks_tx.send(format!("DIE")).await.unwrap();
  tokio::select!
  {
    res = bot_handle =>
    {
      res.ok();
    }
    _ = tokio::time::sleep(StdDuration::from_secs(2)) =>
    {
      panic!("Bot did not shut down within 2s");
    }
  };

  info!("swat_bot_seppuku_exit_long_stock test completed");
}

#[tokio::test]
async fn swat_bot_seppuku_exit_short_position()
{
  let _ = setup_log4rs("swat_bot_seppuku_exit_short_position");
  info!("Starting swat_bot_seppuku_exit_short_position test");

  let bot_name = "USER_SPY";
  let tracking_tick = "SPY";

  let (trade_tx, trade_rx) = mpsc::channel(100);
  let (check_tx, check_rx) = mpsc::channel(100);
  let (sucks_tx, sucks_rx) = mpsc::channel(10);

  let (hand, mut foot) = fn_make_bot_api_bridge_dr_handles();

  let config = dsta::MakeSwatBotsGo
  {
    my_accounting: dsta::CommonBotInfo
    {
      friendly_name: format!("{}", bot_name),
      tracking_tick: format!("{}", tracking_tick),
      ..Default::default()
    },
  };
  let (ticks_tx, tickss_rx) = mpsc::channel(10);
  debug!("Spawning SWAT bot task");
  let bot_handle = tokio::spawn(fn_run_main_swat(
    config,
    trade_rx,
    check_rx,
    sucks_rx,
    hand,
    tokio::sync::mpsc::channel(1).0,
    None,
    ticks_tx,
  ));

  tokio::time::sleep(StdDuration::from_millis(50)).await;

  log::trace!("Waiting for trackingn ticker subscription request...");
  let (ticker_name, reply_tx) = tokio::select!
  {
    res = foot.told_dr_pls_ticks.recv() =>
    {
      res.expect("ticks channel closed")
    }
    _ = tokio::time::sleep(StdDuration::from_secs(2)) =>
    {
      panic!("timeout waiting for ticker subscription");
    }
  };
  assert_eq!(ticker_name, tracking_tick);

  let mut stk = dsta::Stk::default();
  stk.ticker_name = format!("{}",tracking_tick);

  reply_tx
    .send(Some(dsta::PushTickerResult
    { ass_deets: dsta::FinAss::StkDeets(stk),
      opt_chain: vec![],
    }))
    .await
    .unwrap();

  let mut alloc = crate::Allocation
  {
    bot_name: format!("{}", bot_name),
    bot_safe: crate::BotaAccount::default(),
    bot_mind: crate::BotMindState::Running,
    num_swap: 0,
  };
  alloc.bot_safe.m_cash = 10000.0;
  alloc.bot_safe.m_stock = Some(dsta::OwnedPosition
  {
    owned: -5.0,
    price: 450.0,
    asset: dsta::FinAss::StkDeets(dsta::Stk
    {
      ticker_name: format!("{}", tracking_tick),
      ..Default::default()
    }),
  });

  check_tx.send(alloc.clone()).await.unwrap();
  tokio::time::sleep(StdDuration::from_millis(50)).await;



  trade_tx.send(dsta::Ticker
  {
    name: format!("{}", tracking_tick),
    bid: 450.2,
    ask: 450.4,
    bid_size: 200.0,
    ask_size: 200.0,
    time: chrono::Utc::now(),
    theo_price: None,
  }).await.unwrap();
  tokio::time::sleep(StdDuration::from_millis(50)).await;



  log::trace!("Waiting for botaaccount ticker subscription request...");
  let (ticker_name, reply_tx) = tokio::select!
  {
    res = foot.told_dr_pls_ticks.recv() =>
    {
      res.expect("ticks channel closed")
    }
    _ = tokio::time::sleep(StdDuration::from_secs(2)) =>
    {
      panic!("timeout waiting for ticker subscription");
    }
  };
  assert_eq!(ticker_name, tracking_tick);

  let mut stk = dsta::Stk::default();
  stk.ticker_name = format!("{}",tracking_tick);

  reply_tx
    .send(Some(dsta::PushTickerResult
    { ass_deets: dsta::FinAss::StkDeets(stk),
      opt_chain: vec![],
    }))
    .await
    .unwrap();

  trade_tx.send(dsta::Ticker
  {
    name: format!("{}", tracking_tick),
    bid: 450.2,
    ask: 450.4,
    bid_size: 200.0,
    ask_size: 200.0,
    time: chrono::Utc::now(),
    theo_price: None,
  }).await.unwrap();
  tokio::time::sleep(StdDuration::from_millis(50)).await;


  let mut seppuku_alloc = alloc.clone();
  seppuku_alloc.bot_mind = crate::BotMindState::Seppuku;
  check_tx.send(seppuku_alloc).await.unwrap();
  tokio::time::sleep(StdDuration::from_millis(100)).await;

  let exit_order = tokio::select!
  {
    res = foot.told_dr_pls_trade.recv() =>
    {
      res.expect("trade channel closed")
    }
    _ = tokio::time::sleep(StdDuration::from_secs(2)) =>
    {
      panic!("timeout waiting for exit order");
    }
  };

  assert_eq!(exit_order.order_legs.len(), 1);
  let leg = &exit_order.order_legs[0];
  assert_eq!(leg.action, dsta::BuyOrSell::BuyToClose);

  let epsilon = 1e-10; 
  assert!( (leg.quantity - 5.0).abs() < epsilon);
  assert!(leg.price >= 450.4);

  info!("[TEST] ✓ Short exit order validated: {} {} @ {:.2}", leg.quantity, leg.action as i32, leg.price);
  sucks_tx.send(format!("{}",crate::glossary::ORDER_FLY)).await.expect("Failed to send SWAP_SUCC");

  sucks_tx.send(format!("DIE")).await.unwrap();
  tokio::select!
  {
    res = bot_handle =>
    {
      res.ok();
    }
    _ = tokio::time::sleep(StdDuration::from_secs(2)) =>
    {
      panic!("Bot did not shut down within 2s");
    }
  };

  info!("swat_bot_seppuku_exit_short_position test completed");
}
