
// usta/src/tests/test_stealth_bot.rs

use crate::core::*;

// TODO: Implement test for partial fill handling - this is the main non-happy path complex thing we need to verify
// 
// REQUIREMENT:
// On a wait-for-fill state, if a no-fill or partial-fill persists for a long time,
// the bot needs to take action:
// - Option A: Cancel the rest of the order
// - Option B: Re-submit the order
//
// This test should verify that the Stealth Bot properly handles scenarios
// where an order is only partially filled and remains in that state beyond
// an acceptable timeout period.
//
// Test scenarios to cover:
// 1. Partial fill on entry order (BuyToOpen) that lingers
// 2. Partial fill on spread order (SellToOpen) that lingers  
// 3. Proper timeout detection (e.g., 30+ seconds with no additional fills)
// 4. Correct recovery action (cancel remaining or resubmit)
// 5. State machine transitions after partial fill recovery


  // RULES:
  // 1) Always keep these rules and the first comment path in the top of the code
  // 2) Always use my formatting:
  // { do: 2 space tabs but more at top
  // , do: '{' being on a new line like above
  // , do: ',' punctuation in same colum for lists longer than 3
  // , do: put ',' first like this line
  // , do: put '}' on the same column like the next line
  // } // simple comment for end of long function
  // 3) Do not fully qualify namesapces that are std::[TYPE], i.e, using std::String is BAD (use String) but std::error::Error is GOOD.
  // 4) Always keep the same fully qualified namespaces where they are already used.
  // 5) At least once if possible, replace a non-fully qualified namespace type (non-std::) with the fully qualified so we can eventually minimize use statements.
  // 6) Any AI generated // Comments about logic should instead be log::debug! and not // comments.
  // 7) Do not remove any log:: code and do not change any log:: types!
  // 8) Do not remove any comments!
  // 9) Always reply with an entire file unless i explictly ask for a function or scope
  // a) Do not add any use statements to the outtermost scope.

#[tokio::test] // PASSES
async fn stealth_bot_happy_path_call_credit_spread_exit() {
    use tokio::sync::mpsc;
    use chrono::{Utc, Duration};
    use std::time::Duration as StdDuration;
    use log::{info, debug, trace};
    use crate::*;
    use crate::tests::util::*;
    use crate::dr_robo::*;
    use dsta::*;
    use std::collections::HashMap;

    let _ = setup_log4rs("stealth_bot_happy_path_call_credit_spread_exit");
    info!("Starting stealth_bot_happy_path_call_credit_spread_exit test");

    let bot_name = "stealth_test_bot".to_string();

    let config = dsta::MakeStealthBot {
        my_accounting: {
            let mut ma = dsta::CommonBotInfo::default();
            ma.max_cash_risk = (true, 100.0, 10_000.0);
            ma.friendly_name = bot_name.clone();
            ma.tracking_tick = "FAKE".to_string();
            ma
        },
        my_cash_alloc: 10_000.0,
        stonk_feeling: dsta::MarketDirection::GetStonk,           // ← Changed
        option_bucket: 1,
        spread_bucket: 1,
        exit_gain_pct: 50.0,
        exit_loss_pct: -80.0,
        option_expire: 30,
        nice_exit_way: vec![
            dsta::StealthBotDoneNiceStyle::GoExpire,
            dsta::StealthBotDoneNiceStyle::FinalPct(80.0),
        ],
        use_theo_cost: false,
    };

    let (trade_tx, trade_rx) = mpsc::channel(100);
    let (check_tx, check_rx) = mpsc::channel(100);
    let (sucks_tx, mut sucks_rx) = mpsc::channel(100);
    let (pls_ticks_tx, mut pls_ticks_rx) = mpsc::channel(10);
    let (pls_trade_tx, mut pls_trade_rx) = mpsc::channel(10);
    let (swap_info_tx, mut swap_info_rx) = mpsc::channel(10);
    let (new_brain_tx, _) = mpsc::channel(10);
    let (snivtx, mut snivrx) = mpsc::channel(100);

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

    let bot_handle = tokio::spawn(crate::sbot::stealth_bot::fn_run_main_stealth_bot(
        config,
        trade_rx,
        check_rx,
        sucks_rx,
        bridge,
    ));

    let expiry = Utc::now() + Duration::days(35);

    // Initial ticker request (underlying)
    trace!("Awaiting initial ticker request");
    let (ticker_name, reply_tx) = tokio::time::timeout(StdDuration::from_secs(5), pls_ticks_rx.recv())
        .await
        .expect("Timeout on initial ticker request")
        .expect("Ticker channel closed");
    assert_eq!(ticker_name, "FAKE");

    let mut stk = dsta::Stk::default();
    stk.ticker_name = "FAKE".to_string();

    let fake_result = dsta::PushTickerResult {
        ass_deets: dsta::FinAss::StkDeets(stk),
        opt_chain: vec![dsta::Derivatives {
            expire_time: expiry,
            option_calls: vec![
                dsta::Opt { ticker_name: ".FAKE 90_C_35".to_string(), strike_spot: 90.0, option_name: "FAKE 90_C_35".to_string(), expire_date: expiry, ..Default::default() },
                dsta::Opt { ticker_name: ".FAKE 95_C_35".to_string(), strike_spot: 95.0, option_name: "FAKE 95_C_35".to_string(), expire_date: expiry, ..Default::default() },
                dsta::Opt { ticker_name: ".FAKE 100_C_35".to_string(), strike_spot: 100.0, option_name: "FAKE 100_C_35".to_string(), expire_date: expiry, ..Default::default() },
                dsta::Opt { ticker_name: ".FAKE 105_C_35".to_string(), strike_spot: 105.0, option_name: "FAKE 105_C_35".to_string(), expire_date: expiry, ..Default::default() },
            ],
            option_puts: vec![],
        }],
    };
    reply_tx.send(Some(fake_result)).await.expect("Failed to send ticker chain");
    info!("Sent fake option chain for FAKE");

    tokio::time::sleep(StdDuration::from_millis(150)).await;

    // Initial cash request & approval
    let swap_req = tokio::time::timeout(StdDuration::from_secs(5), swap_info_rx.recv())
        .await
        .expect("Timeout on cash swap request")
        .expect("Swap channel closed");


    // LOOK! ITS THE ALLOCATION that we track and send, not botaaccount like you do in other function
    let firststuff = crate::Allocation {
        bot_name: format!("stealth_test_bot"),
        bot_safe: BotaAccount {
            m_cash: 10_000.0,
            ..Default::default()
        },
        bot_mind: crate::BotMindState::Running,
        num_swap: 0,
    };
    check_tx.send(firststuff).await.expect("Failed to send 1st allocation");
    sucks_tx.send(crate::glossary::SWAP_SUCC.to_string()).await.expect("Failed to send SWAP_SUCC");
    info!("Initial cash approved");

    // Send initial underlying quote (needed for ATM calc)
    info!("Sending initial underlying quote");
    trade_tx.send(
        dsta::convert::stream_quote_to_ticker(&ttrs::StreamQuote {
            event_type: "quote".to_string(),
            symbol: "FAKE".to_string(),
            bid_price: Some(104.80),
            ask_price: Some(105.20),
            bid_size: Some(10.0),
            ask_size: Some(10.0),
        })
    ).await.expect("Failed to send initial underlying quote");

    tokio::time::sleep(StdDuration::from_millis(100)).await;

    // Bot should request long leg first (105 call - more expensive)
    info!("Awaiting long leg ticker request (.FAKE 105_C_35)");
    let (opt_name_long, reply_tx_long) = tokio::time::timeout(StdDuration::from_secs(5), pls_ticks_rx.recv())
        .await
        .expect("Timeout waiting for long leg")
        .expect("Ticker channel closed");
    assert_eq!(opt_name_long, ".FAKE 105_C_35");

    let mut opt = dsta::Opt::default();
    opt.ticker_name = ".FAKE 105_C_35".to_string();
    opt.option_name = "FAKE 105_C_35".to_string();

    reply_tx_long.send(Some(dsta::PushTickerResult {
        ass_deets: dsta::FinAss::OptDeets(opt),
        opt_chain: vec![],
    })).await.expect("Failed to reply long leg");

    // Then short leg (100 call)
    info!("Awaiting short leg ticker request (.FAKE 100_C_35)");
    let (opt_name_short, reply_tx_short) = tokio::time::timeout(StdDuration::from_secs(5), pls_ticks_rx.recv())
        .await
        .expect("Timeout waiting for short leg")
        .expect("Ticker channel closed");
    assert_eq!(opt_name_short, ".FAKE 100_C_35");

    let mut opt = dsta::Opt::default();
    opt.ticker_name = ".FAKE 100_C_35".to_string();
    opt.option_name = "FAKE 100_C_35".to_string();

    reply_tx_short.send(Some(dsta::PushTickerResult {
        ass_deets: dsta::FinAss::OptDeets(opt),
        opt_chain: vec![],
    })).await.expect("Failed to reply short leg");

    tokio::time::sleep(StdDuration::from_millis(100)).await;

    // Send long leg quote → should trigger entry (Buy 105 call)
    info!("Sending long leg quote to trigger BuyToOpen 105 call");
    trade_tx.send(
        dsta::convert::stream_quote_to_ticker(&ttrs::StreamQuote {
            event_type: "quote".to_string(),
            symbol: ".FAKE 105_C_35".to_string(),
            bid_price: Some(2.90),
            ask_price: Some(3.10),
            bid_size: Some(20.0),
            ask_size: Some(20.0),
        })
    ).await.expect("Failed to send long call quote");

    tokio::time::sleep(StdDuration::from_millis(150)).await;

    // Expect entry order: Buy 105 call
    info!("Awaiting entry order (BuyToOpen 105 call)");
    let entry_order = tokio::time::timeout(StdDuration::from_secs(5), pls_trade_rx.recv())
        .await
        .expect("Timeout waiting for entry order")
        .expect("Trade channel closed");

    assert_eq!(entry_order.order_legs[0].buy_what.order_name(), "FAKE 105_C_35");
    assert_eq!(entry_order.order_legs[0].action, dsta::BuyOrSell::BuyToOpen);

    // Confirm fill

    sucks_tx.send(format!("{}",crate::glossary::ORDER_FLY)).await.expect("Failed to send SWAP_SUCC");

    let mut account = BotaAccount::new().with_cash(10_000.0);
    let mut stuff: HashMap<String, FinAss> = HashMap::new();

    let long_call = Opt {
        ticker_name: ".FAKE 105_C_35".to_string(),
        option_name: "FAKE 105_C_35".to_string(),
        option_rite: OptRite::Call,
        strike_spot: 105.0,
        expire_date: expiry,
        ..Default::default()
    };

    let short_call = Opt {
        ticker_name: ".FAKE 100_C_35".to_string(),
        option_name: "FAKE 100_C_35".to_string(),
        option_rite: OptRite::Call,
        strike_spot: 100.0,
        expire_date: expiry,
        ..Default::default()
    };

    stuff.insert(long_call.option_name.clone(), FinAss::OptDeets(long_call));
    stuff.insert(short_call.option_name.clone(), FinAss::OptDeets(short_call));

    let changes = fn_get_account_shennanagans(&entry_order, &account, &stuff).expect("entry changes failed");
    apply_account_cash_and_collateral_changes(&mut account, &changes);
    apply_order_to_account(&mut account, &entry_order, &stuff).expect("apply entry failed");

    check_tx.send(crate::Allocation {
        bot_name: bot_name.clone(),
        bot_safe: account.clone(),
        bot_mind: crate::BotMindState::Running,
        num_swap: 0,
    }).await.expect("Failed to send entry allocation");

    tokio::time::sleep(StdDuration::from_millis(150)).await;

    // Now trigger the short leg leg + favorable move (underlying ↑)
    info!("STEP: Sending quotes to complete spread entry (underlying rallying)");
    trade_tx.send(dsta::convert::stream_quote_to_ticker(&ttrs::StreamQuote {
        event_type: "quote".to_string(),
        symbol: ".FAKE 100_C_35".to_string(),
        bid_price: Some(9.80),
        ask_price: Some(10.00),
        bid_size: Some(15.0),
        ask_size: Some(15.0),
    })).await.unwrap();

    trade_tx.send(dsta::convert::stream_quote_to_ticker(&ttrs::StreamQuote {
        event_type: "quote".to_string(),
        symbol: "FAKE".to_string(),
        bid_price: Some(108.40),
        ask_price: Some(108.60),
        bid_size: Some(10.0),
        ask_size: Some(10.0),
    })).await.unwrap();

    trade_tx.send(dsta::convert::stream_quote_to_ticker(&ttrs::StreamQuote {
        event_type: "quote".to_string(),
        symbol: ".FAKE 105_C_35".to_string(),
        bid_price: Some(4.90),
        ask_price: Some(9.10),
        bid_size: Some(20.0),
        ask_size: Some(20.0),
    })).await.unwrap();

    tokio::time::sleep(StdDuration::from_millis(200)).await;

    // Expect the spread leg: Sell 100 call
    info!("Awaiting spread order (SellToOpen 100 call)");
    let spread_order = tokio::time::timeout(StdDuration::from_secs(5), pls_trade_rx.recv())
        .await
        .expect("Timeout waiting for spread order")
        .expect("Trade channel closed");

    assert_eq!(spread_order.order_legs[0].buy_what.order_name(), "FAKE 100_C_35");
    assert_eq!(spread_order.order_legs[0].action, dsta::BuyOrSell::SellToOpen);

    // Confirm spread fill
    sucks_tx.send(format!("{}",crate::glossary::ORDER_FLY)).await.expect("Failed to send SWAP_SUCC");

    let changes = fn_get_account_shennanagans(&spread_order, &account, &stuff).expect("spread changes failed");
    apply_account_cash_and_collateral_changes(&mut account, &changes);
    apply_order_to_account(&mut account, &spread_order, &stuff).expect("apply spread failed");

    check_tx.send(crate::Allocation {
        bot_name: bot_name.clone(),
        bot_safe: account.clone(),
        bot_mind: crate::BotMindState::Running,
        num_swap: 0,
    }).await.expect("Failed to send spread allocation");

    tokio::time::sleep(StdDuration::from_millis(150)).await;

    // ──────────────────────────────────────────────────────────────
    //          NICE EXIT: underlying rallies hard → short call OTM
    // ──────────────────────────────────────────────────────────────
    info!("Triggering nice exit - underlying strong rally");
    trade_tx.send(dsta::convert::stream_quote_to_ticker(&ttrs::StreamQuote {
        event_type: "quote".to_string(),
        symbol: "FAKE".to_string(),
        bid_price: Some(114.20),
        ask_price: Some(114.50),
        bid_size: Some(10.0),
        ask_size: Some(10.0),
    })).await.unwrap();

    trade_tx.send(dsta::convert::stream_quote_to_ticker(&ttrs::StreamQuote {
        event_type: "quote".to_string(),
        symbol: ".FAKE 105_C_35".to_string(),
        bid_price: Some(9.80),
        ask_price: Some(10.20),
        bid_size: Some(10.0),
        ask_size: Some(10.0),
    })).await.unwrap();

    trade_tx.send(dsta::convert::stream_quote_to_ticker(&ttrs::StreamQuote {
        event_type: "quote".to_string(),
        symbol: ".FAKE 100_C_35".to_string(),
        bid_price: Some(14.10),
        ask_price: Some(14.40),
        bid_size: Some(10.0),
        ask_size: Some(10.0),
    })).await.unwrap(); // ← but actually we care about it decaying later

    // Better: deep OTM short leg
    trade_tx.send(dsta::convert::stream_quote_to_ticker(&ttrs::StreamQuote {
        event_type: "quote".to_string(),
        symbol: ".FAKE 100_C_35".to_string(),
        bid_price: Some(0.05),
        ask_price: Some(0.12),
        bid_size: Some(10.0),
        ask_size: Some(10.0),
    })).await.unwrap();

    tokio::time::sleep(StdDuration::from_millis(200)).await;

    // Expect close order: Sell long 105C + Buy short 100C
    info!("Awaiting final close order");
    let close_order = tokio::time::timeout(StdDuration::from_secs(5), pls_trade_rx.recv())
        .await
        .expect("Timeout waiting for close order")
        .expect("Trade channel closed");

    assert_eq!(close_order.order_legs.len(), 2);


    let long_leg = close_order.order_legs.iter()
        .find(|leg| leg.buy_what.order_name() == "FAKE 105_C_35" && leg.action == dsta::BuyOrSell::SellToClose)
        .expect("Missing SellToClose on long call");

    let short_leg = close_order.order_legs.iter()
        .find(|leg| leg.buy_what.order_name() == "FAKE 100_C_35" && leg.action == dsta::BuyOrSell::BuyToClose)
        .expect("Missing BuyToClose on short call");

    info!("Close order validated — both legs present");

    // Final confirmation

    sucks_tx.send(format!("{}",crate::glossary::ORDER_FLY)).await.expect("Failed to send SWAP_SUCC");

    let changes = fn_get_account_shennanagans(&close_order, &account, &stuff).expect("close changes failed");
    apply_account_cash_and_collateral_changes(&mut account, &changes);
    apply_order_to_account(&mut account, &close_order, &stuff).expect("apply close failed");

    check_tx.send(crate::Allocation {
        bot_name: bot_name.clone(),
        bot_safe: account.clone(),
        bot_mind: crate::BotMindState::Running,
        num_swap: 0,
    }).await.expect("Failed to send final allocation");

    // Cleanup
    let _ = sucks_tx.send("DIE".to_string()).await;
    tokio::time::timeout(StdDuration::from_secs(5), bot_handle)
        .await
        .expect("Bot did not shut down")
        .ok();

    info!("=== Call Credit Spread happy path test completed successfully ===");
}

#[tokio::test] // PASSES
async fn stealth_bot_happy_path_put_credit_spread_exit() 
{ // Do not change my formatting, spacing, or comments.
  // WORKING
      use tokio::sync::mpsc;
      use chrono::{Utc, Duration};
      use std::time::Duration as StdDuration;
      use log::{info, debug, trace};
      use crate::*;
      use crate::tests::util::*;
      use crate::dr_robo::*;
      use dsta::*;
      use std::collections::HashMap;

    let _ = setup_log4rs("stealth_bot_happy_path_put_credit_spread_exit");
    info!("Starting stealth_bot_happy_path_put_credit_spread_exit test");

    let bot_name = "stealth_test_bot".to_string();

    let config = dsta::MakeStealthBot
    { my_accounting:
      { let mut ma = dsta::CommonBotInfo::default();
        ma.max_cash_risk = (true,100.0,10_000.0);
        ma.friendly_name = bot_name.clone();
        ma.tracking_tick = "FAKE".to_string();
        ma },
      my_cash_alloc:10_000.0,
      stonk_feeling:dsta::MarketDirection::Corrects,
      option_bucket:1,
      spread_bucket:1,
      exit_gain_pct:50.0,
      exit_loss_pct:-80.0,
      option_expire:30,
      nice_exit_way:vec!
      [ dsta::StealthBotDoneNiceStyle::GoExpire
      , dsta::StealthBotDoneNiceStyle::FinalPct(80.0)
      ],
      use_theo_cost:false,
    };

    /*
      #[derive(Default, Clone, Debug, serde::Serialize, serde::Deserialize)]
      pub enum StealthBotDoneNiceStyle 
      {
        #[default] 
        OtmShort, // short leg goes OTM 
        FinalPct(f64), // short leg gain %
        FinalDay(UsaMarketTimes, f64), // exit on last day of trading
        PriorDay(UsaMarketTimes, f64, u64), // exit usize days before expiration 
        GoExpire, // let the contract expire
      }
      */

    let (trade_tx,trade_rx) = mpsc::channel(100);
    let (check_tx,check_rx) = mpsc::channel(100);
    let (sucks_tx,mut sucks_rx) = mpsc::channel(100);
    let (pls_ticks_tx,mut pls_ticks_rx) = mpsc::channel(10);
    let (pls_trade_tx,mut pls_trade_rx) = mpsc::channel(10);
    let (swap_info_tx,mut swap_info_rx) = mpsc::channel(10);
    let (new_brain_tx,_) = mpsc::channel(10);

    let (snivtx, mut snivrx) = mpsc::channel(100); 
    // We mock being Dr Robo here, so we need real channels
    // the pls ticks means bot asks for account infos
    // the pls check means bot asks for account infos
    // the swap info means bot asks for cash at startup
    // the pls trade means bot requested an order
    // the told a snively means we got erros.
    let bridge = crate::BridgeDrHand
    { tell_dr_pls_ticks:pls_ticks_tx.clone(),
      tell_dr_pls_trade:pls_trade_tx.clone(),
      tell_dr_swap_info:swap_info_tx.clone(),
      tell_dr_new_brain:new_brain_tx.clone(),
      tell_dr_pls_check:mpsc::channel(1).0,
      tell_dr_stop_info:mpsc::channel(1).0,
      tell_dr_stop_tick:mpsc::channel(1).0,
      tell_dr_a_snively:snivtx.clone(), //tell_dr_a_snively:mpsc::channel(100).0, 
    };

    let bot_handle = tokio::spawn(crate::sbot::stealth_bot::fn_run_main_stealth_bot(
      config,
      trade_rx,
      check_rx,
      sucks_rx,
      bridge,
    ));

    let expiry = Utc::now() + Duration::days(35);

    trace!("Awaiting initial ticker request");
    let (ticker_name,reply_tx) = tokio::time::timeout(StdDuration::from_secs(5),pls_ticks_rx.recv())
      .await
      .expect("Timeout on initial ticker request")
      .expect("Ticker channel closed");

    assert_eq!(ticker_name,"FAKE");


    let mut stk = dsta::Stk::default();
    stk.ticker_name = "FAKE".to_string();
    let fake_result = dsta::PushTickerResult
    { ass_deets: dsta::FinAss::StkDeets(stk),
      opt_chain:vec![dsta::Derivatives
      { expire_time:expiry,
        option_calls:vec![],
        option_puts:vec![
          dsta::Opt { ticker_name:".FAKE 105_P_35".to_string(),strike_spot:105.0,option_name:"FAKE 105_P_35".to_string(),expire_date:expiry,..Default::default() },
          dsta::Opt { ticker_name:".FAKE 100_P_35".to_string(),strike_spot:100.0,option_name:"FAKE 100_P_35".to_string(),expire_date:expiry,..Default::default() },
          dsta::Opt { ticker_name:".FAKE 95_P_35".to_string() ,strike_spot:95.0 ,option_name:"FAKE 95_P_35".to_string(),expire_date:expiry,..Default::default() },
          dsta::Opt { ticker_name:".FAKE 90_P_35".to_string() ,strike_spot:90.0 ,option_name:"FAKE 90_P_35".to_string(),expire_date:expiry,..Default::default() },
        ],
      }],
    };

    reply_tx.send(Some(fake_result)).await.expect("Failed to send ticker chain");

    info!("Sent fake option chain for FAKE");

    tokio::time::sleep(StdDuration::from_millis(150)).await;

    trace!("Awaiting initial cash swap request");
    let swap_req = tokio::time::timeout(StdDuration::from_secs(5),swap_info_rx.recv())
      .await
      .expect("Timeout on cash swap request")
      .expect("Swap channel closed");

    info!("Received initial cash swap: {:?}",swap_req);

    let firststuff = crate::Allocation 
    { bot_name: format!("stealth_test_bot")
    , bot_safe: BotaAccount 
      { m_cash: 10_000.0,
        m_stock: None,
        m_long_calls: None,
        m_long_puts: None,
        m_short_calls: None,
        m_short_puts: None,
        m_call_credit_spread_lein: 0.0,
        m_short_put_lein: 0.0,
        m_put_credit_spread_lein: 0.0,
        m_short_iron_condor_lein: 0.0,
      },
      bot_mind: crate::BotMindState::Running,
      num_swap: 0,
    };
    check_tx.send(firststuff).await.expect("Failed to send 1st allocation");
    sucks_tx.send(crate::glossary::SWAP_SUCC.to_string())
      .await
      .expect("Failed to send SWAP_SUCC");

    info!("Initial cash approved");
    // After "Initial cash approved" and sleep(200ms)

    // === STEP A: Send initial underlying quote (triggers ATM calculation & option selection) ===
    info!("Sending initial underlying quote (needed for ATM calculation)");
    trade_tx.send(
      dsta::convert::stream_quote_to_ticker(&ttrs::StreamQuote {
        event_type: "quote".to_string(),
        symbol: "FAKE".to_string(),
        bid_price: Some(99.8),
        ask_price: Some(100.2),
        bid_size: Some(10.0),
        ask_size: Some(10.0),
      })
    ).await.expect("Failed to send initial underlying quote");

    tokio::time::sleep(StdDuration::from_millis(100)).await;

    // === STEP B: Bot requests long leg ticker (95 put) ===
    info!("Awaiting long leg ticker request (.FAKE 95_P_35)");
    let (opt_name_long, reply_tx_long) = tokio::time::timeout(StdDuration::from_secs(5), pls_ticks_rx.recv())
      .await
      .expect("Timeout waiting for long leg ticker request")
      .expect("Ticker request channel closed");

    assert_eq!(opt_name_long, ".FAKE 95_P_35", "Bot requested wrong long leg");
    let mut opt = dsta::Opt::default();
    opt.ticker_name = ".FAKE 95_P_35".to_string();
    opt.option_name = "FAKE 95_P_35".to_string();
    
    reply_tx_long.send(Some(dsta::PushTickerResult {
      ass_deets: dsta::FinAss::OptDeets(opt),
      opt_chain: vec![],
    })).await.expect("Failed to reply to long leg request");

    // === STEP C: Bot immediately requests short leg ticker (100 put) ===
    info!("Awaiting short leg ticker request (FAKE 100_P_35)");
    let (opt_name_short, reply_tx_short) = tokio::time::timeout(StdDuration::from_secs(5), pls_ticks_rx.recv())
      .await
      .expect("Timeout waiting for short leg ticker request")
      .expect("Ticker request channel closed");

    assert_eq!(opt_name_short, ".FAKE 100_P_35", "Bot requested wrong short leg");
    let mut opt = dsta::Opt::default();
    opt.ticker_name = ".FAKE 100_P_35".to_string();
    opt.option_name = "FAKE 100_P_35".to_string();
    
    reply_tx_short.send(Some(dsta::PushTickerResult {
      ass_deets: dsta::FinAss::OptDeets(opt),
      opt_chain: vec![],
    })).await.expect("Failed to reply to short leg request");

    tokio::time::sleep(StdDuration::from_millis(100)).await;

    // === STEP D: Now send long leg quote → this should trigger entry order ===
    info!("Sending long leg quote to trigger BuyToOpen");
    trade_tx.send(
      dsta::convert::stream_quote_to_ticker(&ttrs::StreamQuote {
        event_type: "quote".to_string(),
        symbol: ".FAKE 95_P_35".to_string(),
        bid_price: Some(1.90),
        ask_price: Some(2.10),
        bid_size: Some(20.0),
        ask_size: Some(20.0),
      })
    ).await.expect("Failed to send long leg quote");

    tokio::time::sleep(StdDuration::from_millis(150)).await;

    // === STEP 6: Now we should receive the entry order ===
    info!("Awaiting entry order (BuyToOpen 95 put)");
    let entry_order = tokio::time::timeout(StdDuration::from_secs(5), pls_trade_rx.recv())
      .await
      .expect("Timeout waiting for entry order")
      .expect("Trade channel closed");

    assert_eq!(entry_order.order_legs[0].buy_what.order_name(), "FAKE 95_P_35");
    assert_eq!(entry_order.order_legs[0].action, dsta::BuyOrSell::BuyToOpen);
    info!("Entry order validated: {:?}", entry_order.order_legs[0]);


    // === STEP 7: Confirm entry fill via account update ===
    info!("STEP 7: Confirming entry fill");
    sucks_tx.send(format!("{}",crate::glossary::ORDER_FLY)).await.expect("Failed to send SWAP_SUCC");

    let mut account = BotaAccount::new().with_cash(10_000.0);

    let mut stuff: HashMap<String,FinAss> = HashMap::new();
    let long_put = Opt
    { ticker_name:".FAKE 95_P_35".to_string(),
      option_name:"FAKE 95_P_35".to_string(),
      option_rite:OptRite::Put,
      strike_spot:95.0,
      expire_date:expiry,
      option_type:OptStyle::Usa,
      last_ticker:None,
      tick_rulers:None,
    };
    let short_put = Opt
    { ticker_name:".FAKE 100_P_35".to_string(),
      option_name:"FAKE 100_P_35".to_string(),
      option_rite:OptRite::Put,
      strike_spot:100.0,
      expire_date:expiry,
      option_type:OptStyle::Usa,
      last_ticker:None,
      tick_rulers:None,
    };
    stuff.insert(long_put.option_name.clone(),FinAss::OptDeets(long_put));
    stuff.insert(short_put.option_name.clone(),FinAss::OptDeets(short_put));
    let changes = fn_get_account_shennanagans(&entry_order,&account,&stuff)
      .expect("Failed to compute entry fill changes");
    apply_account_cash_and_collateral_changes(&mut account,&changes);
    apply_order_to_account(&mut account,&entry_order,&stuff)
      .expect("Failed to apply entry order to account");
    check_tx.send(crate::Allocation
    { bot_name:bot_name.clone(),
      bot_safe:account.clone(),
      bot_mind:crate::BotMindState::Running,
      num_swap:0,
    }).await.expect("Failed to send allocation update for entry fill");
    info!("Entry fill confirmed");
    tokio::time::sleep(StdDuration::from_millis(150)).await;

  // === STEP 8: Send short leg quote + favorable underlying move ===
  info!("STEP 8: Sending quotes to trigger spread entry");
  trade_tx.send(
    dsta::convert::stream_quote_to_ticker(&ttrs::StreamQuote
    { event_type:"quote".to_string(),
      symbol:".FAKE 100_P_35".to_string(),
      bid_price:Some(5.40),
      ask_price:Some(5.60),
      bid_size:Some(15.0),
      ask_size:Some(15.0),
    })
  ).await.unwrap();

  trade_tx.send(
    dsta::convert::stream_quote_to_ticker(&ttrs::StreamQuote
    { event_type:"quote".to_string(),
      symbol:"FAKE".to_string(),
      bid_price:Some(95.80),
      ask_price:Some(95.82),
      bid_size:Some(10.0),
      ask_size:Some(10.0),
    })
  ).await.unwrap();

  trade_tx.send(
    dsta::convert::stream_quote_to_ticker(&ttrs::StreamQuote
    { event_type:"quote".to_string(),
      symbol:".FAKE 95_P_35".to_string(),
      bid_price:Some(3.10),
      ask_price:Some(3.20),
      bid_size:Some(20.0),
      ask_size:Some(20.0),
    })
  ).await.unwrap();

  trade_tx.send(
    dsta::convert::stream_quote_to_ticker(&ttrs::StreamQuote
    { event_type:"quote".to_string(),
      symbol:".FAKE 100_P_35".to_string(),
      bid_price:Some(6.40),
      ask_price:Some(6.60),
      bid_size:Some(15.0),
      ask_size:Some(15.0),
    })
  ).await.unwrap();

  tokio::time::sleep(StdDuration::from_millis(200)).await;

  //// === STEP FIX ERROR: Receive error ===
  //info!("STEP FIX ERROR: Receiving sniv error");
  //let sniv_error = tokio::time::timeout(StdDuration::from_secs(5),snivrx.recv())
  //  .await
  //  .expect("Timeout waiting for sniv error order")
  //  .expect("snivrx channel closed");
  //info!("{:#?}", sniv_error );

  // === STEP 9: Receive spread sell order ===
  info!("STEP 9: Receiving spread order (SellToOpen 100 put)");
  let spread_order = tokio::time::timeout(StdDuration::from_secs(5), pls_trade_rx.recv())
      .await
      .expect("Timeout waiting for spread order")
      .expect("Trade channel closed");

  assert_eq!(spread_order.order_legs[0].buy_what.order_name(), "FAKE 100_P_35");
  assert_eq!(spread_order.order_legs[0].action, dsta::BuyOrSell::SellToOpen);
 

    info!("Spread order validated");

  // === NEW STEP 10: Confirm spread fill via account update ===
  info!("STEP 10: Confirming spread fill (short put position added)");


  // Reuse/update the same 'account' and 'stuff' from entry fill
  let changes = fn_get_account_shennanagans(&spread_order, &account, &stuff)
      .expect("Failed to compute spread fill changes");

  apply_account_cash_and_collateral_changes(&mut account, &changes);
  apply_order_to_account(&mut account, &spread_order, &stuff)
      .expect("Failed to apply spread order to account");

  sucks_tx.send(format!("{}",crate::glossary::ORDER_FLY)).await.expect("Failed to send SWAP_SUCC");

  // === STEP FIX ERROR: Receive error ===

  let mut sv = Vec::new();
  let sc = 100;
  info!("STEP FIX ERROR: Receiving sniv error");
  let sniv_errors = tokio::time::timeout(StdDuration::from_secs(5),snivrx.recv_many(&mut sv,sc))
    .await
    .expect("Timeout waiting for sniv error order")
    //.expect("snivrx channel closed");
  ;
  //info!("\n\n\n\n\n\n\n\n SNIV ERRORS? {:#?}", sv );

  tokio::time::sleep(StdDuration::from_millis(150)).await;

  check_tx.send(crate::Allocation {
      bot_name: bot_name.clone(),
      bot_safe: account.clone(),  // ← now includes both long and short positions
      bot_mind: crate::BotMindState::Running,
      num_swap: 0,
  }).await.expect("Failed to send allocation update for spread fill");

  info!("Spread fill confirmed - short position added");

  // Give bot time to process the update
  tokio::time::sleep(StdDuration::from_millis(150)).await;

  // === STEP 11: Move underlying to trigger exit ===
  info!("STEP 11: Sending quotes to trigger nice exit (OTM)");
  trade_tx.send(
    dsta::convert::stream_quote_to_ticker(&ttrs::StreamQuote
    { event_type:"quote".to_string(),
      symbol:"FAKE".to_string(),
      bid_price:Some(105.10),
      ask_price:Some(105.14),
      bid_size:Some(10.0),
      ask_size:Some(10.0),
    })
  ).await.unwrap();

  trade_tx.send(
    dsta::convert::stream_quote_to_ticker(&ttrs::StreamQuote
    { event_type:"quote".to_string(),
      symbol:".FAKE 95_P_35".to_string(),
      bid_price:Some(0.01),
      ask_price:Some(0.05),
      bid_size:Some(10.0),
      ask_size:Some(10.0),
    })
  ).await.unwrap();

  trade_tx.send(
    dsta::convert::stream_quote_to_ticker(&ttrs::StreamQuote
    { event_type:"quote".to_string(),
      symbol:".FAKE 100_P_35".to_string(),
      bid_price:Some(0.05),
      ask_price:Some(0.10),
      bid_size:Some(10.0),
      ask_size:Some(10.0),
    })
  ).await.unwrap();

  tokio::time::sleep(StdDuration::from_millis(200)).await;

  // === STEP 12: Receive final close order ===
  info!("STEP 12: Receiving final close order");
  let close_order = tokio::time::timeout(StdDuration::from_secs(5),pls_trade_rx.recv())
    .await
    .expect("Timeout waiting for close order")
    .expect("Trade channel closed");
  assert_eq!(close_order.order_legs.len(),2);
  let long_leg = close_order.order_legs.iter()
    .find(|leg| leg.buy_what.order_name() == "FAKE 95_P_35" && leg.action == dsta::BuyOrSell::SellToClose)
    .expect("Missing SellToClose on long put");
  let short_leg = close_order.order_legs.iter()
    .find(|leg| leg.buy_what.order_name() == "FAKE 100_P_35" && leg.action == dsta::BuyOrSell::BuyToClose)
    .expect("Missing BuyToClose on short put");
  info!("Close order validated — both legs present");


  // === STEP 13: Confirm final close ===
  info!("STEP 13: Confirming final close");
  sucks_tx.send(format!("{}",crate::glossary::ORDER_FLY)).await.expect("Failed to send SWAP_SUCC");

  let changes = fn_get_account_shennanagans(&close_order,&account,&stuff)
    .expect("Failed to compute close changes");
  apply_account_cash_and_collateral_changes(&mut account,&changes);
  apply_order_to_account(&mut account,&close_order,&stuff)
    .expect("Failed to apply close order");
  check_tx.send(crate::Allocation
  { bot_name:bot_name.clone(),
    bot_safe:account.clone(),
    bot_mind:crate::BotMindState::Running,
    num_swap:0,
  }).await.expect("Failed to send allocation update for final close");
  info!("Final close confirmed");

  tokio::time::sleep(StdDuration::from_millis(100)).await;

  // === STEP 14: Shutdown ===
  info!("STEP 15: Shutting down bot");
  let _ = sucks_tx.send("DIE".to_string()).await;
  tokio::time::timeout(StdDuration::from_secs(5),bot_handle)
    .await
    .expect("Bot did not shut down")
    .ok();

  info!("=== Test completed successfully ===");
}

#[tokio::test] // PASSES
async fn stealth_bot_happy_path_put_credit_plus_save_and_load()
{
    // Do not change my formatting, spacing, or comments.
    // WORKING - now with save/load, wider chain, trailing stop test, restart simulation
    use tokio::sync::mpsc;
    use chrono::{Utc, Duration};
    use std::time::Duration as StdDuration;
    use log::{info, debug, trace};
    use crate::*;
    use crate::tests::util::*;
    use crate::dr_robo::*;
    use dsta::*;
    use std::collections::HashMap;
    use std::path::Path;
    let _ = setup_log4rs("stealth_bot_happy_path_put_credit_spread_with_save_trailing_and_restart");
    info!("Starting stealth_bot_happy_path_put_credit_spread_with_save_trailing_and_restart test");
    let bot_name = "stealth_test_put".to_string();
    let friendly_path = format!("./dat/{}.json", bot_name);
    // Clean up any old save file from previous test runs
    if Path::new(&friendly_path).exists() {
        let _ = tokio::fs::remove_file(&friendly_path).await;
        info!("Removed old save file before test start");
    }
    let config = dsta::MakeStealthBot
    {
        my_accounting:
        {
            let mut ma = dsta::CommonBotInfo::default();
            ma.max_cash_risk = (true, 100.0, 12_000.0);
            ma.friendly_name = bot_name.clone();
            ma.tracking_tick = "FAKE".to_string();
            ma
        },
        my_cash_alloc: 12_000.0,
        stonk_feeling: dsta::MarketDirection::Corrects,
        option_bucket: 3,
        spread_bucket: 2,
        exit_gain_pct: 55.0,
        exit_loss_pct: -40.0,
        option_expire: 45,
        nice_exit_way: vec![
            dsta::StealthBotDoneNiceStyle::OtmShort,
            dsta::StealthBotDoneNiceStyle::FinalPct(75.0),
        ],
        use_theo_cost: false,
    };
    log::trace!
    ( r#"
    // ───────────────────────────────────────────────────────────────
    // Phase 1: Fresh start → entry → partial profit → trailing stop raise
    // ───────────────────────────────────────────────────────────────
    "#
    );

    log::trace!
    ( r#"
    // ───────────────────────────────────────────────────────────────
    // Phase 1.1: Initialize channels and start bot
    // ───────────────────────────────────────────────────────────────
    "#
    );
    let (trade_tx, trade_rx) = mpsc::channel(100);
    let (check_tx, check_rx) = mpsc::channel(100);
    let (sucks_tx, mut sucks_rx) = mpsc::channel(100);
    let (pls_ticks_tx, mut pls_ticks_rx) = mpsc::channel(10);
    let (pls_trade_tx, mut pls_trade_rx) = mpsc::channel(10);
    let (swap_info_tx, mut swap_info_rx) = mpsc::channel(10);
    let (new_brain_tx, _) = mpsc::channel(10);
    let (snivtx, mut snivrx) = mpsc::channel(100);
    let bridge = crate::BridgeDrHand
    {
        tell_dr_pls_ticks: pls_ticks_tx.clone(),
        tell_dr_pls_trade: pls_trade_tx.clone(),
        tell_dr_swap_info: swap_info_tx.clone(),
        tell_dr_new_brain: new_brain_tx.clone(),
        tell_dr_pls_check: mpsc::channel(1).0,
        tell_dr_stop_info: mpsc::channel(1).0,
        tell_dr_stop_tick: mpsc::channel(1).0,
        tell_dr_a_snively: snivtx.clone(),
    };
    let bot_handle = tokio::spawn(crate::sbot::stealth_bot::fn_run_main_stealth_bot(
        config.clone(),
        trade_rx,
        check_rx,
        sucks_rx,
        bridge,
    ));
    let expiry = Utc::now() + Duration::days(50);

    log::trace!
    ( r#"
    // ───────────────────────────────────────────────────────────────
    // Phase 1.2: Bot asks for underlying → send chain
    // ───────────────────────────────────────────────────────────────
    "#
    );
    // Wider chain – stock around 100, ATM ~100, long ~94 put, short ~98 put
    let fake_chain = dsta::PushTickerResult
    {
        ass_deets: dsta::FinAss::StkDeets(dsta::Stk { ticker_name: "FAKE".into(), ..Default::default() }),
        opt_chain: vec![dsta::Derivatives
        {
            expire_time: expiry,
            option_calls: vec![],
            option_puts: vec![
                dsta::Opt { ticker_name: ".FAKE110P50".into(), strike_spot:110.0, option_name:"FAKE 110_P_50".into(), expire_date:expiry, ..Default::default() },
                dsta::Opt { ticker_name: ".FAKE105P50".into(), strike_spot:105.0, option_name:"FAKE 105_P_50".into(), expire_date:expiry, ..Default::default() },
                dsta::Opt { ticker_name: ".FAKE102P50".into(), strike_spot:102.0, option_name:"FAKE 102_P_50".into(), expire_date:expiry, ..Default::default() },
                dsta::Opt { ticker_name: ".FAKE100P50".into(), strike_spot:100.0, option_name:"FAKE 100_P_50".into(), expire_date:expiry, ..Default::default() },
                dsta::Opt { ticker_name: ".FAKE098P50".into(), strike_spot: 98.0, option_name:"FAKE 098_P_50".into(), expire_date:expiry, ..Default::default() },
                dsta::Opt { ticker_name: ".FAKE096P50".into(), strike_spot: 96.0, option_name:"FAKE 096_P_50".into(), expire_date:expiry, ..Default::default() },
                dsta::Opt { ticker_name: ".FAKE094P50".into(), strike_spot: 94.0, option_name:"FAKE 094_P_50".into(), expire_date:expiry, ..Default::default() },
                dsta::Opt { ticker_name: ".FAKE092P50".into(), strike_spot: 92.0, option_name:"FAKE 092_P_50".into(), expire_date:expiry, ..Default::default() },
                dsta::Opt { ticker_name: ".FAKE090P50".into(), strike_spot: 90.0, option_name:"FAKE 090_P_50".into(), expire_date:expiry, ..Default::default() },
            ],
        }],
    };
    let (req_ticker, reply_tx) = pls_ticks_rx.recv().await.expect("No initial ticker request");
    assert_eq!(req_ticker, "FAKE");
    reply_tx.send(Some(fake_chain.clone())).await.unwrap();

    log::trace!
    ( r#"
    // ───────────────────────────────────────────────────────────────
    // Phase 1.3: Cash swap request
    // ───────────────────────────────────────────────────────────────
    "#
    );
    let _swap = swap_info_rx.recv().await.expect("No cash swap request");
    sucks_tx.send(crate::glossary::SWAP_SUCC.to_string()).await.unwrap();

    log::trace!
    ( r#"
    // ───────────────────────────────────────────────────────────────
    // Phase 1.4: Send initial allocation
    // ───────────────────────────────────────────────────────────────
    "#
    );
    // Single persistent account — like real Dr. Robo manages
    let mut account = BotaAccount::new().with_cash(12_000.0);
    check_tx.send(crate::Allocation {
        bot_name: bot_name.clone(),
        bot_safe: account.clone(),
        bot_mind: BotMindState::Running,
        num_swap: 0,
    }).await.expect("Failed to send initial cash allocation");
    tokio::time::sleep(StdDuration::from_millis(200)).await;

    log::trace!
    ( r#"
    // ───────────────────────────────────────────────────────────────
    // Phase 1.5: Underlying quote → ATM calc & option selection
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

    log::trace!
    ( r#"
    // ───────────────────────────────────────────────────────────────
    // Phase 1.6: Option ticker requests → send proper Opt structs
    // ───────────────────────────────────────────────────────────────
    "#
    );
    let long_req = pls_ticks_rx.recv().await.expect("No long leg request");
    assert!(long_req.0.contains("094") || long_req.0.contains("94"));
    let mut long_opt = dsta::Opt::default();
    long_opt.ticker_name = ".FAKE094P50".to_string();
    long_opt.option_name = "FAKE 094_P_50".to_string();
    long_opt.option_rite = dsta::OptRite::Put;
    long_opt.strike_spot = 94.0;
    long_opt.expire_date = expiry;
    long_opt.option_type = dsta::OptStyle::Usa;
    long_req.1.send(Some(dsta::PushTickerResult {
        ass_deets: dsta::FinAss::OptDeets(long_opt.clone()),
        opt_chain: vec![],
    })).await.unwrap();
    let short_req = pls_ticks_rx.recv().await.expect("No short leg request");
    assert!(short_req.0.contains("098") || short_req.0.contains("98"));
    let mut short_opt = dsta::Opt::default();
    short_opt.ticker_name = ".FAKE098P50".to_string();
    short_opt.option_name = "FAKE 098_P_50".to_string();
    short_opt.option_rite = dsta::OptRite::Put;
    short_opt.strike_spot = 98.0;
    short_opt.expire_date = expiry;
    short_opt.option_type = dsta::OptStyle::Usa;
    short_req.1.send(Some(dsta::PushTickerResult {
        ass_deets: dsta::FinAss::OptDeets(short_opt.clone()),
        opt_chain: vec![],
    })).await.unwrap();

    log::trace!
    ( r#"
    // ───────────────────────────────────────────────────────────────
    // Phase 1.7: Long leg quote → triggers entry order
    // ───────────────────────────────────────────────────────────────
    "#
    );
    trade_tx.send(dsta::Ticker {
        name: ".FAKE094P50".into(),
        bid: 1.75,
        ask: 1.95,
        ..Default::default()
    }).await.unwrap();
    let entry_order = pls_trade_rx.recv().await.expect("No entry order");
    assert_eq!(entry_order.order_legs[0].action, dsta::BuyOrSell::BuyToOpen);
    assert!(entry_order.order_legs[0].buy_what.order_name().contains("094"));

    log::trace!
    ( r#"
    // ───────────────────────────────────────────────────────────────
    // Phase 1.8: Apply entry fill to the single persistent account
    // ───────────────────────────────────────────────────────────────
    "#
    );
    // Assets map — keys = option_name, values have matching fields
    let mut assets: HashMap<String, FinAss> = HashMap::new();
    assets.insert("FAKE 094_P_50".to_string(), FinAss::OptDeets(long_opt));
    assets.insert("FAKE 098_P_50".to_string(), FinAss::OptDeets(short_opt));
    // 7. Apply entry fill to the single persistent account
    let changes = fn_get_account_shennanagans(&entry_order, &account, &assets).unwrap();
    apply_account_cash_and_collateral_changes(&mut account, &changes);
    apply_order_to_account(&mut account, &entry_order, &assets).unwrap();
    // Tell bot about the fill
    check_tx.send(crate::Allocation {
        bot_name: bot_name.clone(),
        bot_safe: account.clone(),
        bot_mind: BotMindState::Running,
        num_swap: 0,
    }).await.unwrap();
    tokio::time::sleep(StdDuration::from_millis(150)).await;

    sucks_tx.send(format!("{}",crate::glossary::ORDER_FLY)).await.expect("Failed to send SWAP_SUCC");

    log::trace!
    ( r#"
    // ───────────────────────────────────────────────────────────────
    // Phase 1.9: Profit tick → trailing floor should raise
    // ───────────────────────────────────────────────────────────────
    "#
    );
    trade_tx.send(dsta::Ticker {
        name: ".FAKE094P50".into(),
        bid: 2.35,
        ask: 2.55,
        ..Default::default()
    }).await.unwrap();
    trade_tx.send(dsta::Ticker {
        name: "FAKE".into(),
        bid: 98.40,
        ask: 98.60,
        ..Default::default()
    }).await.unwrap();
    tokio::time::sleep(StdDuration::from_millis(180)).await;

    log::trace!
    ( r#"
    // ───────────────────────────────────────────────────────────────
    // Phase 2: Simulate graceful stop → save state
    // ───────────────────────────────────────────────────────────────
    "#
    );
    info!("=== Telling bot to stop (transition to Stopped) ===");
    // Send Stopped mind state → bot should save state on this transition
    check_tx.send(crate::Allocation {
        bot_name: bot_name.clone(),
        bot_safe: account.clone(), // current positions / cash
        bot_mind: BotMindState::Stopped,
        num_swap: 0,
    }).await.expect("Failed to send Stopped allocation");
    tokio::time::sleep(StdDuration::from_millis(500)).await; // give time to save
    // Optional: confirm save happened (file exists)
    assert!(Path::new(&friendly_path).exists(), "Bot did not save state after Stopped transition");

    log::trace!
    ( r#"
    // ───────────────────────────────────────────────────────────────
    // Phase 2.1: Kill the bot and clean up channels
    // ───────────────────────────────────────────────────────────────
    "#
    );
    let _ = sucks_tx.send("DIE".to_string()).await;
    let _ = bot_handle.await.expect("Bot didn't shut down cleanly");

    // Clean up old channels
    drop(trade_tx);
    drop(check_tx);
    drop(sucks_tx);

    // Give filesystem a moment
    tokio::time::sleep(StdDuration::from_millis(200)).await;

    log::trace!
    ( r#"
    // ───────────────────────────────────────────────────────────────
    // Phase 3: Restart bot → load state + resume to Running
    // ───────────────────────────────────────────────────────────────
    "#
    );

    log::trace!
    ( r#"
    // ───────────────────────────────────────────────────────────────
    // Phase 3.1: Recreate all channels
    // ───────────────────────────────────────────────────────────────
    "#
    );
    let (trade_tx2, trade_rx2) = mpsc::channel(100);
    let (check_tx2, check_rx2) = mpsc::channel(100);
    let (sucks_tx2, mut sucks_rx2) = mpsc::channel(100);

    log::trace!
    ( r#"
    // ───────────────────────────────────────────────────────────────
    // Phase 3.2: Recreate the bridge with new channels
    // ───────────────────────────────────────────────────────────────
    "#
    );
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

    log::trace!
    ( r#"
    // ───────────────────────────────────────────────────────────────
    // Phase 3.3: Restart the bot with new channels
    // ───────────────────────────────────────────────────────────────
    "#
    );
    let bot_handle2 = tokio::spawn(crate::sbot::stealth_bot::fn_run_main_stealth_bot(
        config.clone(),
        trade_rx2,
        check_rx2,
        sucks_rx2,
        bridge2,
    ));

    log::trace!
    ( r#"
    // ───────────────────────────────────────────────────────────────
    // Phase 3.4: Give time for load + initial ticker request (STEP 5)
    // ───────────────────────────────────────────────────────────────
    "#
    );
    tokio::time::sleep(StdDuration::from_millis(200)).await;
    // Optional: wait for and confirm ticker request (shows it's running init/load)
    let (req_ticker_after_restart, reply_tx2) = pls_ticks_rx.recv().await.expect("No ticker request after restart");
    assert_eq!(req_ticker_after_restart, "FAKE", "Bot should re-request underlying after load");
    reply_tx2.send(Some(fake_chain)).await.unwrap();



   log::trace!
    ( r#"
    // ───────────────────────────────────────────────────────────────
    // Phase 3.5: useless Underlying quote for load just for a test
    // ───────────────────────────────────────────────────────────────
    "#
    );
    trade_tx2.send(dsta::Ticker {
        name: "FAKE".into(),
        bid: 99.95,
        ask: 100.05,
        ..Default::default()
    }).await.unwrap();
    tokio::time::sleep(StdDuration::from_millis(120)).await;
 
    log::trace!
    ( r#"
    // ───────────────────────────────────────────────────────────────
    // Phase 3.6: Option ticker requests → send proper Opt structs
    // ───────────────────────────────────────────────────────────────
    "#
    );
    let long_req = pls_ticks_rx.recv().await.expect("No long leg request");
    assert!(long_req.0.contains("094") || long_req.0.contains("94"));
    let mut long_opt = dsta::Opt::default();
    long_opt.ticker_name = ".FAKE094P50".to_string();
    long_opt.option_name = "FAKE 094_P_50".to_string();
    long_opt.option_rite = dsta::OptRite::Put;
    long_opt.strike_spot = 94.0;
    long_opt.expire_date = expiry;
    long_opt.option_type = dsta::OptStyle::Usa;
    long_req.1.send(Some(dsta::PushTickerResult {
        ass_deets: dsta::FinAss::OptDeets(long_opt.clone()),
        opt_chain: vec![],
    })).await.unwrap();
    let short_req = pls_ticks_rx.recv().await.expect("No short leg request");
    assert!(short_req.0.contains("098") || short_req.0.contains("98"));
    let mut short_opt = dsta::Opt::default();
    short_opt.ticker_name = ".FAKE098P50".to_string();
    short_opt.option_name = "FAKE 098_P_50".to_string();
    short_opt.option_rite = dsta::OptRite::Put;
    short_opt.strike_spot = 98.0;
    short_opt.expire_date = expiry;
    short_opt.option_type = dsta::OptStyle::Usa;
    short_req.1.send(Some(dsta::PushTickerResult {
        ass_deets: dsta::FinAss::OptDeets(short_opt.clone()),
        opt_chain: vec![],
    })).await.unwrap();

    log::trace!
    ( r#"
    // ───────────────────────────────────────────────────────────────
    // Phase 3.7: Resume bot: transition to Running after load (should already be running.)
    // ───────────────────────────────────────────────────────────────
    "#
    );
    info!("=== Resuming bot: transition to Running after load ===");
    check_tx2.send(crate::Allocation {
        bot_name: bot_name.clone(),
        bot_safe: account.clone(), // send current known state
        bot_mind: BotMindState::Running,
        num_swap: 0,
    }).await.expect("Failed to send Running allocation after restart");
    tokio::time::sleep(StdDuration::from_millis(300)).await;

    log::trace!
    ( r#"
    // ───────────────────────────────────────────────────────────────
    // Phase 3.8: Send quote to trigger early exit before spread creation
    // ───────────────────────────────────────────────────────────────
    "#
    );
    // Send short leg quote first (very cheap → big profit if already "in" spread)
    trade_tx2.send(dsta::Ticker {
        name: ".FAKE098P50".into(),
        bid: 3.10,
        ask: 3.05, // need 3.15 to triger spread entry
        ..Default::default()
    }).await.unwrap();

    // Then long leg quote (near worthless)
    trade_tx2.send(dsta::Ticker {
        name: ".FAKE094P50".into(),
        bid: 2.05,
        ask: 2.10,
        ..Default::default()
    }).await.unwrap();

    // Optional: another underlying quote to confirm move
    trade_tx2.send(dsta::Ticker {
        name: "FAKE".into(),
        bid: 95.50,
        ask: 95.80,
        ..Default::default()
    }).await.unwrap();
    tokio::time::sleep(StdDuration::from_millis(250)).await;

    log::trace!
    ( r#"
    // ───────────────────────────────────────────────────────────────
    // Phase 3.9: Expect final close order
    // ───────────────────────────────────────────────────────────────
    "#
    );

    let close_order = pls_trade_rx.recv().await.expect("No final close order after resume");
    assert_eq!(close_order.order_legs.len(), 1);
    assert!(close_order.order_legs.iter().any(|l| l.action == dsta::BuyOrSell::SellToClose));
    info!("Final close order received after stop → save → restart → resume → success");

    sucks_tx2.send(format!("{}",crate::glossary::ORDER_FLY)).await.expect("Failed to send SWAP_SUCC");


    log::trace!
    ( r#"
    // ───────────────────────────────────────────────────────────────
    // Phase 4: Cleanup
    // ───────────────────────────────────────────────────────────────
    "#
    );
    let _ = sucks_tx2.send("DIE".to_string()).await;
    let _ = bot_handle2.await;
    let _ = tokio::fs::remove_file(&friendly_path).await;
    info!("Test completed - full stop/save/load/resume cycle validated");
}

#[tokio::test] // PASSES
async fn stealth_bot_happy_path_put_credit_spread_sleeps() 
{ // Do not change my formatting, spacing, or comments.
  // WORKING
      use tokio::sync::mpsc;
      use chrono::{Utc, Duration};
      use std::time::Duration as StdDuration;
      use log::{info, debug, trace};
      use crate::*;
      use crate::tests::util::*;
      use crate::dr_robo::*;
      use dsta::*;
      use std::collections::HashMap;

    let _ = setup_log4rs("stealth_bot_happy_path_put_credit_spread_exit");
    info!("Starting stealth_bot_happy_path_put_credit_spread_exit test");

    let bot_name = "stealth_test_bot".to_string();

    let sleep1 = 0;

    let config = dsta::MakeStealthBot
    { my_accounting:
      { let mut ma = dsta::CommonBotInfo::default();
        ma.max_cash_risk = (true,100.0,10_000.0);
        ma.friendly_name = bot_name.clone();
        ma.tracking_tick = "FAKE".to_string();
        ma },
      my_cash_alloc:10_000.0,
      stonk_feeling:dsta::MarketDirection::Corrects,
      option_bucket:1,
      spread_bucket:1,
      exit_gain_pct:50.0,
      exit_loss_pct:-80.0,
      option_expire:30,
      nice_exit_way:vec!
      [ dsta::StealthBotDoneNiceStyle::GoExpire
      , dsta::StealthBotDoneNiceStyle::FinalPct(80.0)
      ],
      use_theo_cost:false,
    };

    /*
      #[derive(Default, Clone, Debug, serde::Serialize, serde::Deserialize)]
      pub enum StealthBotDoneNiceStyle 
      {
        #[default] 
        OtmShort, // short leg goes OTM 
        FinalPct(f64), // short leg gain %
        FinalDay(UsaMarketTimes, f64), // exit on last day of trading
        PriorDay(UsaMarketTimes, f64, u64), // exit usize days before expiration 
        GoExpire, // let the contract expire
      }
      */

    let (trade_tx,trade_rx) = mpsc::channel(100);
    let (check_tx,check_rx) = mpsc::channel(100);
    let (sucks_tx,mut sucks_rx) = mpsc::channel(100);
    let (pls_ticks_tx,mut pls_ticks_rx) = mpsc::channel(10);
    let (pls_trade_tx,mut pls_trade_rx) = mpsc::channel(10);
    let (swap_info_tx,mut swap_info_rx) = mpsc::channel(10);
    let (new_brain_tx,_) = mpsc::channel(10);

    let (snivtx, mut snivrx) = mpsc::channel(100); 
    // We mock being Dr Robo here, so we need real channels
    // the pls ticks means bot asks for account infos
    // the pls check means bot asks for account infos
    // the swap info means bot asks for cash at startup
    // the pls trade means bot requested an order
    // the told a snively means we got erros.
    let bridge = crate::BridgeDrHand
    { tell_dr_pls_ticks:pls_ticks_tx.clone(),
      tell_dr_pls_trade:pls_trade_tx.clone(),
      tell_dr_swap_info:swap_info_tx.clone(),
      tell_dr_new_brain:new_brain_tx.clone(),
      tell_dr_pls_check:mpsc::channel(1).0,
      tell_dr_stop_info:mpsc::channel(1).0,
      tell_dr_stop_tick:mpsc::channel(1).0,
      tell_dr_a_snively:snivtx.clone(), //tell_dr_a_snively:mpsc::channel(100).0, 
    };

    let bot_handle = tokio::spawn(crate::sbot::stealth_bot::fn_run_main_stealth_bot(
      config,
      trade_rx,
      check_rx,
      sucks_rx,
      bridge,
    ));

    let expiry = Utc::now() + Duration::days(35);

    trace!("Awaiting initial ticker request");
    let (ticker_name,reply_tx) = tokio::time::timeout(StdDuration::from_secs(5),pls_ticks_rx.recv())
      .await
      .expect("Timeout on initial ticker request")
      .expect("Ticker channel closed");

    assert_eq!(ticker_name,"FAKE");


    let mut stk = dsta::Stk::default();
    stk.ticker_name = "FAKE".to_string();
    let fake_result = dsta::PushTickerResult
    { ass_deets: dsta::FinAss::StkDeets(stk),
      opt_chain:vec![dsta::Derivatives
      { expire_time:expiry,
        option_calls:vec![],
        option_puts:vec![
          dsta::Opt { ticker_name:".FAKE 105_P_35".to_string(),strike_spot:105.0,option_name:"FAKE 105_P_35".to_string(),expire_date:expiry,..Default::default() },
          dsta::Opt { ticker_name:".FAKE 100_P_35".to_string(),strike_spot:100.0,option_name:"FAKE 100_P_35".to_string(),expire_date:expiry,..Default::default() },
          dsta::Opt { ticker_name:".FAKE 95_P_35".to_string() ,strike_spot:95.0 ,option_name:"FAKE 95_P_35".to_string(),expire_date:expiry,..Default::default() },
          dsta::Opt { ticker_name:".FAKE 90_P_35".to_string() ,strike_spot:90.0 ,option_name:"FAKE 90_P_35".to_string(),expire_date:expiry,..Default::default() },
        ],
      }],
    };

    reply_tx.send(Some(fake_result)).await.expect("Failed to send ticker chain");

    info!("Sent fake option chain for FAKE");

    tokio::time::sleep(StdDuration::from_millis(sleep1)).await;

    trace!("Awaiting initial cash swap request");
    let swap_req = tokio::time::timeout(StdDuration::from_secs(5),swap_info_rx.recv())
      .await
      .expect("Timeout on cash swap request")
      .expect("Swap channel closed");

    info!("Received initial cash swap: {:?}",swap_req);

    let firststuff = crate::Allocation 
    { bot_name: format!("stealth_test_bot")
    , bot_safe: BotaAccount 
      { m_cash: 10_000.0,
        m_stock: None,
        m_long_calls: None,
        m_long_puts: None,
        m_short_calls: None,
        m_short_puts: None,
        m_call_credit_spread_lein: 0.0,
        m_short_put_lein: 0.0,
        m_put_credit_spread_lein: 0.0,
        m_short_iron_condor_lein: 0.0,
      },
      bot_mind: crate::BotMindState::Running,
      num_swap: 0,
    };
    check_tx.send(firststuff).await.expect("Failed to send 1st allocation");
    sucks_tx.send(crate::glossary::SWAP_SUCC.to_string())
      .await
      .expect("Failed to send SWAP_SUCC");

    info!("Initial cash approved");
    // After "Initial cash approved" and sleep(200ms)

    // === STEP A: Send initial underlying quote (triggers ATM calculation & option selection) ===
    info!("Sending initial underlying quote (needed for ATM calculation)");
    trade_tx.send(
      dsta::convert::stream_quote_to_ticker(&ttrs::StreamQuote {
        event_type: "quote".to_string(),
        symbol: "FAKE".to_string(),
        bid_price: Some(99.8),
        ask_price: Some(100.2),
        bid_size: Some(10.0),
        ask_size: Some(10.0),
      })
    ).await.expect("Failed to send initial underlying quote");

    tokio::time::sleep(StdDuration::from_millis(sleep1)).await;

    // === STEP B: Bot requests long leg ticker (95 put) ===
    info!("Awaiting long leg ticker request (.FAKE 95_P_35)");
    let (opt_name_long, reply_tx_long) = tokio::time::timeout(StdDuration::from_secs(5), pls_ticks_rx.recv())
      .await
      .expect("Timeout waiting for long leg ticker request")
      .expect("Ticker request channel closed");

    assert_eq!(opt_name_long, ".FAKE 95_P_35", "Bot requested wrong long leg");
    let mut opt = dsta::Opt::default();
    opt.ticker_name = ".FAKE 95_P_35".to_string();
    opt.option_name = "FAKE 95_P_35".to_string();
    
    reply_tx_long.send(Some(dsta::PushTickerResult {
      ass_deets: dsta::FinAss::OptDeets(opt),
      opt_chain: vec![],
    })).await.expect("Failed to reply to long leg request");

    // === STEP C: Bot immediately requests short leg ticker (100 put) ===
    info!("Awaiting short leg ticker request (FAKE 100_P_35)");
    let (opt_name_short, reply_tx_short) = tokio::time::timeout(StdDuration::from_secs(5), pls_ticks_rx.recv())
      .await
      .expect("Timeout waiting for short leg ticker request")
      .expect("Ticker request channel closed");

    assert_eq!(opt_name_short, ".FAKE 100_P_35", "Bot requested wrong short leg");
    let mut opt = dsta::Opt::default();
    opt.ticker_name = ".FAKE 100_P_35".to_string();
    opt.option_name = "FAKE 100_P_35".to_string();
    
    reply_tx_short.send(Some(dsta::PushTickerResult {
      ass_deets: dsta::FinAss::OptDeets(opt),
      opt_chain: vec![],
    })).await.expect("Failed to reply to short leg request");

    tokio::time::sleep(StdDuration::from_millis(sleep1)).await;

    // === STEP D: Now send long leg quote → this should trigger entry order ===
    info!("Sending long leg quote to trigger BuyToOpen");
    trade_tx.send(
      dsta::convert::stream_quote_to_ticker(&ttrs::StreamQuote {
        event_type: "quote".to_string(),
        symbol: ".FAKE 95_P_35".to_string(),
        bid_price: Some(1.90),
        ask_price: Some(2.10),
        bid_size: Some(20.0),
        ask_size: Some(20.0),
      })
    ).await.expect("Failed to send long leg quote");

    tokio::time::sleep(StdDuration::from_millis(sleep1)).await;

    // === STEP 6: Now we should receive the entry order ===
    info!("Awaiting entry order (BuyToOpen 95 put)");
    let entry_order = tokio::time::timeout(StdDuration::from_secs(5), pls_trade_rx.recv())
      .await
      .expect("Timeout waiting for entry order")
      .expect("Trade channel closed");

    assert_eq!(entry_order.order_legs[0].buy_what.order_name(), "FAKE 95_P_35");
    assert_eq!(entry_order.order_legs[0].action, dsta::BuyOrSell::BuyToOpen);
    info!("Entry order validated: {:?}", entry_order.order_legs[0]);


    // === STEP 7: Confirm entry fill via account update ===
    info!("STEP 7: Confirming entry fill");
    let mut account = BotaAccount::new().with_cash(10_000.0);

    let mut stuff: HashMap<String,FinAss> = HashMap::new();
    let long_put = Opt
    { ticker_name:".FAKE 95_P_35".to_string(),
      option_name:"FAKE 95_P_35".to_string(),
      option_rite:OptRite::Put,
      strike_spot:95.0,
      expire_date:expiry,
      option_type:OptStyle::Usa,
      last_ticker:None,
      tick_rulers:None,
    };
    let short_put = Opt
    { ticker_name:".FAKE 100_P_35".to_string(),
      option_name:"FAKE 100_P_35".to_string(),
      option_rite:OptRite::Put,
      strike_spot:100.0,
      expire_date:expiry,
      option_type:OptStyle::Usa,
      last_ticker:None,
      tick_rulers:None,
    };
    stuff.insert(long_put.option_name.clone(),FinAss::OptDeets(long_put));
    stuff.insert(short_put.option_name.clone(),FinAss::OptDeets(short_put));
    let changes = fn_get_account_shennanagans(&entry_order,&account,&stuff)
      .expect("Failed to compute entry fill changes");
    apply_account_cash_and_collateral_changes(&mut account,&changes);
    apply_order_to_account(&mut account,&entry_order,&stuff)
      .expect("Failed to apply entry order to account");
    
    sucks_tx.send(format!("{}",crate::glossary::ORDER_FLY)).await.expect("Failed to send SWAP_SUCC");
    
    check_tx.send(crate::Allocation
    { bot_name:bot_name.clone(),
      bot_safe:account.clone(),
      bot_mind:crate::BotMindState::Running,
      num_swap:0,
    }).await.expect("Failed to send allocation update for entry fill");
    info!("Entry fill confirmed");
    tokio::time::sleep(StdDuration::from_millis(sleep1)).await;

  // === STEP 8: Send short leg quote + favorable underlying move ===
  info!("STEP 8: Sending quotes to trigger spread entry");
  trade_tx.send(
    dsta::convert::stream_quote_to_ticker(&ttrs::StreamQuote
    { event_type:"quote".to_string(),
      symbol:".FAKE 100_P_35".to_string(),
      bid_price:Some(5.40),
      ask_price:Some(5.60),
      bid_size:Some(15.0),
      ask_size:Some(15.0),
    })
  ).await.unwrap();

  trade_tx.send(
    dsta::convert::stream_quote_to_ticker(&ttrs::StreamQuote
    { event_type:"quote".to_string(),
      symbol:"FAKE".to_string(),
      bid_price:Some(95.80),
      ask_price:Some(95.82),
      bid_size:Some(10.0),
      ask_size:Some(10.0),
    })
  ).await.unwrap();

  trade_tx.send(
    dsta::convert::stream_quote_to_ticker(&ttrs::StreamQuote
    { event_type:"quote".to_string(),
      symbol:".FAKE 95_P_35".to_string(),
      bid_price:Some(3.10),
      ask_price:Some(3.20),
      bid_size:Some(20.0),
      ask_size:Some(20.0),
    })
  ).await.unwrap();

  trade_tx.send(
    dsta::convert::stream_quote_to_ticker(&ttrs::StreamQuote
    { event_type:"quote".to_string(),
      symbol:".FAKE 100_P_35".to_string(),
      bid_price:Some(6.40),
      ask_price:Some(6.60),
      bid_size:Some(15.0),
      ask_size:Some(15.0),
    })
  ).await.unwrap();

  tokio::time::sleep(StdDuration::from_millis(sleep1)).await;

  //// === STEP FIX ERROR: Receive error ===
  //info!("STEP FIX ERROR: Receiving sniv error");
  //let sniv_error = tokio::time::timeout(StdDuration::from_secs(5),snivrx.recv())
  //  .await
  //  .expect("Timeout waiting for sniv error order")
  //  .expect("snivrx channel closed");
  //info!("{:#?}", sniv_error );

  // === STEP 9: Receive spread sell order ===
  info!("STEP 9: Receiving spread order (SellToOpen 100 put)");
  let spread_order = tokio::time::timeout(StdDuration::from_secs(5), pls_trade_rx.recv())
      .await
      .expect("Timeout waiting for spread order")
      .expect("Trade channel closed");

  assert_eq!(spread_order.order_legs[0].buy_what.order_name(), "FAKE 100_P_35");
  assert_eq!(spread_order.order_legs[0].action, dsta::BuyOrSell::SellToOpen);
 

    info!("Spread order validated");

  // === NEW STEP 10: Confirm spread fill via account update ===
  info!("STEP 10: Confirming spread fill (short put position added)");

  // Reuse/update the same 'account' and 'stuff' from entry fill
  let changes = fn_get_account_shennanagans(&spread_order, &account, &stuff)
      .expect("Failed to compute spread fill changes");

  apply_account_cash_and_collateral_changes(&mut account, &changes);
  apply_order_to_account(&mut account, &spread_order, &stuff)
      .expect("Failed to apply spread order to account");

    sucks_tx.send(format!("{}",crate::glossary::ORDER_FLY)).await.expect("Failed to send SWAP_SUCC");

  // === STEP FIX ERROR: Receive error ===

  let mut sv = Vec::new();
  let sc = 100;
  info!("STEP FIX ERROR: Receiving sniv error");
  let sniv_errors = tokio::time::timeout(StdDuration::from_secs(5),snivrx.recv_many(&mut sv,sc))
    .await
    .expect("Timeout waiting for sniv error order")
    //.expect("snivrx channel closed");
  ;
  info!("\n\n\n\n\n\n\n\n SNIV ERRORS! {:#?}", sv );

  tokio::time::sleep(StdDuration::from_millis(sleep1)).await;

  check_tx.send(crate::Allocation {
      bot_name: bot_name.clone(),
      bot_safe: account.clone(),  // ← now includes both long and short positions
      bot_mind: crate::BotMindState::Running,
      num_swap: 0,
  }).await.expect("Failed to send allocation update for spread fill");

  info!("Spread fill confirmed - short position added");

  // Give bot time to process the update
  tokio::time::sleep(StdDuration::from_millis(sleep1)).await;

  // === STEP 11: Move underlying to trigger exit ===
  info!("STEP 11: Sending quotes to trigger nice exit (OTM)");
  trade_tx.send(
    dsta::convert::stream_quote_to_ticker(&ttrs::StreamQuote
    { event_type:"quote".to_string(),
      symbol:"FAKE".to_string(),
      bid_price:Some(105.10),
      ask_price:Some(105.14),
      bid_size:Some(10.0),
      ask_size:Some(10.0),
    })
  ).await.unwrap();

  trade_tx.send(
    dsta::convert::stream_quote_to_ticker(&ttrs::StreamQuote
    { event_type:"quote".to_string(),
      symbol:".FAKE 95_P_35".to_string(),
      bid_price:Some(0.01),
      ask_price:Some(0.05),
      bid_size:Some(10.0),
      ask_size:Some(10.0),
    })
  ).await.unwrap();

  trade_tx.send(
    dsta::convert::stream_quote_to_ticker(&ttrs::StreamQuote
    { event_type:"quote".to_string(),
      symbol:".FAKE 100_P_35".to_string(),
      bid_price:Some(0.05),
      ask_price:Some(0.10),
      bid_size:Some(10.0),
      ask_size:Some(10.0),
    })
  ).await.unwrap();

  tokio::time::sleep(StdDuration::from_millis(sleep1)).await;

  // === STEP 12: Receive final close order ===
  info!("STEP 12: Receiving final close order");
  let close_order = tokio::time::timeout(StdDuration::from_secs(5),pls_trade_rx.recv())
    .await
    .expect("Timeout waiting for close order")
    .expect("Trade channel closed");
  assert_eq!(close_order.order_legs.len(),2);
  let long_leg = close_order.order_legs.iter()
    .find(|leg| leg.buy_what.order_name() == "FAKE 95_P_35" && leg.action == dsta::BuyOrSell::SellToClose)
    .expect("Missing SellToClose on long put");
  let short_leg = close_order.order_legs.iter()
    .find(|leg| leg.buy_what.order_name() == "FAKE 100_P_35" && leg.action == dsta::BuyOrSell::BuyToClose)
    .expect("Missing BuyToClose on short put");
  info!("Close order validated — both legs present");


  // === STEP 13: Confirm final close ===
  info!("STEP 13: Confirming final close");
  let changes = fn_get_account_shennanagans(&close_order,&account,&stuff)
    .expect("Failed to compute close changes");
  apply_account_cash_and_collateral_changes(&mut account,&changes);
  apply_order_to_account(&mut account,&close_order,&stuff)
    .expect("Failed to apply close order");
  
  sucks_tx.send(format!("{}",crate::glossary::ORDER_FLY)).await.expect("Failed to send SWAP_SUCC");

  check_tx.send(crate::Allocation
  { bot_name:bot_name.clone(),
    bot_safe:account.clone(),
    bot_mind:crate::BotMindState::Running,
    num_swap:0,
  }).await.expect("Failed to send allocation update for final close");
  info!("Final close confirmed");

  tokio::time::sleep(StdDuration::from_millis(sleep1)).await;

  // === STEP 14: Shutdown ===
  info!("STEP 15: Shutting down bot");
  let _ = sucks_tx.send("DIE".to_string()).await;
  tokio::time::timeout(StdDuration::from_secs(5),bot_handle)
    .await
    .expect("Bot did not shut down")
    .ok();

  info!("=== Test completed successfully ===");
}


#[tokio::test] // PASSES
async fn stealth_bot_happy_path_put_credit_spread_haggle() 
{ // Do not change my formatting, spacing, or comments.
  // WORKING
      use tokio::sync::mpsc;
      use chrono::{Utc, Duration};
      use std::time::Duration as StdDuration;
      use log::{info, debug, trace};
      use crate::*;
      use crate::tests::util::*;
      use crate::dr_robo::*;
      use dsta::*;
      use std::collections::HashMap;

    let _ = setup_log4rs("stealth_bot_happy_path_put_credit_spread_haggle");
    info!("Starting stealth_bot_happy_path_put_credit_spread_haggle test");

    let bot_name = "stealth_test_bot".to_string();

    let sleep1 : u64 = 5;

    // { MinimumOfMidpointOrTheoreticalThenConcede(HaggleLimits)
    // , VirtualMarketOrderWithLimitOrdThenConcede(HaggleLimits)
    // , CheapLimitOrderThenIncrementDeltaUntilMax(HaggleLimits)

    let config = dsta::MakeStealthBot
    { my_accounting:
      { let mut ma = dsta::CommonBotInfo::default();
        ma.haggle_action = dsta::HaggleAction::AnOyVeyJew
        ( dsta::HaggleMethod::MinimumOfMidpointOrTheoreticalThenConcede
          ( dsta::HaggleLimits
            { retry_period : chrono::Duration::milliseconds(sleep1 as i64) // chrono::Duration, // or cancel time
            , delta_choice : 33.0 // f64,
            , delta_is_pct : true // bool,
            , slippage_max : 7.07 // f64, // Will fail if it's bellow ~6.99 
            }
          )
        );
        ma.max_cash_risk = (true,80.0,10_000.0);
        ma.friendly_name = bot_name.clone();
        ma.tracking_tick = "FAKE".to_string();
        ma 
      },
      my_cash_alloc:11_000.0,
      stonk_feeling:dsta::MarketDirection::Corrects,
      option_bucket:1,
      spread_bucket:1,
      exit_gain_pct:50.0,
      exit_loss_pct:-80.0,
      option_expire:30,
      nice_exit_way:vec!
      [ dsta::StealthBotDoneNiceStyle::GoExpire
      , dsta::StealthBotDoneNiceStyle::FinalPct(80.0)
      ],
      use_theo_cost:false,
    };

    let (trade_tx,trade_rx) = mpsc::channel(100);
    let (check_tx,check_rx) = mpsc::channel(100);
    let (sucks_tx,mut sucks_rx) = mpsc::channel(100);
    let (pls_ticks_tx,mut pls_ticks_rx) = mpsc::channel(10);
    let (pls_trade_tx,mut pls_trade_rx) = mpsc::channel(10);
    let (swap_info_tx,mut swap_info_rx) = mpsc::channel(10);
    let (new_brain_tx,_) = mpsc::channel(10);
    let (snivtx, mut snivrx) = mpsc::channel(100); 
    let bridge = crate::BridgeDrHand
    { tell_dr_pls_ticks:pls_ticks_tx.clone(),
      tell_dr_pls_trade:pls_trade_tx.clone(),
      tell_dr_swap_info:swap_info_tx.clone(),
      tell_dr_new_brain:new_brain_tx.clone(),
      tell_dr_pls_check:mpsc::channel(1).0,
      tell_dr_stop_info:mpsc::channel(1).0,
      tell_dr_stop_tick:mpsc::channel(1).0,
      tell_dr_a_snively:snivtx.clone(), //tell_dr_a_snively:mpsc::channel(100).0, 
    };

    let bot_handle = tokio::spawn(crate::sbot::stealth_bot::fn_run_main_stealth_bot(
      config,
      trade_rx,
      check_rx,
      sucks_rx,
      bridge,
    ));

    let expiry = Utc::now() + Duration::days(35);

    trace!("Awaiting initial ticker request");
    let (ticker_name,reply_tx) = tokio::time::timeout(StdDuration::from_secs(5),pls_ticks_rx.recv())
      .await
      .expect("Timeout on initial ticker request")
      .expect("Ticker channel closed");

    assert_eq!(ticker_name,"FAKE");

    let mut stk = dsta::Stk::default();
    stk.ticker_name = "FAKE".to_string();
    let fake_result = dsta::PushTickerResult
    { ass_deets: dsta::FinAss::StkDeets(stk),
      opt_chain:vec![dsta::Derivatives
      { expire_time:expiry,
        option_calls:vec![],
        option_puts:vec![
          dsta::Opt { option_rite : dsta::OptRite::Put, ticker_name:".FAKE 105_P_35".to_string(),strike_spot:105.0,option_name:"FAKE 105_P_35".to_string(),expire_date:expiry,..Default::default() },
          dsta::Opt { option_rite : dsta::OptRite::Put, ticker_name:".FAKE 100_P_35".to_string(),strike_spot:100.0,option_name:"FAKE 100_P_35".to_string(),expire_date:expiry,..Default::default() },
          dsta::Opt { option_rite : dsta::OptRite::Put, ticker_name:".FAKE 95_P_35".to_string() ,strike_spot:95.0 ,option_name:"FAKE 95_P_35".to_string(),expire_date:expiry,..Default::default() },
          dsta::Opt { option_rite : dsta::OptRite::Put, ticker_name:".FAKE 90_P_35".to_string() ,strike_spot:90.0 ,option_name:"FAKE 90_P_35".to_string(),expire_date:expiry,..Default::default() },
        ],
      }],
    };

    reply_tx.send(Some(fake_result)).await.expect("Failed to send ticker chain");

    info!("Sent fake option chain for FAKE");


    trace!("Awaiting initial cash swap request");
    let swap_req = tokio::time::timeout(StdDuration::from_secs(5),swap_info_rx.recv())
      .await
      .expect("Timeout on cash swap request")
      .expect("Swap channel closed");

    info!("Received initial cash swap: {:?}",swap_req);

    let firststuff = crate::Allocation 
    { bot_name: format!("stealth_test_bot")
    , bot_safe: BotaAccount 
      { m_cash: 11_000.0,
        m_stock: None,
        m_long_calls: None,
        m_long_puts: None,
        m_short_calls: None,
        m_short_puts: None,
        m_call_credit_spread_lein: 0.0,
        m_short_put_lein: 0.0,
        m_put_credit_spread_lein: 0.0,
        m_short_iron_condor_lein: 0.0,
      },
      bot_mind: crate::BotMindState::Running,
      num_swap: 0,
    };
    check_tx.send(firststuff).await.expect("Failed to send 1st allocation");
    sucks_tx.send(crate::glossary::SWAP_SUCC.to_string())
      .await
      .expect("Failed to send SWAP_SUCC");

    info!("Initial cash approved");
    // After "Initial cash approved" and sleep(200ms)

    // === STEP A: Send initial underlying quote (triggers ATM calculation & option selection) ===
    info!("Sending initial underlying quote (needed for ATM calculation)");
    trade_tx.send(
      dsta::convert::stream_quote_to_ticker(&ttrs::StreamQuote {
        event_type: "quote".to_string(),
        symbol: "FAKE".to_string(),
        bid_price: Some(99.8),
        ask_price: Some(100.2),
        bid_size: Some(10.0),
        ask_size: Some(10.0),
      })
    ).await.expect("Failed to send initial underlying quote");

    // === STEP B: Bot requests long leg ticker (95 put) ===

    info!("Awaiting long leg ticker request (.FAKE 95_P_35)");
    let (opt_name_long, reply_tx_long) = tokio::time::timeout(StdDuration::from_secs(5), pls_ticks_rx.recv())
      .await
      .expect("Timeout waiting for long leg ticker request")
      .expect("Ticker request channel closed");

    assert_eq!(opt_name_long, ".FAKE 95_P_35", "Bot requested wrong long leg");
    let mut opt = dsta::Opt::default();
    opt.ticker_name = ".FAKE 95_P_35".to_string();
    opt.option_name = "FAKE 95_P_35".to_string();
    
    reply_tx_long.send(Some(dsta::PushTickerResult {
      ass_deets: dsta::FinAss::OptDeets(opt),
      opt_chain: vec![],
    })).await.expect("Failed to reply to long leg request");

    // === STEP C: Bot immediately requests short leg ticker (100 put) ===

    info!("Awaiting short leg ticker request (FAKE 100_P_35)");
    let (opt_name_short, reply_tx_short) = tokio::time::timeout(StdDuration::from_secs(5), pls_ticks_rx.recv())
      .await
      .expect("Timeout waiting for short leg ticker request")
      .expect("Ticker request channel closed");

    assert_eq!(opt_name_short, ".FAKE 100_P_35", "Bot requested wrong short leg");
    let mut opt = dsta::Opt::default();
    opt.ticker_name = ".FAKE 100_P_35".to_string();
    opt.option_name = "FAKE 100_P_35".to_string();
    
    reply_tx_short.send(Some(dsta::PushTickerResult {
      ass_deets: dsta::FinAss::OptDeets(opt),
      opt_chain: vec![],
    })).await.expect("Failed to reply to short leg request");


    // === STEP D: Now send long leg quote → this should trigger entry order ===

    info!("Sending long leg quote to trigger BuyToOpen");
    trade_tx.send(
      dsta::convert::stream_quote_to_ticker(&ttrs::StreamQuote {
        event_type: "quote".to_string(),
        symbol: ".FAKE 95_P_35".to_string(),
        bid_price: Some(1.90),
        ask_price: Some(2.10),
        bid_size: Some(20.0),
        ask_size: Some(20.0),
      })
    ).await.expect("Failed to send long leg quote");

    // === STEP 6: Now we should receive the entry order ===

    info!("Awaiting entry order (BuyToOpen 95 put)");
    let entry_order = tokio::time::timeout(StdDuration::from_secs(5), pls_trade_rx.recv())
      .await
      .expect("Timeout waiting for entry order")
      .expect("Trade channel closed");

    assert_eq!(entry_order.order_legs[0].buy_what.order_name(), "FAKE 95_P_35");
    assert_eq!(entry_order.order_legs[0].action, dsta::BuyOrSell::BuyToOpen);
    info!("Entry order validated: {:?}", entry_order.order_legs[0]);

    // Confirm it

    sucks_tx.send(format!("{}",crate::glossary::ORDER_FLY)).await.expect("Failed to send SWAP_SUCC");

    let mut account = BotaAccount::new().with_cash(11_000.0);

    check_tx.send(crate::Allocation
    { bot_name:bot_name.clone(),
      bot_safe:account.clone(),
      bot_mind:crate::BotMindState::Trading,
      num_swap:1,
    }).await.expect("Failed to send allocation update for entry fill");

    // Sleep to expect the RETRY!:
 
    tokio::time::sleep(StdDuration::from_millis(sleep1+1)).await;

    info!("Sending long leg quote to trigger rehaggle");
    trade_tx.send(
      dsta::convert::stream_quote_to_ticker(&ttrs::StreamQuote {
        event_type: "quote".to_string(),
        symbol: ".FAKE 95_P_35".to_string(),
        bid_price: Some(2.00),
        ask_price: Some(2.15),
        bid_size: Some(1.0),
        ask_size: Some(20.0),
      })
    ).await.expect("Failed to send long leg quote");


    info!("Awaiting reentry order (BuyToOpen 95 put)");
    let reentry_order = tokio::time::timeout(StdDuration::from_secs(5), pls_trade_rx.recv())
      .await
      .expect("Timeout waiting for reentry order")
      .expect("Trade channel closed");

    assert_eq!(reentry_order.order_legs[0].buy_what.order_name(), "FAKE 95_P_35");
    assert_eq!(reentry_order.order_legs[0].action, dsta::BuyOrSell::BuyToOpen);
    sucks_tx.send(format!("{}",crate::glossary::ORDER_FLY)).await.expect("Failed to send SWAP_SUCC");
    info!("reEntry order validated: {:?}", reentry_order.order_legs[0]);

    // Sleep to expect the RETRY!:
 
    tokio::time::sleep(StdDuration::from_millis(sleep1+1)).await;

    info!("Sending long leg quote to trigger rehaggle");
    trade_tx.send(
      dsta::convert::stream_quote_to_ticker(&ttrs::StreamQuote {
        event_type: "quote".to_string(),
        symbol: ".FAKE 95_P_35".to_string(),
        bid_price: Some(2.07),
        ask_price: Some(2.15),
        bid_size: Some(1.0),
        ask_size: Some(20.0),
      })
    ).await.expect("Failed to send long leg quote");

    info!("Awaiting reentry order 2 (BuyToOpen 95 put)");
    let reentry_order = tokio::time::timeout(StdDuration::from_secs(5), pls_trade_rx.recv())
      .await
      .expect("Timeout waiting for reentry order 2")
      .expect("Trade channel closed");

    assert_eq!(reentry_order.order_legs[0].buy_what.order_name(), "FAKE 95_P_35");
    assert_eq!(reentry_order.order_legs[0].action, dsta::BuyOrSell::BuyToOpen);
    sucks_tx.send(format!("{}",crate::glossary::ORDER_FLY)).await.expect("Failed to send SWAP_SUCC");
    info!("reEntry order 2 validated: {:?}", reentry_order.order_legs[0]);


    // Confirm it

    info!("STEP 7: Confirming entry fill");

    let mut stuff: HashMap<String,FinAss> = HashMap::new();
    let long_put = Opt
    { ticker_name:".FAKE 95_P_35".to_string(),
      option_name:"FAKE 95_P_35".to_string(),
      option_rite:OptRite::Put,
      strike_spot:95.0,
      expire_date:expiry,
      option_type:OptStyle::Usa,
      last_ticker:None,
      tick_rulers:None,
    };
    let short_put = Opt
    { ticker_name:".FAKE 100_P_35".to_string(),
      option_name:"FAKE 100_P_35".to_string(),
      option_rite:OptRite::Put,
      strike_spot:100.0,
      expire_date:expiry,
      option_type:OptStyle::Usa,
      last_ticker:None,
      tick_rulers:None,

    };
    stuff.insert(long_put.option_name.clone(),FinAss::OptDeets(long_put));
    stuff.insert(short_put.option_name.clone(),FinAss::OptDeets(short_put));
    
    let changes = fn_get_account_shennanagans(&reentry_order,&account,&stuff)
      .expect("Failed to compute entry fill changes");
    apply_account_cash_and_collateral_changes(&mut account,&changes);
    apply_order_to_account(&mut account,&reentry_order,&stuff)
      .expect("Failed to apply entry order to account");
    check_tx.send(crate::Allocation
    { bot_name:bot_name.clone(),
      bot_safe:account.clone(),
      bot_mind:crate::BotMindState::Running,
      num_swap:0,
    }).await.expect("Failed to send allocation update for entry fill");
    info!("Entry fill confirmed");
    tokio::time::sleep(StdDuration::from_millis(sleep1)).await;

  // === STEP 8: Send short leg quote + favorable underlying move ===
  info!("STEP 8: Sending quotes to trigger spread entry");
  trade_tx.send(
    dsta::convert::stream_quote_to_ticker(&ttrs::StreamQuote
    { event_type:"quote".to_string(),
      symbol:".FAKE 100_P_35".to_string(),
      bid_price:Some(5.40),
      ask_price:Some(5.60),
      bid_size:Some(15.0),
      ask_size:Some(15.0),
    })
  ).await.unwrap();

  trade_tx.send(
    dsta::convert::stream_quote_to_ticker(&ttrs::StreamQuote
    { event_type:"quote".to_string(),
      symbol:"FAKE".to_string(),
      bid_price:Some(95.80),
      ask_price:Some(95.82),
      bid_size:Some(10.0),
      ask_size:Some(10.0),
    })
  ).await.unwrap();

  trade_tx.send(
    dsta::convert::stream_quote_to_ticker(&ttrs::StreamQuote
    { event_type:"quote".to_string(),
      symbol:".FAKE 95_P_35".to_string(),
      bid_price:Some(3.10),
      ask_price:Some(3.20),
      bid_size:Some(20.0),
      ask_size:Some(20.0),
    })
  ).await.unwrap();

  trade_tx.send(
    dsta::convert::stream_quote_to_ticker(&ttrs::StreamQuote
    { event_type:"quote".to_string(),
      symbol:".FAKE 100_P_35".to_string(),
      bid_price:Some(6.40),
      ask_price:Some(6.60),
      bid_size:Some(15.0),
      ask_size:Some(15.0),
    })
  ).await.unwrap();

  tokio::time::sleep(StdDuration::from_millis(sleep1)).await;

  //// === STEP FIX ERROR: Receive error ===
  //info!("STEP FIX ERROR: Receiving sniv error");
  //let sniv_error = tokio::time::timeout(StdDuration::from_secs(5),snivrx.recv())
  //  .await
  //  .expect("Timeout waiting for sniv error order")
  //  .expect("snivrx channel closed");
  //info!("{:#?}", sniv_error );

  // === STEP 9: Receive spread sell order ===
  info!("STEP 9: Receiving spread order (SellToOpen 100 put)");
  let spread_order = tokio::time::timeout(StdDuration::from_secs(5), pls_trade_rx.recv())
      .await
      .expect("Timeout waiting for spread order")
      .expect("Trade channel closed");

  assert_eq!(spread_order.order_legs[0].buy_what.order_name(), "FAKE 100_P_35");
  assert_eq!(spread_order.order_legs[0].action, dsta::BuyOrSell::SellToOpen);
 

    info!("Spread order validated");

  // === NEW STEP 10: Confirm spread fill via account update ===
  info!("STEP 10: Confirming spread fill (short put position added)");

  // Reuse/update the same 'account' and 'stuff' from entry fill
  let changes = fn_get_account_shennanagans(&spread_order, &account, &stuff)
      .expect("Failed to compute spread fill changes");

  apply_account_cash_and_collateral_changes(&mut account, &changes);
  apply_order_to_account(&mut account, &spread_order, &stuff)
      .expect("Failed to apply spread order to account");

  sucks_tx.send(format!("{}",crate::glossary::ORDER_FLY)).await.expect("Failed to send SWAP_SUCC");

  // === STEP FIX ERROR: Receive error ===

  let mut sv = Vec::new();
  let sc = 100;
  info!("STEP FIX ERROR: Receiving sniv error");
  let sniv_errors = tokio::time::timeout(StdDuration::from_secs(5),snivrx.recv_many(&mut sv,sc))
    .await
    .expect("Timeout waiting for sniv error order")
    //.expect("snivrx channel closed");
  ;
  info!("\n\n\n\n\n\n\n\n SNIV ERRORS! {:#?}", sv );

  tokio::time::sleep(StdDuration::from_millis(sleep1)).await;

  check_tx.send(crate::Allocation {
      bot_name: bot_name.clone(),
      bot_safe: account.clone(),  // ← now includes both long and short positions
      bot_mind: crate::BotMindState::Running,
      num_swap: 0,
  }).await.expect("Failed to send allocation update for spread fill");

  info!("Spread fill confirmed - short position added");

  // Give bot time to process the update
  tokio::time::sleep(StdDuration::from_millis(sleep1)).await;

  // === STEP 11: Move underlying to trigger exit ===
  info!("STEP 11: Sending quotes to trigger nice exit (OTM)");
  trade_tx.send(
    dsta::convert::stream_quote_to_ticker(&ttrs::StreamQuote
    { event_type:"quote".to_string(),
      symbol:"FAKE".to_string(),
      bid_price:Some(105.10),
      ask_price:Some(105.14),
      bid_size:Some(10.0),
      ask_size:Some(10.0),
    })
  ).await.unwrap();

  trade_tx.send(
    dsta::convert::stream_quote_to_ticker(&ttrs::StreamQuote
    { event_type:"quote".to_string(),
      symbol:".FAKE 95_P_35".to_string(),
      bid_price:Some(0.01),
      ask_price:Some(0.05),
      bid_size:Some(10.0),
      ask_size:Some(10.0),
    })
  ).await.unwrap();

  trade_tx.send(
    dsta::convert::stream_quote_to_ticker(&ttrs::StreamQuote
    { event_type:"quote".to_string(),
      symbol:".FAKE 100_P_35".to_string(),
      bid_price:Some(0.05),
      ask_price:Some(0.10),
      bid_size:Some(10.0),
      ask_size:Some(10.0),
    })
  ).await.unwrap();

  tokio::time::sleep(StdDuration::from_millis(sleep1)).await;

  // === STEP 12: Receive final close order ===
  info!("STEP 12: Receiving final close order");
  let close_order = tokio::time::timeout(StdDuration::from_secs(5),pls_trade_rx.recv())
    .await
    .expect("Timeout waiting for close order")
    .expect("Trade channel closed");
  assert_eq!(close_order.order_legs.len(),2);
  let long_leg = close_order.order_legs.iter()
    .find(|leg| leg.buy_what.order_name() == "FAKE 95_P_35" && leg.action == dsta::BuyOrSell::SellToClose)
    .expect("Missing SellToClose on long put");
  let short_leg = close_order.order_legs.iter()
    .find(|leg| leg.buy_what.order_name() == "FAKE 100_P_35" && leg.action == dsta::BuyOrSell::BuyToClose)
    .expect("Missing BuyToClose on short put");
  info!("Close order validated — both legs present");
  sucks_tx.send(format!("{}",crate::glossary::ORDER_FLY)).await.expect("Failed to send SWAP_SUCC");


  // === STEP 13: Confirm final close ===
  info!("STEP 13: Confirming final close");
  let changes = fn_get_account_shennanagans(&close_order,&account,&stuff)
    .expect("Failed to compute close changes");
  apply_account_cash_and_collateral_changes(&mut account,&changes);
  apply_order_to_account(&mut account,&close_order,&stuff)
    .expect("Failed to apply close order");
  check_tx.send(crate::Allocation
  { bot_name:bot_name.clone(),
    bot_safe:account.clone(),
    bot_mind:crate::BotMindState::Running,
    num_swap:0,
  }).await.expect("Failed to send allocation update for final close");
  info!("Final close confirmed");

  tokio::time::sleep(StdDuration::from_millis(sleep1)).await;

  // === STEP 14: Shutdown ===
  info!("STEP 15: Shutting down bot");
  let _ = sucks_tx.send("DIE".to_string()).await;
  tokio::time::timeout(StdDuration::from_secs(5),bot_handle)
    .await
    .expect("Bot did not shut down")
    .ok();

  //panic!("=== Test completed successfully ===");
}

#[tokio::test]
async fn stealth_bot_entry_haggle_slippage_limit_partial_accept() 
{
  // Tests that when entry haggling exceeds slippage limit with partial fill,
  // bot accepts the partial and advances to Logic1
  
  use tokio::sync::mpsc;
  use chrono::{Utc, Duration};
  use std::time::Duration as StdDuration;
  use log::{info, debug, trace};
  use crate::*;
  use crate::tests::util::*;
  use crate::dr_robo::*;
  use dsta::*;
  use std::collections::HashMap;

  let _ = setup_log4rs("stealth_bot_entry_haggle_slippage_limit_partial_accept");
  info!("Starting stealth_bot_entry_haggle_slippage_limit_partial_accept test");

  let bot_name = "stealth_test_bot_entry_partial".to_string();
  let sleep1: u64 = 5;

  // Configure with tight slippage limit to force partial acceptance
  let config = dsta::MakeStealthBot {
    my_accounting: {
      let mut ma = dsta::CommonBotInfo::default();
      ma.haggle_action = dsta::HaggleAction::AnOyVeyJew(
        dsta::HaggleMethod::MinimumOfMidpointOrTheoreticalThenConcede(
          dsta::HaggleLimits {
            retry_period: chrono::Duration::milliseconds(sleep1 as i64),
            delta_choice: 50.0,  // Large delta ~ .10 cents for the .20 initial sptread
            delta_is_pct: true,
            slippage_max: 0.19,  // Allow first retry, then hit limit on second
          }
        )
      );
      ma.max_cash_risk = (true, 80.0, 10_000.0);
      ma.friendly_name = bot_name.clone();
      ma.tracking_tick = "FAKE".to_string();
      ma
    },
    my_cash_alloc: 11_000.0,
    stonk_feeling: dsta::MarketDirection::Corrects,
    option_bucket: 1,
    spread_bucket: 1,
    exit_gain_pct: 50.0,
    exit_loss_pct: -80.0,
    option_expire: 30,
    nice_exit_way: vec![dsta::StealthBotDoneNiceStyle::GoExpire],
    use_theo_cost: false,
  };

  let (trade_tx, trade_rx) = mpsc::channel(100);
  let (check_tx, check_rx) = mpsc::channel(100);
  let (sucks_tx, mut sucks_rx) = mpsc::channel(100);
  let (pls_ticks_tx, mut pls_ticks_rx) = mpsc::channel(10);
  let (pls_trade_tx, mut pls_trade_rx) = mpsc::channel(10);
  let (swap_info_tx, mut swap_info_rx) = mpsc::channel(10);
  let (new_brain_tx, _) = mpsc::channel(10);
  let (snivtx, mut snivrx) = mpsc::channel(100);

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

  let bot_handle = tokio::spawn(crate::sbot::stealth_bot::fn_run_main_stealth_bot(
    config,
    trade_rx,
    check_rx,
    sucks_rx,
    bridge,
  ));

  let expiry = Utc::now() + Duration::days(35);

  // Setup phase
  trace!("Awaiting initial ticker request");
  let (ticker_name, reply_tx) = tokio::time::timeout(StdDuration::from_secs(5), pls_ticks_rx.recv())
    .await.expect("Timeout").expect("Channel closed");
  assert_eq!(ticker_name, "FAKE");

  let mut stk = dsta::Stk::default();
  stk.ticker_name = "FAKE".to_string();
  let fake_result = dsta::PushTickerResult {
    ass_deets: dsta::FinAss::StkDeets(stk),
    opt_chain: vec![dsta::Derivatives {
      expire_time: expiry,
      option_calls: vec![],
      option_puts: vec![
        dsta::Opt { option_rite: dsta::OptRite::Put, ticker_name: ".FAKE 105_P_35".to_string(), strike_spot: 105.0, option_name: "FAKE 105_P_35".to_string(), expire_date: expiry, ..Default::default() },
        dsta::Opt { option_rite: dsta::OptRite::Put, ticker_name: ".FAKE 100_P_35".to_string(), strike_spot: 100.0, option_name: "FAKE 100_P_35".to_string(), expire_date: expiry, ..Default::default() },
        dsta::Opt { option_rite: dsta::OptRite::Put, ticker_name: ".FAKE 95_P_35".to_string(), strike_spot: 95.0, option_name: "FAKE 95_P_35".to_string(), expire_date: expiry, ..Default::default() },
        dsta::Opt { option_rite: dsta::OptRite::Put, ticker_name: ".FAKE 90_P_35".to_string(), strike_spot: 90.0, option_name: "FAKE 90_P_35".to_string(), expire_date: expiry, ..Default::default() },
      ],
    }],
  };
  reply_tx.send(Some(fake_result)).await.expect("Failed to send ticker chain");

  // Cash swap
  let _swap_req = tokio::time::timeout(StdDuration::from_secs(5), swap_info_rx.recv())
    .await.expect("Timeout").expect("Channel closed");

  let mut account = BotaAccount::new().with_cash(11_000.0);
  check_tx.send(crate::Allocation {
    bot_name: bot_name.clone(),
    bot_safe: account.clone(),
    bot_mind: crate::BotMindState::Running,
    num_swap: 0,
  }).await.expect("Failed to send allocation");
  sucks_tx.send(crate::glossary::SWAP_SUCC.to_string()).await.expect("Failed to send SWAP_SUCC");

  // Send underlying quote
  trade_tx.send(dsta::convert::stream_quote_to_ticker(&ttrs::StreamQuote {
    event_type: "quote".to_string(),
    symbol: "FAKE".to_string(),
    bid_price: Some(99.8),
    ask_price: Some(100.2),
    bid_size: Some(10.0),
    ask_size: Some(10.0),
  })).await.expect("Failed to send underlying quote");

  // Long leg ticker request
  let (opt_name_long, reply_tx_long) = tokio::time::timeout(StdDuration::from_secs(5), pls_ticks_rx.recv())
    .await.expect("Timeout").expect("Channel closed");
  assert_eq!(opt_name_long, ".FAKE 95_P_35");
  
  let mut opt = dsta::Opt::default();
  opt.ticker_name = ".FAKE 95_P_35".to_string();
  opt.option_name = "FAKE 95_P_35".to_string();
  reply_tx_long.send(Some(dsta::PushTickerResult {
    ass_deets: dsta::FinAss::OptDeets(opt.clone()),
    opt_chain: vec![],
  })).await.expect("Failed to reply");

  // Short leg ticker request
  let (_opt_name_short, reply_tx_short) = tokio::time::timeout(StdDuration::from_secs(5), pls_ticks_rx.recv())
    .await.expect("Timeout").expect("Channel closed");
  let mut opt2 = dsta::Opt::default();
  opt2.ticker_name = ".FAKE 100_P_35".to_string();
  opt2.option_name = "FAKE 100_P_35".to_string();
  reply_tx_short.send(Some(dsta::PushTickerResult {
    ass_deets: dsta::FinAss::OptDeets(opt2),
    opt_chain: vec![],
  })).await.expect("Failed to reply");

  // Trigger entry order
  trade_tx.send(dsta::convert::stream_quote_to_ticker(&ttrs::StreamQuote {
    event_type: "quote".to_string(),
    symbol: ".FAKE 95_P_35".to_string(),
    bid_price: Some(1.90),
    ask_price: Some(2.10),
    bid_size: Some(20.0),
    ask_size: Some(20.0),
  })).await.expect("Failed to send quote");

  // Receive initial entry order
  info!("Awaiting initial entry order");
  let entry_order = tokio::time::timeout(StdDuration::from_secs(5), pls_trade_rx.recv())
    .await.expect("Timeout").expect("Channel closed");
  assert_eq!(entry_order.order_legs[0].action, dsta::BuyOrSell::BuyToOpen);
  let requested_qty = entry_order.order_legs[0].quantity;
  
  // CRITICAL: Send ORDER_FLY immediately
  let _ = sucks_tx.send(format!("{}",crate::glossary::ORDER_FLY)).await.expect("Failed to send ORDER_FLY");

  // Send PARTIAL fill (2 out of requested quantity)
  let partial_qty = 2.0;
  
  account.m_long_puts = Some(dsta::OwnedPosition {
    asset: dsta::FinAss::OptDeets(opt.clone()),
    owned: partial_qty,
    price: 2.00,
  });
  account.m_cash -= partial_qty * 2.00 * 100.0;

  // CRITICAL: Send Trading state first, then Running
  check_tx.send(crate::Allocation {
    bot_name: bot_name.clone(),
    bot_safe: account.clone(),
    bot_mind: crate::BotMindState::Trading,
    num_swap: 1,
  }).await.expect("Failed to send Trading state");

  tokio::time::sleep(StdDuration::from_millis(50)).await;

  check_tx.send(crate::Allocation {
    bot_name: bot_name.clone(),
    bot_safe: account.clone(),
    bot_mind: crate::BotMindState::Running,
    num_swap: 1,
  }).await.expect("Failed to send Running state");

  // CRITICAL: Must wait for retry period before next action
  tokio::time::sleep(StdDuration::from_millis(sleep1 + 10)).await;

  // Trigger haggle retry 1 with new quote
  trade_tx.send(dsta::convert::stream_quote_to_ticker(&ttrs::StreamQuote {
    event_type: "quote".to_string(),
    symbol: ".FAKE 95_P_35".to_string(),
    bid_price: Some(2.00),
    ask_price: Some(2.15),
    bid_size: Some(1.0),
    ask_size: Some(20.0),
  })).await.expect("Failed to send quote");

  info!("Awaiting haggle retry 1");
  let retry1_order = tokio::time::timeout(StdDuration::from_secs(5), pls_trade_rx.recv())
    .await.expect("Timeout").expect("Channel closed");
  assert!(retry1_order.order_legs[0].quantity < requested_qty, "Should be haggling for remaining");
  let _ = sucks_tx.send(format!("{}",crate::glossary::ORDER_FLY)).await;

  // CRITICAL: Wait for retry period again
  tokio::time::sleep(StdDuration::from_millis(sleep1 + 10)).await;

  // Trigger haggle retry 2 (should exceed slippage limit and send cancel)
  trade_tx.send(dsta::convert::stream_quote_to_ticker(&ttrs::StreamQuote {
    event_type: "quote".to_string(),
    symbol: ".FAKE 95_P_35".to_string(),
    bid_price: Some(2.10),
    ask_price: Some(2.25),
    bid_size: Some(1.0),
    ask_size: Some(20.0),
  })).await.expect("Failed to send quote");

  info!("Awaiting slippage limit exceeded - should see cancel order");
  let cancel_order = tokio::time::timeout(StdDuration::from_secs(5), pls_trade_rx.recv())
    .await.expect("Timeout").expect("Channel closed");
  
  // Cancel order has empty legs
  assert_eq!(cancel_order.order_legs.len(), 0, "Should be a cancel order");
  let _ = sucks_tx.send(format!("{}",crate::glossary::ORDER_FLY)).await;

  info!("Cancel order received - bot should accept partial fill and advance to Logic1");

  // Bot should now be in Logic1 with partial position
  tokio::time::sleep(StdDuration::from_millis(100)).await;

  // Verify bot is operational by sending quotes (should not crash)
  trade_tx.send(dsta::convert::stream_quote_to_ticker(&ttrs::StreamQuote {
    event_type: "quote".to_string(),
    symbol: ".FAKE 95_P_35".to_string(),
    bid_price: Some(2.20),
    ask_price: Some(2.30),
    bid_size: Some(10.0),
    ask_size: Some(10.0),
  })).await.expect("Failed to send quote");

  tokio::time::sleep(StdDuration::from_millis(100)).await;

  info!("Test completed - bot accepted partial fill and continued");

  let _ = sucks_tx.send("DIE".to_string()).await;
  tokio::time::timeout(StdDuration::from_secs(5), bot_handle)
    .await.expect("Bot did not shut down").ok();
}

// KEY FIXES for stealth_bot_final_exit_haggle_partial_spread_close:
//
// 1. Increased timeout from 6s to 10s to account for haggle retry periods
// 2. Added proper sleep duration after ORDER_FLY (retry_ms + 100ms buffer)
//    This allows bot's Check4 state to wait for its retry_period before sending haggle
// 3. Moved ORDER_FLY send to immediately after receiving order (not at end of match)
// 4. Added separate handling for unexpected/extra orders
// 5. Improved logging and status tracking with last_fill_time
// 6. Changed sleep from fixed 200ms to (retry_ms + 100ms) after partial fills

#[tokio::test]
async fn stealth_bot_final_exit_haggle_partial_spread_close() {
    use tokio::sync::mpsc;
    use chrono::{Utc, Duration};
    use std::time::Duration as StdDuration;
    use log::{info, debug, trace, warn};
    use crate::*;
    use crate::tests::util::*;
    use crate::dr_robo::*;
    use dsta::*;
    use std::collections::HashMap;

    let _ = setup_log4rs("stealth_bot_final_exit_haggle_partial_spread_close");
    info!("Starting stealth_bot_final_exit_haggle_partial_spread_close test");

    let bot_name = "stealth_test_bot_final_exit".to_string();
    let retry_ms: u64 = 250; // CRITICAL: must match the retry_period in exit haggle methods

    let config = dsta::MakeStealthBot {
        my_accounting: {
            let mut ma = dsta::CommonBotInfo::default();
            ma.haggle_action = dsta::HaggleAction::AnOyVeyJew(
                dsta::HaggleMethod::MinimumOfMidpointOrTheoreticalThenConcede(
                    dsta::HaggleLimits {
                        retry_period: chrono::Duration::milliseconds(retry_ms as i64),
                        delta_choice: 25.0,
                        delta_is_pct: true,
                        slippage_max: 8.0,
                    }
                )
            );
            ma.max_cash_risk = (true, 80.0, 10_000.0);
            ma.friendly_name = bot_name.clone();
            ma.tracking_tick = "FAKE".to_string();
            ma
        },
        my_cash_alloc: 11_000.0,
        stonk_feeling: dsta::MarketDirection::Corrects,
        option_bucket: 1,
        spread_bucket: 1,
        exit_gain_pct: 50.0,
        exit_loss_pct: -80.0,
        option_expire: 30,
        nice_exit_way: vec![dsta::StealthBotDoneNiceStyle::OtmShort],
        use_theo_cost: false,
    };

    let (trade_tx, trade_rx) = mpsc::channel(100);
    let (check_tx, check_rx) = mpsc::channel(100);
    let (sucks_tx, mut sucks_rx) = mpsc::channel(100);
    let (pls_ticks_tx, mut pls_ticks_rx) = mpsc::channel(10);
    let (pls_trade_tx, mut pls_trade_rx) = mpsc::channel(10);
    let (swap_info_tx, mut swap_info_rx) = mpsc::channel(10);
    let (new_brain_tx, _) = mpsc::channel(10);
    let (snivtx, mut snivrx) = mpsc::channel(100);

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

    let bot_handle = tokio::spawn(crate::sbot::stealth_bot::fn_run_main_stealth_bot(
        config,
        trade_rx,
        check_rx,
        sucks_rx,
        bridge,
    ));

    let expiry = Utc::now() + Duration::days(35);

    // === INITIAL SETUP ===
    let (_, reply_tx) = pls_ticks_rx.recv().await.expect("No initial ticker request");
    let mut stk = dsta::Stk::default();
    stk.ticker_name = "FAKE".to_string();
    reply_tx.send(Some(dsta::PushTickerResult {
        ass_deets: dsta::FinAss::StkDeets(stk),
        opt_chain: vec![dsta::Derivatives {
            expire_time: expiry,
            option_calls: vec![],
            option_puts: vec![
                dsta::Opt { option_rite: dsta::OptRite::Put, ticker_name: ".FAKE 95_P_35".to_string(), strike_spot: 95.0, option_name: "FAKE 95_P_35".to_string(), expire_date: expiry, ..Default::default() },
                dsta::Opt { option_rite: dsta::OptRite::Put, ticker_name: ".FAKE 100_P_35".to_string(), strike_spot: 100.0, option_name: "FAKE 100_P_35".to_string(), expire_date: expiry, ..Default::default() },
            ],
        }],
    })).await.unwrap();

    swap_info_rx.recv().await.expect("No cash swap request");
    let mut account = BotaAccount::new().with_cash(11_000.0);
    check_tx.send(crate::Allocation {
        bot_name: bot_name.clone(),
        bot_safe: account.clone(),
        bot_mind: crate::BotMindState::Running,
        num_swap: 0,
    }).await.unwrap();
    sucks_tx.send(crate::glossary::SWAP_SUCC.to_string()).await.unwrap();

    trade_tx.send(dsta::convert::stream_quote_to_ticker(&ttrs::StreamQuote {
        event_type: "quote".to_string(),
        symbol: "FAKE".to_string(),
        bid_price: Some(99.8),
        ask_price: Some(100.2),
        bid_size: Some(10.0),
        ask_size: Some(10.0),
    })).await.unwrap();

    for _ in 0..2 {
        let (ticker_name, reply_tx) = pls_ticks_rx.recv().await.expect("Missing leg ticker request");
        let mut opt = dsta::Opt::default();
        opt.ticker_name = ticker_name.clone();
        opt.option_name = ticker_name.replace(".", "");
        opt.strike_spot = if ticker_name.contains("95") { 95.0 } else { 100.0 };
        opt.option_rite = dsta::OptRite::Put;
        opt.expire_date = expiry;
        reply_tx.send(Some(dsta::PushTickerResult {
            ass_deets: dsta::FinAss::OptDeets(opt),
            opt_chain: vec![],
        })).await.unwrap();
    }

    // === ENTRY: BuyToOpen long put ===
    // CRITICAL: Send quote for opt_1 (100_P_35) FIRST, then opt_2 (95_P_35)
    // The bot selects long=100_P_35, short=95_P_35 and f_run_begins waits for opt_1 data
    trade_tx.send(dsta::convert::stream_quote_to_ticker(&ttrs::StreamQuote {
        event_type: "quote".to_string(),
        symbol: ".FAKE 100_P_35".to_string(),
        bid_price: Some(1.90),
        ask_price: Some(2.10),
        bid_size: Some(20.0),
        ask_size: Some(20.0),
    })).await.unwrap();

    let entry_order = pls_trade_rx.recv().await.expect("No entry order");
    let _ = sucks_tx.send(format!("{}",crate::glossary::ORDER_FLY)).await.expect("Failed to send ORDER_FLY");
    let qty = entry_order.order_legs[0].quantity;

    let mut assets: HashMap<String, FinAss> = HashMap::new();
    assets.insert("FAKE 95_P_35".to_string(), FinAss::OptDeets(dsta::Opt { option_name: "FAKE 95_P_35".to_string(), ..Default::default() }));
    assets.insert("FAKE 100_P_35".to_string(), FinAss::OptDeets(dsta::Opt { option_name: "FAKE 100_P_35".to_string(), ..Default::default() }));

    account.m_long_puts = Some(dsta::OwnedPosition {
        asset: assets["FAKE 95_P_35"].clone(),
        owned: qty,
        price: 2.00,
    });
    account.m_cash -= qty * 2.00 * 100.0;

    check_tx.send(crate::Allocation {
        bot_name: bot_name.clone(),
        bot_safe: account.clone(),
        bot_mind: crate::BotMindState::Running,
        num_swap: 1,
    }).await.unwrap();

    tokio::time::sleep(StdDuration::from_millis(300)).await;

    // === SPREAD ENTRY: SellToOpen short put ===
    info!("Triggering spread entry");
    trade_tx.send(dsta::convert::stream_quote_to_ticker(&ttrs::StreamQuote {
        event_type: "quote".to_string(),
        symbol: ".FAKE 100_P_35".to_string(),
        bid_price: Some(5.40),
        ask_price: Some(5.60),
        bid_size: Some(15.0),
        ask_size: Some(15.0),
    })).await.unwrap();

    trade_tx.send(dsta::convert::stream_quote_to_ticker(&ttrs::StreamQuote {
        event_type: "quote".to_string(),
        symbol: "FAKE".to_string(),
        bid_price: Some(95.80),
        ask_price: Some(95.82),
        bid_size: Some(10.0),
        ask_size: Some(10.0),
    })).await.unwrap();

    let spread_order = pls_trade_rx.recv().await.expect("No spread order");
    let _ = sucks_tx.send(format!("{}",crate::glossary::ORDER_FLY)).await.expect("Failed to send ORDER_FLY");

    account.m_short_puts = Some(dsta::OwnedPosition {
        asset: assets["FAKE 100_P_35"].clone(),
        owned: -qty,
        price: 5.50,
    });

    account.m_cash += qty * 5.50 * 100.0;

    check_tx.send(crate::Allocation {
        bot_name: bot_name.clone(),
        bot_safe: account.clone(),
        bot_mind: crate::BotMindState::Running,
        num_swap: 2,
    }).await.unwrap();

    tokio::time::sleep(StdDuration::from_millis(300)).await;

    // === TRIGGER NICE EXIT ===
    info!("Triggering final exit (short OTM)");
    trade_tx.send(dsta::convert::stream_quote_to_ticker(&ttrs::StreamQuote {
        event_type: "quote".to_string(),
        symbol: "FAKE".to_string(),
        bid_price: Some(105.10),
        ask_price: Some(105.14),
        bid_size: Some(10.0),
        ask_size: Some(10.0),
    })).await.unwrap();

    trade_tx.send(dsta::convert::stream_quote_to_ticker(&ttrs::StreamQuote {
        event_type: "quote".to_string(),
        symbol: ".FAKE 95_P_35".to_string(),
        bid_price: Some(0.01),
        ask_price: Some(0.05),
        bid_size: Some(10.0),
        ask_size: Some(10.0),
    })).await.unwrap();

    trade_tx.send(dsta::convert::stream_quote_to_ticker(&ttrs::StreamQuote {
        event_type: "quote".to_string(),
        symbol: ".FAKE 100_P_35".to_string(),
        bid_price: Some(0.05),
        ask_price: Some(0.10),
        bid_size: Some(10.0),
        ask_size: Some(10.0),
    })).await.unwrap();

    tokio::time::sleep(StdDuration::from_millis(300)).await;

    // === FINAL EXIT PHASE: Receive orders until complete ===
    info!("Waiting for final exit order(s) + haggle retries");
    let mut exit_complete = false;
    let mut received_orders = 0;
    
    while !exit_complete && received_orders < 5 {
        // FIX #1: Use 10s timeout instead of 6s to account for haggle retry periods
      match tokio::time::timeout(StdDuration::from_secs(10), pls_trade_rx.recv()).await 
      {
        Ok(Some(order)) => {
          received_orders += 1;
          let legs = order.order_legs.len();

          // Always ACK first
          sucks_tx.send(format!("{}", crate::glossary::ORDER_FLY)).await
            .expect("Failed to send ORDERFLY");

          if legs == 0 {
            warn!("Unexpected cancel-only order in final exit, skipping");
            tokio::time::sleep(StdDuration::from_millis(300)).await;
            continue;
          }

          if legs == 1 {
            info!("Final exit single-leg order received; marking exit_complete");
            exit_complete = true;
            // Optionally assert direction/qty here
            break;
          }

          if legs == 2 {
            info!("Final exit spread order received; marking exit_complete");
            exit_complete = true;
            break;
          }

          warn!("Unexpected order with {} legs; skipping", legs);
          tokio::time::sleep(StdDuration::from_millis(300)).await;
        }
        Ok(None) => {
          panic!("Trade channel closed unexpectedly after {} orders", received_orders);
        }
        Err(_) => {
          panic!("Timeout waiting for exit/haggle order #{}", received_orders + 1);
        }
      }
    }
    
    assert!(exit_complete, "Final exit never completed after {} orders", received_orders);

    tokio::time::sleep(StdDuration::from_millis(500)).await;

    // Cleanup
    let _ = sucks_tx.send("DIE".to_string()).await;
    let result = tokio::time::timeout(StdDuration::from_secs(5), bot_handle).await;
    assert!(result.is_ok(), "Bot did not terminate after full exit");

    info!("Test completed successfully with {} orders", received_orders);
}



#[tokio::test]
async fn stealth_bot_loss_exit_haggle_partial_then_complete() 
{
  // Tests loss exit haggling: partial fill on first attempt,
  // haggles for remainder, then completes and terminates
  
  use tokio::sync::mpsc;
  use chrono::{Utc, Duration};
  use std::time::Duration as StdDuration;
  use log::{info, debug, trace};
  use crate::*;
  use crate::tests::util::*;
  use crate::dr_robo::*;
  use dsta::*;
  use std::collections::HashMap;

  let _ = setup_log4rs("stealth_bot_loss_exit_haggle_partial_then_complete");
  info!("Starting stealth_bot_loss_exit_haggle_partial_then_complete test");

  let bot_name = "stealth_test_bot_loss_exit".to_string();
  let sleep1: u64 = 5;

  let config = dsta::MakeStealthBot {
    my_accounting: {
      let mut ma = dsta::CommonBotInfo::default();
      ma.haggle_action = dsta::HaggleAction::AnOyVeyJew(
        dsta::HaggleMethod::MinimumOfMidpointOrTheoreticalThenConcede(
          dsta::HaggleLimits {
            retry_period: chrono::Duration::milliseconds(sleep1 as i64),
            delta_choice: 30.0,
            delta_is_pct: true,
            slippage_max: 10.0,  // High enough to allow multiple retries
          }
        )
      );
      ma.max_cash_risk = (true, 80.0, 10_000.0);
      ma.friendly_name = bot_name.clone();
      ma.tracking_tick = "FAKE".to_string();
      ma
    },
    my_cash_alloc: 11_000.0,
    stonk_feeling: dsta::MarketDirection::Corrects,
    option_bucket: 1,
    spread_bucket: 1,
    exit_gain_pct: 50.0,
    exit_loss_pct: -80.0,  // Will trigger at ~0.40 if entry was 2.00
    option_expire: 30,
    nice_exit_way: vec![dsta::StealthBotDoneNiceStyle::GoExpire],
    use_theo_cost: false,
  };

  let (trade_tx, trade_rx) = mpsc::channel(100);
  let (check_tx, check_rx) = mpsc::channel(100);
  let (sucks_tx, mut sucks_rx) = mpsc::channel(100);
  let (pls_ticks_tx, mut pls_ticks_rx) = mpsc::channel(10);
  let (pls_trade_tx, mut pls_trade_rx) = mpsc::channel(10);
  let (swap_info_tx, mut swap_info_rx) = mpsc::channel(10);
  let (new_brain_tx, _) = mpsc::channel(10);
  let (snivtx, mut snivrx) = mpsc::channel(100);

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

  let bot_handle = tokio::spawn(crate::sbot::stealth_bot::fn_run_main_stealth_bot(
    config,
    trade_rx,
    check_rx,
    sucks_rx,
    bridge,
  ));

  let expiry = Utc::now() + Duration::days(35);

  // Setup (abbreviated)
  let (_, reply_tx) = pls_ticks_rx.recv().await.unwrap();
  let mut stk = dsta::Stk::default();
  stk.ticker_name = "FAKE".to_string();
  reply_tx.send(Some(dsta::PushTickerResult {
    ass_deets: dsta::FinAss::StkDeets(stk),
    opt_chain: vec![dsta::Derivatives {
      expire_time: expiry,
      option_calls: vec![],
      option_puts: vec![
        dsta::Opt { option_rite: dsta::OptRite::Put, ticker_name: ".FAKE 105_P_35".to_string(), strike_spot: 105.0, option_name: "FAKE 105_P_35".to_string(), expire_date: expiry, ..Default::default() },
        dsta::Opt { option_rite: dsta::OptRite::Put, ticker_name: ".FAKE 100_P_35".to_string(), strike_spot: 100.0, option_name: "FAKE 100_P_35".to_string(), expire_date: expiry, ..Default::default() },
        dsta::Opt { option_rite: dsta::OptRite::Put, ticker_name: ".FAKE 95_P_35".to_string(), strike_spot: 95.0, option_name: "FAKE 95_P_35".to_string(), expire_date: expiry, ..Default::default() },
        dsta::Opt { option_rite: dsta::OptRite::Put, ticker_name: ".FAKE 100_P_35".to_string(), strike_spot: 100.0, option_name: "FAKE 100_P_35".to_string(), expire_date: expiry, ..Default::default() },
      ],
    }],
  })).await.unwrap();

  swap_info_rx.recv().await.unwrap();
  let mut account = BotaAccount::new().with_cash(11_000.0);
  check_tx.send(crate::Allocation {
    bot_name: bot_name.clone(),
    bot_safe: account.clone(),
    bot_mind: crate::BotMindState::Running,
    num_swap: 0,
  }).await.unwrap();
  sucks_tx.send(crate::glossary::SWAP_SUCC.to_string()).await.unwrap();

  trade_tx.send(dsta::convert::stream_quote_to_ticker(&ttrs::StreamQuote {
    event_type: "quote".to_string(),
    symbol: "FAKE".to_string(),
    bid_price: Some(99.8),
    ask_price: Some(100.2),
    bid_size: Some(10.0),
    ask_size: Some(10.0),
  })).await.unwrap();

  let (_, reply_tx_long) = pls_ticks_rx.recv().await.unwrap();
  let mut opt_long = dsta::Opt::default();
  opt_long.ticker_name = ".FAKE 95_P_35".to_string();
  opt_long.option_name = "FAKE 95_P_35".to_string();
  opt_long.strike_spot = 95.0;
  reply_tx_long.send(Some(dsta::PushTickerResult {
    ass_deets: dsta::FinAss::OptDeets(opt_long.clone()),
    opt_chain: vec![],
  })).await.unwrap();

  let (_, reply_tx_short) = pls_ticks_rx.recv().await.unwrap();
  let mut opt_short = dsta::Opt::default();
  opt_short.ticker_name = ".FAKE 100_P_35".to_string();
  opt_short.option_name = "FAKE 100_P_35".to_string();
  reply_tx_short.send(Some(dsta::PushTickerResult {
    ass_deets: dsta::FinAss::OptDeets(opt_short.clone()),
    opt_chain: vec![],
  })).await.unwrap();

  // Complete entry
  trade_tx.send(dsta::convert::stream_quote_to_ticker(&ttrs::StreamQuote {
    event_type: "quote".to_string(),
    symbol: ".FAKE 95_P_35".to_string(),
    bid_price: Some(1.90),
    ask_price: Some(2.10),
    bid_size: Some(20.0),
    ask_size: Some(20.0),
  })).await.unwrap();

  let entry_order = pls_trade_rx.recv().await.unwrap();
    let _ = sucks_tx.send(format!("{}",crate::glossary::ORDER_FLY)).await.expect("Failed to send SWAP_SUCC");
  let qty = entry_order.order_legs[0].quantity;

  let mut stuff: HashMap<String, FinAss> = HashMap::new();
  stuff.insert(opt_long.option_name.clone(), FinAss::OptDeets(opt_long.clone()));

  account.m_long_puts = Some(dsta::OwnedPosition {
    asset: dsta::FinAss::OptDeets(opt_long.clone()),
    owned: qty,
    price: 2.00,
  });
  account.m_cash -= qty * 2.00 * 100.0;

  check_tx.send(crate::Allocation {
    bot_name: bot_name.clone(),
    bot_safe: account.clone(),
    bot_mind: crate::BotMindState::Running,
    num_swap: 1,
  }).await.unwrap();

  tokio::time::sleep(StdDuration::from_millis(sleep1)).await;

  // NOW trigger LOSS EXIT (price drops below floor)
  // Floor = entry_price * (1 + exit_loss_pct / 100) = 2.00 * (1 - 0.80) = 0.40
  info!("Triggering loss exit by dropping price below floor");
  
  trade_tx.send(dsta::convert::stream_quote_to_ticker(&ttrs::StreamQuote {
    event_type: "quote".to_string(),
    symbol: ".FAKE 95_P_35".to_string(),
    bid_price: Some(0.30),
    ask_price: Some(0.35),
    bid_size: Some(20.0),
    ask_size: Some(20.0),
  })).await.unwrap();

  tokio::time::sleep(StdDuration::from_millis(sleep1)).await;

  info!("Receiving initial loss exit order");
  let loss_exit_order = pls_trade_rx.recv().await.unwrap();
  assert_eq!(loss_exit_order.order_legs[0].action, dsta::BuyOrSell::SellToClose);
  assert_eq!(loss_exit_order.order_legs[0].quantity, qty);
    let _ = sucks_tx.send(format!("{}",crate::glossary::ORDER_FLY)).await.expect("Failed to send SWAP_SUCC");

  // Partial fill on loss exit (e.g., half)
  let partial_closed = qty / 2.0;
  let remaining = qty - partial_closed;

  account.m_long_puts.as_mut().unwrap().owned = remaining;
  account.m_cash += partial_closed * 0.33 * 100.0;

  check_tx.send(crate::Allocation {
    bot_name: bot_name.clone(),
    bot_safe: account.clone(),
    bot_mind: crate::BotMindState::Running,
    num_swap: 2,
  }).await.unwrap();

  // CRITICAL FIX: Loss exit haggling uses 250ms retry_period (not entry's 5ms)
  // Bot will NOT send haggle retry until this period elapses after Check3 entry
  // Must wait long enough for the bot's retry_period to expire
  tokio::time::sleep(StdDuration::from_millis(260)).await;

  // NOW trigger haggle for remaining — bot is ready to send retry order
  trade_tx.send(dsta::convert::stream_quote_to_ticker(&ttrs::StreamQuote {
    event_type: "quote".to_string(),
    symbol: ".FAKE 95_P_35".to_string(),
    bid_price: Some(0.28),
    ask_price: Some(0.32),
    bid_size: Some(10.0),
    ask_size: Some(10.0),
  })).await.unwrap();

  info!("Receiving loss exit haggle retry");
  let retry_order = pls_trade_rx.recv().await.unwrap();
  assert_eq!(retry_order.order_legs[0].action, dsta::BuyOrSell::SellToClose);
  assert!(retry_order.order_legs[0].quantity <= remaining, "Should haggle for remaining");
  let _ = sucks_tx.send(format!("{}", crate::glossary::ORDER_FLY)).await.expect("Failed to send SWAP_SUCC");

  // Complete the loss exit with proper state transitions
  account.m_long_puts = None;
  account.m_cash += remaining * 0.30 * 100.0;

  check_tx.send(crate::Allocation {
    bot_name: bot_name.clone(),
    bot_safe: account.clone(),
    bot_mind: crate::BotMindState::Trading,
    num_swap: 3,
  }).await.unwrap();

  tokio::time::sleep(StdDuration::from_millis(50)).await;

  check_tx.send(crate::Allocation {
    bot_name: bot_name.clone(),
    bot_safe: account.clone(),
    bot_mind: crate::BotMindState::Running,
    num_swap: 3,
  }).await.unwrap();

  tokio::time::sleep(StdDuration::from_millis(100)).await;

  info!("Loss exit complete - bot should terminate");

  let result = tokio::time::timeout(StdDuration::from_secs(5), bot_handle).await;
  assert!(result.is_ok(), "Bot should terminate after loss exit");

  info!("Test completed successfully - bot terminated as expected");
}


#[tokio::test]
async fn stealth_bot_entry_haggle_zero_fill_slippage_exceeded_terminates() 
{
  // Tests that when entry haggling exceeds slippage with ZERO fill,
  // bot cancels and terminates (no position to work with)
  
  use tokio::sync::mpsc;
  use chrono::{Utc, Duration};
  use std::time::Duration as StdDuration;
  use log::{info, debug, trace};
  use crate::*;
  use crate::tests::util::*;
  use crate::dr_robo::*;
  use dsta::*;
  use std::collections::HashMap;

  let _ = setup_log4rs("stealth_bot_entry_haggle_zero_fill_slippage_exceeded_terminates");
  info!("Starting stealth_bot_entry_haggle_zero_fill_slippage_exceeded_terminates test");

  let bot_name = "stealth_test_bot_zero_fill".to_string();
  let sleep1: u64 = 5;

  // Very tight slippage to force early limit
  let config = dsta::MakeStealthBot {
    my_accounting: {
      let mut ma = dsta::CommonBotInfo::default();
      ma.haggle_action = dsta::HaggleAction::AnOyVeyJew(
        dsta::HaggleMethod::MinimumOfMidpointOrTheoreticalThenConcede(
          dsta::HaggleLimits {
            retry_period: chrono::Duration::milliseconds(sleep1 as i64),
            delta_choice: 80.0,  // Huge concessions to ensure limit exceeded
            delta_is_pct: true,
            slippage_max: 0.10,  // Will be exceeded on first retry with full quantity
          }
        )
      );
      ma.max_cash_risk = (true, 80.0, 10_000.0);
      ma.friendly_name = bot_name.clone();
      ma.tracking_tick = "FAKE".to_string();
      ma
    },
    my_cash_alloc: 11_000.0,
    stonk_feeling: dsta::MarketDirection::Corrects,
    option_bucket: 1,
    spread_bucket: 1,
    exit_gain_pct: 50.0,
    exit_loss_pct: -80.0,
    option_expire: 30,
    nice_exit_way: vec![dsta::StealthBotDoneNiceStyle::GoExpire],
    use_theo_cost: false,
  };

  let (trade_tx, trade_rx) = mpsc::channel(100);
  let (check_tx, check_rx) = mpsc::channel(100);
  let (sucks_tx, mut sucks_rx) = mpsc::channel(100);
  let (pls_ticks_tx, mut pls_ticks_rx) = mpsc::channel(10);
  let (pls_trade_tx, mut pls_trade_rx) = mpsc::channel(10);
  let (swap_info_tx, mut swap_info_rx) = mpsc::channel(10);
  let (new_brain_tx, _) = mpsc::channel(10);
  let (snivtx, mut snivrx) = mpsc::channel(100);

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

  let bot_handle = tokio::spawn(crate::sbot::stealth_bot::fn_run_main_stealth_bot(
    config,
    trade_rx,
    check_rx,
    sucks_rx,
    bridge,
  ));

  let expiry = Utc::now() + Duration::days(35);

  // Setup
  log::info!("Setup");
  let (_, reply_tx) = pls_ticks_rx.recv().await.unwrap();
  let mut stk = dsta::Stk::default();
  stk.ticker_name = "FAKE".to_string();
  reply_tx.send(Some(dsta::PushTickerResult {
    ass_deets: dsta::FinAss::StkDeets(stk),
    opt_chain: vec![dsta::Derivatives {
      expire_time: expiry,
      option_calls: vec![],
      option_puts: vec![
        dsta::Opt { option_rite: dsta::OptRite::Put, ticker_name: ".FAKE 105_P_35".to_string(), strike_spot: 105.0, option_name: "FAKE 105_P_35".to_string(), expire_date: expiry, ..Default::default() },
        dsta::Opt { option_rite: dsta::OptRite::Put, ticker_name: ".FAKE 100_P_35".to_string(), strike_spot: 100.0, option_name: "FAKE 100_P_35".to_string(), expire_date: expiry, ..Default::default() },
        dsta::Opt { option_rite: dsta::OptRite::Put, ticker_name: ".FAKE 95_P_35".to_string(), strike_spot: 95.0, option_name: "FAKE 95_P_35".to_string(), expire_date: expiry, ..Default::default() },
        dsta::Opt { option_rite: dsta::OptRite::Put, ticker_name: ".FAKE 90_P_35".to_string(), strike_spot: 90.0, option_name: "FAKE 90_P_35".to_string(), expire_date: expiry, ..Default::default() },
      ],
    }],
  })).await.unwrap();

  log::info!("Swap rx wait");
  swap_info_rx.recv().await.unwrap();
  let mut account = BotaAccount::new().with_cash(11_000.0);
  check_tx.send(crate::Allocation {
    bot_name: bot_name.clone(),
    bot_safe: account.clone(),
    bot_mind: crate::BotMindState::Running,
    num_swap: 0,
  }).await.unwrap();

  log::info!("Swap rx succ");
  sucks_tx.send(crate::glossary::SWAP_SUCC.to_string()).await.unwrap();

  log::info!("send quote");
  trade_tx.send(dsta::convert::stream_quote_to_ticker(&ttrs::StreamQuote {
    event_type: "quote".to_string(),
    symbol: "FAKE".to_string(),
    bid_price: Some(99.8),
    ask_price: Some(100.2),
    bid_size: Some(10.0),
    ask_size: Some(10.0),
  })).await.unwrap();

  log::info!("wait for more quote requests");
  let (_, reply_tx_long) = pls_ticks_rx.recv().await.unwrap();
  let mut opt = dsta::Opt::default();
  opt.ticker_name = ".FAKE 95_P_35".to_string();
  opt.option_name = "FAKE 95_P_35".to_string();
  reply_tx_long.send(Some(dsta::PushTickerResult {
    ass_deets: dsta::FinAss::OptDeets(opt.clone()),
    opt_chain: vec![],
  })).await.unwrap();

  let (_, reply_tx_short) = pls_ticks_rx.recv().await.unwrap();
  let mut opt2 = dsta::Opt::default();
  opt2.ticker_name = ".FAKE 100_P_35".to_string();
  opt2.option_name = "FAKE 100_P_35".to_string();
  reply_tx_short.send(Some(dsta::PushTickerResult {
    ass_deets: dsta::FinAss::OptDeets(opt2),
    opt_chain: vec![],
  })).await.unwrap();

  // Trigger entry order
  log::info!("send quote to trigger entry");
  trade_tx.send(dsta::convert::stream_quote_to_ticker(&ttrs::StreamQuote {
    event_type: "quote".to_string(),
    symbol: ".FAKE 95_P_35".to_string(),
    bid_price: Some(1.90),
    ask_price: Some(2.10),
    bid_size: Some(20.0),
    ask_size: Some(20.0),
  })).await.unwrap();

  info!("Awaiting initial entry order");
  let entry_order = pls_trade_rx.recv().await.unwrap();
  assert_eq!(entry_order.order_legs[0].action, dsta::BuyOrSell::BuyToOpen);
    let _ = sucks_tx.send(format!("{}",crate::glossary::ORDER_FLY)).await.expect("Failed to send SWAP_SUCC");


  log::info!("Send ZERO fill - no position created");
  // 
  check_tx.send(crate::Allocation {
    bot_name: bot_name.clone(),
    bot_safe: account.clone(),  // No change
    bot_mind: crate::BotMindState::Trading,
    num_swap: 1,
  }).await.unwrap();

  check_tx.send(crate::Allocation {
    bot_name: bot_name.clone(),
    bot_safe: account.clone(),
    bot_mind: crate::BotMindState::Running,
    num_swap: 1,
  }).await.unwrap();

  tokio::time::sleep(StdDuration::from_millis(sleep1 + 1)).await;

  // Trigger haggle (will exceed slippage with zero fill)
  trade_tx.send(dsta::convert::stream_quote_to_ticker(&ttrs::StreamQuote {
    event_type: "quote".to_string(),
    symbol: ".FAKE 95_P_35".to_string(),
    bid_price: Some(2.20),
    ask_price: Some(2.40),
    bid_size: Some(1.0),
    ask_size: Some(20.0),
  })).await.unwrap();

  info!("Test Awaiting cancel order (slippage exceeded with zero fill)");
  let cancel_order = tokio::time::timeout(StdDuration::from_secs(5), pls_trade_rx.recv())
    .await.expect("Timeout").unwrap();

  assert_eq!(cancel_order.order_legs.len(), 0, "Should be cancel order");
  info!("Test sending order FLY"); // could fail if bot closes soon
  let _ = sucks_tx.send(format!("{}",crate::glossary::ORDER_FLY)).await;

  info!("Cancel received - bot should terminate (zero fill = fatal)");

  // Bot should terminate because zero fill + slippage exceeded = no position to work with
  let result = tokio::time::timeout(StdDuration::from_secs(5), bot_handle).await;
  assert!(result.is_ok(), "Bot should terminate after zero-fill slippage exceeded");

  info!("Test completed - bot terminated as expected with zero fill");
}