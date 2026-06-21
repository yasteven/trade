// trade/usta/src/sbot/swat_bot.rs
// UPDATED: Order confirmation checks + all 4 TODOs implemented

use std::time::{Instant, Duration as StdDuration};
use dsta::*;
use crate::*;
use crate::sbot::f_report_error;
use crate::sbot::f_report_info;
use crate::sbot::errors::{BotError, BotResult};
use std::collections::HashMap;

//===================================================================
// LOCAL DATA STRUCTURES
//===================================================================
#[derive(Clone, Debug, Default, serde::Serialize, serde::Deserialize, PartialEq, Eq)]
pub(crate) enum SwatState
{
  #[default]
  Monitoring,
  ExitInProgress,
  Exited,
}

#[derive(Clone, Debug, Default, serde::Serialize, serde::Deserialize)]
pub(crate) struct SwatBot
{ pub m_state: SwatState,
  pub m_pending_order: Option<dsta::Order>,
  pub m_begin: Option<chrono::DateTime<chrono::Utc>>,
  pub haggle_context: dsta::HaggleContext,
  pub retry_at_delta: chrono::TimeDelta,
  pub total_slippage: f64, // accumulated slippage (starts at 0.0)
  pub haggle_attempt: u32, // how many times we already applied haggle
  pub last_haggle_at: Option<chrono::DateTime<chrono::Utc>>,
  pub original_exit_quantity: f64,
  pub order_fly_confirmed: bool, // Track if order was routed
}

//===================================================================
// BOT CREATION & MAIN LOOP
//===================================================================
use crate::sbot::position_for_order_name;

pub(crate) async fn f_make_bot_swat_2
( stone: MakeSwatBotsGo,
  dr_robo_api: &DrRoboApi,
  // Has EXTRA args due to it's nature within the system to directly report to frontend and usage as a SINK for deleted bots
  swap_info_0: Option<(usize, crate::BotSwaper)>,
  tell_dr_got_tick00: tokio::sync::mpsc::Sender<dsta::GotTick00>
) -> ( tokio::task::JoinHandle<()>, crate::CtorBot )
{ let friendly_name = &stone.my_accounting.friendly_name;
  let tracking_tick = &stone.my_accounting.tracking_tick;
  let dbgspt = format!("[SWAT_FN_MAKE - {}]", friendly_name);
  log::debug!
  ( "{} [ENTRY] fn_make_bot_swat() - tracking {}"
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
    tell_bot_to_trade_api,
    tell_bot_to_check_api,
    tell_bot_it_sucks_api,
    told_the_bot_requests: bot_bridge_to_dr_foot,
  };
  let zero_alerter = dr_robo_api.tell_dr_swap_info.clone();
  log::debug!("{} [EXIT] fn_make_bot_swat()", dbgspt);
  ( tokio::spawn
   ( async move 
     { fn_run_main_swat
       ( stone,
         told_bot_to_trade_api,
         told_bot_to_check_api,
         told_bot_it_sucks_api,
         bot_to_dr,
         zero_alerter,
         swap_info_0,
         tell_dr_got_tick00,
       ).await;
       log::error!("{}.SPAWN [EXIT] fn_run_main_swat()", dbgspt);
     }
   )
  , sperm
  )
}

pub(crate) async fn fn_run_main_swat
( constructor_stuff: MakeSwatBotsGo
  , mut told_to_trade_api: tokio::sync::mpsc::Receiver<dsta::Ticker>
  , mut told_to_check_api: tokio::sync::mpsc::Receiver<crate::Allocation>
  , mut told_it_sucks_api: tokio::sync::mpsc::Receiver<String>
  , to_dr: crate::BridgeDrHand
  , tell_dr_swap_info: tokio::sync::mpsc::Sender<(usize, BotSwaper)>
  , swap_info_0: Option<(usize, crate::BotSwaper)>
  , tell_dr_got_tick00: tokio::sync::mpsc::Sender<dsta::GotTick00>
) 
{ let dbgspt = format!("[SWAT_RUN {}]", constructor_stuff.my_accounting.friendly_name);
  log::debug!("{} [ENTRY] fn_run_main_swat()", dbgspt);

  let mut stone = constructor_stuff;
  let mut stuff = crate::Allocation {
    bot_name: stone.my_accounting.friendly_name.clone(),
    bot_safe: crate::BotaAccount::default(),
    bot_mind: crate::BotMindState::Running,
    num_swap: 0,
  };

  // STEP 1: Load or create fresh brain state
  let msgmsg = format!("{} STEP 1: Attempting to load bot brain state...", dbgspt);
  let temp_brain = SwatBot::default();
  let _ = f_report_info(&stone, &temp_brain, &stuff, &to_dr, &msgmsg).await;
  let ( mut brain, is_fresh_bot) = match crate::sbot::fn_load_bot_state(&stone.my_accounting.friendly_name).await 
  { Ok( ( load_stone, load_brain, load_stuff ) ) => 
    { stone = load_stone; stuff = load_stuff;
      let msgmsg = format!("{} [LOAD SUCCESS] Loaded existing state from disk", dbgspt);
      let _ = f_report_info(&stone, &load_brain, &stuff, &to_dr, &msgmsg).await;
      (load_brain, false)
    }
    Err(e) => 
    { log::info!("{} [NEW] No saved state found: {}. Creating fresh bot.", dbgspt, e);
      let fresh = SwatBot 
      { m_state: SwatState::Monitoring,
        m_begin: Some(chrono::Utc::now()),
        ..Default::default()
      };
      (fresh, true)
    }
  };

  if let Some((idx, sw)) = swap_info_0 {
    let msgmsg = format!("{} STEP 2: Processing initial swap_info...", dbgspt);
    let _ = f_report_info(&stone, &brain, &stuff, &to_dr, &msgmsg).await;

    let _ = tell_dr_swap_info.send((idx, sw)).await;

    let swap_begin = chrono::Utc::now();
    loop 
    { let swap_reply = tokio::select! 
      { res = told_it_sucks_api.recv() => res,
        _ = tokio::time::sleep(StdDuration::from_secs(1)) => None,
      };

      match swap_reply 
      {
        Some(msg) if msg == crate::glossary::SWAP_SUCC => {
          let msgmsg = format!("{} STEP 2.1: SWAP SUCCESS - initial swap completed", dbgspt);
          let _ = f_report_info(&stone, &brain, &stuff, &to_dr, &msgmsg).await;
          break;
        }
        Some(msg) if msg == crate::glossary::SWAP_FAIL => {
          let errmsg = format!("{} STEP 2.1: SWAP FAIL - initial swap rejected", dbgspt);
          let _ = f_report_error(&stone, &brain, &stuff, &to_dr, &errmsg).await;
          stuff.bot_mind = crate::BotMindState::Zombied;
          let _ = to_dr.tell_dr_new_brain.send(stuff).await;
          return;
        }
        None if chrono::Utc::now() - swap_begin > chrono::Duration::seconds(15) => {
          let errmsg = format!("{} STEP 2.1: 15s Timeout waiting for SWAP reply", dbgspt);
          let _ = f_report_error(&stone, &brain, &stuff, &to_dr, &errmsg).await;
          stuff.bot_mind = crate::BotMindState::Zombied;
          let _ = to_dr.tell_dr_new_brain.send(stuff).await;
          return;
        }
        _ => continue,
      }
    }
  }
  else
  { let msgmsg = format!("{} STEP 2: SKIP! no initial swap_info...", dbgspt);
    let _ = f_report_info(&stone, &brain, &stuff, &to_dr, &msgmsg).await;
  }

  // STEP 3: Get tracking_tick asset details and add to bot_safe, then subscribe to position tickers
  let msgmsg = format!("{} STEP 3: Setting up tracking tick and ticker subscriptions...", dbgspt);
  let _ = f_report_info(&stone, &brain, &stuff, &to_dr, &msgmsg).await;

  // Step 3.1: Request ticker/asset details for tracking_tick
  if stone.my_accounting.tracking_tick.is_empty() 
  {
    let errmsg = format!("{} {}", dbgspt, glossary::EMPTY_SWAT_BOT);
    let _ = f_report_error(&stone, &brain, &stuff, &to_dr, &errmsg).await;
    stuff.bot_mind = crate::BotMindState::Zombied;
    let _ = to_dr.tell_dr_new_brain.send(stuff).await;
    return;
  }

  let msgmsg = format!(
    "{} STEP 3.1: Requesting asset details for tracking_tick: {}",
    dbgspt,
    stone.my_accounting.tracking_tick
  );
  let _ = f_report_info(&stone, &brain, &stuff, &to_dr, &msgmsg).await;

  let (tx_asset, mut rx_asset) = tokio::sync::mpsc::channel(2);
  if to_dr.tell_dr_pls_ticks.send((stone.my_accounting.tracking_tick.clone(), tx_asset)).await.is_err() 
  {
    let errmsg = format!("{} STEP 3.1: Ticker request send failed for tracking_tick", dbgspt);
    let _ = f_report_error(&stone, &brain, &stuff, &to_dr, &errmsg).await;
    stuff.bot_mind = crate::BotMindState::Zombied;
    let _ = to_dr.tell_dr_new_brain.send(stuff).await;
    return;
  }

  let tracking_asset = match tokio::time::timeout
  ( StdDuration::from_secs(60), rx_asset.recv()
  ).await 
  { Ok(Some(Some(details))) => 
    { let msgmsg = format!("{} STEP 3.1: Tracking tick asset details received", dbgspt);
      let _ = f_report_info(&stone, &brain, &stuff, &to_dr, &msgmsg).await;
      details
    }
  , _ => 
    { let errmsg = format!
      ( "{} STEP 3.1: 60 Second Timeout/error getting tracking_tick asset details"
      , dbgspt
      );
      let _ = f_report_error(&stone, &brain, &stuff, &to_dr, &errmsg).await;
      stuff.bot_mind = crate::BotMindState::Zombied;
      let _ = to_dr.tell_dr_new_brain.send(stuff).await;
      return;
    }
  };

  // Step 3.2: Add tracking_tick asset to bot_safe with zero quantity if not already present
  let msgmsg = format!("{} STEP 3.2: Tracking tick asset details received", dbgspt);
  let _ = f_report_info(&stone, &brain, &stuff, &to_dr, &msgmsg).await;
      
  let tracking_ticker_name = tracking_asset.ass_deets.ticker_name();
  
  match &tracking_asset.ass_deets {
    dsta::FinAss::StkDeets(stk) => {
      let already_tracked = stuff.bot_safe.m_stock.as_ref()
        .map(|p| p.asset.ticker_name() == tracking_ticker_name)
        .unwrap_or(false);
      
      if !already_tracked {
        if stuff.bot_safe.m_stock.is_none() {
          stuff.bot_safe.m_stock = Some(dsta::OwnedPosition {
            asset: dsta::FinAss::StkDeets(stk.clone()),
            owned: 0.0,
            price: 0.0,
          });
          let msgmsg = format!(
            "{} STEP 3.2: Added tracking_tick {} to m_stock with zero quantity",
            dbgspt,
            tracking_ticker_name
          );
          let _ = f_report_info(&stone, &brain, &stuff, &to_dr, &msgmsg).await;
        } else {
          let msgmsg = format!(
            "{} STEP 3.2: WARNING - m_stock slot occupied, cannot add tracking_tick {}",
            dbgspt,
            tracking_ticker_name
          );
          let _ = f_report_info(&stone, &brain, &stuff, &to_dr, &msgmsg).await;
        }
      }
    }
    dsta::FinAss::OptDeets(opt) => {
      let is_call = opt.option_rite == dsta::OptRite::Call;
      
      let already_tracked = 
        stuff.bot_safe.m_long_calls.as_ref().map(|p| p.asset.ticker_name() == tracking_ticker_name).unwrap_or(false) ||
        stuff.bot_safe.m_long_puts.as_ref().map(|p| p.asset.ticker_name() == tracking_ticker_name).unwrap_or(false) ||
        stuff.bot_safe.m_short_calls.as_ref().map(|p| p.asset.ticker_name() == tracking_ticker_name).unwrap_or(false) ||
        stuff.bot_safe.m_short_puts.as_ref().map(|p| p.asset.ticker_name() == tracking_ticker_name).unwrap_or(false);
      
      if !already_tracked {
        let target_slot = if is_call { &mut stuff.bot_safe.m_long_calls } else { &mut stuff.bot_safe.m_long_puts };
        
        if target_slot.is_none() {
          *target_slot = Some(dsta::OwnedPosition {
            asset: dsta::FinAss::OptDeets(opt.clone()),
            owned: 0.0,
            price: 0.0,
          });
          let slot_name = if is_call { "m_long_calls" } else { "m_long_puts" };
          let msgmsg = format!(
            "{} STEP 3.2: Added tracking_tick {} to {} with zero quantity",
            dbgspt,
            tracking_ticker_name,
            slot_name
          );
          let _ = f_report_info(&stone, &brain, &stuff, &to_dr, &msgmsg).await;
        } else {
          let slot_name = if is_call { "m_long_calls" } else { "m_long_puts" };
          let msgmsg = format!(
            "{} STEP 3.2: WARNING - {} slot occupied, cannot add tracking_tick {}",
            dbgspt,
            slot_name,
            tracking_ticker_name
          );
          let _ = f_report_info(&stone, &brain, &stuff, &to_dr, &msgmsg).await;
        }
      }
    }
    _ => {
      let msgmsg = format!(
        "{} STEP 3.2: Tracking tick is neither stock nor option, skipping bot_safe insertion",
        dbgspt
      );
      let _ = f_report_info(&stone, &brain, &stuff, &to_dr, &msgmsg).await;
    }
  }

  let mut tickers_to_subscribe = Vec::new();
  
  // Always include tracking tick
  tickers_to_subscribe.push(stone.my_accounting.tracking_tick.clone());
  
  // Add all position tickers
  if let Some(pos) = &stuff.bot_safe.m_stock 
  { let ticker = pos.asset.ticker_name();
    if !ticker.is_empty() {
      tickers_to_subscribe.push(ticker);
    }
  }
  if let Some(pos) = &stuff.bot_safe.m_long_calls {
    let ticker = pos.asset.ticker_name();
    if !ticker.is_empty() {
      tickers_to_subscribe.push(ticker);
    }
  }
  if let Some(pos) = &stuff.bot_safe.m_long_puts {
    let ticker = pos.asset.ticker_name();
    if !ticker.is_empty() {
      tickers_to_subscribe.push(ticker);
    }
  }
  if let Some(pos) = &stuff.bot_safe.m_short_calls {
    let ticker = pos.asset.ticker_name();
    if !ticker.is_empty() {
      tickers_to_subscribe.push(ticker);
    }
  }
  if let Some(pos) = &stuff.bot_safe.m_short_puts {
    let ticker = pos.asset.ticker_name();
    if !ticker.is_empty() {
      tickers_to_subscribe.push(ticker);
    }
  }

  // Deduplicate
  tickers_to_subscribe.sort();
  tickers_to_subscribe.dedup();

  // Step 3.4: Request all ticker subscriptions (non-fatal timeouts)
  let msgmsg = format!(
        "{} STEP 3.4: Full account Ticker subscriptions count : {}",
        dbgspt,
        tickers_to_subscribe.len()
      );
  let _ = f_report_info(&stone, &brain, &stuff, &to_dr, &msgmsg).await;
  for ticker_name in tickers_to_subscribe 
  { let (tx, mut rx) = tokio::sync::mpsc::channel(2);
    if to_dr.tell_dr_pls_ticks.send((ticker_name.clone(), tx)).await.is_err() {
      let msgmsg = format!(
        "{} STEP 3.4: Failed to request ticker {} (non-fatal)",
        dbgspt,
        ticker_name
      );
      let _ = f_report_info(&stone, &brain, &stuff, &to_dr, &msgmsg).await;
      continue;
    }
    if is_fresh_bot 
    { match tokio::time::timeout(StdDuration::from_secs(15), rx.recv()).await 
      { Ok(Some(Some(_))) => 
        { let msgmsg = format!(
            "{} STEP 3.4: Ticker subscription confirmed: {}",
            dbgspt,
            ticker_name
          );
          let _ = f_report_info(&stone, &brain, &stuff, &to_dr, &msgmsg).await;
        }
        _ => 
        { let msgmsg = format!
          ( "{} STEP 3.4: Ticker 15s timeout for {} (non-fatal, continuing)",
            dbgspt,
            ticker_name
          );
          let _ = f_report_info(&stone, &brain, &stuff, &to_dr, &msgmsg).await;
        }
      }
    }
  }

  let msgmsg = format!("{} STEP 4: Entering main event loop...", dbgspt);
  let _ = f_report_info(&stone, &brain, &stuff, &to_dr, &msgmsg).await;

  let mut last_log = Instant::now();
  let tick_updates = StdDuration::from_secs(1);
  let mut report_ticks: HashMap<String, Vec<dsta::Ticker>> = HashMap::new();
  
  loop 
  { let now = Instant::now();
    if now.duration_since(last_log) >= tick_updates 
    { // MAIN FRONTEND TICKER UPDATE LOGIC:
      // For each ticker, find the most recent of the 3 kinds:
      // 1) normal ticker => bid != ask
      // 2) theo price => theo price not none (ignore if fin ass is stk)
      // 3) last price ticker bid == ask (and no theo)
      for (_ticker_name, tickers) in &report_ticks 
      { if let Some(best_ticker) = select_best_ticker(tickers) 
        { let _ = tell_dr_got_tick00.send(dsta::GotTick00 { ticker: best_ticker }).await;
        }
      }
      report_ticks.clear();
      last_log = now;
    }

    tokio::select! 
    { res = told_to_trade_api.recv() => 
      { if let Some(ticker) = res 
        { // MAIN TODO 2: IMPLEMENTED
          // Update tickers for all of our assets, not just tracking_tick
          update_position_ticker(&mut stuff.bot_safe, &ticker);
          
          // Also collect for reporting
          report_ticks.entry(ticker.name.clone())
            .or_insert_with(Vec::new)
            .push(ticker.clone());
          
          slog::trace!
          ( crate::glossary::TICK_SLOG,
           "{} [TICK] {} bid={:.2} ask={:.2}",
            dbgspt, ticker.name, ticker.bid, ticker.ask
          );
        } 
        else 
        {
          log::error!("{} {}", dbgspt, glossary::TICK_CHANNEL_CLOSED);
          break;
        }
      }

      res = told_to_check_api.recv() => 
      {
        if let Some(mut incoming) = res {
          if incoming.bot_name == stone.my_accounting.friendly_name {
            merge_fresher_tickers(&mut incoming.bot_safe, &stuff.bot_safe);
            if let Err(e) = state_machine_mind(
              &mut stone,
              &mut brain,
              &mut stuff,
              &to_dr,
              &mut told_it_sucks_api,
              &incoming,
            ).await {
              log::info!("{} [EXIT] State machine error: {}", dbgspt, e);
              break;
            }
          }
        } else {
          log::error!("{} {}", dbgspt, glossary::ALLOC_CHANNEL_CLOSED);
          break;
        }
      }

      res = told_it_sucks_api.recv() => 
      {
        if let Some(msg) = res 
        {
          if msg == "DIE" 
          { log::error!("\n\n \t\t OH NO WE GOT TOLD TO DEI \n\n");
            log::debug!("{} {}", dbgspt, glossary::DIE_COMMAND);
            break;
          }
          // Ignore ORDER_FLY, ORDER_BAD, SWAP messages here - they're handled in state machine
        } else {
          log::error!("{} {}", dbgspt, glossary::SUCKS_CHANNEL_CLOSED);
          break;
        }
      }
    }

    if brain.m_state == SwatState::Exited {
      let msgmsg = format!("{} Exit completed - SwatBot exiting cleanly", dbgspt);
      log::info!("{}", msgmsg);
      break;
    }
  }

  // Final cleanup
  if stuff.bot_mind != crate::BotMindState::Zombied {
    stuff.bot_mind = crate::BotMindState::Zombied;
    let _ = to_dr.tell_dr_new_brain.send(stuff).await;
  }

  log::debug!("{} [EXIT] fn_run_main_swat()", dbgspt);
}

//===================================================================
// HELPER: Select best ticker from candidates (MAIN TODO 1)
//===================================================================
fn select_best_ticker(tickers: &[dsta::Ticker]) -> Option<dsta::Ticker> {
    if tickers.is_empty() {
        return None;
    }

    // Sort by time (most recent first)
    let mut sorted = tickers.to_vec();
    sorted.sort_by(|a, b| b.time.cmp(&a.time));

    // Prefer: normal ticker (bid != ask) > theo price > last price (bid == ask)
    for ticker in &sorted {
        if (ticker.bid - ticker.ask).abs() > 0.001 {
            // Normal ticker with bid-ask spread
            return Some(ticker.clone());
        }
    }

    // Then look for theo price (if available and not stock)
    for ticker in &sorted {
        if let Some(_theo) = &ticker.theo_price {
            return Some(ticker.clone());
        }
    }

    // Finally, take the most recent regardless
    sorted.into_iter().next()
}

//===================================================================
// STATE MACHINE PROCESSORS
//===================================================================
async fn state_machine_mind(
    stone: &mut dsta::MakeSwatBotsGo,
    brain: &mut SwatBot,
    stuff: &mut crate::Allocation,
    to_dr: &crate::BridgeDrHand,
    told_it_sucks_api: &mut tokio::sync::mpsc::Receiver<String>,
    incoming: &crate::Allocation,
) -> BotResult {
    let dbgspt = format!("[SWAT_MIND - {}]", stone.my_accounting.friendly_name);
    let timestamp = chrono::Utc::now().format("%Y-%m-%d %H:%M:%S%.f").to_string();
    let msgmsg = format!("{} [{}] SwatBot mind transitioning to {:#?} from {:#?}", dbgspt, timestamp, incoming.bot_mind, stuff.bot_mind);
    let _ = f_report_info(&stone, &brain, &stuff, &to_dr, &msgmsg).await;

    match incoming.bot_mind {
        crate::BotMindState::Seppuku => {
            match stuff.bot_mind {
                crate::BotMindState::Running | crate::BotMindState::Stopped | crate::BotMindState::Trading => {
                    log::info!("{} {}", dbgspt, glossary::SEPPUKU_TRANSITION);
                    *stuff = incoming.clone();

                    let tracking_tick = stone.my_accounting.tracking_tick.clone();
                    let friendly_name = stone.my_accounting.friendly_name.clone();

                    if let Some(ticker) = ticker_from_last_or_request(
                        &to_dr,
                        &tracking_tick,
                        &friendly_name,
                        brain,
                        stuff,
                    ).await {
                        return state_machine_tick_seppuku(
                            stone, brain, stuff,
                            &ticker,
                            to_dr,
                            told_it_sucks_api,
                        ).await;
                    }
                    Ok(())
                }
                crate::BotMindState::Seppuku => {
                    log::trace!("{} {}", dbgspt, glossary::SEPPUKU_DURING_SEPPUKU);
                    *stuff = incoming.clone();
                    if is_account_empty(&stuff.bot_safe) {
                        log::info!("{} {}", dbgspt, glossary::ACCOUNT_EMPTY_ZOMBIED);
                        stuff.bot_mind = crate::BotMindState::Zombied;
                        let _ = to_dr.tell_dr_new_brain.send(stuff.clone()).await;
                        return Err(BotError::Other(format!("Account empty after seppuku")));
                    }
                    Ok(())
                }
                crate::BotMindState::Zombied => {
                    log::warn!("{} {}", dbgspt, glossary::SEPPUKU_AFTER_ZOMBIED);
                    return Err(BotError::LogicError(format!("Already zombied")));
                }
            }
        }
        crate::BotMindState::Stopped => {
            let msgmsg = format!("{} [{}] Mind transition indicates save request", dbgspt, timestamp);
            let _ = f_report_info(&stone, &brain, &stuff, &to_dr, &msgmsg).await;
            *stuff = incoming.clone();
            return crate::sbot::fn_save_bot_state(&stone, &brain, &stuff, &to_dr).await;
        }
        crate::BotMindState::Zombied => {
            log::info!("{} [ZOMBIED] Received zombied command, exiting", dbgspt);
            return Err(BotError::Other(format!("Zombied")));
        }
        _ => {
            log::trace!("{} {}", dbgspt, glossary::BASIC_ACCOUNT_UPDATE);
            *stuff = incoming.clone();
            Ok(())
        }
    }
}

// ================================================================
// Main seppuku entry point — creates INITIAL haggled order
// ================================================================
async fn state_machine_tick_seppuku(
    stone: &mut MakeSwatBotsGo,
    brain: &mut SwatBot,
    stuff: &mut crate::Allocation,
    ticker: &dsta::Ticker,
    to_dr: &crate::BridgeDrHand,
    told_it_sucks_api: &mut tokio::sync::mpsc::Receiver<String>,
) -> BotResult 
{
    let dbgspt = format!("[SWAT_SEPPUKU {}]", stone.my_accounting.friendly_name);
    let timestamp = chrono::Utc::now().format("%Y-%m-%d %H:%M:%S%.f").to_string();

    if brain.m_state == SwatState::ExitInProgress {
        return check_exit_fill(stone, brain, stuff, to_dr, told_it_sucks_api).await;
    }

    // ── Find position to exit ────────────────────────────────────────
    let pos = match find_position(&stuff.bot_safe) {
        Some(p) => p,
        None => {
            let msg = format!("{} No position found to exit", dbgspt);
            let _ = f_report_info(stone, brain, stuff, to_dr, &msg).await;
            brain.m_state = SwatState::Exited;
            return Ok(());
        }
    };

    let qty_abs = pos.owned.abs();
    if qty_abs < glossary::MIN_POSITION {
        let msg = format!("{} Position already flat ({:.2})", dbgspt, qty_abs);
        let _ = f_report_info(stone, brain, stuff, to_dr, &msg).await;
        brain.m_state = SwatState::Exited;
        return Ok(());
    }

    // ── Determine direction & aggressive starting price ──────────────
    let is_long = pos.owned > 0.0;
    let (bid, ask) = (ticker.bid.max(0.0), ticker.ask.max(0.0));
    if bid <= 0.01 || ask <= 0.01 {
        let msg = format!("{} Invalid prices (bid={:.3}, ask={:.3})", dbgspt, bid, ask);
        let _ = f_report_error(stone, brain, stuff, to_dr, &msg).await;
        return Ok(());
    }

    const AGGRESSION_OFFSET: f64 = 0.01;

    let start_price = if is_long {
        (bid - AGGRESSION_OFFSET).max(0.01)
    } else {
        (ask + AGGRESSION_OFFSET).max(0.01)
    };

    let action = if is_long { BuyOrSell::SellToClose } else { BuyOrSell::BuyToClose };

    // ── Build BASE order ─────────────────────────────────────────────
    let base_leg = OrderLeg {
        buy_what: pos.asset.clone(),
        quantity: qty_abs,
        remaining: qty_abs,
        action,
        price: start_price,
    };

    let base_order = Order {
        order_legs: vec![base_leg],
        order_type: MarketOrLimit::Limit,
    };

    // ── Create INITIAL haggled order ──────────────────────────────────
    let exit_haggle_method = dsta::HaggleMethod::VirtualMarketOrderWithLimitOrdThenConcede
    ( dsta::HaggleLimits
      { retry_period : chrono::Duration::milliseconds(2000) 
      , delta_choice : 25.0
      , delta_is_pct : true
      , slippage_max : 150.00
      }
    );

    let (initial_order, initial_ctx) = dsta::create_initial_haggle_order(
        &base_order,
        &exit_haggle_method,
    );

    stone.my_accounting.haggle_action = dsta::HaggleAction::AnOyVeyJew(exit_haggle_method);

    // Store state
    brain.m_pending_order = Some(initial_order.clone());
    brain.haggle_context = initial_ctx;
    brain.total_slippage = 0.0;
    brain.haggle_attempt = 0;
    brain.retry_at_delta = stone.my_accounting.haggle_action.get_retry_period();
    brain.last_haggle_at = Some(chrono::Utc::now());
    brain.m_begin = Some(chrono::Utc::now());
    brain.original_exit_quantity = qty_abs;
    brain.m_state = SwatState::ExitInProgress;
    brain.order_fly_confirmed = false;

    let msg = format!(
        "{} [{}] Initial exit haggle submitted | {} {:.1} @ {:.3}",
        dbgspt, timestamp, action, qty_abs, initial_order.order_legs[0].price
    );
    let _ = f_report_info(stone, brain, stuff, to_dr, &msg).await;

    to_dr.tell_dr_pls_trade.send(initial_order).await.map_err
    ( |e| BotError::OrderSendFailed
      ( format!
        ( "{} Failed to send initial exit: {}", dbgspt, e
        )
      )
    )?;

    // MAIN TODO 3: IMPLEMENTED - Wait for ORDER_FLY / ORDER_BAD confirmation
    wait_for_order_confirmation(stone, brain, stuff, to_dr, told_it_sucks_api, &dbgspt).await?;

    Ok(())
}

// ================================================================
// Check fill status + haggle retry logic (called every tick during ExitInProgress)
// ================================================================
async fn check_exit_fill(
    stone: &mut MakeSwatBotsGo,
    brain: &mut SwatBot,
    stuff: &mut crate::Allocation,
    to_dr: &crate::BridgeDrHand,
    told_it_sucks_api: &mut tokio::sync::mpsc::Receiver<String>,
) -> BotResult
{
    let dbgspt = format!("[SWAT_FILL_CHECK {}]", stone.my_accounting.friendly_name);
    let timestamp = chrono::Utc::now().format("%Y-%m-%d %H:%M:%S%.f").to_string();

    let order = match &brain.m_pending_order {
        Some(o) => o,
        None => return Ok(()),
    };

    // ── 1. Check current fill level ──────────────────────────────────
    let leg = &order.order_legs[0];
    let pos_opt = position_for_order_name(&stuff.bot_safe, &leg.buy_what.order_name());

    let (filled_qty, remaining_qty) = match pos_opt {
        Some(pos) => {
            let filled = if leg.action.is_sell() {
                brain.original_exit_quantity - pos.owned.max(0.0)
            } else {
                (-pos.owned).max(0.0)
            };
            (filled, brain.original_exit_quantity - filled)
        }
        None => (0.0, brain.original_exit_quantity),
    };

    if remaining_qty <= 0.05 {
        let msg = format!("{} Exit completed — filled {:.1}/{:.1}", dbgspt, filled_qty, brain.original_exit_quantity);
        let _ = f_report_info(stone, brain, stuff, to_dr, &msg).await;
        brain.m_state = SwatState::Exited;
        return Ok(());
    }

    // ── 2. Check if we should haggle now ─────────────────────────────
    let should_haggle = match brain.last_haggle_at {
        Some(last) => chrono::Utc::now() - last >= brain.retry_at_delta,
        None => true,
    };

    if !should_haggle {
        // Also have absolute timeout
        if let Some(begin) = brain.m_begin {
            if chrono::Utc::now() - begin > chrono::Duration::seconds(180) {
                let msg = format!("{} Absolute timeout (3 min) — canceling", dbgspt);
                let _ = f_report_info(stone, brain, stuff, to_dr, &msg).await;
                let _ = to_dr.tell_dr_pls_trade.send(Order { order_legs: vec![], order_type: MarketOrLimit::Limit }).await;
                brain.m_state = SwatState::Exited;
            }
        }
        return Ok(());
    }

    // ── 3. Build haggled retry order ─────────────────────────────────
    let mut haggle_order = order.clone();

    // Update remaining
    for leg in &mut haggle_order.order_legs {
        leg.quantity = remaining_qty;
        leg.remaining = remaining_qty;
    }

    // Apply next concession step
    let slippage_delta = dsta::HaggleMethod::VirtualMarketOrderWithLimitOrdThenConcede(
        HaggleLimits {
            retry_period: chrono::Duration::milliseconds(800),
            delta_choice: 30.0,
            delta_is_pct: true,
            slippage_max: 120.0,
        }
    ).apply_haggle_changes(&mut haggle_order, &brain.haggle_context);

    brain.total_slippage += slippage_delta;
    brain.haggle_attempt += 1;
    brain.last_haggle_at = Some(chrono::Utc::now());

    // ── 4. Slippage guard (lenient for exit) ─────────────────────────
    if brain.total_slippage > 1.20 {
        let msg = format!("{} Slippage limit reached ({:.3}) — canceling", dbgspt, brain.total_slippage);
        let _ = f_report_info(stone, brain, stuff, to_dr, &msg).await;
        let _ = to_dr.tell_dr_pls_trade.send(Order { order_legs: vec![], order_type: MarketOrLimit::Limit }).await;
        brain.m_state = SwatState::Exited;
        return Ok(());
    }

    // ── 5. Send haggled order ────────────────────────────────────────
    let new_price = haggle_order.order_legs[0].price;
    let msg = format!(
        "{} Haggle #{} | filled {:.1}/{:.1} | slippage Δ{:.3} total {:.3} | new price {:.3}",
        dbgspt, brain.haggle_attempt, filled_qty, brain.original_exit_quantity,
        slippage_delta, brain.total_slippage, new_price
    );
    let _ = f_report_info(stone, brain, stuff, to_dr, &msg).await;

    to_dr.tell_dr_pls_trade
        .send(haggle_order.clone())
        .await
        .map_err(|e| {
          let errmsg = format!("{} [{}] Failed to send haggle retry order: {}", dbgspt, timestamp, e);
          BotError::OrderSendFailed(errmsg)
        })?;

    brain.m_pending_order = Some(haggle_order);

    // MAIN TODO 3: IMPLEMENTED - Wait for ORDER_FLY confirmation on retry
    if !brain.order_fly_confirmed {
        wait_for_order_confirmation(stone, brain, stuff, to_dr, told_it_sucks_api, &dbgspt).await?;
    }

    Ok(())
}

//===================================================================
// HELPER: Wait for ORDER_FLY / ORDER_BAD (MAIN TODO 3)
//===================================================================
async fn wait_for_order_confirmation(
    stone: &dsta::MakeSwatBotsGo,
    brain: &mut SwatBot,
    stuff: &crate::Allocation,
    to_dr: &crate::BridgeDrHand,
    told_it_sucks_api: &mut tokio::sync::mpsc::Receiver<String>,
    dbgspt: &str,
) -> BotResult 
{
  let timestamp = chrono::Utc::now().format("%Y-%m-%d %H:%M:%S%.f").to_string();
  let order_begin = chrono::Utc::now();
  let order_timeout = chrono::Duration::seconds(86400); // 1 day timeout
  let mut last_status_print = Instant::now();
  let status_interval = StdDuration::from_secs(3);

  loop 
  { let now = chrono::Utc::now();
    let elapsed = now - order_begin;

    // Print status every 3 seconds
    if last_status_print.elapsed() >= status_interval 
    { let msg = format!
      ( "{} [{}] Waiting for ORDER_FLY confirmation... (elapsed: {:.0}s)",
        dbgspt, timestamp, elapsed.num_seconds()
      );
      log::info!("{}", msg);
      let _ = f_report_info(stone, brain, stuff, to_dr, &msg).await;
      last_status_print = Instant::now();
    }

    // Check if we've exceeded 1-day timeout
    if elapsed > order_timeout 
    { let errmsg = format!(
          "{} [{}] ORDER_FLY timeout after 1 day - canceling order",
          dbgspt, timestamp
      );
      log::error!("{}", errmsg);
      let _ = f_report_error(stone, brain, stuff, to_dr, &errmsg).await;
      let _ = to_dr.tell_dr_pls_trade.send(Order { order_legs: vec![], order_type: MarketOrLimit::Limit }).await;
      return Err(BotError::Timeout(errmsg));
    }

    // Wait for message with timeout
    match tokio::time::timeout(StdDuration::from_secs(1), told_it_sucks_api.recv()).await 
    { Ok(Some(msg)) => 
      { if msg == glossary::ORDER_FLY 
        { let msg = format!("{} [{}] ORDER_FLY confirmed - order routed successfully", dbgspt, timestamp);
          log::info!("{}", msg);
          let _ = f_report_info(stone, brain, stuff, to_dr, &msg).await;
          brain.order_fly_confirmed = true;
          return Ok(());
        } 
        else if msg == glossary::ORDER_BAD 
        { let errmsg = format!("{} [{}] ORDER_BAD received - order rejected", dbgspt, timestamp);
          log::error!("{}", errmsg);
          let _ = f_report_error(stone, brain, stuff, to_dr, &errmsg).await;
          return Err(BotError::OrderSendFailed(errmsg));
        }
        // Ignore other messages (SWAP_SUCC, SWAP_FAIL, etc.)
      }
      ,
      Ok(None) => 
      { // Channel closed
        let errmsg = format!("{} [{}] Order confirmation channel closed", dbgspt, timestamp);
        log::error!("{}", errmsg);
        return Err(BotError::OrderSendFailed(errmsg));
        //return Err(BotError::ChannelClosed(errmsg));
      }
      ,
      Err(_) => 
      { // Timeout on this iteration - loop and check again
        continue;
      }
    }
  }
}

//===================================================================
// HELPERS
//===================================================================

async fn ticker_from_last_or_request(
    _bridge: &crate::BridgeDrHand,
    ticker_name: &str,
    _friendly_name: &str,
    _brain: &mut SwatBot,
    stuff: &crate::Allocation,
) -> Option<dsta::Ticker> {
    let find_ticker = |pos_opt: &Option<dsta::OwnedPosition>| -> Option<dsta::Ticker> {
        pos_opt.as_ref().and_then(|pos| {
            match &pos.asset {
                dsta::FinAss::StkDeets(stk) if stk.ticker_name == ticker_name => stk.last_ticker.clone(),
                dsta::FinAss::OptDeets(opt) if opt.ticker_name == ticker_name => opt.last_ticker.clone(),
                _ => None,
            }
        })
    };

    find_ticker(&stuff.bot_safe.m_stock)
        .or_else(|| find_ticker(&stuff.bot_safe.m_long_calls))
        .or_else(|| find_ticker(&stuff.bot_safe.m_long_puts))
        .or_else(|| find_ticker(&stuff.bot_safe.m_short_calls))
        .or_else(|| find_ticker(&stuff.bot_safe.m_short_puts))
        .or_else(|| {
            log::warn!("[SWAT_TICKER_GET] No last_ticker found for {}", ticker_name);
            None
        })
}

fn is_account_empty(account: &crate::BotaAccount) -> bool
{
  log::trace!("[SWAT_ACCOUNT_EMPTY] checking if account is completely empty");

  let result = account.m_cash.abs() < glossary::MIN_CASH
    && account.m_stock.is_none()
    && account.m_long_calls.is_none()
    && account.m_long_puts.is_none()
    && account.m_short_calls.is_none()
    && account.m_short_puts.is_none();

  log::trace!("[SWAT_ACCOUNT_EMPTY] account empty: {}", result);
  result
}

fn find_position(account: &crate::BotaAccount) -> Option<dsta::OwnedPosition>
{
  if let Some(p) = &account.m_stock { Some(p.clone()) }
  else if let Some(p) = &account.m_long_calls { Some(p.clone()) }
  else if let Some(p) = &account.m_long_puts { Some(p.clone()) }
  else if let Some(p) = &account.m_short_calls { Some(p.clone()) }
  else if let Some(p) = &account.m_short_puts { Some(p.clone()) }
  else { None }
}

//===================================================================
// HELPERS: Ticker preservation across all assets
//===================================================================
use crate::sbot::update_position_ticker;
use crate::sbot::merge_fresher_tickers;


mod glossary
{
  pub const TICK_CHANNEL_CLOSED: &str = "Trade channel closed";
  pub const ALLOC_CHANNEL_CLOSED: &str = "Allocation channel closed";
  pub const SUCKS_CHANNEL_CLOSED: &str = "Sucks channel closed";
  pub const SEPPUKU_TRANSITION: &str = "Transition to [SEPPUKU] => submitting aggressive exit order with haggle";
  pub const SEPPUKU_DURING_SEPPUKU: &str = "Received another seppuku while already seppuku";
  pub const ACCOUNT_EMPTY_ZOMBIED: &str = "Account is empty, reporting Zombied";
  pub const SEPPUKU_AFTER_ZOMBIED: &str = "Received seppuku after already zombied, exiting";
  pub const BASIC_ACCOUNT_UPDATE: &str = "Basic account changes";
  pub const DIE_COMMAND: &str = "Received DIE command";
  pub const EMPTY_SWAT_BOT: &str = "Empty swat bot shall track a tick from the swap info";
  pub const ORDER_FLY: &str = "ORDER_FLY";
  pub const ORDER_BAD: &str = "ORDER_BAD";
  pub const MIN_POSITION: f64 = 1.0;
  pub const MIN_CASH: f64 = 0.01;
}

use crate::sbot::HasFriendlyName;

impl HasFriendlyName for dsta::MakeSwatBotsGo {
    fn friendly_name(&self) -> &str {
        &self.my_accounting.friendly_name
    }
}

impl HasFriendlyName for &dsta::MakeSwatBotsGo {
    fn friendly_name(&self) -> &str {
        HasFriendlyName::friendly_name(*self)
    }
}

impl HasFriendlyName for &mut dsta::MakeSwatBotsGo {
    fn friendly_name(&self) -> &str {
        HasFriendlyName::friendly_name(*self)
    }
}