
// trade/usta/src/stealth_bot.rs

use std::time::{Duration};
use crate::sbot::errors::{BotError, BotResult};
use crate::sbot::{f_report_info, f_report_error};



#[derive(Default, Clone, Debug, serde::Serialize, serde::Deserialize)]
pub(crate) enum StealthBotState 
{ #[default] Begins,
  Check1,
  Logic1,
  Check2,
  Check3,
  Logic2,
  Check4,
}

#[derive(Default, Clone, Debug, serde::Serialize, serde::Deserialize)]
pub(crate) struct StealthBot
{   pub m_state : StealthBotState
  , pub m_deets : dsta::PushTickerResult
  , pub m_stock : dsta::Stk // always made with a stock in mind
  , pub m_opt_1 : Option<dsta::Opt> // the option we buy 
  , pub m_opt_2 : Option<dsta::Opt> // the option we may sell
  // Things to make tracking easier so we don't need to deal with above stuff:
  , pub m_str_1 : String // option_name for opt_1, set when opt_1 found
  , pub m_str_2 : String // option_name for opt_2, set when opt_2 found
  , pub m_val_1 : f64 // opt_1 strike, set when opt_1 found
  , pub m_val_2 : f64 // opt_2 strike, set when opt_2 found
  , pub m_tik_1 : String // ticker_name for opt_1, set when opt_1 found
  , pub m_tik_2 : String // ticker_name for opt_2, set when opt_2 found
  , pub m_price : f64 // opt_1 entry price paid
  , pub m_floor : f64 // exit price trailing stop
  , pub m_nicep : f64 // price sold for opt_2
  , pub m_count : f64 // quantity entered for open/close order - original request, stays constant (so we can tell if we got partials; always use actual counts in the account for closing! )

  , pub m_entry: Option<dsta::Order> 
  , pub m_begin: Option<chrono::DateTime<chrono::Utc>> // when the bot loop / trade started

  , pub haggle_context: dsta::HaggleContext           // ← new
  , pub retry_at_delta: chrono::TimeDelta             // ← new
  , pub total_slippage: f64                           // ← new
  , pub haggle_attempt: u32                           // ← new
  , pub last_haggle_at: Option<chrono::DateTime<chrono::Utc>> // ← new

  , pub m_start: Option<chrono::DateTime<chrono::Utc>> // when the second leg / spread started}
  , pub m_exits: Option<dsta::Order> 

}

// Basic Bots really only have their constructors and everything else is the same (e.g., check out make_bot_sally_2)
// so we really can make this a macro or template function in the mod.rs probably
pub(crate) async fn f_make_bot_stealth_2
( stone: dsta::MakeStealthBot,
) -> ( tokio::task::JoinHandle<()>, crate::CtorBot )
{ let friendly_name = &stone.my_accounting.friendly_name;
  let tracking_tick = &stone.my_accounting.tracking_tick;
  let dbgspt = format!("[STEALTH_FN_MAKE2 - {}]", friendly_name);
  log::debug!
  ( "{} [ENTRY] f_make_bot_stealth_2() - tracking {}"
  , dbgspt
  , tracking_tick
  );
  log::trace!
  ( "{} verifying friendly_name follows USER_ or DEAD_ convention"
  , dbgspt
  );
  let (tell_bot_to_trade_api, told_bot_to_trade_api) = tokio::sync::mpsc::channel::<dsta::Ticker>(50);
  let (tell_bot_to_check_api, told_bot_to_check_api) = tokio::sync::mpsc::channel::<crate::Allocation>(50);
  let (tell_bot_it_sucks_api, told_bot_it_sucks_api) = tokio::sync::mpsc::channel::<String>(10);
  let (bot_to_dr, bot_bridge_to_dr_foot) = crate::fn_make_bot_api_bridge_dr_handles();
  let sperm = crate::CtorBot 
  { robot_api_brain_state: crate::Allocation 
    { bot_name: friendly_name.clone(),
      bot_safe: crate::BotaAccount::default(),
      bot_mind: crate::BotMindState::Running,
      num_swap: 0,
    },
    tell_bot_to_trade_api: tell_bot_to_trade_api,
    tell_bot_to_check_api: tell_bot_to_check_api,
    tell_bot_it_sucks_api: tell_bot_it_sucks_api,
    told_the_bot_requests: bot_bridge_to_dr_foot,
  };
  log::debug!("{} [EXIT] fn_make_bot_stealth_2", dbgspt);
  ( tokio::spawn
    ( async move
      { fn_run_main_stealth_bot
        ( stone,
          told_bot_to_trade_api,
          told_bot_to_check_api,
          told_bot_it_sucks_api,
          bot_to_dr,
        ).await;
      }
    )
  , sperm
  )
}

// The main stealth bot state machine task
pub(crate) async fn fn_run_main_stealth_bot
( constructor_stuff : dsta::MakeStealthBot
, mut told_to_trade_api : tokio::sync::mpsc::Receiver<dsta::Ticker>
, mut told_to_check_api : tokio::sync::mpsc::Receiver<crate::Allocation>
, mut told_it_sucks_api : tokio::sync::mpsc::Receiver<String>
, to_dr : crate::BridgeDrHand
)
{ let dbgspt = format!("[MAIN-STEALTH {}]", constructor_stuff.my_accounting.friendly_name);
  log::debug!("{} ENTRY! fn_run_main_stealth_bot", dbgspt);
  let mut stone = constructor_stuff;
  let mut stuff = crate::Allocation
  { bot_name: format!("{}", stone.my_accounting.friendly_name)
    , bot_safe: crate::BotaAccount::default()
    , bot_mind: crate::BotMindState::Running
    , num_swap: 0
  };

  // STEP 1: Request asset details & ticker for tracking_tick
  let msgmsg = format!("{} STEP 1: Request asset details & ticker for {}...", dbgspt, stone.my_accounting.tracking_tick);
  let temp_brain = StealthBot::default();
  let _ = f_report_info(&stone,&temp_brain,&stuff,&to_dr,&msgmsg).await;
  let deets =
  { let (tx_asset, mut rx_asset) = tokio::sync::mpsc::channel(2);
    if to_dr.tell_dr_pls_ticks.send((stone.my_accounting.tracking_tick.clone(), tx_asset)).await.is_err()
    { let errmsg = format!("{} Ticker request send failed",dbgspt);
      let temp_brain = StealthBot::default();
      let _ = f_report_error (&stone,&temp_brain,&stuff,&to_dr,&errmsg).await;
      return;
    }
    match tokio::time::timeout(Duration::from_secs(5), rx_asset.recv()).await
    { Ok(Some(Some(d))) => d,
      _ =>
      { let errmsg = format!("{} Asset details request 5 second timeout or channel closed", dbgspt);
        let temp_brain = StealthBot::default();
        let _ = f_report_error (&stone,&temp_brain,&stuff,&to_dr,&errmsg).await;
        return;
      }
    }
  };

  let msgmsg = format!("{} STEP 2: Attempting to load bot brain state...", dbgspt);
  let _ = f_report_info(&stone,&temp_brain,&stuff,&to_dr,&msgmsg).await;
  let (mut brain, is_fresh_bot) = match crate::sbot::fn_load_bot_state(&stone.my_accounting.friendly_name).await 
  {
    Ok ( ( load_stone, load_brain, load_stuff ) ) => 
    { stone = load_stone; stuff = load_stuff;
      let msgmsg = format!("{} [LOAD SUCCESS] Loaded existing state from disk", dbgspt);
      let _ = f_report_info(&stone, &load_brain, &stuff, &to_dr, &msgmsg).await;
      ( load_brain, false )
    }
    Err(e) => 
    { log::trace!("{} STEP 2: Attempting to load bot brain state error = {:#?} => fresh stealth bot!", dbgspt, e);
      let fresh = StealthBot 
      { m_state: StealthBotState::Begins,
        m_deets: deets.clone(),
        m_stock: 
        { if let dsta::FinAss::StkDeets(ref fsd) = deets.ass_deets
          { fsd.clone()
          }
          else
          { let errmsg = "Asset details request indicates bad underlying".to_string();
            let _ = f_report_error (&stone,&temp_brain,&stuff,&to_dr,&errmsg).await;
            return;
          }
        },
        m_opt_1: None,
        m_opt_2: None,
        m_str_1: String::new(),
        m_str_2: String::new(),
        m_tik_1: String::new(),
        m_tik_2: String::new(),
        m_val_1: 0.0,
        m_val_2: 0.0,
        m_price: 0.0,
        m_floor: 0.0,
        m_nicep: 0.0,
        m_count: 0.0,
        m_begin: Some(chrono::Utc::now()),
        m_start: None,
        m_entry: None,
        m_exits: None,
        haggle_context: dsta::HaggleContext::default(),  // or initialize properly if needed
        retry_at_delta: stone.my_accounting.haggle_action.get_retry_period(),
        total_slippage: 0.0,
        haggle_attempt: 0,
        last_haggle_at: None,
      };
      (fresh, true)
    }
  };

  // Step 3 - get cash & tickers if fresh bot / only reqest tickers for saved bot
  if is_fresh_bot
  { let msgmsg = format!("{} STEP 3.0a: Requesting initial cash allocation for fresh bot...", dbgspt);
    let _ = f_report_info(&stone,&brain,&stuff,&to_dr,&msgmsg).await;
    let _ = to_dr.tell_dr_swap_info.send
    ( crate::BotSwaper
      { source_offering : Some
        ( crate::SwapWat
          { who: format!("{}", stone.my_accounting.friendly_name)
          , wat: dsta::PositionsList::default()
          , how: 0.0
          }
        )
      , target_released : Some
        ( crate::SwapWat
          { who: format!("USER_CASH")
          , wat: dsta::PositionsList::default()
          , how: stone.my_cash_alloc
          }
        )
      }
    ).await;

    let msgmsg = format!("{} STEP 3.1a: Confirm cash via wait for SWAP reply or timeout...", dbgspt);
    let _ = f_report_info(&stone,&brain,&stuff,&to_dr,&msgmsg).await;
    let m_begin = Some(chrono::Utc::now());
    loop
    { let swap_reply = tokio::select!
      { res = told_it_sucks_api.recv() =>
        { match res
          { Some(msg) =>
            { log::trace!("Stealth bot got a message from robo: {}", msg);
              Some(msg)
            }
          , None =>
            { log::warn!("Stealth Bot got a closed pipe from robo:");
              None
            }
          }
        }
        _ = tokio::time::sleep(tokio::time::Duration::from_secs(1)) =>
        { let errmsg = format!("{} ERROR in STEP 3.1a : Stealth Bot got no swap reply from dr robo after 1 second !", dbgspt);
          let _ = f_report_error(&stone, &brain, &stuff, &to_dr, &errmsg).await;
          None
        }
      };
      let zombie_reason = if let Some(val) = swap_reply
      { if val == crate::glossary::SWAP_SUCC
        { let msgmsg = format!("{} SUCCESS in Step 3.1a : Initial cash allocation received.", dbgspt);
          let _ = f_report_info(&stone,&brain,&stuff,&to_dr,&msgmsg).await;
          break;
        }
        else if val == crate::glossary::SWAP_FAIL
        { let errmsg = format!("{} ERROR in Initial Step 3.1a : cash allocation rejected (SWAP FAIL)!", dbgspt);
          let _ = f_report_error (&stone, &brain, &stuff, &to_dr, &errmsg).await;
          Some(errmsg)
        }
        else
        { let errmsg = format!("{} WARNING in Step 3.1a : Unexpected message on sucks channel during init: {}", dbgspt, val);
          log::warn!("{}", errmsg);
          continue;
        }
      }
      else if chrono::Utc::now().signed_duration_since(m_begin.unwrap()) > chrono::Duration::seconds(2)
      { let errmsg = format!("{} ERROR IN Step 3.1a : Timeout/channel closed on initial cash swap",dbgspt);
        let _ = f_report_error (&stone, &brain, &stuff, &to_dr, &errmsg).await;
        Some(errmsg)
      }
      else
      { continue; 
      };
    
      if let Some(reason) = zombie_reason
      { log::trace!("{} Step 3.1a : [ZOMBIE] Reporting zombied state via new_brain channel: {}", dbgspt, reason);
        stuff.bot_mind =  crate::BotMindState::Zombied;
        let _ = to_dr.tell_dr_new_brain.send(stuff.clone()).await;
        let errmsg = format!("{} [EXIT] [ZOMBIE] in Step 3.1a : due to initial allocation failure!", dbgspt);
        let _ = f_report_error (&stone, &brain, &stuff, &to_dr, &errmsg).await;
        return;
      }
    }

    let msgmsg = format!("{} STEP 3.2a: Confirm cash via wait for Account update or timeout...", dbgspt);
    let _ = f_report_info(&stone,&brain,&stuff,&to_dr,&msgmsg).await;
    let m_begin = Some(chrono::Utc::now());
    loop 
    { // Timeout after 2 seconds
      if chrono::Utc::now().signed_duration_since(m_begin.unwrap()) > chrono::Duration::seconds(2) 
      { stuff.bot_mind =  crate::BotMindState::Zombied;
        let _ = to_dr.tell_dr_new_brain.send(stuff.clone()).await;
        let errmsg = format!("{} [EXIT] [ZOMBIE] in STEP 3.2a : due to cash allocation timeout", dbgspt);
        let _ = f_report_error (&stone, &brain, &stuff, &to_dr, &errmsg).await;
        return;
      }

      let alloc_update = tokio::select! 
      { res = told_to_check_api.recv() => 
        { match res 
          { Some(alloc) => 
            { log::trace!("{} STEP 3.2a :Received account update for bot: {}", dbgspt, alloc.bot_name);
              Some(alloc)
            },
            None => 
            { log::warn!("{} STEP 3.2a :Account update channel closed", dbgspt);
              None // alloc update is none
            }
          }
        },
        _ = tokio::time::sleep(tokio::time::Duration::from_secs(1)) => 
        { let errmsg = format!("{} ERROR in STEP 3.2a : No account update received after 1 second", dbgspt);
          let _ = f_report_error (&stone, &brain, &stuff, &to_dr, &errmsg).await;
          None
        }
      };

      // If no update, continue or timeout
      let Some(alloc) = alloc_update else 
      { continue;
      };

      // Only process updates for this bot
      if alloc.bot_name != stuff.bot_name 
      { log::warn!("{} STEP 3.2a : Ignoring account update for bot: {}", dbgspt, alloc.bot_name);
        continue;
      }

      // Verify the cash amount
      if alloc.bot_safe.m_cash >= (stone.my_cash_alloc - 1.00)
      { let msgmsg = format!("{} SUCCESS in STEP 3.2a : Cash allocation confirmed: ${:.2} (requested: ${:.2})", dbgspt, alloc.bot_safe.m_cash, stone.my_cash_alloc);
        let _ = f_report_info(&stone, &brain, &stuff, &to_dr, &msgmsg).await;
        stuff = alloc;
        break;
      } 
      else 
      { stuff.bot_mind = crate::BotMindState::Zombied;
        let _ = to_dr.tell_dr_new_brain.send(stuff.clone()).await;
        let errmsg = format!("{} [EXIT] [ZOMBIE] in STEP 3.2a Cash allocation mismatch: got ${:.2}, expected ${:.2}", dbgspt, alloc.bot_safe.m_cash, stone.my_cash_alloc);
        let _ = f_report_error (&stone, &brain, &stuff, &to_dr, &errmsg).await;
        return;
      }
    }

    // ───────────────────────────────────────────────────────────────
    // STEP 3.3a: Only for fresh bots — wait for underlying quote → calc ATM → select legs
    // ───────────────────────────────────────────────────────────────
    let msgmsg = format!("{} STEP 3.3a: Waiting for first underlying quote to select ATM options...", dbgspt);
    let _ = f_report_info(&stone, &brain, &stuff, &to_dr, &msgmsg).await;

    let underlying_ticker = loop 
    { tokio::select! 
      { res = told_to_trade_api.recv() => 
        { match res 
          { Some(ticker) if ticker.name == stone.my_accounting.tracking_tick => 
            { let msgmsg = format!("{} STEP 3.3a: Received first underlying quote: bid=${:.2}, ask=${:.2}", 
                dbgspt, ticker.bid, ticker.ask);
              let _ = f_report_info(&stone, &brain, &stuff, &to_dr, &msgmsg).await;
              break ticker;
            }
            Some(ticker) => 
            { log::debug!("{} STEP 3.3a: Ignoring non-underlying ticker: {}", dbgspt, ticker.name);
              continue;
            }
            None => 
            { let errmsg = format!("{} STEP 3.3a: trade channel closed while waiting for underlying quote", dbgspt);
              let _ = f_report_error(&stone, &brain, &stuff, &to_dr, &errmsg).await;
              stuff.bot_mind = crate::BotMindState::Zombied;
              let _ = to_dr.tell_dr_new_brain.send(stuff).await;
              return;
            }
          }
        }
        _ = tokio::time::sleep(tokio::time::Duration::from_secs(10)) => 
        {
          let errmsg = format!("{} STEP 3.3a: Timeout (10s) waiting for first underlying quote", dbgspt);
          let _ = f_report_error(&stone, &brain, &stuff, &to_dr, &errmsg).await;
          stuff.bot_mind = crate::BotMindState::Zombied;
          let _ = to_dr.tell_dr_new_brain.send(stuff).await;
          return;
        }
      }
    };

    let underlying_price = (underlying_ticker.bid + underlying_ticker.ask) / 2.0;
    let msgmsg = format!("{} STEP 3.3a: Underlying mid-price calculated = ${:.2}", dbgspt, underlying_price);
    let _ = f_report_info(&stone, &brain, &stuff, &to_dr, &msgmsg).await;

    // Find suitable expiry
    let now = chrono::Utc::now();
    let min_expiry = now + chrono::Duration::days(stone.option_expire as i64);

    let deriv = match deets.opt_chain.iter().find(|d| d.expire_time >= min_expiry) {
        Some(d) => d,
        None => {
            let errmsg = format!("{} STEP 3.3a: No option chain found with expiry ≥ {} days", 
                dbgspt, stone.option_expire);
            let _ = f_report_error(&stone, &brain, &stuff, &to_dr, &errmsg).await;
            stuff.bot_mind = crate::BotMindState::Seppuku;
            let _ = to_dr.tell_dr_new_brain.send(stuff).await;
            return;
        }
    };

    let options = match stone.stonk_feeling {
        dsta::MarketDirection::Corrects => &deriv.option_puts,
        dsta::MarketDirection::GetStonk   => &deriv.option_calls,
        dsta::MarketDirection::Sideways   => {
            let errmsg = format!("{} STEP 3.3a: Sideways direction not supported for stealth bot", dbgspt);
            let _ = f_report_error(&stone, &brain, &stuff, &to_dr, &errmsg).await;
            stuff.bot_mind = crate::BotMindState::Seppuku;
            let _ = to_dr.tell_dr_new_brain.send(stuff).await;
            return;
        }
    };

    // ── Find ATM ───────────────────────────────────────────────────────
    let mut atm_index = 0;
    let mut min_diff = f64::MAX;

    for (i, opt) in options.iter().enumerate() {
        let diff = (opt.strike_spot - underlying_price).abs();
        if diff < min_diff {
            min_diff = diff;
            atm_index = i;
        }
    }

    let msgmsg = format!("{} STEP 3.3a: ATM strike found at ${:.0} (index {}, diff={:.2})", 
        dbgspt, options[atm_index].strike_spot, atm_index, min_diff);
    let _ = f_report_info(&stone, &brain, &stuff, &to_dr, &msgmsg).await;

    // ── Select legs with clamping ──────────────────────────────────────
    let (opt_1_index, opt_2_index) = match stone.stonk_feeling {
        dsta::MarketDirection::Corrects | dsta::MarketDirection::GetStonk => {
            let target_1 = atm_index + (stone.option_bucket as usize);
            let opt_1 = target_1.min(options.len() - 1);

            let target_2 = opt_1.saturating_sub(stone.spread_bucket as usize);
            let opt_2 = target_2.max(0);

            (opt_1, opt_2)
        }
        _ => unreachable!(),
    };

    if opt_1_index != atm_index + (stone.option_bucket as usize) {
        let msgmsg = format!("{} STEP 3.3a: option_bucket clamped — using index {} instead of {}", 
            dbgspt, opt_1_index, atm_index + stone.option_bucket as usize);
        let _ = f_report_info(&stone, &brain, &stuff, &to_dr, &msgmsg).await;
    }

    if opt_2_index != opt_1_index.saturating_sub(stone.spread_bucket as usize) {
        let msgmsg = format!("{} STEP 3.3a: spread_bucket clamped — using index {} instead of {}", 
            dbgspt, opt_2_index, opt_1_index.saturating_sub(stone.spread_bucket as usize));
        let _ = f_report_info(&stone, &brain, &stuff, &to_dr, &msgmsg).await;
    }

    // ── Assign selected options ────────────────────────────────────────
    brain.m_str_1  = options[opt_1_index].option_name.clone();
    brain.m_str_2  = options[opt_2_index].option_name.clone();
    brain.m_tik_1  = options[opt_1_index].ticker_name.clone();
    brain.m_tik_2  = options[opt_2_index].ticker_name.clone();
    brain.m_val_1  = options[opt_1_index].strike_spot;
    brain.m_val_2  = options[opt_2_index].strike_spot;
    brain.m_opt_1  = Some(options[opt_1_index].clone());
    brain.m_opt_2  = Some(options[opt_2_index].clone());

    let msgmsg = format!("{} STEP 3.3a: Selected legs → long: {} (${:.0}), short: {} (${:.0})",
        dbgspt, brain.m_str_1, brain.m_val_1, brain.m_str_2, brain.m_val_2);
    let _ = f_report_info(&stone, &brain, &stuff, &to_dr, &msgmsg).await;

    // ── Request tickers for both options ───────────────────────────────
    for ticker_name in [&brain.m_tik_1, &brain.m_tik_2] {
        let msgmsg = format!("{} STEP 3.3a: Requesting ticker data for {}", dbgspt, ticker_name);
        let _ = f_report_info(&stone, &brain, &stuff, &to_dr, &msgmsg).await;

        let (tx, mut rx) = tokio::sync::mpsc::channel(1);
        if to_dr.tell_dr_pls_ticks.send((ticker_name.to_string(), tx)).await.is_err() {
            let errmsg = format!("{} STEP 3.3a: Failed to send ticker request for {}", dbgspt, ticker_name);
            let _ = f_report_error(&stone, &brain, &stuff, &to_dr, &errmsg).await;
            stuff.bot_mind = crate::BotMindState::Zombied;
            let _ = to_dr.tell_dr_new_brain.send(stuff).await;
            return;
        }

        match tokio::time::timeout(Duration::from_secs(5), rx.recv()).await {
            Ok(Some(Some(_deets))) => {
                let msgmsg = format!("{} STEP 3.3a: Successfully received ticker details for {}", dbgspt, ticker_name);
                let _ = f_report_info(&stone, &brain, &stuff, &to_dr, &msgmsg).await;
            }
            _ => {
                let errmsg = format!("{} STEP 3.3a: Timeout / error receiving ticker for {}", dbgspt, ticker_name);
                let _ = f_report_error(&stone, &brain, &stuff, &to_dr, &errmsg).await;
                stuff.bot_mind = crate::BotMindState::Zombied;
                let _ = to_dr.tell_dr_new_brain.send(stuff).await;
                return;
            }
        }
    }

    let msgmsg = format!("{} STEP 3.3a: Complete — initial options selected and tickers requested", dbgspt);
    let _ = f_report_info(&stone, &brain, &stuff, &to_dr, &msgmsg).await;
  }

  else // We need to request our prior FinAss::ticker_name() for stk and opt 1 & 2
  { let msgmsg =format!("{} STEP 3.0b: Requesting ticker updates for prior options...", dbgspt);
    let _ = f_report_info(&stone, &brain, &stuff, &to_dr, &msgmsg).await;
    if let Some(x) = brain.m_opt_1.as_ref()
    { let (tx_asset, mut rx_asset) = tokio::sync::mpsc::channel(2);
      if to_dr.tell_dr_pls_ticks.send((x.ticker_name.clone(), tx_asset)).await.is_err()
      { stuff.bot_mind = crate::BotMindState::Zombied;
        let _ = to_dr.tell_dr_new_brain.send(stuff.clone()).await;
        let errmsg = format!("{} [EXIT] [ZOMBIED] at Step 3.0b  :Ticker opt_1 request send failed",dbgspt);
        let _ = f_report_error (&stone,&brain,&stuff,&to_dr,&errmsg).await;
        return;
      }
      match tokio::time::timeout(Duration::from_secs(5), rx_asset.recv()).await
      { Ok(Some(Some(d))) => {log::trace!("{} SUCCESS at STEP 3.0b Ticker opt_1 found: {:#?}",dbgspt, d );},
        _ =>
        { stuff.bot_mind = crate::BotMindState::Zombied;
          let _ = to_dr.tell_dr_new_brain.send(stuff.clone()).await;
          let errmsg = format!("{} [EXIT] [ZOMBIED] at Step 3.0b : Asset opt_1 details request 5 second timeout or channel closed", dbgspt);
          let _ = f_report_error (&stone,&brain,&stuff,&to_dr,&errmsg).await;
          return;
        }
      }
    }
    else
    { stuff.bot_mind = crate::BotMindState::Zombied;
      let _ = to_dr.tell_dr_new_brain.send(stuff.clone()).await;
      let errmsg = format!("{} [EXIT] [ZOMBIED] at STEP 3.0b : Asset opt_1 does not exit", dbgspt);
      let _ = f_report_error (&stone,&brain,&stuff,&to_dr,&errmsg).await;
      return;
    }

    if let Some(x) = brain.m_opt_2.as_ref()
    { let (tx_asset, mut rx_asset) = tokio::sync::mpsc::channel(2);
      if to_dr.tell_dr_pls_ticks.send((x.ticker_name.clone(), tx_asset)).await.is_err()
      { stuff.bot_mind = crate::BotMindState::Zombied;
        let _ = to_dr.tell_dr_new_brain.send(stuff.clone()).await;
        let errmsg = format!("{} [EXIT] [ZOMBIED] at STEP 3.0b : Ticker opt_2 request send failed",dbgspt);
        let _ = f_report_error (&stone,&brain,&stuff,&to_dr,&errmsg).await;
        return;
      }
      match tokio::time::timeout(Duration::from_secs(5), rx_asset.recv()).await
      { Ok(Some(Some(d))) => {log::trace!("{} [EXIT] [ZOMBIED] at Step 3.0b : Ticker opt_2 found : {:#?}" ,dbgspt, d );},
        _ =>
        { stuff.bot_mind = crate::BotMindState::Zombied;
          let _ = to_dr.tell_dr_new_brain.send(stuff.clone()).await;
          let errmsg = format!("{} 3b Asset opt_2 details request 5 second timeout or channel closed", dbgspt);
          let _ = f_report_error (&stone,&brain,&stuff,&to_dr,&errmsg).await;
          return;
        }
      }
    }
    else
    { stuff.bot_mind = crate::BotMindState::Zombied;
      let _ = to_dr.tell_dr_new_brain.send(stuff.clone()).await;
      let errmsg = format!("{} [EXIT] [ZOMBIED] at STEP 3.0b Asset opt_2 does not exit", dbgspt);
      let _ = f_report_error (&stone,&brain,&stuff,&to_dr,&errmsg).await;
      return;
    }

    let msgmsg =format!("{} STEP 3.0: Stealth bot is waiting for some action...", dbgspt);
    let _ = f_report_info(&stone, &brain, &stuff, &to_dr, &msgmsg).await;
  }

  // STEP 4: Start the main event loop
  let msgmsg =format!("{} STEP 4: Start forever loop...", dbgspt);
  let _ = f_report_info(&stone, &brain, &stuff, &to_dr, &msgmsg).await;
  loop
  { tokio::select!
    { res = told_to_trade_api.recv() =>
      { log::trace!("{} FOREVER LOOP Told to trade api recv() = {:#?}", dbgspt, res);
        if let Some(ticker) = res
        { if ticker.name == stone.my_accounting.tracking_tick
          || ticker.name == brain.m_tik_1
          || ticker.name == brain.m_tik_2
          { if let Err(e) = handle_quote(&ticker, &mut stone, &mut brain, &mut stuff, &to_dr, &mut told_it_sucks_api).await
            { let errmsg = format!("Quote handling failed: {}", e);
              let _ = f_report_error (&mut stone, &mut brain, &mut stuff, &to_dr, &errmsg).await;
              break;
            }
          }
          else
          { //panic!("Illegal ticker!");
          }
        }
        else
        { let errmsg = "trade channel closed".to_string();
          let _ = f_report_error (&mut stone, &mut brain, &mut stuff, &to_dr, &errmsg).await;
          break;
        }
      }
      res = told_to_check_api.recv() =>
      { log::trace!("{} FOREVER LOOP Told to check account recv() = {:#?}", dbgspt, res);
        if let Some(alloc) = res
        { if alloc.bot_name == stone.my_accounting.friendly_name
          { if let Err(e) = handle_check(&alloc, &mut stone, &mut brain, &mut stuff, &to_dr, &mut told_it_sucks_api).await
            { let errmsg = format!("Account update handling failed: {}", e);
              let _ = f_report_error (&mut stone, &mut brain, &mut stuff, &to_dr, &errmsg).await;
              break;
            }
          }
        }
      }
      res = told_it_sucks_api.recv() =>
      { log::trace!("{} FOREVER LOOP Told it sucks from dr robo recv() = {:#?}", dbgspt, res);
        if let Some(msg) = res
        { if msg == "DIE"
          { log::debug!("{} [EXIT] Received DIE", dbgspt);
            break;
          }
        }
      }
      _ = tokio::time::sleep(tokio::time::Duration::from_secs(30)) =>
      { let msgmsg =format!("{} STEP 4: Stealth bot is waiting for some action...", dbgspt);
        let _ = f_report_info(&stone, &brain, &stuff, &to_dr, &msgmsg).await;
      }
    }
  }

  let msgmsg =format!("{} STEP 5: Stealth bot shutting down!", dbgspt);
  let _ = f_report_info(&stone, &brain, &stuff, &to_dr, &msgmsg).await;
  stuff.bot_mind = crate::BotMindState::Zombied;
  let _ = to_dr.tell_dr_new_brain.send(stuff).await;
}

//===================================================================
// Core ROBO API handlers 
//===================================================================

async fn handle_check // incoming update for a bot allocation
( check: &crate::Allocation,
  stone: &mut dsta::MakeStealthBot,
  brain: &mut StealthBot,
  stuff: &mut crate::Allocation,
  to_dr: &crate::BridgeDrHand,
  told_it_sucks_api: &mut tokio::sync::mpsc::Receiver<String>,
) -> BotResult
{ if check.bot_name != stuff.bot_name
  { log::warn!("Stealth bot got unexpected allocation update: {:#?}", check);
    return Ok(());
  }
  if check.bot_mind != stuff.bot_mind
  { return state_machine_mind(check,stone,brain,stuff,to_dr, told_it_sucks_api).await;
  }
  else
  { *stuff = check.clone();
    return state_machine_tick(stone,brain,stuff,to_dr, told_it_sucks_api).await;
  }
}

async fn handle_quote // incoming quote update
( ticks: &dsta::Ticker,
  stone: &mut dsta::MakeStealthBot,
  brain: &mut StealthBot,
  stuff: &mut crate::Allocation,
  to_dr: &crate::BridgeDrHand,
  told_it_sucks_api: &mut tokio::sync::mpsc::Receiver<String>,
) -> BotResult
{ let dbgspt = format!("[STEALTH QUOTE {}]", stone.my_accounting.friendly_name);
  let mut found = false;
  if brain.m_stock.ticker_name == ticks.name 
  { found = true;
    log::trace!("{} setting brain.m_stock...", dbgspt);
    brain.m_stock.last_ticker = Some(ticks.clone())
  }
  if !found 
  { if let Some(opt) = brain.m_opt_1.as_mut()
    { if opt.ticker_name == ticks.name
      { log::trace!("{} it's brain.m_opt_1...", dbgspt);
        found = true;
        match &mut opt.last_ticker
        { Some(lt) =>
          { if let Some(tp) = ticks.theo_price
            { lt.theo_price = Some(tp);
            }
            else
            { let preserved_theo = lt.theo_price;
              *lt = ticks.clone();
              lt.theo_price = preserved_theo;
            }
          }
          None =>
          { let mut fake = ticks.clone();
            if let Some(tp) = ticks.theo_price
            { fake.bid = tp - 0.02;
              fake.ask = tp + 0.02;
              fake.bid_size = 10.0;
              fake.ask_size = 10.0;
            }
            opt.last_ticker = Some(fake)
          }
        }
      }
    }
  }

  if !found 
  { if let Some(opt) = brain.m_opt_2.as_mut()
    { if opt.ticker_name == ticks.name
      { found = true;
        log::trace!("{} it's m_opt_2...", dbgspt);
        match &mut opt.last_ticker
        { Some(lt) =>
          { if let Some(tp) = ticks.theo_price
            { lt.theo_price = Some(tp);
            }
            else
            { let preserved_theo = lt.theo_price;
              *lt = ticks.clone();
              lt.theo_price = preserved_theo;
            }
          }
          None =>
          { let mut fake = ticks.clone();
            if let Some(tp) = ticks.theo_price
            { fake.bid = tp - 0.02;
              fake.ask = tp + 0.02;
              fake.bid_size = 10.0;
              fake.ask_size = 10.0;
            }
            opt.last_ticker = Some(fake)
          }
        }
      }
    }
  }

  if !found
  { let errmsg = format!("{} handle quote got garbage ticker; ticker = {:#?}; \n brain = {:#?}"
    , dbgspt
    , ticks
    , brain
    );
    Err(BotError::Other(errmsg))
  }
  else
  { state_machine_tick(stone,brain,stuff,to_dr, told_it_sucks_api).await
  }
}

//===================================================================
// How to process from the core robo api - from mind or ticker
//===================================================================


// This is for handling an external mind transition and alerts.
// this must set *stuff to check in every path
// This is a great skeleton to use for any bot.
async fn state_machine_mind
( check: &crate::Allocation,
  stone: &mut dsta::MakeStealthBot,
  brain: &mut StealthBot,
  stuff: &mut crate::Allocation,
  to_dr: &crate::BridgeDrHand,
  told_it_sucks_api: &mut tokio::sync::mpsc::Receiver<String>,

) -> BotResult
{ let dbgspt = format!("[MIND_SWAP {}]", stone.my_accounting.friendly_name);
  let timestamp = chrono::Utc::now().format("%Y-%m-%d %H:%M:%S%.f").to_string();
  let msgmsg = format!("{} [{}] Stealth bot mind is transitioning to {:#?} from {:#?}", dbgspt, timestamp, check.bot_mind, stuff.bot_mind); let _ = f_report_info(&stone, &brain, &stuff, &to_dr, &msgmsg).await;
  match check.bot_mind
  { crate::BotMindState::Running =>
    { // if we transition to running, just continue as normal
      match stuff.bot_mind
      { crate::BotMindState::Trading =>
        { let msgmsg = format!("{} mind transition indicates a trade completed!",dbgspt); let _ = f_report_info(&stone, &brain, &stuff, &to_dr, &msgmsg).await;
          *stuff = check.clone();
          return state_machine_tick(stone,brain,stuff,to_dr, told_it_sucks_api).await;
        }
        crate::BotMindState::Stopped =>
        { let msgmsg = format!("{} mind transition indicates a (from stopped) save request!",dbgspt); let _ = f_report_info(&stone, &brain, &stuff, &to_dr, &msgmsg).await;
          *stuff = check.clone();
          return crate::sbot::fn_save_bot_state(stone,brain,stuff,to_dr).await;
        }
        crate::BotMindState::Seppuku | crate::BotMindState::Zombied  =>
        { *stuff = check.clone();
          let reason = format!("Illegal transition to {:#?} while current is {:#?}",
            check.bot_mind, stuff.bot_mind);
          let errmsg = format!("{} [{}] {}", dbgspt, timestamp, reason);
          return Err(BotError::InvalidStateTransition(errmsg));
        }
        crate::BotMindState::Running =>
        { log::warn!("Impossible code");
          *stuff = check.clone();
        }
      }
    }
    crate::BotMindState::Trading =>
    { let msgmsg = format!("{} mind transition indicates successfull order dispatch!", dbgspt); let _ = f_report_info(&stone, &brain, &stuff, &to_dr, &msgmsg).await;
      *stuff = check.clone(); // MUST BE DONE before this function returns
    }
    crate::BotMindState::Stopped =>
    { let msgmsg = format!("mind transition indicates a (to stopped) save request!"); let _ = f_report_info(&stone, &brain, &stuff, &to_dr, &msgmsg).await;
      *stuff = check.clone();
      return crate::sbot::fn_save_bot_state(stone,brain,stuff,to_dr).await;
    }
    crate::BotMindState::Seppuku =>
    { *stuff = check.clone();
      let reason = "Mind transition to Seppuku - self-terminate requested".to_string();
      let errmsg = format!("{} [{}] {}", dbgspt, timestamp, reason);
      return Err(BotError::Other(errmsg));
    }
    crate::BotMindState::Zombied =>
    { *stuff = check.clone();
      let reason = "Mind transition to Zombied - external kill requested".to_string();
      let errmsg = format!("{} [{}] {}", dbgspt, timestamp, reason);
      return Err(BotError::Other(errmsg));
    }
  }
  Ok(())
}

async fn state_machine_tick
( stone: &mut dsta::MakeStealthBot,
  brain: &mut StealthBot,
  stuff: &mut crate::Allocation,
  to_dr: &crate::BridgeDrHand,
  told_it_sucks_api: &mut tokio::sync::mpsc::Receiver<String>,
) -> BotResult
{ // Tickers are updated!
  match brain.m_state
  { StealthBotState::Begins => f_run_begins(stone,brain,stuff,to_dr, told_it_sucks_api).await?,
    StealthBotState::Check1 => f_run_check1(stone,brain,stuff,to_dr).await?,
    StealthBotState::Logic1 => f_run_logic1(stone,brain,stuff,to_dr, told_it_sucks_api).await?,
    StealthBotState::Check2 => f_run_check2(stone,brain,stuff,to_dr).await?,
    StealthBotState::Check3 => f_run_check3(stone,brain,stuff,to_dr).await?,
    StealthBotState::Logic2 => f_run_logic2(stone,brain,stuff,to_dr, told_it_sucks_api).await?,
    StealthBotState::Check4 => f_run_check4(stone,brain,stuff,to_dr).await?,
  }
  Ok(())
}

//===================================================================
// Core State Machines processes for stealth bot behavior
//===================================================================

// submit order
// submit order
async fn f_run_begins
( stone: &mut dsta::MakeStealthBot,
  brain: &mut StealthBot,
  stuff: &mut crate::Allocation,
  to_dr: &crate::BridgeDrHand,
  told_it_sucks_api: &mut tokio::sync::mpsc::Receiver<String>,
) -> BotResult
{ 
  let dbgspt = format!("[STEALTH BEGINS {}]", stone.my_accounting.friendly_name);
  let timestamp = chrono::Utc::now().format("%Y-%m-%d %H:%M:%S%.f").to_string();

  log::debug!("{} [{}] ENTRY! - generate entry order", dbgspt, timestamp);

  let opt_1 = if let Some(t) = &brain.m_opt_1 { t } else { return Ok(()); };
  
  let last_ticker = if let Some(lt) = &opt_1.last_ticker { lt } else
  { log::warn!("{} [{}] No ticker data yet for opt_1 - delaying order", dbgspt, timestamp);
    return Ok(());
  };

  let (use_max, pct_limit, dollar_limit) = stone.my_accounting.max_cash_risk;

  let pct_cash = stuff.bot_safe.m_cash * (pct_limit / 100.0);
  let limit_cash = if use_max 
  { pct_cash.max(dollar_limit) 
  } else 
  { pct_cash.min(dollar_limit) 
  };
  let available_cash = limit_cash.min(stuff.bot_safe.m_cash);

  if available_cash < 100.0
  { let errmsg = format!
    ( "{} [{}] Available_cash under risk rules too small: {:.2}",
      dbgspt, timestamp, available_cash
    );
    log::debug!("{} [{}] BAD EXIT! - going to zombie (insufficient cash)", dbgspt, timestamp);
    return Err(BotError::Other(errmsg));
  }

  let target_price = if stone.use_theo_cost
  { if let Some(tp) = last_ticker.theo_price
    { tp + 0.01
    }
    else
    { log::warn!("{} [{}] EXIT! stealth bot startup delay due to lacking theo price", dbgspt, timestamp);
      return Ok(());
    }
  }
  else
  { let spread = last_ticker.ask - last_ticker.bid;
    let bid75 = 0.75 * spread;
    let tp = last_ticker.bid + bid75;
    if tp <= 0.01
    { let errmsg = format!(
        "{} [{}] [MAKE ORDER FAIL] Invalid target price: {:.2}",
        dbgspt, timestamp, tp
      );
      log::debug!("{} [{}] BAD EXIT! - going to zombie (invalid price)", dbgspt, timestamp);
      return Err(BotError::Other(errmsg));
    }
    tp
  };

  let max_contracts = (available_cash / (target_price * 100.0)).floor();
  let qty = max_contracts;

  if qty <= 0.0
  { let errmsg = format!(
      "{} [{}] [MAKE ORDER FAIL] Quantity <= 0 (max={:.0}, price={:.2}, avail_cash={:.2})",
      dbgspt, timestamp, max_contracts, target_price, available_cash
    );
    log::debug!("{} [{}] BAD EXIT! - going to zombie (zero quantity)", dbgspt, timestamp);
    return Err(BotError::Other(errmsg));
  }

  brain.m_count = qty;

  let order_leg = dsta::OrderLeg
  { buy_what: dsta::FinAss::OptDeets(brain.m_opt_1.as_ref().unwrap().clone()),
    quantity: qty,
    remaining: qty,
    action: dsta::BuyOrSell::BuyToOpen,
    price: target_price
  };

  let order_cost = -target_price * qty * 100.0;

  let base_order = dsta::Order
  { order_legs: vec![order_leg],
    order_type: dsta::MarketOrLimit::Limit,
  };

  // ─────────────────────────────────────────────────────────────────────
  // CREATE INITIAL HAGGLE ORDER (like Sally does)
  // ─────────────────────────────────────────────────────────────────────
  let haggle_method = match &stone.my_accounting.haggle_action {
    dsta::HaggleAction::JustCancel(m) | dsta::HaggleAction::AnOyVeyJew(m) => m,
  };

  let (initial_order, initial_ctx) = dsta::create_initial_haggle_order(
    &base_order,
    haggle_method,
  );

  // Store the initial haggle order and context
  brain.m_entry = Some(initial_order.clone());
  brain.haggle_context = initial_ctx;
  brain.total_slippage = 0.0;
  brain.haggle_attempt = 0;
  brain.retry_at_delta = stone.my_accounting.haggle_action.get_retry_period();
  
  let msgmsg = format!(
    "{} [{}] Initial haggle order created | spread: {:.4} | method: {:?}",
    dbgspt, timestamp, brain.haggle_context.initial_spread, haggle_method
  );
  let _ = f_report_info(stone, brain, stuff, to_dr, &msgmsg).await;

  // ─────────────────────────────────────────────────────────────────────
   
  let major_info = format!(
    "{} [{}] [MAKE ORDER SEND] BuyToOpen {} x{:.0} @ {:.2} (cost {:.0}, avail_cash={:.2})",
    dbgspt, timestamp, brain.m_str_1, qty, initial_order.order_legs[0].price, order_cost, available_cash
  );
  let msgmsg = format!("{}", major_info); 
  let _ = f_report_info(&stone, &brain, &stuff, &to_dr, &msgmsg).await;

  to_dr.tell_dr_pls_trade
    .send(initial_order.clone())
    .await
    .map_err(|e| 
    { log::debug!("{} [{}] BAD EXIT! - going to zombie (order send failed)", dbgspt, timestamp);
      BotError::OrderSendFailed
      ( format!
        ( "{} [{}] Failed to send order for {} ({} legs, cost?): {}",
          dbgspt, timestamp, brain.m_str_1, initial_order.order_legs.len(), e
        )
      )
    })?;

  // WAIT FOR ORDER_FLY CONFIRMATION
  wait_for_stealth_order_confirmation(stone, brain, stuff, to_dr, told_it_sucks_api, &dbgspt, "entry").await?;

  // transition to the checking order status state:
  brain.last_haggle_at = Some(chrono::Utc::now());
  brain.m_begin = Some(chrono::Utc::now());
  brain.m_state = StealthBotState::Check1;

  let msgmsg = format!("{} [{}] EXIT! - next brain state = {:#?}", dbgspt, timestamp, brain.m_state); 
  let _ = f_report_info(&stone, &brain, &stuff, &to_dr, &msgmsg).await;
  
  Ok(())
}

// wait for order fill
// wait for order fill
async fn f_run_check1
( stone: &mut dsta::MakeStealthBot,
  brain: &mut StealthBot,
  stuff: &mut crate::Allocation,
  to_dr: &crate::BridgeDrHand,
) -> BotResult
{ 
  let dbgspt = format!("[STEALTH CHECK1 {}]", stuff.bot_name);
  let timestamp = chrono::Utc::now().format("%Y-%m-%d %H:%M:%S%.f").to_string();
  log::debug!("{} [{}] ENTRY!", dbgspt, timestamp);

  // ─────────────────────────────────────────────────────────────────────
  // Step 1: Check current position
  // ─────────────────────────────────────────────────────────────────────
  let mut avg_price = 0.0;
  let mut owned_num = 0.0;
  let mut found_position = false;

  if let Some(pos) = &stuff.bot_safe.m_long_calls {
    if let dsta::FinAss::OptDeets(info) = &pos.asset {
      if info.option_name == brain.m_str_1 {
        avg_price = pos.price;
        owned_num = pos.owned;
        found_position = true;
      }
    }
  } else if let Some(pos) = &stuff.bot_safe.m_long_puts {
    if let dsta::FinAss::OptDeets(info) = &pos.asset {
      if info.option_name == brain.m_str_1 {
        avg_price = pos.price;
        owned_num = pos.owned;
        found_position = true;
      }
    }
  }

  // ─────────────────────────────────────────────────────────────────────
  // Step 2: If filled (full or partial), check if we can advance
  // ─────────────────────────────────────────────────────────────────────
  if found_position {
    if avg_price <= 0.01 {
      let errmsg = format!("{} [{}] Invalid avg_price {:.4}!", dbgspt, timestamp, avg_price);
      return Err(BotError::Other(errmsg));
    }

    // Check for overfill
    if owned_num > brain.m_count + 0.01 {
      let msgmsg = format!(
        "{} [{}] OVERFILL: owned {:.2} > expected {:.2} — adjusting",
        dbgspt, timestamp, owned_num, brain.m_count
      );
      let _ = f_report_info(stone, brain, stuff, to_dr, &msgmsg).await;
    }

    // If fully filled, advance immediately
    if owned_num >= brain.m_count - 0.01 {
      brain.m_price = avg_price;
      brain.m_floor = avg_price * (1.0 + stone.exit_loss_pct / 100.0);
      brain.m_start = Some(chrono::Utc::now());
      brain.m_state = StealthBotState::Logic1;
      
      let msgmsg = format!(
        "{} [{}] FULL FILL — {} haggle attempts | entry: {:.3} | floor: {:.3}",
        dbgspt, timestamp, brain.haggle_attempt, avg_price, brain.m_floor
      );
      let _ = f_report_info(stone, brain, stuff, to_dr, &msgmsg).await;
      return Ok(());
    }

    // Partial fill - continue haggling for remaining
    let remaining = brain.m_count - owned_num;
    let msgmsg = format!(
      "{} [{}] PARTIAL: {:.2}/{:.2} filled, {:.2} remaining",
      dbgspt, timestamp, owned_num, brain.m_count, remaining
    );
    let _ = f_report_info(stone, brain, stuff, to_dr, &msgmsg).await;
  }

  // ─────────────────────────────────────────────────────────────────────
  // Step 3: Calculate remaining quantity to haggle for
  // ─────────────────────────────────────────────────────────────────────
  let remaining_qty = brain.m_count - owned_num;

  // ─────────────────────────────────────────────────────────────────────
  // Step 4: Check if we should haggle (retry period check)
  // ─────────────────────────────────────────────────────────────────────
  let last = brain.last_haggle_at.expect("last_haggle_at must be set in f_run_begins");
  let should_haggle = chrono::Utc::now() - last >= stone.my_accounting.haggle_action.get_retry_period();

  if !should_haggle {
    return Ok(()); // Wait for retry period
  }

  // ─────────────────────────────────────────────────────────────────────
  // Step 5: Build and send haggle order for remaining quantity
  // ─────────────────────────────────────────────────────────────────────
  let orig_order = brain.m_entry.as_ref().expect("m_entry must be set in f_run_begins");

  let base_price = orig_order.order_legs[0].price;  // this is now the LAST sent price

  let mut haggle_order = dsta::Order {
      order_legs: vec![dsta::OrderLeg {
          buy_what: orig_order.order_legs[0].buy_what.clone(),
          quantity: remaining_qty,
          remaining: remaining_qty,
          action: dsta::BuyOrSell::BuyToOpen,
          price: base_price,   // ← start from previous limit, not original
      }],
      order_type: dsta::MarketOrLimit::Limit,
  };

  // Apply haggle concession
  let slippage_delta = stone.my_accounting.haggle_action.apply_haggle_changes(
    &mut haggle_order,
    &brain.haggle_context,
  );

  brain.total_slippage += slippage_delta;
  brain.haggle_attempt += 1;
  brain.last_haggle_at = Some(chrono::Utc::now());

  // ─────────────────────────────────────────────────────────────────────
  // Step 6: Check slippage limit
  // ─────────────────────────────────────────────────────────────────────
  let slippage_max = stone.my_accounting.haggle_action.get_slippage_max();

  if brain.total_slippage > slippage_max {
    let msgmsg = format!(
      "{} [{}] Slippage limit exceeded ({:.4} > {:.4})",
      dbgspt, timestamp, brain.total_slippage, slippage_max
    );
    let _ = f_report_info(stone, brain, stuff, to_dr, &msgmsg).await;

    // Send cancel
    let cancel_order = dsta::Order {
      order_legs: vec![],
      order_type: dsta::MarketOrLimit::Limit,
    };
    let _ = to_dr.tell_dr_pls_trade.send(cancel_order).await;

    // IMPORTANT: In the partial-accept branch, we DO allow shrinking m_count
    // because we're accepting that the full size is impossible
    if owned_num <= 0.01 {
        return Err(BotError::Other("Max slippage exceeded with zero fill — entry canceled".into()));
    }

    // Accept partial and advance
    let msgmsg = format!(
        "{} [{}] Accepting partial fill of {:.2} contracts and advancing",
        dbgspt, timestamp, owned_num
    );
    let _ = f_report_info(stone, brain, stuff, to_dr, &msgmsg).await;

    // Here we shrink the target — this is the ONLY place m_count should ever change after init
    brain.m_count = owned_num;

    brain.m_price = avg_price;
    brain.m_floor = avg_price * (1.0 + stone.exit_loss_pct / 100.0);

    if let Some(ref mut entry) = brain.m_entry {
        entry.order_legs[0].quantity = owned_num;
        entry.order_legs[0].remaining = owned_num;
    }

    brain.m_start = Some(chrono::Utc::now());
    brain.m_state = StealthBotState::Logic1;
    return Ok(());
  }

  // ─────────────────────────────────────────────────────────────────────
  // Step 7: Send haggled order
  // ─────────────────────────────────────────────────────────────────────
  let new_price = haggle_order.order_legs[0].price;
  let msgmsg = format!(
    "{} [{}] Haggle #{} | qty {:.2} | slippage Δ{:.4} total {:.4} | price {:.3}",
    dbgspt, timestamp, brain.haggle_attempt, remaining_qty, 
    slippage_delta, brain.total_slippage, new_price
  );
  let _ = f_report_info(stone, brain, stuff, to_dr, &msgmsg).await;

  to_dr.tell_dr_pls_trade.send(haggle_order.clone()).await.map_err(|e| {
    BotError::Other(format!("Failed to send haggled order: {}", e))
  })?;

  brain.m_entry = Some(haggle_order);  // or haggle_order.clone() if you need to keep the sent version immutable

  log::debug!("{} [{}] EXIT! - haggle sent, awaiting fill", dbgspt, timestamp);
  Ok(())
}

// monitor the long option we opened first
async fn f_run_logic1
( stone: &mut dsta::MakeStealthBot,
  brain: &mut StealthBot,
  stuff: &mut crate::Allocation,
  to_dr: &crate::BridgeDrHand,
  told_it_sucks_api: &mut tokio::sync::mpsc::Receiver<String>,
) -> BotResult
{ let dbgspt = format!("[STEALTH LOGIC1 {}]", stuff.bot_name);
  let timestamp = chrono::Utc::now().format("%Y-%m-%d %H:%M:%S%.f").to_string();

  // Get opt_1 ticker
  let opt_1_ticker = brain.m_opt_1.as_ref()
    .and_then(|opt| opt.last_ticker.as_ref())
    .ok_or_else(|| { 
      let msg = format!("{} [{}] Missing opt_1 ticker data", dbgspt, timestamp);
      BotError::MissingData(msg)
    })?;

  let bid1 = opt_1_ticker.bid;
  let ask1 = opt_1_ticker.ask;
  let mid1 = if stone.use_theo_cost
  { opt_1_ticker.theo_price.unwrap_or((bid1 + ask1) / 2.0) }
  else
  { (bid1 + ask1) / 2.0 };

  let ptto = mid1;           // current market price of owned option
  let peto = brain.m_price;  // entry price

  // Stop loss check
  if ptto < brain.m_floor
  { let msgmsg = format!("{} [{}] STOP LOSS HIT - exiting at a loss (current: {:.2}, floor: {:.2})", dbgspt, timestamp, ptto, brain.m_floor); let _ = f_report_info(&stone, &brain, &stuff, &to_dr, &msgmsg).await;
    brain.m_start = Some(chrono::Utc::now());
    return f_run_exit_order_1(stone, brain, stuff, to_dr, told_it_sucks_api).await;
  }

  // Profit calculations
  let opft = ptto - peto;
  let opct = if peto.abs() > 1e-6 { 100.0 * opft / peto } else { 0.0 };

  // Trailing stop update (at 50% of target gain)
  if opct > (stone.exit_gain_pct / 2.0)
  { let new_floor = peto + (opft / 2.0);
    if new_floor > brain.m_floor
    { brain.m_floor = new_floor;
      let msgmsg = format!("{} [{}] Raised trailing floor to {:.2} for profit protection", dbgspt, timestamp, brain.m_floor); let _ = f_report_info(&stone, &brain, &stuff, &to_dr, &msgmsg).await;
    }
  }

  // Get opt_2 ticker (required for spread logic) - skip quietly if not ready yet
  let opt_2_ticker = match brain.m_opt_2.as_ref()
    .and_then(|opt| opt.last_ticker.as_ref())
  {
    Some(t) => t,
    None => 
    { log::debug!("{} EXIT! [{}] Waiting for opt_2 ticker data - skipping this tick", dbgspt, timestamp);
      return Ok(());
    }
  };

  let bid2 = opt_2_ticker.bid;
  let ask2 = opt_2_ticker.ask;
  let mid2 = if stone.use_theo_cost
  { opt_2_ticker.theo_price.unwrap_or((bid2 + ask2) / 2.0) }
  else
  { (bid2 + ask2) / 2.0 };
  
  // Get stock ticker (underlying) - skip quietly if not ready yet
  let _stock_ticker = match brain.m_stock.last_ticker.as_ref() 
  { Some(t) => t,
    None => 
    { log::debug!("{} EXIT [{}] Waiting for underlying stock ticker - skipping this tick", dbgspt, timestamp);
      return Ok(());
    }
  };

  // Key variables
  //let mids = (stock_ticker.bid + stock_ticker.ask) / 2.0;
  //let pstk = mids;
  //let psbo = mid2;
  let pst2 = brain.m_val_1;   // long strike
  let pso1 = brain.m_val_2;   // short strike

  // Your logic: H = G + p·X (for calls) or H = G - p·X (for puts)
  let p = stone.exit_gain_pct / 100.0;
  let required_short_price = if stone.stonk_feeling == dsta::MarketDirection::Corrects
  { // PUT: breakeven = strike - premium
    // We want: (strike_short - required_price) = (strike_long + peto) - p*peto
    // Solving: required_price = strike_short - (strike_long + peto - p*peto)
    //                         = strike_short - strike_long - peto*(1 - p)
    //                         = spread_width - peto*(1 - p)
    let spread_width = pso1 - pst2;  // short strike - long strike (both positive)
    spread_width - peto * (1.0 - p)
  }
  else
  { // CALL: breakeven = strike + premium  
    // We want: (strike_short + required_price) = (strike_long + peto) + p*peto
    // Solving: required_price = strike_long + peto + p*peto - strike_short
    //                         = strike_long - strike_short + peto*(1 + p)
    let spread_width = pst2 - pso1;  // long strike - short strike (positive for call spreads)
    peto * (1.0 + p) + spread_width
  };

  // Check if current short leg price meets or exceeds required price
  if mid2 >= required_short_price
  { brain.m_nicep = mid2;
    brain.m_start = Some(chrono::Utc::now());
    
    let msg111 = format!(
    "{} [{}] SPREAD ENTRY TRIGGERED! Required credit ≥ ${:.2}, Got: ${:.2}",
    dbgspt, timestamp, required_short_price, mid2
    );  
    let target_breakeven = if stone.stonk_feeling == dsta::MarketDirection::Corrects {
        pso1 - required_short_price   // lower BE = better for put credit
    } else {
        pso1 + required_short_price   // higher BE = better for call credit
    };
    let long_breakeven = if stone.stonk_feeling == dsta::MarketDirection::Corrects {
        pst2 + peto                   // long put BE = strike + debit
    } else {
        pst2 + peto                   // long call BE = strike + debit
    };
    let profit_improvement = peto * p;
    let msg222 = format!(
    "→ locks breakeven at ≤ ${:.2} (improved from long BE ${:.2} by capturing {:.0}% of debit (${:.2}))",
    target_breakeven,
    long_breakeven,
    stone.exit_gain_pct,
    profit_improvement
    );
    let msgmsg = format!("{}\n{}", msg111,msg222 ); let _ = f_report_info(&stone, &brain, &stuff, &to_dr, &msgmsg).await;

    return f_run_makes_spread(stone, brain, stuff, to_dr,told_it_sucks_api).await;  
  }

  log::debug!("{} [{}] EXIT! - no action, state remains {:#?}", dbgspt, timestamp, brain.m_state);
  Ok(())
}

// check 2 and 3 both stem from the logic1, so the chould check from m_start

// wait for spread order fill (short leg SellToOpen)
async fn f_run_check2
( stone: &mut dsta::MakeStealthBot,
  brain: &mut StealthBot,
  stuff: &mut crate::Allocation,
  to_dr: &crate::BridgeDrHand,
) -> BotResult
{ 
  let dbgspt = format!("[STEALTH CHECK2 {}]", stuff.bot_name);
  let timestamp = chrono::Utc::now().format("%Y-%m-%d %H:%M:%S%.f").to_string();
  log::debug!("{} [{}] ENTRY! - waiting for short leg fill", dbgspt, timestamp);
  
  let short_filled : bool;
  let owned_short : f64;

  // ─────────────────────────────────────────────────────────────────────
  // Check short calls or short puts depending on direction
  // ─────────────────────────────────────────────────────────────────────
  if stone.stonk_feeling == dsta::MarketDirection::Corrects
  { 
    // bearish → short puts
    if let Some(pos) = &stuff.bot_safe.m_short_puts
    { 
      owned_short = pos.owned;
      if pos.asset.order_name() == brain.m_str_2
      { 
        short_filled = owned_short.abs() >= brain.m_count - 0.01;
      }
      else
      { 
        let errmsg = format!("{} incorrect name of short put! ", dbgspt);
        return Err(BotError::Other(errmsg));
      }
    }
    else
    { 
      log::warn!("{} short put doesn't exist yet", dbgspt);
      return Ok(());
    }
  }
  else
  { 
    // bullish → short calls
    if let Some(pos) = &stuff.bot_safe.m_short_calls
    { 
      owned_short = pos.owned;
      if pos.asset.order_name() == brain.m_str_2
      { 
        short_filled = owned_short.abs() >= brain.m_count - 0.01;
      }
      else
      { 
        let errmsg = format!("{} incorrect name of short call! ", dbgspt);
        return Err(BotError::Other(errmsg));
      }
    }
    else
    { 
      log::warn!("{} short call doesn't exist yet", dbgspt);
      return Ok(());
    }
  }

  if !short_filled && owned_short == 0.0
  { 
    log::debug!("{} EXIT [{}] No short position detected yet - still waiting", dbgspt, timestamp);
    return Ok(());
  }

  if owned_short.abs() > brain.m_count + 0.01
  { 
    let msgmsg = format!(
      "{} [{}] OVERFILL on short leg: {:.2} > expected {:.2} for {}", 
      dbgspt, timestamp, owned_short, brain.m_count, brain.m_str_2
    );
    let _ = f_report_info(&stone, &brain, &stuff, &to_dr, &msgmsg).await;
  }

  // ─────────────────────────────────────────────────────────────────────
  // Success: short leg fully filled
  // ─────────────────────────────────────────────────────────────────────
  if short_filled
  { 
    brain.m_state = StealthBotState::Logic2;
    let msgmsg = format!("{} [{}] SHORT LEG FULL FILL detected - moving to Logic2", dbgspt, timestamp);
    let _ = f_report_info(&stone, &brain, &stuff, &to_dr, &msgmsg).await;
    return Ok(());
  }

  // ─────────────────────────────────────────────────────────────────────
  // Partial fill: haggle for the remainder
  // ─────────────────────────────────────────────────────────────────────
  if owned_short.abs() > 0.01 && owned_short.abs() < brain.m_count - 0.01
  { 
    let filled_short = owned_short.abs();
    let remaining_short = brain.m_count - filled_short;

    log::warn!(
      "{} [{}] Partial fill on short leg: {:.2}/{:.2} filled, {:.2} remaining",
      dbgspt, timestamp, filled_short, brain.m_count, remaining_short
    );

    // ── Check timeout first ──────────────────────────────────────────
    if let Some(start_time) = brain.m_start
    { 
      let elapsed = chrono::Utc::now() - start_time;
      if elapsed > chrono::TimeDelta::seconds(20)
      { 
        let errmsg = format!(
          "{} [{}] TIMEOUT after {:.0}s waiting for short leg fill (got {:.2}/{:.2})",
          dbgspt, timestamp, elapsed.num_seconds(), filled_short, brain.m_count
        );
        return Err(BotError::Timeout(errmsg));
      }
    }
    else
    { 
      log::warn!("{} [{}] m_start not set — cannot check short fill timeout", dbgspt, timestamp);
    }

    // ── Retry timing check ───────────────────────────────────────────
    if let Some(last) = brain.last_haggle_at 
    {
      if chrono::Utc::now() - last < brain.retry_at_delta 
      {
        return Ok(()); // Wait for retry period
      }
    } 
    else 
    {
      // Safety: if never set, don't haggle
      return Ok(());
    }

    // ── Build haggle order for remaining quantity ────────────────────
    let orig_order = match brain.m_exits.as_ref() 
    {
      Some(o) => o,
      None => return Ok(()),
    };

    if remaining_short <= 0.0 
    {
      return Ok(());
    }

    // Build new haggle order from last sent price
    let base_leg = &orig_order.order_legs[0];
    let base_price = base_leg.price;

    let mut haggle_order = dsta::Order {
      order_legs: vec![dsta::OrderLeg {
        buy_what: base_leg.buy_what.clone(),
        quantity: remaining_short,
        remaining: remaining_short,
        action: dsta::BuyOrSell::SellToOpen,
        price: base_price,
      }],
      order_type: dsta::MarketOrLimit::Limit,
    };

    // Apply spread haggle method
    let slippage_delta = stone.my_accounting.haggle_action.apply_haggle_changes(
      &mut haggle_order,
      &brain.haggle_context,
    );

    brain.total_slippage += slippage_delta;
    brain.haggle_attempt += 1;
    brain.last_haggle_at = Some(chrono::Utc::now());

    // ── Check slippage cap ───────────────────────────────────────────
    let slippage_max = stone.my_accounting.haggle_action.get_slippage_max();
    if brain.total_slippage > slippage_max 
    {
      let msgmsg = format!(
        "{} [{}] Spread entry slippage limit exceeded ({:.4} > {:.4})",
        dbgspt, timestamp, brain.total_slippage, slippage_max
      );
      let _ = f_report_info(stone, brain, stuff, to_dr, &msgmsg).await;

      // Send cancel
      let cancel_order = dsta::Order {
        order_legs: vec![],
        order_type: dsta::MarketOrLimit::Limit,
      };
      let _ = to_dr.tell_dr_pls_trade.send(cancel_order).await;

      // If no fill, treat as fatal; if partial, accept that partial
      if filled_short <= 0.01 
      {
        return Err(BotError::Other("Max slippage exceeded on spread entry with zero fill".into()));
      }

      // Accept partial and advance
      let msgmsg = format!(
        "{} [{}] Accepting partial spread of {:.2} contracts and advancing to Logic2",
        dbgspt, timestamp, filled_short
      );
      let _ = f_report_info(stone, brain, stuff, to_dr, &msgmsg).await;

      // Note: Unlike entry, we don't shrink m_count here because the long leg is already fully filled
      // We just have a partial short leg, which affects the spread but doesn't change our position size
      brain.m_state = StealthBotState::Logic2;
      return Ok(());
    }

    // ── Send the haggled spread order ────────────────────────────────
    let new_price = haggle_order.order_legs[0].price;
    let msgmsg = format!(
      "{} [{}] Spread entry haggle #{} | qty {:.2} | slippage Δ{:.4} total {:.4} | price {:.3}",
      dbgspt, timestamp, brain.haggle_attempt, remaining_short, 
      slippage_delta, brain.total_slippage, new_price
    );
    let _ = f_report_info(stone, brain, stuff, to_dr, &msgmsg).await;

    to_dr.tell_dr_pls_trade
      .send(haggle_order.clone())
      .await
      .map_err(|e| BotError::Other(format!("Failed to send spread haggled order: {}", e)))?;

    // Track it as current spread order
    brain.m_exits = Some(haggle_order.clone());
    brain.m_nicep = new_price; // Update the credit price we're getting

    return Ok(());
  }

  log::debug!("{} [{}] EXIT! - no full fill yet, state remains {:#?}", dbgspt, timestamp, brain.m_state);
  Ok(())
}

// wait for loss exit fill (SellToClose long leg → owned should go to ~0)
async fn f_run_check3
( stone: &mut dsta::MakeStealthBot,
  brain: &mut StealthBot,
  stuff: &mut crate::Allocation,
  to_dr: &crate::BridgeDrHand,
) -> BotResult
{ 
  let dbgspt = format!("[STEALTH CHECK3 {}]", stuff.bot_name);
  let timestamp = chrono::Utc::now().format("%Y-%m-%d %H:%M:%S%.f").to_string();
  log::debug!("{} [{}] ENTRY! - waiting for loss exit fill", dbgspt, timestamp);
  
  let mut remaining_long = 0.0;

  // Check long calls or long puts depending on direction
  let mut found = false;
  if stone.stonk_feeling == dsta::MarketDirection::Corrects
  { 
    // bearish → long puts
    if let Some(pos) = &stuff.bot_safe.m_long_puts
    { 
      remaining_long = pos.owned;
      if let dsta::FinAss::OptDeets(opt) = &pos.asset
      { 
        if opt.option_name == brain.m_str_1
        { 
          found = true;
        }
      }
    }
  }
  else
  { 
    // bullish → long calls
    if let Some(pos) = &stuff.bot_safe.m_long_calls
    { 
      remaining_long = pos.owned;
      if let dsta::FinAss::OptDeets(opt) = &pos.asset
      { 
        if opt.option_name == brain.m_str_1
        { 
          found = true;
        }
      }
    }
  }
  
  if !found
  { 
    log::warn!("{} remaining long is zero because it wasn't found!", dbgspt);
  }

  // ─────────────────────────────────────────────────────────────────────
  // Success condition: long leg basically closed
  // ─────────────────────────────────────────────────────────────────────
  if remaining_long <= 0.01
  { 
    let msgmsg = format!("{} [{}] LOSS EXIT COMPLETE - long leg closed (remaining: {:.2})", dbgspt, timestamp, remaining_long);
    let _ = f_report_info(&stone, &brain, &stuff, &to_dr, &msgmsg).await;
    // Terminal - zombie out after loss exit
    return Err(BotError::Other(format!("{} [{}] Loss exit completed - bot terminating", dbgspt, timestamp)));
  }

  // ─────────────────────────────────────────────────────────────────────
  // Partial fill: haggle for the remainder
  // ─────────────────────────────────────────────────────────────────────
  if remaining_long < brain.m_count && remaining_long > 0.0
  {
    let owned_loss_exit = brain.m_count - remaining_long;
    
    log::warn!("{} [{}] Partial close on loss exit: {:.2} filled, {:.2} remaining",
      dbgspt, timestamp, owned_loss_exit, remaining_long);

    // ── Check timeout first ──────────────────────────────────────────
    if let Some(start_time) = brain.m_start
    { 
      let elapsed = chrono::Utc::now() - start_time;
      if elapsed > chrono::TimeDelta::seconds(20)
      { 
        let errmsg = format!(
          "{} [{}] TIMEOUT after {:.0}s waiting for loss exit fill (remaining {:.2}/{:.2})",
          dbgspt, timestamp, elapsed.num_seconds(), remaining_long, brain.m_count
        );
        return Err(BotError::Timeout(errmsg));
      }
    }

    // ── Retry timing check ───────────────────────────────────────────
    if let Some(last) = brain.last_haggle_at 
    {
      if chrono::Utc::now() - last < brain.retry_at_delta 
      {
        return Ok(()); // Wait for retry period
      }
    } 
    else 
    {
      // Safety: if never set, don't haggle
      return Ok(());
    }

    // ── Build haggle order for remaining quantity ────────────────────
    let orig_order = match brain.m_exits.as_ref() 
    {
      Some(o) => o,
      None => return Ok(()),
    };

    let remaining_qty = remaining_long;
    if remaining_qty <= 0.0 
    {
      return Ok(());
    }

    // Build new haggle order from last sent price
    let base_leg = &orig_order.order_legs[0];
    let base_price = base_leg.price;

    let mut haggle_order = dsta::Order {
      order_legs: vec![dsta::OrderLeg {
        buy_what: base_leg.buy_what.clone(),
        quantity: remaining_qty,
        remaining: remaining_qty,
        action: dsta::BuyOrSell::SellToClose,
        price: base_price,
      }],
      order_type: dsta::MarketOrLimit::Limit,
    };

    // Apply exit haggle method
    let slippage_delta = stone.my_accounting.haggle_action.apply_haggle_changes(
      &mut haggle_order,
      &brain.haggle_context,
    );

    brain.total_slippage += slippage_delta;
    brain.haggle_attempt += 1;
    brain.last_haggle_at = Some(chrono::Utc::now());

    // ── Check slippage cap ───────────────────────────────────────────
    let slippage_max = stone.my_accounting.haggle_action.get_slippage_max();
    if brain.total_slippage > slippage_max 
    {
      let msgmsg = format!(
        "{} [{}] Loss exit slippage limit exceeded ({:.4} > {:.4})",
        dbgspt, timestamp, brain.total_slippage, slippage_max
      );
      let _ = f_report_info(stone, brain, stuff, to_dr, &msgmsg).await;

      // Send cancel
      let cancel_order = dsta::Order {
        order_legs: vec![],
        order_type: dsta::MarketOrLimit::Limit,
      };
      let _ = to_dr.tell_dr_pls_trade.send(cancel_order).await;

      // If no fill, treat as fatal; if partial, accept that partial
      if owned_loss_exit <= 0.01 
      {
        return Err(BotError::Other("Max slippage exceeded on loss exit with zero fill".into()));
      }

      let msgmsg = format!(
        "{} [{}] Accepting partial loss exit of {:.2} contracts and terminating",
        dbgspt, timestamp, owned_loss_exit
      );
      let _ = f_report_info(stone, brain, stuff, to_dr, &msgmsg).await;
      
      return Err(BotError::Other(format!(
        "{} [{}] Loss exit partially completed - bot terminating", 
        dbgspt, timestamp
      )));
    }

    // ── Send the haggled exit order ──────────────────────────────────
    let new_price = haggle_order.order_legs[0].price;
    let msgmsg = format!(
      "{} [{}] Loss exit haggle #{} | qty {:.2} | slippage Δ{:.4} total {:.4} | price {:.3}",
      dbgspt, timestamp, brain.haggle_attempt, remaining_qty, 
      slippage_delta, brain.total_slippage, new_price
    );
    let _ = f_report_info(stone, brain, stuff, to_dr, &msgmsg).await;

    to_dr.tell_dr_pls_trade
      .send(haggle_order.clone())
      .await
      .map_err(|e| BotError::Other(format!("Failed to send loss exit haggled order: {}", e)))?;

    // Track it as current exit order
    brain.m_exits = Some(haggle_order);

    return Ok(());
  }

  // ─────────────────────────────────────────────────────────────────────
  // No fill yet - continue waiting
  // ─────────────────────────────────────────────────────────────────────
  if remaining_long >= brain.m_count - 0.01
  { 
    log::trace!("{} [{}] No fill detected yet on loss exit order", dbgspt, timestamp);
  }

  log::debug!("{} [{}] EXIT! - waiting continues, state remains {:#?}", dbgspt, timestamp, brain.m_state);
  Ok(())
}

// monitor spread for nice exit conditions (after short leg is opened)
async fn f_run_logic2
( stone: &mut dsta::MakeStealthBot,
  brain: &mut StealthBot,
  stuff: &mut crate::Allocation,
  to_dr: &crate::BridgeDrHand,
  told_it_sucks_api: &mut tokio::sync::mpsc::Receiver<String>,
) -> BotResult
{
  let dbgspt = format!("[STEALTH LOGIC2 {}]", stuff.bot_name);
  let timestamp = chrono::Utc::now().format("%Y-%m-%d %H:%M:%S%.f").to_string();

  // ── Get current prices ───────────────────────────────────────────────
  let opt_1_ticker = brain.m_opt_1.as_ref()
    .and_then(|opt| opt.last_ticker.as_ref())
    .ok_or_else(|| BotError::MissingData(format!("long leg ticker")))?;

  let mid1 = if stone.use_theo_cost 
  { opt_1_ticker.theo_price.unwrap_or((opt_1_ticker.bid + opt_1_ticker.ask) / 2.0)
  } 
  else 
  { (opt_1_ticker.bid + opt_1_ticker.ask) / 2.0
  };

  let opt_2_ticker = brain.m_opt_2.as_ref()
    .and_then(|opt| opt.last_ticker.as_ref())
    .ok_or_else(|| BotError::MissingData(format!("short leg ticker")))?;

  let mid2 = if stone.use_theo_cost 
  { opt_2_ticker.theo_price.unwrap_or((opt_2_ticker.bid + opt_2_ticker.ask) / 2.0)
  } else 
  { (opt_2_ticker.bid + opt_2_ticker.ask) / 2.0
  };

  let stock_ticker = brain.m_stock.last_ticker.as_ref()
    .ok_or_else(|| BotError::MissingData(format!("underlying ticker")))?;

  let pstk = (stock_ticker.bid + stock_ticker.ask) / 2.0;

  // ── Core spread P/L calculations ─────────────────────────────────────
  let credit_received = brain.m_nicep;
  let current_spread_value = mid2 - mid1;           // current debit value of the spread
  let max_profit = credit_received - brain.m_price; // theoretical max if expires worthless
  let current_profit = credit_received - current_spread_value;
 
  log::trace!("PL CALC: \n  credit_received = {:#?} \n  current_spread_value = {:#?} \n  max_profit = {:#?} \n  current_profit = {:#?} \n", credit_received , current_spread_value , max_profit , current_profit);

  let profit_pct = if max_profit > 0.01 
  { 100.0 * current_profit / max_profit
  } 
  else 
  { 0.0
  };

  // ── Multiple exit conditions check ───────────────────────────────────
  let mut should_exit = false;
  let mut exit_reasons = Vec::new();

  // We now expect nice_exit_way to be a Vec<StealthBotDoneNiceStyle>
  for condition in &stone.nice_exit_way
  {
    match condition 
    { dsta::StealthBotDoneNiceStyle::OtmShort => 
      { let short_strike = brain.m_val_2;
        let is_otm = if stone.stonk_feeling == dsta::MarketDirection::Corrects 
        { pstk > short_strike  // put short OTM when underlying above strike
        } else 
        { pstk < short_strike  // call short OTM when underlying below strike
        };
        if is_otm 
        { should_exit = true;
          exit_reasons.push(format!("short leg OTM (underlying {:.2} vs strike {:.2})", pstk, short_strike));
        }
      }

      dsta::StealthBotDoneNiceStyle::FinalPct(target_pct) => 
      { log::trace!("Profit check: {} vs target {} ", profit_pct, target_pct);
        if profit_pct >= *target_pct 
        { should_exit = true;
          exit_reasons.push(format!("{}% of max profit captured", profit_pct));
        }
      }

      dsta::StealthBotDoneNiceStyle::PriorDay(market_time) => 
      { let days_before = market_time.relative_days;
        let expiry = brain.m_opt_1.as_ref()
          .and_then(|o| Some(o.expire_date))
          .unwrap_or(chrono::Utc::now());

        // days until expiry
        let days_left = (expiry - chrono::Utc::now()).num_days();
        if days_left == 0
        { should_exit = market_time.has_exit_time_passed(chrono::Utc::now());
          exit_reasons.push(format!("within {} days of expiry ({})", days_before, market_time));
        }
      }

      dsta::StealthBotDoneNiceStyle::GoExpire => 
      { // No action — hold until expiry (can be combined with others)
      }
    }
  }

  // ── Exit if any condition is met ─────────────────────────────────────
  if should_exit 
  { let reason_str = if exit_reasons.len() == 1 
    { exit_reasons[0].clone()
    } else 
    { format!("multiple conditions: {}", exit_reasons.join(", "))
    };
    let msgmsg = format!("{} [{}] NICE EXIT TRIGGERED: {}", dbgspt, timestamp, reason_str); let _ = f_report_info(&stone, &brain, &stuff, &to_dr, &msgmsg).await;
    brain.m_start = Some(chrono::Utc::now());
    return f_run_exit_order_2(stone, brain, stuff, to_dr, told_it_sucks_api).await;
  }

  log::debug!("{} [{}] EXIT! - no exit condition met yet", dbgspt, timestamp);
  Ok(())
}

// wait for full exit (both legs closed → all positions ≈0), then zombie
async fn f_run_check4
( stone: &mut dsta::MakeStealthBot,
  brain: &mut StealthBot,
  stuff: &mut crate::Allocation,
  to_dr: &crate::BridgeDrHand,
) -> BotResult
{ 
  let dbgspt = format!("[STEALTH_CHECK4 {}]", stuff.bot_name);
  let timestamp = chrono::Utc::now().format("%Y-%m-%d %H:%M:%S%.f").to_string();
  log::debug!("{} [{}] ENTRY! - waiting for final spread close", dbgspt, timestamp);
  
  let mut remaining_long = 0.0;
  let mut remaining_short = 0.0;

  // ─────────────────────────────────────────────────────────────────────
  // Check current positions
  // ─────────────────────────────────────────────────────────────────────
  if let Some(pos) = stuff.bot_safe.m_long_calls.as_ref().or(stuff.bot_safe.m_long_puts.as_ref())
  { 
    if let dsta::FinAss::OptDeets(opt) = &pos.asset
    { 
      if opt.option_name == brain.m_str_1
      { 
        remaining_long = pos.owned;
      }
    }
  }

  if let Some(pos) = stuff.bot_safe.m_short_calls.as_ref().or(stuff.bot_safe.m_short_puts.as_ref())
  { 
    if let dsta::FinAss::OptDeets(opt) = &pos.asset
    { 
      if opt.option_name == brain.m_str_2
      { 
        remaining_short = pos.owned.abs();
      }
    }
  }

  // ─────────────────────────────────────────────────────────────────────
  // Success: both legs basically closed
  // ─────────────────────────────────────────────────────────────────────
  if remaining_long <= 0.01 && remaining_short <= 0.01
  { 
    let msgmsg = format!(
      "{} [{}] FINAL EXIT COMPLETE - all positions closed (long: {:.2}, short: {:.2})", 
      dbgspt, timestamp, remaining_long, remaining_short
    );
    let _ = f_report_info(&stone, &brain, &stuff, &to_dr, &msgmsg).await;

    // Terminal state: zombie out (bot finished successfully)
    return Err(BotError::Other(format!(
      "{} EXIT OK! [{}] All positions closed - stealth bot mission complete → zombied",
      dbgspt, timestamp
    )));
  }

  // ─────────────────────────────────────────────────────────────────────
  // Partial close handling: haggle for remainder
  // ─────────────────────────────────────────────────────────────────────
  let partial = (remaining_long > 0.01 && remaining_long < brain.m_count - 0.01) ||
                (remaining_short > 0.01 && remaining_short < brain.m_count - 0.01);

  if partial
  { let filled_long = brain.m_count - remaining_long;
    let filled_short = brain.m_count - remaining_short;
   
    log::warn!(
        "{} [{}] Partial final close detected (long: {:.2}/{:.2}, short: {:.2}/{:.2})",
        dbgspt, timestamp, filled_long, brain.m_count, filled_short, brain.m_count
    );

    // ── TIMEOUT CHECK ────────────────────────────────────────────────
    if let Some(start_time) = brain.m_start
    {
        let elapsed = chrono::Utc::now() - start_time;
        if elapsed > chrono::Duration::seconds(20)
        {
            log::error!("{} TIMEOUT in Check4 - giving up after {}s", dbgspt, elapsed.num_seconds());
            return Err(BotError::Timeout("Timeout waiting for full exit".into()));
        }
        log::trace!("{} Check4 timeout check: elapsed {}s / 20s", dbgspt, elapsed.num_seconds());
    }

    // ── RETRY TIMING CHECK ───────────────────────────────────────────
    if let Some(last) = brain.last_haggle_at {
        let time_since_last = chrono::Utc::now() - last;
        log::trace!(
            "{} Check4 retry check: time since last haggle = {}ms, required >= {}ms",
            dbgspt,
            time_since_last.num_milliseconds(),
            brain.retry_at_delta.num_milliseconds()
        );
        if time_since_last < brain.retry_at_delta {
            log::debug!("{} Check4: retry timer not elapsed yet - skipping haggle send", dbgspt);
            return Ok(());
        }
    } else {
        log::warn!("{} Check4: last_haggle_at is None - allowing haggle anyway", dbgspt);
    }

    // ── BUILD HAGGLE ORDER ───────────────────────────────────────────
    let orig_order = match brain.m_exits.as_ref() {
        Some(o) => {
            log::trace!("{} Check4: using previous exit order for base prices", dbgspt);
            o
        }
        None => {
            log::error!("{} Check4: brain.m_exits is None - cannot build haggle order!", dbgspt);
            return Ok(());
        }
    };

    let filled_qty = filled_long.min(filled_short);
    let remaining_qty = brain.m_count - filled_qty;
   
    if remaining_qty <= 0.0 {
        log::warn!("{} Check4: calculated remaining_qty <= 0 - skipping", dbgspt);
        return Ok(());
    }

    // Build new haggle order from last sent prices
    let base_leg_long = &orig_order.order_legs[0];
    let base_leg_short = &orig_order.order_legs[1];
    
    let base_price_long = base_leg_long.price;
    let base_price_short = base_leg_short.price;

    let mut haggle_order = dsta::Order {
      order_legs: vec![
        dsta::OrderLeg {
          buy_what: base_leg_long.buy_what.clone(),
          quantity: remaining_qty,
          remaining: remaining_qty,
          action: dsta::BuyOrSell::SellToClose,
          price: base_price_long,
        },
        dsta::OrderLeg {
          buy_what: base_leg_short.buy_what.clone(),
          quantity: remaining_qty,
          remaining: remaining_qty,
          action: dsta::BuyOrSell::BuyToClose,
          price: base_price_short,
        },
      ],
      order_type: dsta::MarketOrLimit::Limit,
    };

    log::trace!(
        "{} Check4: built haggle order for {} contracts @ long {:.3} / short {:.3}",
        dbgspt, remaining_qty,
        haggle_order.order_legs[0].price, haggle_order.order_legs[1].price
    );

    // Apply haggle concession
    let slippage_delta = stone.my_accounting.haggle_action.apply_haggle_changes(
        &mut haggle_order,
        &brain.haggle_context,
    );
    log::trace!(
        "{} Check4: applied haggle concession, delta={:.4}, total_slippage={:.4}",
        dbgspt, slippage_delta, brain.total_slippage
    );
    brain.total_slippage += slippage_delta;
    brain.haggle_attempt += 1;
    brain.last_haggle_at = Some(chrono::Utc::now());

    // ── Check slippage cap ───────────────────────────────────────────
    let slippage_max = stone.my_accounting.haggle_action.get_slippage_max();
    if brain.total_slippage > slippage_max 
    { log::warn!(
            "{} Check4: slippage limit exceeded ({:.4} > {:.4}) - canceling & accepting partial",
            dbgspt, brain.total_slippage, slippage_max
        );

      let msgmsg = format!(
        "{} [{}] Final exit slippage limit exceeded ({:.4} > {:.4})",
        dbgspt, timestamp, brain.total_slippage, slippage_max
      );
      let _ = f_report_info(stone, brain, stuff, to_dr, &msgmsg).await;

      // Send cancel
      let cancel_order = dsta::Order {
        order_legs: vec![],
        order_type: dsta::MarketOrLimit::Limit,
      };
      let _ = to_dr.tell_dr_pls_trade.send(cancel_order).await;

      // If no fill, treat as fatal; if partial, accept and terminate
      if filled_qty <= 0.01 
      {
        return Err(BotError::Other("Max slippage exceeded on final exit with zero fill".into()));
      }

      let msgmsg = format!(
        "{} [{}] Accepting partial final exit ({:.2} contracts) and terminating",
        dbgspt, timestamp, filled_qty
      );
      let _ = f_report_info(stone, brain, stuff, to_dr, &msgmsg).await;
      
      return Err(BotError::Other(format!(
        "{} EXIT OK! [{}] Final exit partially completed - bot terminating", 
        dbgspt, timestamp
      )));
    }

    // ── Send the haggled exit order ──────────────────────────────────
    let new_price_long = haggle_order.order_legs[0].price;
    let new_price_short = haggle_order.order_legs[1].price;
    
    let msgmsg = format!(
      "{} [{}] Final exit haggle #{} | qty {:.2} | slippage Δ{:.4} total {:.4} | long {:.3} short {:.3}",
      dbgspt, timestamp, brain.haggle_attempt, remaining_qty, 
      slippage_delta, brain.total_slippage, new_price_long, new_price_short, 
    );
    let _ = f_report_info(stone, brain, stuff, to_dr, &msgmsg).await;


    log::info!(
        "{} Check4: *** PREPARING TO SEND HAGGLE RETRY #{} FOR REMAINING {:.2} *** long @ {:.3} short @ {:.3}",
        dbgspt, brain.haggle_attempt + 1, remaining_qty,
        haggle_order.order_legs[0].price, haggle_order.order_legs[1].price
    );

    to_dr.tell_dr_pls_trade
      .send(haggle_order.clone())
      .await
      .map_err(|e| BotError::Other(format!("Failed to send final exit haggled order: {}", e)))?;

    // Track it as current exit order
    brain.m_exits = Some(haggle_order);

    return Ok(());
  }

  // ─────────────────────────────────────────────────────────────────────
  // No fill yet - continue waiting
  // ─────────────────────────────────────────────────────────────────────
  if remaining_long >= brain.m_count - 0.01 || remaining_short >= brain.m_count - 0.01
  { 
    log::trace!("{} [{}] No fill detected yet on final close order(s)", dbgspt, timestamp);
  }

  log::debug!("{} [{}] EXIT! - waiting continues, state remains {:#?}", dbgspt, timestamp, brain.m_state);

  Ok(())
}

//===================================================================
// HELPER: Wait for ORDER_FLY / ORDER_BAD confirmation
//===================================================================
async fn wait_for_stealth_order_confirmation(
    stone: &dsta::MakeStealthBot,
    brain: &StealthBot,
    stuff: &crate::Allocation,
    to_dr: &crate::BridgeDrHand,
    told_it_sucks_api: &mut tokio::sync::mpsc::Receiver<String>,
    dbgspt: &str,
    context: &str, // "entry", "spread", "exit1", "exit2"
) -> BotResult {
    let timestamp = chrono::Utc::now().format("%Y-%m-%d %H:%M:%S%.f").to_string();
    let order_begin = chrono::Utc::now();
    let order_timeout = chrono::Duration::seconds(30);
    let mut last_log = std::time::Instant::now();
    let log_interval = std::time::Duration::from_secs(3);

    loop 
    {
        let elapsed = chrono::Utc::now() - order_begin;

        // Log status every 3 seconds
        if last_log.elapsed() >= log_interval {
            let msg = format!(
                "{} [{}] ({}) Waiting for ORDER_FLY... (elapsed: {:.0}s)",
                dbgspt, timestamp, context, elapsed.num_seconds()
            );
            log::info!("{}", msg);
            let _ = f_report_info(stone, brain, stuff, to_dr, &msg).await;
            last_log = std::time::Instant::now();
        }

        if elapsed > order_timeout {
            let errmsg = format!(
                "{} [{}] ({}) ORDER_FLY timeout after {:.0}s - canceling",
                dbgspt, timestamp, context, elapsed.num_seconds()
            );
            log::error!("{}", errmsg);
            let _ = f_report_error(stone, brain, stuff, to_dr, &errmsg).await;
            let _ = to_dr.tell_dr_pls_trade.send(dsta::Order {
                order_legs: vec![],
                order_type: dsta::MarketOrLimit::Limit,
            }).await;
            return Err(BotError::Timeout(errmsg));
        }

        match tokio::time::timeout
        ( std::time::Duration::from_secs(4)
        , told_it_sucks_api.recv()).await 
          { Ok(Some(msg)) if msg == "ORDER_FLY" => 
            { let msg_str = format!("{} [{}] ({}) ORDER_FLY confirmed", dbgspt, timestamp, context);
              log::info!("{}", msg_str);
              let _ = f_report_info(stone, brain, stuff, to_dr, &msg_str).await;
              return Ok(());
            }
            Ok(Some(msg)) if msg == "ORDER_BAD" => 
            {
              let errmsg = format!("{} [{}] ({}) ORDER_BAD received - order rejected", dbgspt, timestamp, context);
              log::error!("{}", errmsg);
              let _ = f_report_error(stone, brain, stuff, to_dr, &errmsg).await;
              return Err(BotError::OrderSendFailed(errmsg));
            }
            Ok(Some(msg)) =>
            { let errmsg= format!("Stealth bot got a transient messsage while whaiting for order fly/bad! : {:#?} ", msg);
              log::warn!("{}",errmsg);
              let _ = f_report_error(stone, brain, stuff, to_dr, &errmsg).await;
              continue;
            }
            Ok(None) | Err(_) => continue, // TODO: Err => break pls
        }
    }
}

// Make loss exit order (SellToClose long leg) - triggered from Logic1 stop loss
async fn f_run_exit_order_1
( stone: &mut dsta::MakeStealthBot,
  brain: &mut StealthBot,
  stuff: &mut crate::Allocation,
  to_dr: &crate::BridgeDrHand,
  told_it_sucks_api: &mut tokio::sync::mpsc::Receiver<String>,
) -> BotResult
{ 
  let dbgspt = format!("[STEALTH_EXIT1 {}]", stuff.bot_name);
  let timestamp = chrono::Utc::now().format("%Y-%m-%d %H:%M:%S%.f").to_string();

  let opt_1_ticker = brain.m_opt_1.as_ref()
    .and_then(|opt| opt.last_ticker.as_ref())
    .ok_or_else(|| BotError::MissingData(format!("opt_1 ticker for exit")))?;

  let mid_price = if stone.use_theo_cost {
    opt_1_ticker.theo_price.unwrap_or((opt_1_ticker.bid + opt_1_ticker.ask) / 2.0)
  } else {
    (opt_1_ticker.bid + opt_1_ticker.ask) / 2.0
  };

  if mid_price <= 0.01 {
    let errmsg = format!("{} [{}] Invalid exit price: {:.2} - cannot place loss exit", dbgspt, timestamp, mid_price);
    return Err(BotError::Other(errmsg));
  }

  // ─────────────────────────────────────────────────────────────────────
  // Build BASE order (before haggling)
  // ─────────────────────────────────────────────────────────────────────
  let order_leg = dsta::OrderLeg {
    buy_what: dsta::FinAss::OptDeets(brain.m_opt_1.as_ref().unwrap().clone()), 
    quantity: brain.m_count,
    remaining: brain.m_count,
    action: dsta::BuyOrSell::SellToClose,
    price: mid_price,
  };

  let base_exit_order = dsta::Order {
    order_legs: vec![order_leg],
    order_type: dsta::MarketOrLimit::Limit,
  };

  // ─────────────────────────────────────────────────────────────────────
  // Wrap in initial haggle using VirtualMarketOrderWithLimitOrdThenConcede
  // ─────────────────────────────────────────────────────────────────────
  let exit_haggle_method = dsta::HaggleMethod::VirtualMarketOrderWithLimitOrdThenConcede
  ( dsta::HaggleLimits
    { retry_period : chrono::Duration::milliseconds(250) // chrono::Duration, // or cancel time
    , delta_choice : 25.0 // f64,
    , delta_is_pct : true // bool,
    , slippage_max : 50.00 // f64, 
    }
  );

  // Apply haggle concession
  let (initial_exit_order, initial_exit_ctx) = dsta::create_initial_haggle_order(
    &base_exit_order,
    &exit_haggle_method,
  );

  // update our own exit_haggle_method
  stone.my_accounting.haggle_action = dsta::HaggleAction::AnOyVeyJew(exit_haggle_method);

  // Track like entry does
  brain.m_exits = Some(initial_exit_order.clone());
  brain.haggle_context = initial_exit_ctx;
  brain.total_slippage = 0.0;
  brain.haggle_attempt = 0;
  brain.retry_at_delta = stone.my_accounting.haggle_action.get_retry_period();
  brain.last_haggle_at = Some(chrono::Utc::now());

  let msgmsg = format!(
    "{} [{}] Initial loss exit haggle order created | spread: {:.4} | method: {:?}",
    dbgspt, timestamp, brain.haggle_context.initial_spread, stone.my_accounting.haggle_action
  );


  let _ = f_report_info(stone, brain, stuff, to_dr, &msgmsg).await;

  // ─────────────────────────────────────────────────────────────────────
  // Send the haggled order
  // ─────────────────────────────────────────────────────────────────────
  let msgmsg = format!(
    "{} [{}] Sending loss exit: SellToClose {} x{:.0} @ {:.2} (haggled)",
    dbgspt, timestamp, brain.m_str_1, brain.m_count, initial_exit_order.order_legs[0].price
  );
  let _ = f_report_info(&stone, &brain, &stuff, &to_dr, &msgmsg).await;

  to_dr.tell_dr_pls_trade
    .send(initial_exit_order.clone())
    .await
    .map_err(|e| {
      let errmsg = format!("{} [{}] Failed to send loss exit order: {}", dbgspt, timestamp, e);
      BotError::OrderSendFailed(errmsg)
    })?;

  // WAIT FOR ORDER_FLY CONFIRMATION
  wait_for_stealth_order_confirmation(stone, brain, stuff, to_dr, told_it_sucks_api, &dbgspt, "exit1").await?;

  brain.m_state = StealthBotState::Check3;
  brain.m_start = Some(chrono::Utc::now());

  let msgmsg = format!("{} [{}] Loss exit order sent → moving to Check3", dbgspt, timestamp);
  let _ = f_report_info(&stone, &brain, &stuff, &to_dr, &msgmsg).await;
  Ok(())
}

// Make spread order (SellToOpen short leg) - triggered from Logic1 profit condition
// NOW WITH HAGGLING
async fn f_run_makes_spread
( stone: &mut dsta::MakeStealthBot,
  brain: &mut StealthBot,
  stuff: &mut crate::Allocation,
  to_dr: &crate::BridgeDrHand,
  told_it_sucks_api: &mut tokio::sync::mpsc::Receiver<String>,
) -> BotResult
{ 
  let dbgspt = format!("[STEALTH_SPREAD {}]", stuff.bot_name);
  let timestamp = chrono::Utc::now().format("%Y-%m-%d %H:%M:%S%.f").to_string();

  let opt_2_ticker = brain.m_opt_2.as_ref()
    .and_then(|opt| opt.last_ticker.as_ref())
    .ok_or_else(|| BotError::MissingData(format!("opt_2 ticker for spread")))?;

  let mid_price = if stone.use_theo_cost {
    opt_2_ticker.theo_price.unwrap_or((opt_2_ticker.bid + opt_2_ticker.ask) / 2.0)
  } else {
    (opt_2_ticker.bid + opt_2_ticker.ask) / 2.0
  };

  if mid_price <= 0.01 {
    let errmsg = format!("{} [{}] Invalid short leg price: {:.2} - cannot open spread", dbgspt, timestamp, mid_price);
    return Err(BotError::Other(errmsg));
  }

  // ─────────────────────────────────────────────────────────────────────
  // Build BASE order (before haggling)
  // ─────────────────────────────────────────────────────────────────────
  let order_leg = dsta::OrderLeg {
    buy_what: dsta::FinAss::OptDeets(brain.m_opt_2.as_ref().unwrap().clone()),
    quantity: brain.m_count,
    remaining: brain.m_count,
    action: dsta::BuyOrSell::SellToOpen,
    price: mid_price,
  };

  let base_spread_order = dsta::Order {
    order_legs: vec![order_leg],
    order_type: dsta::MarketOrLimit::Limit,
  };

  // ─────────────────────────────────────────────────────────────────────
  // Wrap in initial haggle using VirtualMarketOrderWithLimitOrdThenConcede
  // ─────────────────────────────────────────────────────────────────────
  let spread_haggle_method = dsta::HaggleMethod::VirtualMarketOrderWithLimitOrdThenConcede
  ( dsta::HaggleLimits
    { retry_period : chrono::Duration::milliseconds(250) // chrono::Duration, // or cancel time
    , delta_choice : 25.0 // f64,
    , delta_is_pct : true // bool,
    , slippage_max : 50.00 // f64, 
    }
  );

  let (initial_spread_order, initial_spread_ctx) = dsta::create_initial_haggle_order(
    &base_spread_order,
    &spread_haggle_method,
  );

  // update our own exit_haggle_method
  stone.my_accounting.haggle_action = dsta::HaggleAction::AnOyVeyJew(spread_haggle_method);

  // Track like entry does (reset for spread entry)
  brain.m_exits = Some(initial_spread_order.clone());
  brain.haggle_context = initial_spread_ctx;
  brain.total_slippage = 0.0;
  brain.haggle_attempt = 0;
  brain.retry_at_delta = stone.my_accounting.haggle_action.get_retry_period();
  brain.last_haggle_at = Some(chrono::Utc::now());

  let msgmsg = format!(
    "{} [{}] Initial spread haggle order created | spread: {:.4} | method: {:?}",
    dbgspt, timestamp, brain.haggle_context.initial_spread, spread_haggle_method
  );
  let _ = f_report_info(stone, brain, stuff, to_dr, &msgmsg).await;

  // ─────────────────────────────────────────────────────────────────────
  // Send the haggled order
  // ─────────────────────────────────────────────────────────────────────
  let msgmsg = format!(
    "{} [{}] Sending spread open: SellToOpen {} x{:.0} @ {:.2} (haggled)",
    dbgspt, timestamp, brain.m_str_2, brain.m_count, initial_spread_order.order_legs[0].price
  );
  let _ = f_report_info(&stone, &brain, &stuff, &to_dr, &msgmsg).await;

  to_dr.tell_dr_pls_trade
    .send(initial_spread_order.clone())
    .await
    .map_err(|e| {
      let errmsg = format!("{} [{}] Failed to send spread open order: {}", dbgspt, timestamp, e);
      BotError::OrderSendFailed(errmsg)
    })?;

  // WAIT FOR ORDER_FLY CONFIRMATION
  wait_for_stealth_order_confirmation(stone, brain, stuff, to_dr, told_it_sucks_api, &dbgspt, "spread").await?;

  brain.m_state = StealthBotState::Check2;
  brain.m_nicep = initial_spread_order.order_legs[0].price; // store the initial haggled credit
  brain.m_start = Some(chrono::Utc::now());

  let msgmsg = format!("{} EXIT [{}] Spread order sent → moving to Check2", dbgspt, timestamp);
  let _ = f_report_info(&stone, &brain, &stuff, &to_dr, &msgmsg).await;
  Ok(())
}

// Make final nice exit order (close both legs) - triggered from Logic2
async fn f_run_exit_order_2
( stone: &mut dsta::MakeStealthBot,
  brain: &mut StealthBot,
  stuff: &mut crate::Allocation,
  to_dr: &crate::BridgeDrHand,
  told_it_sucks_api: &mut tokio::sync::mpsc::Receiver<String>,
) -> BotResult
{ 
  let dbgspt = format!("[STEALTH_EXIT2 {}]", stuff.bot_name);
  let timestamp = chrono::Utc::now().format("%Y-%m-%d %H:%M:%S%.f").to_string();
  
  let long_ticker = brain.m_opt_1.as_ref()
    .and_then(|opt| opt.last_ticker.as_ref())
    .ok_or_else(|| BotError::MissingData(format!("long leg ticker for final exit")))?;
  
  let short_ticker = brain.m_opt_2.as_ref()
    .and_then(|opt| opt.last_ticker.as_ref())
    .ok_or_else(|| BotError::MissingData(format!("short leg ticker for final exit")))?;
  
  let mid_long = if stone.use_theo_cost {
    long_ticker.theo_price.unwrap_or((long_ticker.bid + long_ticker.ask) / 2.0)
  } else {
    (long_ticker.bid + long_ticker.ask) / 2.0
  };
  
  let mid_short = if stone.use_theo_cost {
    short_ticker.theo_price.unwrap_or((short_ticker.bid + short_ticker.ask) / 2.0)
  } else {
    (short_ticker.bid + short_ticker.ask) / 2.0
  };
  
  if mid_long <= 0.01 || mid_short <= 0.01 {
    let errmsg = format!("{} [{}] Invalid final close prices (long: {:.2}, short: {:.2})", 
      dbgspt, timestamp, mid_long, mid_short);
    return Err(BotError::Other(errmsg));
  }
  
  let qty = brain.m_count;

  // ─────────────────────────────────────────────────────────────────────
  // Build BASE closing spread order (before haggling)
  // ─────────────────────────────────────────────────────────────────────
  let leg_long = dsta::OrderLeg {
    buy_what: dsta::FinAss::OptDeets(brain.m_opt_1.as_ref().unwrap().clone()),
    quantity: qty,
    remaining: qty,
    action: dsta::BuyOrSell::SellToClose,
    price: mid_long,
  };

  let leg_short = dsta::OrderLeg {
    buy_what: dsta::FinAss::OptDeets(brain.m_opt_2.as_ref().unwrap().clone()),
    quantity: qty,
    remaining: qty,
    action: dsta::BuyOrSell::BuyToClose,
    price: mid_short,
  };

  let base_close_order = dsta::Order {
    order_legs: vec![leg_long, leg_short],
    order_type: dsta::MarketOrLimit::Limit,
  };

  // ─────────────────────────────────────────────────────────────────────
  // Wrap in initial haggle using VirtualMarketOrderWithLimitOrdThenConcede
  // ─────────────────────────────────────────────────────────────────────
  let exit_haggle_method = dsta::HaggleMethod::VirtualMarketOrderWithLimitOrdThenConcede
  ( dsta::HaggleLimits
    { retry_period : chrono::Duration::milliseconds(250) // chrono::Duration, // or cancel time
    , delta_choice : 25.0 // f64,
    , delta_is_pct : true // bool,
    , slippage_max : 50.00 // f64, 
    }
  );

  let (initial_exit_order, initial_exit_ctx) = dsta::create_initial_haggle_order(
    &base_close_order,
    &exit_haggle_method,
  );

  // update our own exit_haggle_method
  stone.my_accounting.haggle_action = dsta::HaggleAction::AnOyVeyJew(exit_haggle_method);


  brain.m_exits = Some(initial_exit_order.clone());
  brain.haggle_context = initial_exit_ctx;
  brain.total_slippage = 0.0;
  brain.haggle_attempt = 0;
  brain.retry_at_delta = stone.my_accounting.haggle_action.get_retry_period();
  brain.last_haggle_at = Some(chrono::Utc::now());
  brain.m_start = Some(chrono::Utc::now());

  let msgmsg = format!(
    "{} [{}] Initial final exit haggle order created | spread: {:.4} | method: {:?}",
    dbgspt, timestamp, brain.haggle_context.initial_spread, stone.my_accounting.haggle_action
  );
  let _ = f_report_info(stone, brain, stuff, to_dr, &msgmsg).await;

  // ─────────────────────────────────────────────────────────────────────
  // Send the haggled order
  // ─────────────────────────────────────────────────────────────────────
  let net_cost = (initial_exit_order.order_legs[0].price - initial_exit_order.order_legs[1].price) * qty * 100.0;

  let msgmsg = format!(
    "{} [{}] Sending final close: SellToClose {} @ {:.2} + BuyToClose {} @ {:.2} (net {:.2}, haggled)",
    dbgspt, timestamp, brain.m_str_1, initial_exit_order.order_legs[0].price, 
    brain.m_str_2, initial_exit_order.order_legs[1].price, net_cost
  );
  let _ = f_report_info(&stone, &brain, &stuff, &to_dr, &msgmsg).await;
  
  to_dr.tell_dr_pls_trade
    .send(initial_exit_order.clone())
    .await
    .map_err(|e| {
      let errmsg = format!("{} [{}] Failed to send final close order: {}", dbgspt, timestamp, e);
      BotError::OrderSendFailed(errmsg)
    })?;

  // WAIT FOR ORDER_FLY CONFIRMATION
  wait_for_stealth_order_confirmation(stone, brain, stuff, to_dr, told_it_sucks_api, &dbgspt, "exit2").await?;

  brain.m_state = StealthBotState::Check4;

  let msgmsg = format!("{} EXIT! [{}] Final close order sent → moving to Check4", dbgspt, timestamp);
  let _ = f_report_info(&stone, &brain, &stuff, &to_dr, &msgmsg).await;
  Ok(())
}

use crate::sbot::HasFriendlyName;

// 1. Original - for owned type (if you pass stone directly anywhere)
impl HasFriendlyName for dsta::MakeStealthBot {
    fn friendly_name(&self) -> &str {
        &self.my_accounting.friendly_name
    }
}

// 2. Immutable reference - for &stone calls
impl HasFriendlyName for &dsta::MakeStealthBot {
    fn friendly_name(&self) -> &str {
        HasFriendlyName::friendly_name(*self)
    }
}

// 3. Mutable reference - for &mut stone calls (line 370)
impl HasFriendlyName for &mut dsta::MakeStealthBot {
    fn friendly_name(&self) -> &str {
        HasFriendlyName::friendly_name(*self)
    }
}


