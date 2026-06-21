// usta/src/sbot/buzz_bomber.rs

use std::time::Duration;
use chrono::Utc;
use crate::sbot::{f_report_info, f_report_error};


use crate::sbot::errors::{BotError, BotResult};  // ← one line

//===================================================================
// LOCAL DATA STRUCTURES
//===================================================================

#[derive(Clone, Debug, Default, serde::Serialize, serde::Deserialize, PartialEq, Eq)]
pub(crate) enum BuzzState
{ #[default]
  Logic0, // waiting for entry time window + monitoring date/time
  Logic1, // inside entry window → scanning chains & quotes → possibly submit
  Check1, // entry order submitted → waiting for fill confirmation
  Cools1, // just finished entry → waiting algo_cooldown before next scan
  Logic2, // active position → monitoring exit conditions
  Check2, // exit order submitted → waiting for fill (close to zero position)
}

#[derive(Default, Clone, Debug, serde::Serialize, serde::Deserialize)]
pub(crate) struct BuzzBomber 
{ pub m_state: BuzzState,
  pub m_deets:          dsta::PushTickerResult, // for storing tickers.
  pub m_stock:          dsta::Stk,
  pub m_ticks:          Vec<dsta::Opt>, // the options we want tickers for, add 20 at logic 0, reduce to final 2 at logic1 
  pub m_str_long:       String, // quick access to the option name to use for orders
  pub m_str_short:      String, // quick access to the option name to use for orders
  pub m_count:          f64, // number of contracts we entered with
  pub m_credit_received:f64, // actual credit we get.
  // Trade statistics / current cycle data
  pub m_cooldown_until: Option<chrono::DateTime<chrono::Utc>>, // next allowed scan
  pub m_last_selected_expiry: Option<chrono::DateTime<chrono::Utc>>, // next allowed scan

  pub m_entry: Option<dsta::Order>,
  pub m_begin: Option<chrono::DateTime<chrono::Utc>>, // cycle start 

  pub haggle_context:  dsta::HaggleContext,
  pub retry_at_delta:  chrono::TimeDelta,
  pub total_slippage:  f64,
  pub haggle_attempt:  u32,
  pub last_haggle_at:  Option<chrono::DateTime<chrono::Utc>>,

  pub m_start: Option<chrono::DateTime<chrono::Utc>>, // when the exit started}
  pub m_exits: Option<dsta::Order> 
}

//===================================================================
// BOT CREATION & MAIN TASK
//===================================================================

pub(crate) async fn f_make_bot_buzz_2
( stone: dsta::MakeBuzzBomber,
) -> ( tokio::task::JoinHandle<()>, crate::CtorBot )
{ let friendly_name = &stone.my_accounting.friendly_name;
  let tracking_tick = &stone.my_accounting.tracking_tick;
  let dbgspt = format!("[BUZZ_FN_MAKE2 - {}]", friendly_name);
  log::debug!
  ( "{} [ENTRY] f_make_bot_buzz_2() - tracking {}"
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
  log::debug!("{} [EXIT] fn_make_bot_buzz_2", dbgspt);
  ( tokio::spawn
    ( async move
      { fn_run_main_buzz_bot
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

pub(crate) async fn fn_run_main_buzz_bot
( constructor_stuff: dsta::MakeBuzzBomber,
  mut told_to_trade_api: tokio::sync::mpsc::Receiver<dsta::Ticker>,
  mut told_to_check_api: tokio::sync::mpsc::Receiver<crate::Allocation>,
  mut told_it_sucks_api: tokio::sync::mpsc::Receiver<String>,
  to_dr: crate::BridgeDrHand,
)
{ let dbgspt = format!("[MAIN-BUZZ {}]", constructor_stuff.my_accounting.friendly_name);
  log::debug!("{} ENTRY!", dbgspt);

  let mut stone = constructor_stuff;
  let mut stuff = crate::Allocation
  { bot_name: format!("{}", stone.my_accounting.friendly_name),
    bot_safe: crate::BotaAccount::default(),
    bot_mind: crate::BotMindState::Running,
    num_swap: 0,
  };

  // ── STEP 1: Request asset details for tracking tick ───────────────────────
  let msgmsg = format!("{} STEP 1: Request asset details for {}...", dbgspt, stone.my_accounting.tracking_tick);
  let temp_brain = BuzzBomber::default();
  let _ = f_report_info(&stone, &temp_brain, &stuff, &to_dr, &msgmsg).await;

  let deets = {
    let (tx_asset, mut rx_asset) = tokio::sync::mpsc::channel(2);
    if to_dr.tell_dr_pls_ticks
      .send((stone.my_accounting.tracking_tick.clone(), tx_asset))
      .await
      .is_err()
    { let errmsg = format!("{} Ticker request send failed", dbgspt);
      let _ = f_report_error(&stone, &temp_brain, &stuff, &to_dr, &errmsg).await;
      return;
    }

    match tokio::time::timeout(Duration::from_secs(5), rx_asset.recv()).await
    { Ok(Some(Some(d))) => d,
      _ =>
      { let errmsg = format!("{} Asset details request timeout or channel closed (5s)", dbgspt);
        let _ = f_report_error(&stone, &temp_brain, &stuff, &to_dr, &errmsg).await;
        return;
      }
    }
  };

  // ── STEP 2: Attempt to load existing bot brain state ──────────────────────
  let msgmsg = format!("{} STEP 2: Attempting to load bot brain state...", dbgspt);
  let _ = f_report_info(&stone, &temp_brain, &stuff, &to_dr, &msgmsg).await;

  let (mut brain, is_fresh_bot) = match crate::sbot::fn_load_bot_state(&stone.my_accounting.friendly_name).await
  { Ok ( ( load_stone, load_brain, load_stuff ) ) => 
    { stone = load_stone; stuff = load_stuff;
      let msgmsg = format!("{} [LOAD SUCCESS] Loaded existing state from disk", dbgspt);
      let _ = f_report_info(&stone, &load_brain, &stuff, &to_dr, &msgmsg).await;
      ( load_brain, false )
    }
    Err(e) =>
    { log::info!("{} [NEW] No saved state found: {}. Creating fresh bot.", dbgspt, e);
      let fresh = BuzzBomber 
      { m_state: BuzzState::Logic0,
        m_deets: deets.clone(),
        m_stock: match deets.ass_deets {
            dsta::FinAss::StkDeets(ref fsd) => fsd.clone(),
            _ => {
                let errmsg = "Asset details indicate invalid underlying".to_string();
                let _ = f_report_error(&stone, &temp_brain, &stuff, &to_dr, &errmsg).await;
                return;
            }
        },
        haggle_context: dsta::HaggleContext::default(),
        retry_at_delta: stone.my_accounting.haggle_action.get_retry_period(),
        total_slippage: 0.0,
        haggle_attempt: 0,
        last_haggle_at: None,
        m_begin: Some(chrono::Utc::now()),
        ..Default::default()
      };

      (fresh, true)
    }
  };

  // ── STEP 3 - initialing bot 
  if is_fresh_bot
  { // STEP 3.0a – Request initial cash
    let msgmsg = format!("{} STEP 3.0a: Requesting initial cash allocation...", dbgspt);
    let _ = f_report_info(&stone, &brain, &stuff, &to_dr, &msgmsg).await;

    let _ = to_dr.tell_dr_swap_info
      .send(crate::BotSwaper
      { source_offering: Some(crate::SwapWat
        { who: format!("{}", stone.my_accounting.friendly_name),
          wat: dsta::PositionsList::default(),
          how: 0.0,
        }),
        target_released: Some(crate::SwapWat
        { who: format!("USER_CASH"),
          wat: dsta::PositionsList::default(),
          how: stone.my_cash_alloc,
        }),
      })
      .await;

    // STEP 3.1a – Wait for SWAP reply confirmation
    // Law 1: wait for SWAP_SUCC before proceeding; also check DIE (Law 8)
    let msgmsg = format!("{} STEP 3.1a: Waiting for SWAP success reply...", dbgspt);
    let _ = f_report_info(&stone, &brain, &stuff, &to_dr, &msgmsg).await;

    let m_begin = chrono::Utc::now();
    loop
    { let swap_reply = tokio::select!
      { res = told_it_sucks_api.recv() => res,
        _ = tokio::time::sleep(Duration::from_secs(1)) => None,
      };

      match swap_reply
      { Some(msg) if msg == "DIE" =>  // Law 8: check DIE in every waiting loop
        { let errmsg = format!("{} STEP 3.1a: DIE received while waiting for SWAP reply", dbgspt);
          log::debug!("{}", errmsg);
          stuff.bot_mind = crate::BotMindState::Zombied;
          let _ = to_dr.tell_dr_new_brain.send(stuff).await;
          return;
        }
        Some(msg) if msg == crate::glossary::SWAP_SUCC =>
        { let msgmsg = format!("{} STEP 3.1a: SWAP SUCCESS - cash allocated", dbgspt);
          let _ = f_report_info(&stone, &brain, &stuff, &to_dr, &msgmsg).await;
          break;
        }
        Some(msg) if msg == crate::glossary::SWAP_FAIL =>
        { let errmsg = format!("{} STEP 3.1a: SWAP FAIL - allocation rejected", dbgspt);
          let _ = f_report_error(&stone, &brain, &stuff, &to_dr, &errmsg).await;
          stuff.bot_mind = crate::BotMindState::Zombied;
          let _ = to_dr.tell_dr_new_brain.send(stuff).await;
          return;
        }
        None if chrono::Utc::now() - m_begin > chrono::Duration::seconds(10) =>
        { let errmsg = format!("{} STEP 3.1a: Timeout waiting for SWAP reply", dbgspt);
          let _ = f_report_error(&stone, &brain, &stuff, &to_dr, &errmsg).await;
          stuff.bot_mind = crate::BotMindState::Zombied;
          let _ = to_dr.tell_dr_new_brain.send(stuff).await;
          return;
        }
        _ => continue,
      }
    }

    // STEP 3.2a – Confirm cash via account update
    let msgmsg = format!("{} STEP 3.2a: Confirming cash allocation via account update...", dbgspt);
    let _ = f_report_info(&stone, &brain, &stuff, &to_dr, &msgmsg).await;

    let m_begin = chrono::Utc::now();
    loop
    { if chrono::Utc::now() - m_begin > chrono::Duration::seconds(10)
      { let errmsg = format!("{} STEP 3.2a: Timeout waiting for cash confirmation", dbgspt);
        let _ = f_report_error(&stone, &brain, &stuff, &to_dr, &errmsg).await;
        stuff.bot_mind = crate::BotMindState::Zombied;
        let _ = to_dr.tell_dr_new_brain.send(stuff).await;
        return;
      }

      let alloc_update = tokio::select!
      { res = told_to_check_api.recv() => res,
        _ = tokio::time::sleep(Duration::from_secs(1)) => None,
      };

      let Some(alloc) = alloc_update else { continue; };

      if alloc.bot_name != stuff.bot_name { continue; }

      if alloc.bot_safe.m_cash >= stone.my_cash_alloc - 1.0
      { let msgmsg = format!("{} STEP 3.2a: Cash confirmed (${:.2})", dbgspt, alloc.bot_safe.m_cash);
        let _ = f_report_info(&stone, &brain, &stuff, &to_dr, &msgmsg).await;
        stuff = alloc;
        break;
      }
      else
      { let errmsg = format!("{} STEP 3.2a: Cash mismatch - got ${:.2}, expected ~${:.2}", dbgspt, alloc.bot_safe.m_cash, stone.my_cash_alloc);
        let _ = f_report_error(&stone, &brain, &stuff, &to_dr, &errmsg).await;
        stuff.bot_mind = crate::BotMindState::Zombied;
        let _ = to_dr.tell_dr_new_brain.send(stuff).await;
        return;
      }
    }

    // STEP 3.3a – Wait for first underlying quote
    let msgmsg = format!("{} STEP 3.3a: Waiting for first underlying quote...", dbgspt);
    let _ = f_report_info(&stone, &brain, &stuff, &to_dr, &msgmsg).await;

    let _first_quote = loop
    { tokio::select!
      { Some(ticker) = told_to_trade_api.recv() =>
        { if ticker.name == stone.my_accounting.tracking_tick
          { let msg = format!("{} STEP 3.3a: First underlying quote received (bid={:.2}, ask={:.2})", dbgspt, ticker.bid, ticker.ask);
            let _ = f_report_info(&stone, &brain, &stuff, &to_dr, &msg).await;
            break ticker;
          }
        }
        _ = tokio::time::sleep(Duration::from_secs(15)) =>
        { let errmsg = format!("{} STEP 3.3a: Timeout waiting for first quote", dbgspt);
          let _ = f_report_error(&stone, &brain, &stuff, &to_dr, &errmsg).await;
          stuff.bot_mind = crate::BotMindState::Zombied;
          let _ = to_dr.tell_dr_new_brain.send(stuff).await;
          return;
        }
      }
    };

    let msgmsg = format!("{} STEP 3.3a: Fresh bot ready, starting in Logic0", dbgspt);
    let _ = f_report_info(&stone, &brain, &stuff, &to_dr, &msgmsg).await;
  }
  else
  { // STEP 3.0b – resuming saved bot
    let msgmsg = format!("{} STEP 3.0b: Resuming saved bot state...", dbgspt);
    let _ = f_report_info(&stone, &brain, &stuff, &to_dr, &msgmsg).await;

    // Re-request tickers for remembered legs if needed
    for opt in &brain.m_ticks {
        let (tx, mut rx) = tokio::sync::mpsc::channel(1);
        if let Err(e) = to_dr.tell_dr_pls_ticks.send((opt.ticker_name.clone(), tx)).await {
            log::warn!("{} Failed to request ticker for {}: {}", dbgspt, opt.ticker_name, e);
            continue;
        }
        let _ = tokio::time::timeout(Duration::from_secs(5), rx.recv()).await;
    }
  }

  // ── STEP 4: Main forever loop ────────────────────────────────────────────
  let msgmsg = format!("{} STEP 4: Entering main event loop...", dbgspt);
  let _ = f_report_info(&stone, &brain, &stuff, &to_dr, &msgmsg).await;

  loop
  { tokio::select!
    { res = told_to_trade_api.recv() =>
      { if let Some(ticker) = res 
        { if ticker.name == stone.my_accounting.tracking_tick ||
            brain.m_ticks.iter().any(|opt| opt.ticker_name == ticker.name) 
          { if let Err(e) = handle_quote(&ticker, &mut stone, &mut brain, &mut stuff, &to_dr, &mut told_it_sucks_api).await {
                let errmsg = format!("Quote handling failed: {}", e);
                let _ = f_report_error(&stone, &brain, &stuff, &to_dr, &errmsg).await;
                break;
              }
          }
        } 
        else 
        {
              let errmsg = "trade channel closed".to_string();
              let _ = f_report_error(&stone, &brain, &stuff, &to_dr, &errmsg).await;
              break;
        }
      },

      res = told_to_check_api.recv() =>
      { if let Some(alloc) = res
        { if alloc.bot_name == stone.my_accounting.friendly_name
          { if let Err(e) = handle_check(&alloc, &mut stone, &mut brain, &mut stuff, &to_dr, &mut told_it_sucks_api).await
            { let errmsg = format!("Account update handling failed: {}", e);
              let _ = f_report_error(&stone, &brain, &stuff, &to_dr, &errmsg).await;
              break;
            }
          }
        }
      }

      res = told_it_sucks_api.recv() =>
      { if let Some(msg) = res
        { if msg == "DIE"   // Law 8: check DIE in main loop
          { log::debug!("{} [EXIT] Received DIE signal", dbgspt);
            break;
          }
          // Other messages (ORDER_FLY, ORDER_BAD, etc.) are handled inside
          // wait_for_buzz_order_confirmation during the blocking wait; if they
          // arrive here it means we were not in a wait — safe to ignore.
          else
          { log::debug!("{} Unhandled sucks message (not in wait): {}", dbgspt, msg);
          }
        }
      }

      _ = tokio::time::sleep(Duration::from_secs(30)) =>
      { let msgmsg = format!("{} STEP 4: waiting for next event...", dbgspt);
        let _ = f_report_info(&stone, &brain, &stuff, &to_dr, &msgmsg).await;
      }
    }
  }

  // ── STEP 5: Shutdown ─────────────────────────────────────────────────────
  let msgmsg = format!("{} STEP 5: Buzz bomber shutting down!", dbgspt);
  let _ = f_report_info(&stone, &brain, &stuff, &to_dr, &msgmsg).await;

  stuff.bot_mind = crate::BotMindState::Zombied;
  let _ = to_dr.tell_dr_new_brain.send(stuff).await;
}

//===================================================================
// CORE ROBO API HANDLERS
//===================================================================

async fn handle_check
( check: &crate::Allocation,
  stone: &mut dsta::MakeBuzzBomber,
  brain: &mut BuzzBomber,
  stuff: &mut crate::Allocation,
  to_dr: &crate::BridgeDrHand,
  told_it_sucks_api: &mut tokio::sync::mpsc::Receiver<String>,  // Law 2: needed for order confirmation in state handlers
) -> BotResult
{ if check.bot_name != stuff.bot_name { return Ok(()); }
  if check.bot_mind != stuff.bot_mind
  { return state_machine_mind(check, stone, brain, stuff, to_dr, told_it_sucks_api).await; }
  else
  { *stuff = check.clone();
    return state_machine_tick(stone, brain, stuff, to_dr, told_it_sucks_api).await;
  }
}

async fn handle_quote(
    ticks: &dsta::Ticker,
    stone: &mut dsta::MakeBuzzBomber,
    brain: &mut BuzzBomber,
    stuff: &mut crate::Allocation,
    to_dr: &crate::BridgeDrHand,
    told_it_sucks_api: &mut tokio::sync::mpsc::Receiver<String>,  // Law 2: needed for order confirmation in state handlers
) -> BotResult 
{
    let dbgspt = format!("[BUZZ QUOTE {}]", stone.my_accounting.friendly_name);

    // ── Handle underlying stock first (highest priority) ──────────────────
    if brain.m_stock.ticker_name == ticks.name {
        log::trace!("{} updating underlying {}", dbgspt, ticks.name);
        brain.m_stock.last_ticker = Some(ticks.clone());
        return state_machine_tick(stone, brain, stuff, to_dr, told_it_sucks_api).await;
    }

    // ── Look for match in candidate options (m_ticks) ─────────────────────
    let mut updated = false;

    for opt in brain.m_ticks.iter_mut() {
        if opt.ticker_name == ticks.name {
            log::trace!("{} updating candidate option {}", dbgspt, ticks.name);
            opt.last_ticker = Some(ticks.clone());
            updated = true;
            break;  // assume ticker names are unique → no need to continue searching
        }
    }

    // ── Log unknown tickers at debug level (no error, no zombie) ───────────
    if !updated 
    {
      log::debug!("{} received ticker not in candidates or underlying: {}", 
                    dbgspt, ticks.name);
      Ok(())
    }
    else
    { state_machine_tick(stone, brain, stuff, to_dr, told_it_sucks_api).await
    }

}

//===================================================================
// STATE MACHINE PROCESSORS
//===================================================================

async fn state_machine_mind
( check: &crate::Allocation,
  stone: &mut dsta::MakeBuzzBomber,
  brain: &mut BuzzBomber,
  stuff: &mut crate::Allocation,
  to_dr: &crate::BridgeDrHand,
  told_it_sucks_api: &mut tokio::sync::mpsc::Receiver<String>,
) -> BotResult
{ let dbgspt = format!("[MIND_SWAP {}]", stone.my_accounting.friendly_name);
  let timestamp = chrono::Utc::now().format("%Y-%m-%d %H:%M:%S%.f").to_string();
  log::info!("{} [{}] Buzz bot mind transitioning to {:#?} from {:#?}", dbgspt, timestamp, check.bot_mind, stuff.bot_mind);
  match check.bot_mind
  { crate::BotMindState::Running =>
    { match stuff.bot_mind
      { crate::BotMindState::Trading =>
        { log::info!("{} mind transition indicates trade completed", dbgspt);
          *stuff = check.clone();
          return state_machine_tick(stone, brain, stuff, to_dr, told_it_sucks_api).await;
        }
        crate::BotMindState::Stopped =>
        { // Law 9: save when transitioning FROM stopped (the non-stopped value)
          let msgmsg = format!("{} mind transition from Stopped → Running: saving state", dbgspt);
          let _ = f_report_info(&stone, &brain, &stuff, &to_dr, &msgmsg).await;
          *stuff = check.clone();
          return crate::sbot::fn_save_bot_state(stone, brain, stuff, to_dr).await;
        }
        _ => { *stuff = check.clone(); }
      }
    }
    crate::BotMindState::Trading => 
    { log::info!("{} mind transition indicates successful order dispatch", dbgspt);
      *stuff = check.clone();
    }
    crate::BotMindState::Stopped =>
    { // Law 9: save when transitioning TO stopped
      log::info!("{} mind transition TO Stopped: saving state", dbgspt);
      *stuff = check.clone();
      return crate::sbot::fn_save_bot_state(stone, brain, stuff, to_dr).await;
    }
    crate::BotMindState::Seppuku =>
    { *stuff = check.clone();
      let errmsg = format!("{} [{}] Mind transition to Seppuku - self-terminate", dbgspt, timestamp);
      return Err(BotError::Other(errmsg));
    }
    crate::BotMindState::Zombied =>
    { *stuff = check.clone();
      let errmsg = format!("{} [{}] Mind transition to Zombied - external kill", dbgspt, timestamp);
      return Err(BotError::Other(errmsg));
    }
  }
  Ok(())
}

async fn state_machine_tick
( stone: &mut dsta::MakeBuzzBomber
, brain: &mut BuzzBomber
, stuff: &mut crate::Allocation
, to_dr: &crate::BridgeDrHand
, told_it_sucks_api: &mut tokio::sync::mpsc::Receiver<String>,  // Law 2: flow through to order-sending states
) -> BotResult<()>
{ log::trace!("state machine ticking with {:#?}", brain.m_state );

  match brain.m_state
  {   BuzzState::Logic0 => f_run_logic0(stone,brain,stuff,to_dr).await?
    , BuzzState::Logic1 => f_run_logic1(stone,brain,stuff,to_dr,told_it_sucks_api).await?
    , BuzzState::Check1 => f_run_check1(stone,brain,stuff,to_dr,told_it_sucks_api).await?
    , BuzzState::Cools1 => f_run_cools1(stone,brain,stuff,to_dr).await?
    , BuzzState::Logic2 => f_run_logic2(stone,brain,stuff,to_dr,told_it_sucks_api).await?
    , BuzzState::Check2 => f_run_check2(stone,brain,stuff,to_dr).await?
  }
  Ok(())
}

//===================================================================
// HELPER: Wait for ORDER_FLY / ORDER_BAD confirmation  (Law 2 + Law 8)
//===================================================================
async fn wait_for_buzz_order_confirmation(
    stone: &dsta::MakeBuzzBomber,
    brain: &mut BuzzBomber,
    stuff: &crate::Allocation,
    to_dr: &crate::BridgeDrHand,
    told_it_sucks_api: &mut tokio::sync::mpsc::Receiver<String>,
    dbgspt: &str,
    context: &str, // e.g. "entry", "entry-haggle", "exit"
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

        match tokio::time::timeout(
            std::time::Duration::from_secs(1),
            told_it_sucks_api.recv()
        ).await
        {
            Ok(Some(msg)) if msg == "DIE" =>
            {
                // Law 8: DIE must be checked FIRST in every waiting loop, before any other message handling
                let errmsg = format!("{} [{}] ({}) DIE received while waiting for ORDER_FLY", dbgspt, timestamp, context);
                log::debug!("{}", errmsg);
                return Err(BotError::Other(errmsg));
            }
            Ok(Some(msg)) if msg == "ORDER_FLY" =>
            {
                let msg_str = format!("{} [{}] ({}) ORDER_FLY confirmed - order routed successfully", dbgspt, timestamp, context);
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
            {
                // Transient / unexpected message — log and continue.
                // NOTE: DIE is already handled above, so this arm never silently swallows it.
                let errmsg = format!("Buzz bot got a transient message while waiting for order fly/bad: {:#?}", msg);
                log::warn!("{}", errmsg);
                let _ = f_report_error(stone, brain, stuff, to_dr, &errmsg).await;
                continue;
            }
            Ok(None) =>
            {
                let errmsg = format!("{} [{}] ({}) Order confirmation channel closed", dbgspt, timestamp, context);
                log::error!("{}", errmsg);
                return Err(BotError::OrderSendFailed(errmsg));
            }
            Err(_) =>
            {
                // 1-second poll timeout — loop and check elapsed again
                continue;
            }
        }
    }
}

//===================================================================
// STATE MACHINE HANDLERS
//===================================================================

async fn f_run_logic0
( stone: &mut dsta::MakeBuzzBomber,
  brain: &mut BuzzBomber,
  stuff: &mut crate::Allocation,
  to_dr: &crate::BridgeDrHand,
) -> BotResult<()> 
{ let dbgspt = format!("[BUZZ L0 {}]", stone.my_accounting.friendly_name);
  let now = Utc::now();
  log::debug!("{} ENTRY! @ {:#?} ", dbgspt, now);

  if let Some(target) = stone.time_to_order.get_entry_time(now)
  { if now < target
    { let remaining = target - now;
      log::trace!("{} Waiting for entry — {} until {}", dbgspt, remaining, target);
      return Ok(());
    }
    // now >= target → proceed
    let msg = format!("{} Entry time reached (target was {})", dbgspt, target);
    let _ = f_report_info(stone, brain, stuff, to_dr, &msg).await;
  }
  else
  { // wrong day of week
    log::trace!("{} wrong day of the week! ", dbgspt);
    return Ok(());
  }

    // Refresh underlying + full chain
    
    async fn request_fresh_asset_details(
        symbol: &str,
        to_dr: &crate::BridgeDrHand,
    ) -> Result<dsta::PushTickerResult, BotError> {
        let (tx, mut rx) = tokio::sync::mpsc::channel(2);
        to_dr.tell_dr_pls_ticks
            .send((symbol.to_string(), tx))
            .await
            .map_err(|_| BotError::ChannelClosed)?;

        tokio::time::timeout(Duration::from_secs(6), rx.recv())
            .await
            .map_err(|_| BotError::Timeout("asset details timeout".into()))?
            .ok_or(BotError::MissingData("no response".into()))
            .and_then(|opt| opt.ok_or(BotError::MissingData("empty response".into())))
    }

    // ── Refresh full chain & underlying quote ─────────────────────────────
    let deets = match request_fresh_asset_details(
        &stone.my_accounting.tracking_tick,
        to_dr,
    ).await {
        Ok(d) => d,
        Err(e) => {
            let msg = format!("{} Failed to refresh chain in Logic0: {}", dbgspt, e);
            let _ = f_report_error(stone, brain, stuff, to_dr, &msg).await;
            return Ok(());
        }
    };
    brain.m_deets = deets;

    let msg = format!
    ( "{} Entry window hit → chain refreshed ({} expiries)",
      dbgspt, brain.m_deets.opt_chain.len()
    );
    let _ = f_report_info(stone, brain, stuff, to_dr, &msg).await;

    // ── Choose target expiry ──────────────────────────────────────────────
    let min_dte = stone.option_expire as i64;
    let min_exp_time = now + chrono::Duration::days(min_dte);

    let Some(target_expiry) = brain.m_deets.opt_chain
        .iter()
        .filter(|e| e.expire_time >= min_exp_time)
        .min_by_key(|e| e.expire_time)           // closest expiry that meets min DTE
    else {
        let msg = format!("{} No expiry ≥ {} DTE found", dbgspt, min_dte);
        let _ = f_report_info(stone, brain, stuff, to_dr, &msg).await;
        return Ok(());
    };

    let dte = (target_expiry.expire_time - now).num_days();
    let msg = format!(
        "{} Targeting expiry {} (DTE ≈ {})",
        dbgspt, target_expiry.expire_time.format("%Y-%m-%d"), dte
    );
    let _ = f_report_info(stone, brain, stuff, to_dr, &msg).await;

    // ── Select chain (puts or calls) ──────────────────────────────────────
    let chain = match stone.stonk_feeling {
        dsta::MarketDirection::GetStonk  => &target_expiry.option_puts,   // bullish → put credit
        dsta::MarketDirection::Corrects  => &target_expiry.option_calls,  // bearish → call credit
        _ => {
            log::warn!("{} Invalid market direction", dbgspt);
            return Ok(());
        }
    };

    if chain.len() < 8 {
        log::warn!("{} Too few strikes ({}) in selected chain", dbgspt, chain.len());
        return Ok(());
    }

    // ── Find approximate ATM ──────────────────────────────────────────────
    let underlying = brain.m_stock
        .last_ticker
        .as_ref()
        .ok_or_else(|| BotError::MissingData("no underlying quote in Logic0".into()))?;

    let mid = (underlying.bid + underlying.ask) / 2.0;

    let mut closest_idx = 0;
    let mut min_diff = f64::MAX;
    for (i, opt) in chain.iter().enumerate() {
        let diff = (opt.strike_spot - mid).abs();
        if diff < min_diff {
            min_diff = diff;
            closest_idx = i;
        }
    }

    // ── Take ~20–25 candidates around ATM ─────────────────────────────────
    let window = 12;
    let start = closest_idx.saturating_sub(window).max(0);
    let end   = (closest_idx + window + 1).min(chain.len());

    brain.m_ticks = chain[start..end].to_vec();

    let msg = format!(
        "{} Selected {} candidate options around ATM (${:.2}) for expiry {}",
        dbgspt, brain.m_ticks.len(), mid, target_expiry.expire_time.format("%m-%d")
    );
    let _ = f_report_info(stone, brain, stuff, to_dr, &msg).await;

    // ── Subscribe to all candidate option tickers ─────────────────────────
    for opt in &brain.m_ticks 
    { let (tx, mut rx) = tokio::sync::mpsc::channel(2);
      
      if to_dr.tell_dr_pls_ticks.send((opt.ticker_name.clone(), tx)).await.is_err() 
      { log::warn!("{} Failed to request ticker for {}", dbgspt, opt.ticker_name);
        continue;
      }

      // non-blocking — we don't wait here, quotes will arrive async
      let _ = tokio::time::timeout(Duration::from_millis(400), rx.recv()).await.ok();
    }

    // Remember which expiry we're working with this cycle
    brain.m_last_selected_expiry = Some(target_expiry.expire_time);

    brain.m_state = BuzzState::Logic1;
    Ok(())
}

async fn f_run_logic1
( stone: &mut dsta::MakeBuzzBomber,
  brain: &mut BuzzBomber,
  stuff: &mut crate::Allocation,
  to_dr: &crate::BridgeDrHand,
  told_it_sucks_api: &mut tokio::sync::mpsc::Receiver<String>,  // Law 2: needed for order confirmation
) -> BotResult<()> 
{
    let dbgspt = format!("[BUZZ LOGIC1 {}]", stone.my_accounting.friendly_name);
    let timestamp = Utc::now().format("%Y-%m-%d %H:%M:%S%.f").to_string();

    // Early exit if we don't have enough candidates yet
    if brain.m_ticks.len() < 3 {
        log::trace!("{} [{}] waiting for more option quotes", dbgspt, timestamp);
        return Ok(());
    }

    let is_put_credit = matches!(stone.stonk_feeling, dsta::MarketDirection::GetStonk);
    let target_credit = stone.target_spread / 100.0;

    let Some((best_credit, short_leg, long_leg)) = select_best_spread
    ( &brain.m_ticks, 
      stone.stonk_feeling, 
      target_credit
    ) 
    else 
    { return Ok(()); // No qualifying spread found
    };

    // ── Report to frontend ─────────────────────────────────────────────────
    let direction = if is_put_credit { "PUT CREDIT" } else { "CALL CREDIT" };

    let msg1 = format!(
        "{} [{}] {} SPREAD ENTRY TRIGGERED! credit {:.2} (target {:.2})",
        dbgspt, timestamp, direction, best_credit, target_credit
    );
    let msg2 = format!(
        "short → {} (${:.0}) | long → {} (${:.0})",
        short_leg.option_name, short_leg.strike_spot,
        long_leg.option_name, long_leg.strike_spot
    );
    let msgmsg = format!("{}\n{}", msg1, msg2);
    let _ = f_report_info(stone, brain, stuff, to_dr, &msgmsg).await;

    // ── Retain only the selected legs ─────────────────────────────────────
    let keep_names = [
        short_leg.ticker_name.clone(),
        long_leg.ticker_name.clone(),
    ];

    let to_unsubscribe: Vec<String> = brain.m_ticks
        .iter()
        .filter(|opt| !keep_names.contains(&opt.ticker_name))
        .map(|opt| opt.ticker_name.clone())
        .collect();

    brain.m_ticks = vec![short_leg, long_leg];

    // ── Unsubscribe from everything else ──────────────────────────────────
    for sym in to_unsubscribe {
        log::trace!("{} unsubscribing {}", dbgspt, sym);
        let _ = to_dr.tell_dr_stop_tick.send(sym).await;
    }

    // ── Quick access for order generation ─────────────────────────────────
    brain.m_str_short = brain.m_ticks[0].option_name.clone();
    brain.m_str_long  = brain.m_ticks[1].option_name.clone();

    // ── Place the order (Law 2: told_it_sucks_api flows through) ──────────
    f_run_entry_order(stone, brain, stuff, to_dr, told_it_sucks_api).await
}

// wait for entry order fill (both legs BuyToOpen long + SellToOpen short)
async fn f_run_check1
( stone: &mut dsta::MakeBuzzBomber,
  brain: &mut BuzzBomber,
  stuff: &mut crate::Allocation,
  to_dr: &crate::BridgeDrHand,
  told_it_sucks_api: &mut tokio::sync::mpsc::Receiver<String>,  // Law 2: needed for haggle retry confirmation
) -> BotResult<()> {
  let dbgspt = format!("[BUZZ CHECK1 {}]", stuff.bot_name);
  let timestamp = chrono::Utc::now().format("%Y-%m-%d %H:%M:%S%.f").to_string();
  log::debug!("{} [{}] ENTRY! - waiting for entry order fill", dbgspt, timestamp);

  let mut long_owned  = 0.0;
  let mut long_price  = 0.0;
  let mut short_owned = 0.0;
  let mut short_price = 0.0;
  let mut long_found  = false;
  let mut short_found = false;

  // long leg (BuyToOpen)
  let long_field = if stone.stonk_feeling == dsta::MarketDirection::GetStonk {
    &stuff.bot_safe.m_long_puts
  } else {
    &stuff.bot_safe.m_long_calls
  };
  if let Some(pos) = long_field {
    if let dsta::FinAss::OptDeets(ref opt) = pos.asset {
      if opt.option_name == brain.m_str_long {
        long_owned = pos.owned;
        long_price = pos.price;
        long_found = true;
      }
    }
  }

  // short leg (SellToOpen)
  let short_field = if stone.stonk_feeling == dsta::MarketDirection::GetStonk {
    &stuff.bot_safe.m_short_puts
  } else {
    &stuff.bot_safe.m_short_calls
  };
  if let Some(pos) = short_field {
    if let dsta::FinAss::OptDeets(ref opt) = pos.asset {
      if opt.option_name == brain.m_str_short {
        short_owned = pos.owned.abs();
        short_price = pos.price;
        short_found = true;
      }
    }
  }

  // no positions detected yet
  if !long_found && !short_found {
    log::trace!("{} [{}] No positions detected yet", dbgspt, timestamp);
    return Ok(());
  }

  // partial fill detection + timeout
  let long_partial  = long_found  && (long_owned  - brain.m_count).abs() > 0.01;
  let short_partial = short_found && (short_owned - brain.m_count).abs() > 0.01;

  if long_partial || short_partial {
    let msgmsg = format!(
      "{} [{}] Partial fill detected: long {:.2}/{:.2}, short {:.2}/{:.2} — waiting or haggling",
      dbgspt, timestamp, long_owned, brain.m_count, short_owned, brain.m_count
    );
    let _ = f_report_info(stone, brain, stuff, to_dr, &msgmsg).await;

    // TIMEOUT check still applies for overall failure
    if let Some(begin) = brain.m_begin {
      let elapsed = Utc::now() - begin;
      if elapsed > chrono::Duration::seconds(30) {
        let errmsg = format!(
          "{} [{}] TIMEOUT waiting for full fill after {:.0}s (long: {:.2}/{:.2}, short: {:.2}/{:.2})",
          dbgspt, timestamp, elapsed.num_seconds(), long_owned, brain.m_count, short_owned, brain.m_count
        );
        return Err(BotError::Timeout(errmsg));
      }
    }

    // HAGGLE RETRY: only if enough time since last haggle
    if let Some(last) = brain.last_haggle_at {
      if Utc::now() - last < brain.retry_at_delta {
        return Ok(());
      }
    } else {
      return Ok(());
    }

    // Build haggle order for remaining quantity
    let orig = match brain.m_entry.as_ref() {
      Some(o) => o,
      None => {
        log::warn!("{} [{}] Partial fill but no m_entry stored – cannot haggle", dbgspt, timestamp);
        return Ok(());
      }
    };

    let filled = long_owned.min(short_owned);
    let remaining_qty = (brain.m_count - filled).max(0.0);
    if remaining_qty < 0.01 {
      return Ok(());
    }

    let mut haggle_order = dsta::Order {
      order_legs: orig.order_legs.iter().map(|leg| {
        let mut l = leg.clone();
        l.quantity = remaining_qty;
        l.remaining = remaining_qty;
        l
      }).collect(),
      order_type: dsta::MarketOrLimit::Limit,
      ..orig.clone()
    };

    let slippage_delta = stone.my_accounting
      .haggle_action
      .apply_haggle_changes(
        &mut haggle_order,
        &mut brain.haggle_context,
      );

    brain.total_slippage += slippage_delta;
    brain.haggle_attempt += 1;
    brain.last_haggle_at = Some(Utc::now());

    let slip_max = stone.my_accounting.haggle_action.get_slippage_max();
    if brain.total_slippage > slip_max {
      let msgmsg = format!(
        "{} [{}] Entry slippage limit exceeded – accepting partial fill {:.2} and moving on",
        dbgspt, timestamp, filled
      );
      let _ = f_report_info(stone, brain, stuff, to_dr, &msgmsg).await;

      brain.m_count = filled.max(0.0);

      if brain.m_count <= 0.01 {
        return Err(BotError::Other(
          "Max slippage exceeded with zero fill – entry canceled".into(),
        ));
      }

      if let Some(ref mut entry) = brain.m_entry {
        entry.order_legs.iter_mut().for_each(|leg| {
          leg.quantity = brain.m_count;
          leg.remaining = brain.m_count;
        });
      }

      return Ok(());
    }

    let msgmsg = format!(
      "{} [{}] Entry haggle attempt {} – remaining {:.2}, slippage_step {:.4}, total_slippage {:.4}",
      dbgspt, timestamp, brain.haggle_attempt, remaining_qty, slippage_delta, brain.total_slippage
    );
    let _ = f_report_info(stone, brain, stuff, to_dr, &msgmsg).await;

    to_dr.tell_dr_pls_trade
      .send(haggle_order.clone())
      .await
      .map_err(|e| BotError::Other(format!("Failed to send entry haggle order: {}", e)))?;

    // Law 2: wait for ORDER_FLY on haggle retry
    wait_for_buzz_order_confirmation(stone, brain, stuff, to_dr, told_it_sucks_api, &dbgspt, "entry-haggle").await?;

    brain.m_entry = Some(haggle_order);
    return Ok(());
  }

  // overfill warning
  if long_found  && long_owned  > brain.m_count + 0.01 ||
     short_found && short_owned > brain.m_count + 0.01 {
    let msgmsg = format!(
      "{} [{}] OVERFILL warning: long {:.2}, short {:.2} (expected {:.2})",
      dbgspt, timestamp, long_owned, short_owned, brain.m_count
    );
    let _ = f_report_info(stone, brain, stuff, to_dr, &msgmsg).await;
  }

  // full fill → transition
  if long_found && short_found &&
     long_owned  >= brain.m_count - 0.01 &&
     short_owned >= brain.m_count - 0.01 {
    brain.m_credit_received = short_price - long_price;
    brain.m_cooldown_until  = Some(Utc::now() + stone.algo_cooldown);
    brain.m_state = BuzzState::Cools1;

    let msgmsg = format!(
      "{} [{}] FULL ENTRY FILL CONFIRMED - {} contracts @ credit {:.2} → moving to Cools1",
      dbgspt, timestamp, brain.m_count as u32, brain.m_credit_received
    );
    let _ = f_report_info(stone, brain, stuff, to_dr, &msgmsg).await;
  }

  Ok(())
}

async fn f_run_cools1
( _stone: &mut dsta::MakeBuzzBomber,
  brain: &mut BuzzBomber,
  _stuff: &mut crate::Allocation,
  _to_dr: &crate::BridgeDrHand,
) -> BotResult<()> 
{ if let Some(until) = brain.m_cooldown_until 
  { if Utc::now() < until 
    { return Ok(());
    }
  }
  brain.m_state = BuzzState::Logic2;
  Ok(())
}

async fn f_run_logic2
( stone: &mut dsta::MakeBuzzBomber,
  brain: &mut BuzzBomber,
  stuff: &mut crate::Allocation,
  to_dr: &crate::BridgeDrHand,
  told_it_sucks_api: &mut tokio::sync::mpsc::Receiver<String>,  // Law 2: needed for exit order confirmation
) -> BotResult<()> {
  let dbgspt = format!("[BUZZ LOGIC2 {}]", stone.my_accounting.friendly_name);
  let timestamp = Utc::now().format("%Y-%m-%d %H:%M:%S%.f").to_string();

  // skip if still cooling (should not happen, but defensive)
  if let Some(until) = brain.m_cooldown_until {
    if Utc::now() < until { return Ok(()); }
  }

  let short_opt = brain.m_ticks.get(0).ok_or_else(|| BotError::MissingData("short leg missing".into()))?;
  let long_opt  = brain.m_ticks.get(1).ok_or_else(|| BotError::MissingData("long leg missing".into()))?;

  let short_mid = short_opt.last_ticker.as_ref()
    .filter(|t| t.bid > 0.0 && t.ask > t.bid)
    .map(|t| (t.bid + t.ask) / 2.0)
    .ok_or_else(|| BotError::MissingData("short leg quote missing".into()))?;

  let long_mid = long_opt.last_ticker.as_ref()
    .filter(|t| t.bid > 0.0 && t.ask > t.bid)
    .map(|t| (t.bid + t.ask) / 2.0)
    .ok_or_else(|| BotError::MissingData("long leg quote missing".into()))?;

  let current_credit = short_mid - long_mid;

  // check any exit rule
  for rule in &stone.follow_a_exit {
    match rule {
      dsta::BuzzBotDoneNiceStyle::SpreadValueGain(gain) => {
        if current_credit <= *gain {
          let msg = format!("{} [{}] Profit target hit: current credit {:.2} ≤ {:.2}", dbgspt, timestamp, current_credit, gain);
          let _ = f_report_info(stone, brain, stuff, to_dr, &msg).await;
          return f_run_exit_order(stone, brain, stuff, to_dr, told_it_sucks_api).await;
        }
      }
      dsta::BuzzBotDoneNiceStyle::SpreadValueLoss(loss) => {
        if current_credit >= *loss {
          let msg = format!("{} [{}] Stop loss hit: current credit {:.2} ≥ {:.2}", dbgspt, timestamp, current_credit, loss);
          let _ = f_report_info(stone, brain, stuff, to_dr, &msg).await;
          return f_run_exit_order(stone, brain, stuff, to_dr, told_it_sucks_api).await;
        }
      }
      dsta::BuzzBotDoneNiceStyle::SpreadValueTime(t) => {
        if t.is_entry_time(Utc::now()) {
          let msg = format!("{} [{}] Time-based exit triggered", dbgspt, timestamp);
          let _ = f_report_info(stone, brain, stuff, to_dr, &msg).await;
          return f_run_exit_order(stone, brain, stuff, to_dr, told_it_sucks_api).await;
        }
      }
    }
  }

  Ok(())
}

// wait for exit order fill (both legs closed → all positions ≈0), then transition
// Note: Check2 does not send orders, so told_it_sucks_api is not needed here.
async fn f_run_check2
( stone: &mut dsta::MakeBuzzBomber,
  brain: &mut BuzzBomber,
  stuff: &mut crate::Allocation,
  to_dr: &crate::BridgeDrHand,
) -> BotResult<()> {
  let dbgspt = format!("[BUZZ CHECK2 {}]", stuff.bot_name);
  let timestamp = chrono::Utc::now().format("%Y-%m-%d %H:%M:%S%.f").to_string();
  log::debug!("{} [{}] ENTRY! - waiting for exit order fill", dbgspt, timestamp);

  let mut remaining_long = 0.0;
  let mut remaining_short = 0.0;

  // Check long leg (should be closed)
  let long_field = if stone.stonk_feeling == dsta::MarketDirection::GetStonk {
    &stuff.bot_safe.m_long_puts
  } else {
    &stuff.bot_safe.m_long_calls
  };
  if let Some(pos) = long_field {
    if let dsta::FinAss::OptDeets(ref opt) = pos.asset {
      if opt.option_name == brain.m_str_long {
        remaining_long = pos.owned;
      }
    }
  }

  // Check short leg (should be closed)
  let short_field = if stone.stonk_feeling == dsta::MarketDirection::GetStonk {
    &stuff.bot_safe.m_short_puts
  } else {
    &stuff.bot_safe.m_short_calls
  };
  if let Some(pos) = short_field {
    if let dsta::FinAss::OptDeets(ref opt) = pos.asset {
      if opt.option_name == brain.m_str_short {
        remaining_short = pos.owned.abs();
      }
    }
  }

  // Success: both legs basically closed
  if remaining_long <= 0.01 && remaining_short <= 0.01 {
    let msgmsg = format!(
      "{} [{}] EXIT COMPLETE - all positions closed (long: {:.2}, short: {:.2})",
      dbgspt, timestamp, remaining_long, remaining_short
    );
    let _ = f_report_info(stone, brain, stuff, to_dr, &msgmsg).await;

    // Transition to Logic0 if repeating, else zombie
    if stone.bombs_forever {
      brain.m_state = BuzzState::Logic0;
      let msgmsg = format!(
        "{} [{}] EXIT OK! Transitioning to Logic0 for next cycle",
        dbgspt, timestamp
      );
      let _ = f_report_info(stone, brain, stuff, to_dr, &msgmsg).await;
    } else {
      return Err(BotError::Other(format!(
        "{} [{}] EXIT OK! All positions closed - one-and-done → zombied",
        dbgspt, timestamp
      )));
    }
  }

  // Partial close handling + timeout
  let partial = (remaining_long > 0.01 && remaining_long < brain.m_count - 0.01) ||
                (remaining_short > 0.01 && remaining_short < brain.m_count - 0.01);

  if partial {
    log::warn!(
      "{} [{}] Partial exit close detected (long: {:.2}, short: {:.2}) — still waiting",
      dbgspt, timestamp, remaining_long, remaining_short
    );

    if let Some(begin) = brain.m_begin {
      let elapsed = Utc::now() - begin;
      if elapsed > chrono::Duration::seconds(20) {
        let errmsg = format!(
          "{} [{}] TIMEOUT after {:.0}s waiting for full exit (long: {:.2}, short: {:.2})",
          dbgspt, timestamp, elapsed.num_seconds(), remaining_long, remaining_short
        );
        return Err(BotError::Timeout(errmsg));
      }
    } else {
      log::warn!("{} [{}] m_begin not set — cannot check exit timeout", dbgspt, timestamp);
    }

    return Ok(());
  }

  // No fill yet
  if remaining_long >= brain.m_count - 0.01 || remaining_short >= brain.m_count - 0.01 {
    log::trace!("{} [{}] No fill detected yet on exit order(s)", dbgspt, timestamp);
  }

  log::debug!("{} [{}] EXIT! - waiting continues, state remains {:#?}", dbgspt, timestamp, brain.m_state);

  Ok(())
}



//===================================================================
// HELPERS
//===================================================================

// entry order – vertical credit spread (limit credit, haggling)
async fn f_run_entry_order(
    stone: &mut dsta::MakeBuzzBomber,
    brain: &mut BuzzBomber,
    stuff: &mut crate::Allocation,
    to_dr: &crate::BridgeDrHand,
    told_it_sucks_api: &mut tokio::sync::mpsc::Receiver<String>,  // Law 2: wait for ORDER_FLY after send
) -> BotResult<()> {
    let dbgspt = format!("[BUZZ ENTRY {}]", stone.my_accounting.friendly_name);
    let timestamp = Utc::now().format("%Y-%m-%d %H:%M:%S%.f").to_string();

    if brain.m_ticks.len() != 2 {
        return Err(BotError::LogicError("expected exactly 2 legs in m_ticks".into()));
    }

    let short_leg = &brain.m_ticks[0];
    let long_leg  = &brain.m_ticks[1];

    let short_mid = short_leg
        .last_ticker
        .as_ref()
        .map(|t| (t.bid + t.ask) / 2.0)
        .unwrap_or(0.0);

    let long_mid = long_leg
        .last_ticker
        .as_ref()
        .map(|t| (t.bid + t.ask) / 2.0)
        .unwrap_or(0.0);

    // Sizing
    let qty = {
        let risk_per_contract =
            (short_leg.strike_spot - long_leg.strike_spot).abs() * 100.0;
        let (use_max, pct, fixed) = stone.my_accounting.max_cash_risk;
        let risk_cap = if use_max {
            (stuff.bot_safe.m_cash * pct / 100.0).max(fixed)
        } else {
            (stuff.bot_safe.m_cash * pct / 100.0).min(fixed)
        };
        (risk_cap / risk_per_contract).floor().max(1.0).min(50.0) as f64
    };

    if qty < 1.0 {
        let msg = format!(
            "{} [{}] Insufficient capital for 1 contract",
            dbgspt, timestamp
        );
        let _ = f_report_info(stone, brain, stuff, to_dr, &msg).await;
        return Ok(());
    }

    brain.m_count = qty;

    let leg_short = dsta::OrderLeg {
        buy_what: dsta::FinAss::OptDeets(short_leg.clone()),
        quantity: qty,
        remaining: qty,
        action: dsta::BuyOrSell::SellToOpen,
        price: short_mid,
    };

    let leg_long = dsta::OrderLeg {
        buy_what: dsta::FinAss::OptDeets(long_leg.clone()),
        quantity: qty,
        remaining: qty,
        action: dsta::BuyOrSell::BuyToOpen,
        price: long_mid,
    };

    let target_credit_per_contract = stone.target_spread / 100.0;

    let base_order = dsta::Order 
    { order_legs: vec![leg_short, leg_long],
      order_type: dsta::MarketOrLimit::Limit,
    };

    let msg = format!(
        "{} [{}] Creating credit spread: STO {} + BTO {} × {:.0} @ target credit {:.2}",
        dbgspt, timestamp,
        brain.m_str_short, brain.m_str_long, qty, target_credit_per_contract,
    );
    let _ = f_report_info(stone, brain, stuff, to_dr, &msg).await;

    // CREATE INITIAL HAGGLE ORDER
    let haggle_method = match &stone.my_accounting.haggle_action {
      dsta::HaggleAction::JustCancel(m) | dsta::HaggleAction::AnOyVeyJew(m) => m,
    };
    let (initial_order, initial_ctx) = dsta::create_initial_haggle_order(
      &base_order,
      haggle_method,
    );

    brain.m_entry         = Some(initial_order.clone());
    brain.haggle_context  = initial_ctx;
    brain.total_slippage  = 0.0;
    brain.haggle_attempt  = 0;
    brain.retry_at_delta  = stone.my_accounting.haggle_action.get_retry_period();

    let msg = format!(
        "{} [{}] ENTRY HAGGLE created (method {:?}) – sending initial order",
        dbgspt, timestamp, haggle_method
    );
    let _ = f_report_info(stone, brain, stuff, to_dr, &msg).await;

    to_dr.tell_dr_pls_trade
        .send(initial_order.clone())
        .await
        .map_err(|e| BotError::OrderSendFailed(e.to_string()))?;

    // Law 2: wait for ORDER_FLY confirmation before returning
    wait_for_buzz_order_confirmation(stone, brain, stuff, to_dr, told_it_sucks_api, &dbgspt, "entry").await?;

    brain.last_haggle_at  = Some(Utc::now());
    brain.m_begin         = Some(Utc::now());
    brain.m_state = BuzzState::Check1;

    let msg = format!(
        "{} [{}] Entry order confirmed (ORDER_FLY) → Check1",
        dbgspt, timestamp
    );
    let _ = f_report_info(stone, brain, stuff, to_dr, &msg).await;

    Ok(())
}


async fn f_run_exit_order
( stone: &mut dsta::MakeBuzzBomber,
  brain: &mut BuzzBomber,
  stuff: &mut crate::Allocation,
  to_dr: &crate::BridgeDrHand,
  told_it_sucks_api: &mut tokio::sync::mpsc::Receiver<String>,  // Law 2: wait for ORDER_FLY after send
) -> BotResult
{ let dbgspt = format!("[BUZZ_EXIT {}]", stuff.bot_name);
  let timestamp = chrono::Utc::now().format("%Y-%m-%d %H:%M:%S%.f").to_string();
  log::debug!("{} [{}] ENTRY! - generate exit order", dbgspt, timestamp);

  if brain.m_ticks.len() != 2 
  { return Err(BotError::LogicError("expected exactly 2 legs for exit".into()));
  }
  let qty = brain.m_count;

    let short_leg = &brain.m_ticks[0]; 
    let long_leg  = &brain.m_ticks[1];

    let short_mid = short_leg.last_ticker.as_ref()
        .and_then(|t| if t.ask > t.bid { Some((t.bid + t.ask)/2.0) } else { None })
        .ok_or_else(|| BotError::MissingData("short leg missing valid quote for exit".into()))?;

    let long_mid = long_leg.last_ticker.as_ref()
        .and_then
        ( |t| if t.bid < t.ask 
          { Some((t.bid + t.ask)/2.0) 
          } else { None })
        .ok_or_else(|| BotError::MissingData("long leg missing valid quote for exit".into()))?;

    let leg_long = dsta::OrderLeg {
        buy_what: dsta::FinAss::OptDeets(long_leg.clone()),
        quantity: qty,
        remaining: qty,
        action: dsta::BuyOrSell::SellToClose,
        price: long_mid,
    };

    let leg_short = dsta::OrderLeg {
        buy_what: dsta::FinAss::OptDeets(short_leg.clone()),
        quantity: qty,
        remaining: qty,
        action: dsta::BuyOrSell::BuyToClose,
        price: short_mid,
    };

  let net_debit = (long_mid - short_mid) * qty * 100.0;

  let exit_order = dsta::Order
  { order_legs: vec![leg_long, leg_short],
    order_type: dsta::MarketOrLimit::Limit,
  };

  let msgmsg = format!("{} [{}] Sending exit: SellToClose {} x{:.0} @ {:.2} + BuyToClose {} x{:.0} @ {:.2} (net debit {:.2})",
    dbgspt, timestamp, brain.m_str_long, qty, long_mid, brain.m_str_short, qty, short_mid, net_debit);
  let _ = f_report_info(stone, brain, stuff, to_dr, &msgmsg).await;

  to_dr.tell_dr_pls_trade
    .send(exit_order.clone())
    .await
    .map_err(|e|
    { let errmsg = format!("{} [{}] Failed to send exit order: {}", dbgspt, timestamp, e);
      BotError::OrderSendFailed(errmsg)
    })?;

  // Law 2: wait for ORDER_FLY confirmation before moving to Check2
  wait_for_buzz_order_confirmation(stone, brain, stuff, to_dr, told_it_sucks_api, &dbgspt, "exit").await?;

  brain.m_exits = Some(exit_order);
  brain.m_state = BuzzState::Check2;
  let msgmsg = format!("{} [{}] Exit order confirmed (ORDER_FLY) → Check2", dbgspt, timestamp);
  let _ = f_report_info(stone, brain, stuff, to_dr, &msgmsg).await;

  Ok(())
}

pub(crate) fn select_best_spread // crate for the tests
( opts: &Vec<dsta::Opt>
, feeling: dsta::MarketDirection
, target_credit: f64
) -> Option<(f64, dsta::Opt, dsta::Opt)> 
{
    let mut best_credit = f64::INFINITY;
    let mut best_short = None;
    let mut best_long = None;

    for i in 0..opts.len().saturating_sub(1) {
        let a = &opts[i];
        let b = &opts[i + 1];

        let (short_opt, long_opt, mid_short, mid_long) = match feeling {
            dsta::MarketDirection::GetStonk => (
                b, a,
                b.last_ticker.as_ref()
                    .filter(|t| t.bid > 0.01 && t.ask > t.bid + 0.01)
                    .map(|t| (t.bid + t.ask) / 2.0),
                a.last_ticker.as_ref()
                    .filter(|t| t.bid > 0.01 && t.ask > t.bid + 0.01)
                    .map(|t| (t.bid + t.ask) / 2.0)
            ),
            dsta::MarketDirection::Corrects => (
                a, b,
                a.last_ticker.as_ref()
                    .filter(|t| t.bid > 0.01 && t.ask > t.bid + 0.01)
                    .map(|t| (t.bid + t.ask) / 2.0),
                b.last_ticker.as_ref()
                    .filter(|t| t.bid > 0.01 && t.ask > t.bid + 0.01)
                    .map(|t| (t.bid + t.ask) / 2.0)
            ),
            _ => continue,
        };

        if let (Some(ms), Some(ml)) = (mid_short, mid_long) {
            let credit = ms - ml;
            if credit >= target_credit && credit < best_credit {
                best_credit = credit;
                best_short = Some(short_opt.clone());
                best_long = Some(long_opt.clone());
            }
        }
    }

    best_short.zip(best_long).map(|(s, l)| (best_credit, s, l))
}



use crate::sbot::HasFriendlyName;

impl HasFriendlyName for dsta::MakeBuzzBomber {
    fn friendly_name(&self) -> &str {
        &self.my_accounting.friendly_name
    }
}

impl HasFriendlyName for &dsta::MakeBuzzBomber {
    fn friendly_name(&self) -> &str {
        HasFriendlyName::friendly_name(*self)
    }
}

impl HasFriendlyName for &mut dsta::MakeBuzzBomber {
    fn friendly_name(&self) -> &str {
        HasFriendlyName::friendly_name(*self)
    }
}