
// trade/usta/src/sbot/sally_bot.rs

use std::time::Duration;
use crate::sbot::errors::{BotError, BotResult};
use crate::sbot::position_for_order_name;
use crate::sbot::{f_report_info, f_report_error};

//===================================================================
// LOCAL DATA STRUCTURES
//===================================================================

#[derive(Clone, Debug, Default, serde::Serialize, serde::Deserialize, PartialEq, Eq)]
pub(crate) enum SallyState
{ #[default]

  WaitingForTrigger,
  OrderActive,
  Filled,
}

#[derive(Default, Clone, Debug, serde::Serialize, serde::Deserialize)]
pub(crate) struct SallyBot
{ pub m_state: SallyState,
  pub m_action: dsta::SallyAction,
  pub m_pending_order: Option<dsta::Order>,
  pub m_begin: Option<chrono::DateTime<chrono::Utc>>,

  pub haggle_context: dsta::HaggleContext,
  pub retry_at_delta: chrono::TimeDelta,
  pub total_slippage: f64,                    // accumulated slippage (starts at 0.0)
  pub haggle_attempt: u32,              // how many times we already applied haggle
  pub last_haggle_at: Option<chrono::DateTime<chrono::Utc>>,
  pub order_fly_confirmed: bool, // Track if order was routed
}

//===================================================================
// BOT CREATION & MAIN TASK
//===================================================================

pub(crate) async fn f_make_bot_sally_2
( stone: dsta::MakeSallyFakes,
) -> ( tokio::task::JoinHandle<()>, crate::CtorBot )
{ let friendly_name = &stone.my_accounting.friendly_name;
  let tracking_tick = &stone.my_accounting.tracking_tick;
  let dbgspt = format!("[SALLY_FN_MAKE2 - {}]", friendly_name);
  log::debug!
  ( "{} [ENTRY] f_make_bot_sally_2() - tracking {}"
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
  log::debug!("{} [EXIT] fn_make_bot_sally_2", dbgspt);
  ( tokio::spawn
    ( async move
      { fn_run_main_sally_bot
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


pub(crate) async fn fn_run_main_sally_bot
( constructor_stuff: dsta::MakeSallyFakes,
  mut told_to_trade_api: tokio::sync::mpsc::Receiver<dsta::Ticker>,
  mut told_to_check_api: tokio::sync::mpsc::Receiver<crate::Allocation>,
  mut sucks: tokio::sync::mpsc::Receiver<String>,
  to_dr: crate::BridgeDrHand,
)
{ let dbgspt = format!("[MAIN-SALLY {}]", constructor_stuff.my_accounting.friendly_name);
  log::debug!("{} ENTRY!", dbgspt);

  let mut stone = constructor_stuff;
  let mut stuff = crate::Allocation
  { bot_name: format!("{}", stone.my_accounting.friendly_name),
    bot_safe: crate::BotaAccount::default(),
    bot_mind: crate::BotMindState::Running,
    num_swap: 0,
  };

  // ── STEP 1: Attempt to load existing bot brain state ───────────────────────
  let msgmsg = format!("{} STEP 1: Attempting to load bot brain state...", dbgspt);
  let temp_brain = SallyBot::default();
  let _ = f_report_info(&stone, &temp_brain, &stuff, &to_dr, &msgmsg).await;

  let (mut brain, is_fresh_bot) = match crate::sbot::fn_load_bot_state
  ( &stone.my_accounting.friendly_name
  ).await
  {  Ok ( ( load_stone, load_brain, load_stuff ) ) => 
    { stone = load_stone; stuff = load_stuff;
      let msgmsg = format!("{} [LOAD SUCCESS] Loaded existing state from disk", dbgspt);
      let _ = f_report_info(&stone, &load_brain, &stuff, &to_dr, &msgmsg).await;
      ( load_brain, false )
    }
    Err(e) =>
    { log::info!("{} [NEW] No saved state found: {}. Creating fresh bot.", dbgspt, e);
      let fresh = SallyBot
      { m_state: SallyState::WaitingForTrigger,
        m_action: stone.my_reveal_way.clone(),
        m_pending_order: None, 
        m_begin: Some(chrono::Utc::now()),
        ..Default::default()
      };
      (fresh, true)
    }
  };

  // ── STEP 2: Initialize fresh bot (request cash) or resume saved bot ────────
  if is_fresh_bot
  { let msgmsg = format!("{} STEP 2.0a: Requesting initial cash allocation...", dbgspt);
    let _ = f_report_info(&stone, &brain, &stuff, &to_dr, &msgmsg).await;
    let mut totcost = match crate::core::fn_calculate_order_cost(&stone.my_hide_order)
    { Ok(cost) => cost - 1.0, // Add a $1 buffer for potential slippage
      Err(e) =>
      { let errmsg = format!("{} STEP 2.0a: Order cost calculation failed: {}", dbgspt, e);
        let _ = f_report_error(&stone, &brain, &stuff, &to_dr, &errmsg).await;
        stuff.bot_mind = crate::BotMindState::Zombied;
        let _ = to_dr.tell_dr_new_brain.send(stuff).await;
        return;
      }
    };

    if totcost > 0.0
    { let mut tmplist = std::collections::HashMap::new();
      for leg in &stone.my_hide_order.order_legs 
      { tmplist.insert(leg.buy_what.order_name(), leg.buy_what.clone());
      }

      let mut bs = crate::BotaAccount::default();
      bs.m_cash = 10_000_000_000_000_000.0;

      let ta = crate::core::fn_get_account_shennanagans
      ( &stone.my_hide_order,
        &bs,
        &tmplist,
      );

      totcost = if let Ok(changes) = ta 
      { crate::core::apply_account_cash_and_collateral_changes(&mut bs, &changes);
        -(10_000_000_000_000_000.0 - bs.m_cash) - 5.0
      } 
      else 
      { let errmsg = format!("{} STEP 2.0a: Failed to calculate required cash for credit spread", dbgspt);
        let _ = f_report_error(&stone, &brain, &stuff, &to_dr, &errmsg).await;
        stuff.bot_mind = crate::BotMindState::Zombied;
        let _ = to_dr.tell_dr_new_brain.send(stuff).await;
        return;
      };
    }

    if totcost < 0.0
    { let msgmsg = format!("{} STEP 2.0a: Order requires ${:.2} cash allocation...", dbgspt, totcost.abs());
      let _ = f_report_info(&stone, &brain, &stuff, &to_dr, &msgmsg).await;
      let _ = to_dr.tell_dr_swap_info.send
      ( crate::BotSwaper
        { source_offering: Some(crate::SwapWat
          { who: format!("{}", stone.my_accounting.friendly_name),
            wat: dsta::PositionsList::default(),
            how: 0.0,
          }),
          target_released: Some(crate::SwapWat
          { who: "USER_CASH".to_string(),
            wat: dsta::PositionsList::default(),
            how: totcost.abs(),
          }),
        }
      ).await;

      let msgmsg = format!("{} STEP 2.1a: Waiting for SWAP success reply...", dbgspt);
      let _ = f_report_info(&stone, &brain, &stuff, &to_dr, &msgmsg).await;
      let m_begin = chrono::Utc::now();
      loop
      { let swap_reply = tokio::select!
        { res = sucks.recv() => res,
          _ = tokio::time::sleep(Duration::from_secs(1)) => None,
        };

        match swap_reply
        { Some(msg) if msg == crate::glossary::SWAP_SUCC =>
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

      let msgmsg = format!("{} STEP 2.Xa: Fresh bot ready, proceeding to wait for trigger", dbgspt);
      let _ = f_report_info(&stone, &brain, &stuff, &to_dr, &msgmsg).await;
    }
    else
    { let errmsg = format!("{} STEP 2.0a: Could not calculate required cash balance (totcost={:.2})", dbgspt, totcost);
      let _ = f_report_error(&stone, &brain, &stuff, &to_dr, &errmsg).await;
      stuff.bot_mind = crate::BotMindState::Zombied;
      let _ = to_dr.tell_dr_new_brain.send(stuff).await;
      return;
    }
  }
  else
  { let msgmsg = format!("{} STEP 2.0b: Resuming saved bot state...", dbgspt);
    let _ = f_report_info(&stone, &brain, &stuff, &to_dr, &msgmsg).await;
  }

  // STEP 3: Request ticker for each asset in the hidden order
  let msgmsg = format!("{} STEP 3: Requesting ticker details for all legs in the hidden order...", dbgspt);
  let _ = f_report_info(&stone, &brain, &stuff, &to_dr, &msgmsg).await;

  for leg in &stone.my_hide_order.order_legs 
  { let ticker_name = leg.buy_what.ticker_name();
    let (tx_asset, mut rx_asset) = tokio::sync::mpsc::channel(2);

    if to_dr.tell_dr_pls_ticks
      .send((ticker_name.clone(), tx_asset))
      .await
      .is_err() 
    { let errmsg = format!("{} Ticker request send failed for {}", dbgspt, ticker_name);
      let _ = f_report_error(&stone, &brain, &stuff, &to_dr, &errmsg).await;
      stuff.bot_mind = crate::BotMindState::Zombied;
      let _ = to_dr.tell_dr_new_brain.send(stuff).await;
      return;
    }

    match tokio::time::timeout(Duration::from_secs(5), rx_asset.recv()).await 
    { Ok(Some(Some(_deets))) => 
      { let msgmsg = format!("{} Successfully received ticker details for {}", dbgspt, ticker_name);
        let _ = f_report_info(&stone, &brain, &stuff, &to_dr, &msgmsg).await;
      }
      _ => 
      { let errmsg = format!("{} Timeout or error receiving ticker for {}", dbgspt, ticker_name);
        let _ = f_report_error(&stone, &brain, &stuff, &to_dr, &errmsg).await;
        stuff.bot_mind = crate::BotMindState::Zombied;
        let _ = to_dr.tell_dr_new_brain.send(stuff).await;
        return;
      }
    }
  }

  // STEP 4: Main forever loop 
  brain.m_begin = Some(chrono::Utc::now());
  let msgmsg = format!("{} STEP 4: Entering main event loop...", dbgspt);
  let _ = f_report_info(&stone, &brain, &stuff, &to_dr, &msgmsg).await;
  loop
  { tokio::select!
    { res = told_to_trade_api.recv() =>
      { if let Some(ticker) = res
        { if let Err(e) = handle_quote(&ticker, &mut stone, &mut brain, &mut stuff, &to_dr, &mut sucks).await
          { let errmsg = format!("Quote handling failed: {}", e);
            let _ = f_report_error(&stone, &brain, &stuff, &to_dr, &errmsg).await;
            break;
          }
        }
        else
        { let errmsg = "trade channel closed".to_string();
          let _ = f_report_error(&stone, &brain, &stuff, &to_dr, &errmsg).await;
          break;
        }
      }

      res = told_to_check_api.recv() =>
      { if let Some(alloc) = res
        { if alloc.bot_name == stone.my_accounting.friendly_name
          { if let Err(e) = handle_check(&alloc, &mut stone, &mut brain, &mut stuff, &to_dr, &mut sucks).await
            { let errmsg = format!("Account update handling failed: {}", e);
              let _ = f_report_error(&stone, &brain, &stuff, &to_dr, &errmsg).await;
              break;
            }
          }
        }
      }

      res = sucks.recv() =>
      { if let Some(msg) = res
        { if msg == "DIE"
          { log::debug!("{} [EXIT] Received DIE signal", dbgspt);
            break;
          }
          else 
          { log::info!("{} Sally bot got an unhandled snively: {:#?}", dbgspt, msg);
          }
        }
      }

      _ = tokio::time::sleep(Duration::from_secs(30)) =>
      { let msgmsg = format!("{} Sally bot waiting on some action!", dbgspt);
        let _ = f_report_info(&stone, &brain, &stuff, &to_dr, &msgmsg).await;
      }
    }

    if brain.m_state == SallyState::Filled
    { let msgmsg = format!("{} Order filled and processed - Sally exiting cleanly", dbgspt);
      let _ = f_report_info(&stone, &brain, &stuff, &to_dr, &msgmsg).await;
      break;
    }
  }

  // ── STEP 5: Shutdown 
  let msgmsg = format!("{} STEP 5: Sally bot shutting down!", dbgspt);
  let _ = f_report_info(&stone, &brain, &stuff, &to_dr, &msgmsg).await;

  stuff.bot_mind = crate::BotMindState::Zombied;
  let _ = to_dr.tell_dr_new_brain.send(stuff).await;
}

//===================================================================
// CORE ROBO API HANDLERS
//===================================================================

async fn handle_check
(
  check: &crate::Allocation,
  stone: &mut dsta::MakeSallyFakes,
  brain: &mut SallyBot,
  stuff: &mut crate::Allocation,
  to_dr: &crate::BridgeDrHand,
  sucks: &mut tokio::sync::mpsc::Receiver<String>,
) -> BotResult
{ if check.bot_name != stuff.bot_name { return Ok(()); }
  if check.bot_mind != stuff.bot_mind
  { return state_machine_mind(check, stone, brain, stuff, to_dr).await;
  }
  else
  { *stuff = check.clone();
    return state_machine_tick(stone, brain, stuff, to_dr, sucks).await;
  }
}

async fn handle_quote
( ticks: &dsta::Ticker,
  stone: &mut dsta::MakeSallyFakes,
  brain: &mut SallyBot,
  stuff: &mut crate::Allocation,
  to_dr: &crate::BridgeDrHand,
  sucks: &mut tokio::sync::mpsc::Receiver<String>,
) -> BotResult
{ let dbgspt = format!("[SALLY QUOTE {}]", stone.my_accounting.friendly_name);
  let mut updated = false;
  for leg in &mut stone.my_hide_order.order_legs 
  { if leg.buy_what.ticker_name() == ticks.name 
    { leg.buy_what.set_ticker(ticks.clone());
      updated = true;
      log::trace!("{} Set ticker on {:#?}", dbgspt, leg);
    }
  }

  if !updated 
  { log::warn!
    ( "{} received ticker not in candidates or underlying: {}"
      , dbgspt, ticks.name
    );
    Ok(())
  }
  else
  { return state_machine_tick(stone, brain, stuff, to_dr,sucks).await;
  }
}

//===================================================================
// STATE MACHINE PROCESSORS
//===================================================================

async fn state_machine_mind
( check: &crate::Allocation,
  stone: &mut dsta::MakeSallyFakes,
  brain: &mut SallyBot,
  stuff: &mut crate::Allocation,
  to_dr: &crate::BridgeDrHand,
) -> BotResult
{ let dbgspt = format!("[MIND_SWAP {}]", stone.my_accounting.friendly_name);
  let timestamp = chrono::Utc::now().format("%Y-%m-%d %H:%M:%S%.f").to_string();
  log::info!("{} [{}] Sally bot mind transitioning to {:#?} from {:#?}", dbgspt, timestamp, check.bot_mind, stuff.bot_mind);
  match check.bot_mind
  { crate::BotMindState::Running =>
    { match stuff.bot_mind
      { crate::BotMindState::Trading =>
        { log::info!("{} mind transition indicates trade completed", dbgspt);
          *stuff = check.clone();
          return Ok(());
        }
        crate::BotMindState::Stopped =>
        { log::info!("{} mind transition indicates save request", dbgspt);
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
    { log::info!("{} mind transition indicates save request", dbgspt);
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
( stone: &mut dsta::MakeSallyFakes,
  brain: &mut SallyBot,
  stuff: &mut crate::Allocation,
  to_dr: &crate::BridgeDrHand,
  sucks: &mut tokio::sync::mpsc::Receiver<String>,
) -> BotResult<()>
{
  log::trace!("sally state machine ticking with {:#?}", brain.m_state);

  let res : BotResult<()> = match brain.m_state
  { SallyState::WaitingForTrigger => f_run_waiting_for_trigger(stone, brain, stuff, to_dr, sucks).await,
    SallyState::OrderActive => f_run_check_order_fill(stone, brain, stuff, to_dr, sucks).await,
    SallyState::Filled => Ok(()),
  };

  res
}

//===================================================================
// STATE HANDLERS
//===================================================================
async fn f_run_waiting_for_trigger
(
  stone: &mut dsta::MakeSallyFakes,
  brain: &mut SallyBot,
  stuff: &mut crate::Allocation,
  to_dr: &crate::BridgeDrHand,
    sucks: &mut tokio::sync::mpsc::Receiver<String>,
) -> BotResult<()> 
{
  let dbgspt = format!("[SALLY TRIGGER {}]", stone.my_accounting.friendly_name);
  let timestamp = chrono::Utc::now().format("%Y-%m-%d %H:%M:%S%.f").to_string();

  // ── 1. Early exit if any leg lacks ticker data ─────────────────────────
  for leg in &stone.my_hide_order.order_legs 
  { if leg.buy_what.last_ticker().is_none() 
    { log::trace!("{} still waiting for quotes on all legs", dbgspt);
      return Ok(());
    }
  }

  log::trace!("Sally Checking trigger loggic: ");

  // ── 2. Minimal market view just for trigger evaluation ─────────────────
  let net_per_contract = stone.my_hide_order.order_legs.iter().fold(0.0, |acc, leg| {
      let signed = if matches!(leg.action, dsta::BuyOrSell::SellToOpen | dsta::BuyOrSell::SellToClose) {
          leg.price
      } else {
          -leg.price
      };
      acc + signed * leg.quantity
  });

  let is_net_credit = net_per_contract > 0.0;
  let mut combo_bid = 0.0_f64;
  let mut combo_ask = 0.0_f64;

  for leg in &stone.my_hide_order.order_legs 
  { let t = leg.buy_what.last_ticker().expect("All legs must have fresh ticker data");
    let leg_sign = if leg.action.is_buy() { -1.0 } else { 1.0 };
    combo_bid += leg_sign * t.bid;
    combo_ask += leg_sign * t.ask;
    log::trace!("{} sign = {}, what={}, t.bid = {}, t.ask = {} \n\n\n{:#?}"
    , dbgspt, leg_sign, t.name,t.bid,t.ask,leg
    );
  }

  let (price_we_have, price_we_want) = if is_net_credit 
  { (combo_bid, combo_ask)
  } 
  else 
  { (combo_ask, combo_bid)
  };
  let combo_mid = (price_we_have + price_we_want) / 2.0;

  log::trace!("{} - combo_bid = {}; combo_ask = {}"
  , dbgspt, combo_bid, combo_ask,
  );

  // ── 3. Evaluate trigger ─────────────────────────────────────────────────
  let triggered = match &brain.m_action 
  {
    dsta::SallyAction::SubmitRightAwayButLikeOnlyUseForTest => 
    { log::info!("{} [{}] TEST MODE => sally submitting immediately", dbgspt, timestamp);
      true
    }

    dsta::SallyAction::SubmitWhenBidAskMidAttainsInputPrice(target) => 
    { log::trace!("{} SubmitWhenBidAskMidAttainsInputPrice combo_mid vs target => {} vs {} "
      ,  dbgspt, combo_mid, *target
      );
      if is_net_credit 
      { combo_mid >= *target 
      } else 
      { combo_mid <= *target 
      }
    }

    dsta::SallyAction::SubmitWhenMarketSideReachesThisPrice(target) => 
    { log::trace!("{} SubmitWhenMarketSideReachesThisPrice combo_mid vs price_we_want => {} vs {} "
      ,  dbgspt, price_we_want, *target
      );
      if is_net_credit 
      { price_we_want >= *target 
      } else 
      { price_we_want <= *target 
      }
    }

    dsta::SallyAction::SubmitAtThisTimeUnlessItReachesPrice(_batime, _this_price) => 
    { log::warn!("{} {} not yet implemented", dbgspt, stringify!(SubmitAtThisTimeUnlessItReachesPrice));
      false
    }

  };

  if !triggered 
  { return Ok(());
  }

    let msg = format!(
        "{} [{}] TRIGGER HIT → creating initial haggle order (mid={:.3}, want={:.3})",
        dbgspt, timestamp, combo_mid, price_we_want
    );
    let _ = f_report_info(stone, brain, stuff, to_dr, &msg).await;

    if brain.m_pending_order.is_none() {
        let method = match &stone.my_accounting.haggle_action {
            dsta::HaggleAction::JustCancel(m) | dsta::HaggleAction::AnOyVeyJew(m) => m,
        };

        let (initial_order, initial_ctx) = dsta::create_initial_haggle_order(
            &stone.my_hide_order,
            method,
        );
        brain.m_pending_order = Some(initial_order);
        brain.haggle_context = initial_ctx;
        brain.total_slippage = 0.0;
        brain.haggle_attempt = 0;
        brain.last_haggle_at = Some(chrono::Utc::now());
        brain.retry_at_delta = stone.my_accounting.haggle_action.get_retry_period();
        brain.order_fly_confirmed = false;

        let msg = format!(
            "{} Initial haggle order created | combo price ? | spread: {:.4} | method: {:?}",
            dbgspt, brain.haggle_context.initial_spread, method
        );
        let _ = f_report_info(stone, brain, stuff, to_dr, &msg).await;
    } else {
        log::warn!("{} Pending order already exists on trigger — skipping", dbgspt);
    }

    // ── 5. Submit ───────────────────────────────────────────────────────────
    let order = brain.m_pending_order.as_ref().unwrap().clone();

    if let Err(e) = to_dr.tell_dr_pls_trade.send(order).await {
        let errmsg = format!("{} Failed to send initial order: {}", dbgspt, e);
        let _ = f_report_error(stone, brain, stuff, to_dr, &errmsg).await;
        return Err(BotError::OrderSendFailed(errmsg));
    }

    // ── 5.5 WAIT FOR ORDER_FLY CONFIRMATION ─────────────────────────────────
    wait_for_sally_order_confirmation(stone, brain, stuff, to_dr, sucks, &dbgspt).await?;

    // ── 6. Move to monitoring state ─────────────────────────────────────────
    brain.m_state = SallyState::OrderActive;
    let msg = format!("{} Initial order submitted → now in OrderActive", dbgspt);
    let _ = f_report_info(stone, brain, stuff, to_dr, &msg).await;

    Ok(())
}

// ─────────────────────────────────────────────────────────────────────────────
// UPGRADED: f_run_check_order_fill with ORDER_FLY/ORDER_BAD checks
// ─────────────────────────────────────────────────────────────────────────────

async fn f_run_check_order_fill(
    stone: &mut dsta::MakeSallyFakes,
    brain: &mut SallyBot,
    stuff: &mut crate::Allocation,
    to_dr: &crate::BridgeDrHand,
    sucks: &mut tokio::sync::mpsc::Receiver<String>,
) -> BotResult<()> {
    let dbgspt = format!("[SALLY CHECK {}]", stone.my_accounting.friendly_name);
    let timestamp = chrono::Utc::now().format("%Y-%m-%d %H:%M:%S%.f").to_string();

    // ── 1. Get the pending order (must exist in OrderActive state) ───────────
    let order = match brain.m_pending_order.as_ref() {
        Some(o) => o,
        None => return Ok(()),
    };

    // ── 2. Check if order is fully filled ────────────────────────────────────
    let filled = order.order_legs.iter().all(|leg| {
        let pos = position_for_order_name(&stuff.bot_safe, &leg.buy_what.order_name());
        pos.map_or(false, |p| {
            let required = leg.quantity.abs() as f64;
            if leg.quantity > 0.0 {
                p.owned >= required
            } else {
                p.owned <= -required
            }
        })
    });

    if filled {
        let msgmsg = format!("{} [{}] Order filled successfully", dbgspt, timestamp);
        let _ = f_report_info(stone, brain, stuff, to_dr, &msgmsg).await;
        brain.m_state = SallyState::Filled;

        let _ = to_dr.tell_dr_swap_info.send(
            crate::BotSwaper {
                source_offering: Some(crate::SwapWat {
                    who: stuff.bot_name.clone(),
                    wat: dsta::PositionsList::default(),
                    how: stuff.bot_safe.m_cash,
                }),
                target_released: Some(crate::SwapWat {
                    who: "USER_CASH".to_string(),
                    wat: dsta::PositionsList::default(),
                    how: crate::glossary::TOUCH_NO_HOW_WHO_IN_A_SWAP,
                }),
            }
        ).await;

        let msgmsg = format!("{} [{}] Returned remaining cash to USER_CASH", dbgspt, timestamp);
        let _ = f_report_info(stone, brain, stuff, to_dr, &msgmsg).await;
        return Ok(());
    }

    // ── 3. Check if we're in a partial fill state ────────────────────────────
    let partial_fill_ratio = calculate_partial_fill_ratio(order, &stuff.bot_safe);
    if partial_fill_ratio > 0.0 && partial_fill_ratio < 1.0 {
        let msgmsg = format!(
            "{} [{}] Order partially filled ({:.1}%) - will adjust remaining quantity",
            dbgspt, timestamp, partial_fill_ratio * 100.0
        );
        let _ = f_report_info(stone, brain, stuff, to_dr, &msgmsg).await;
    }

    // ── 4. Verify ORDER_FLY on first entry ───────────────────────────────────
    if !brain.order_fly_confirmed {
        if let Err(_e) = wait_for_sally_order_confirmation(stone, brain, stuff, to_dr, sucks, &dbgspt).await {
            // Log already handled in wait function
        }
    }

    // ── 5. Check if retry period has elapsed ─────────────────────────────────
    let haggle_action = &stone.my_accounting.haggle_action;
 
    let should_retry = match brain.last_haggle_at {
        Some(last_time) => chrono::Utc::now() - last_time >= brain.retry_at_delta,
        None => false,
    };

    if !should_retry {
        if let Some(begin) = brain.m_begin {
            let elapsed = chrono::Utc::now() - begin;
            let max_wait = chrono::Duration::seconds(300); // 5-minute absolute timeout
            if elapsed > max_wait {
                let errmsg = format!(
                    "{} [{}] Timeout waiting for order fill after {:.0}s",
                    dbgspt, timestamp, elapsed.num_seconds()
                );
                return Err(BotError::Timeout(errmsg));
            }
        }
        return Ok(());
    }

    // ── 6. Retry period has elapsed → execute haggle action ──────────────────
    let msgmsg = format!(
        "{} [{}] Retry period elapsed (attempt #{}) - evaluating haggle action",
        dbgspt, timestamp, brain.haggle_attempt + 1
    );
    let _ = f_report_info(stone, brain, stuff, to_dr, &msgmsg).await;

    match haggle_action 
    { dsta::HaggleAction::JustCancel(method) => 
      { log::info!("{} haggle action => Just Canceling method = {:#?}", dbgspt, method);
        f_handle_cancel_action(stone, brain, stuff, to_dr, &dbgspt, &timestamp).await
      }
      dsta::HaggleAction::AnOyVeyJew(method) => 
      { f_handle_haggle_action(stone, brain, stuff, to_dr, sucks, method, partial_fill_ratio, &dbgspt, &timestamp).await
      }
    }
}

//===================================================================
// HELPER: Wait for ORDER_FLY confirmation
//===================================================================
/* depreciated by AI

  async fn wait_for_order_fly(
    stone: &dsta::MakeSallyFakes,
    brain: &mut SallyBot,
    stuff: &crate::Allocation,
    to_dr: &crate::BridgeDrHand,
    sucks: &mut tokio::sync::mpsc::Receiver<String>,
    dbgspt: &str,
    timestamp: &str,
) -> BotResult {
    // Try to receive ORDER_FLY or ORDER_BAD without blocking main loop
    // Use a small timeout to check once
    match tokio::time::timeout(Duration::from_millis(100), sucks.recv()).await {
        Ok(Some(msg)) => {
            if msg == glossary::ORDER_FLY {
                let msg_str = format!(
                    "{} [{}] ORDER_FLY confirmed - order routed successfully",
                    dbgspt, timestamp
                );
                log::info!("{}", msg_str);
                let _ = f_report_info(stone, brain, stuff, to_dr, &msg_str).await;
                brain.order_fly_confirmed = true;
                Ok(())
            } else if msg == glossary::ORDER_BAD {
                let errmsg = format!("{} [{}] ORDER_BAD received - order rejected", dbgspt, timestamp);
                log::error!("{}", errmsg);
                let _ = f_report_error(stone, brain, stuff, to_dr, &errmsg).await;
                Err(BotError::OrderSendFailed(errmsg))
            } else {
                // Not an order confirmation, put it back by ignoring it
                Ok(())
            }
        }
        Ok(None) => {
            Err(BotError::OrderSendFailed("Order confirmation channel closed".to_string()))
        }
        Err(_) => {
            // Timeout - not ready yet
            Ok(())
        }
    }
}
*/
// Above is replaced with:
async fn wait_for_sally_order_confirmation(
    stone: &dsta::MakeSallyFakes,
    brain: &mut SallyBot,
    stuff: &crate::Allocation,
    to_dr: &crate::BridgeDrHand,
    sucks: &mut tokio::sync::mpsc::Receiver<String>,
    dbgspt: &str,
) -> BotResult {
    let timestamp = chrono::Utc::now().format("%Y-%m-%d %H:%M:%S%.f").to_string();
    let order_begin = chrono::Utc::now();
    let order_timeout = chrono::Duration::seconds(30);
    let mut last_log = std::time::Instant::now();
    let log_interval = std::time::Duration::from_secs(3);

    loop {
        let elapsed = chrono::Utc::now() - order_begin;

        // Log status every 3 seconds
        if last_log.elapsed() >= log_interval {
            let msg = format!(
                "{} [{}] Waiting for ORDER_FLY confirmation... (elapsed: {:.0}s)",
                dbgspt, timestamp, elapsed.num_seconds()
            );
            log::info!("{}", msg);
            let _ = f_report_info(stone, brain, stuff, to_dr, &msg).await;
            last_log = std::time::Instant::now();
        }

        if elapsed > order_timeout {
            let errmsg = format!(
                "{} [{}] ORDER_FLY timeout after {:.0}s - canceling order",
                dbgspt, timestamp, elapsed.num_seconds()
            );
            log::error!("{}", errmsg);
            let _ = f_report_error(stone, brain, stuff, to_dr, &errmsg).await;
            let _ = to_dr.tell_dr_pls_trade.send(dsta::Order {
                order_legs: vec![],
                order_type: dsta::MarketOrLimit::Limit,
            }).await;
            return Err(BotError::Timeout(errmsg));
        }

        match tokio::time::timeout(std::time::Duration::from_secs(1), sucks.recv()).await {
            Ok(Some(msg)) if msg == glossary::ORDER_FLY => {
                let msg_str = format!("{} [{}] ORDER_FLY confirmed - order routed successfully", dbgspt, timestamp);
                log::info!("{}", msg_str);
                let _ = f_report_info(stone, brain, stuff, to_dr, &msg_str).await;
                brain.order_fly_confirmed = true;
                return Ok(());
            }
            Ok(Some(msg)) if msg == glossary::ORDER_BAD => {
                let errmsg = format!("{} [{}] ORDER_BAD received - order rejected", dbgspt, timestamp);
                log::error!("{}", errmsg);
                let _ = f_report_error(stone, brain, stuff, to_dr, &errmsg).await;
                return Err(BotError::OrderSendFailed(errmsg));
            }
            Ok(None) => {
                let errmsg = format!("{} [{}] Order confirmation channel closed", dbgspt, timestamp);
                log::error!("{}", errmsg);
                return Err(BotError::OrderSendFailed(errmsg));
            }
            _ => continue,
        }
    }
}

//===================================================================
// HELPERS
//===================================================================

// Handler: CANCEL action
async fn f_handle_cancel_action(
    stone: &dsta::MakeSallyFakes,
    brain: &mut SallyBot,
    stuff: &mut crate::Allocation,
    to_dr: &crate::BridgeDrHand,
    dbgspt: &str,
    timestamp: &str,
) -> BotResult<()> {
    let msgmsg = format!(
        "{} [{}] JustCancel action triggered → sending empty order to cancel",
        dbgspt, timestamp
    );
    let _ = f_report_info(stone, brain, stuff, to_dr, &msgmsg).await;

    let cancel_order = dsta::Order {
        order_legs: vec![],
        order_type: dsta::MarketOrLimit::Limit,
    };

    if let Err(e) = to_dr.tell_dr_pls_trade.send(cancel_order).await {
        let errmsg = format!("{} [{}] Failed to send cancel order: {}", dbgspt, timestamp, e);
        let _ = f_report_error(stone, brain, stuff, to_dr, &errmsg).await;
        return Err(BotError::OrderSendFailed(errmsg));
    }

    let msgmsg = format!("{} [{}] Cancel order sent - exiting trade", dbgspt, timestamp);
    let _ = f_report_info(stone, brain, stuff, to_dr, &msgmsg).await;

    brain.m_state = SallyState::Filled;
    Ok(())
}

// Handler: HAGGLE action
async fn f_handle_haggle_action
(
    stone: &dsta::MakeSallyFakes,
    brain: &mut SallyBot,
    stuff: &mut crate::Allocation,
    to_dr: &crate::BridgeDrHand,
    sucks: &mut tokio::sync::mpsc::Receiver<String>,
    haggle_method: &dsta::HaggleMethod,
    partial_fill_ratio: f64,
    dbgspt: &str,
    timestamp: &str,
) -> BotResult<()> 
{ let msgmsg = format!
  ( "{} [{}] AnOyVeyJew action triggered → applying haggle logic",
    dbgspt, timestamp
  );
  let _ = f_report_info(stone, brain, stuff, to_dr, &msgmsg).await;
  let mut haggled_order = match brain.m_pending_order.as_ref() 
  { Some(o) => o.clone(),
    None => return Ok(()),
  };
  let remaining_ratio = 1.0 - partial_fill_ratio;
  for leg in &mut haggled_order.order_legs 
  { leg.remaining = leg.quantity * remaining_ratio;
    let orig_qty = leg.quantity;
    leg.quantity = leg.remaining;
    log::debug!
    (
      "{} Leg {} adjusted: {:.0} → {:.0} (fill ratio: {:.1}%)",
      dbgspt, leg.buy_what.order_name(), orig_qty, leg.quantity, partial_fill_ratio * 100.0
    );
  }

  let pre_cost = crate::core::fn_calculate_order_cost(&haggled_order);
  let slippage_delta = haggle_method.apply_haggle_changes(&mut haggled_order, &brain.haggle_context);
  brain.total_slippage += slippage_delta;
  brain.haggle_attempt += 1;
  let new_cost = crate::core::fn_calculate_order_cost(&haggled_order);
  let msgmsg = format!
  ( "{} [{}] Haggle applied | attempt: {} | slippage delta: {:.4} | total: {:.4} \n pre vs new cost {:#?} vs {:#?}"
    , dbgspt, timestamp, brain.haggle_attempt, slippage_delta, brain.total_slippage
    , pre_cost, new_cost, 
  );
  

  let _ = f_report_info(stone, brain, stuff, to_dr, &msgmsg).await;

  let slippage_max = match haggle_method 
  { dsta::HaggleMethod::MinimumOfMidpointOrTheoreticalThenConcede(limits) => limits.slippage_max,
    dsta::HaggleMethod::VirtualMarketOrderWithLimitOrdThenConcede(limits) => limits.slippage_max,
    dsta::HaggleMethod::CheapLimitOrderThenIncrementDeltaUntilMax(limits) => limits.slippage_max,
  };

  if brain.total_slippage > slippage_max 
  { let errmsg = format!
    ( "{} [{}] Slippage exceeded max ({:.4} > {:.4}) - canceling",
      dbgspt, timestamp, brain.total_slippage, slippage_max
    );
    let _ = f_report_error(stone, brain, stuff, to_dr, &errmsg).await;

    let cancel_order = dsta::Order 
    { order_legs: vec![],
      order_type: dsta::MarketOrLimit::Limit,
    };
    let _ = to_dr.tell_dr_pls_trade.send(cancel_order).await;
    brain.m_state = SallyState::Filled;
    return Ok(());
  }

  brain.m_pending_order = Some(haggled_order.clone());
  brain.last_haggle_at = Some(chrono::Utc::now());

  if let Err(e) = to_dr.tell_dr_pls_trade.send(haggled_order).await 
  { let errmsg = format!
    ( "{} [{}] Failed to send haggled order (attempt #{}): {}",
      dbgspt, timestamp, brain.haggle_attempt, e
    );
    let _ = f_report_error(stone, brain, stuff, to_dr, &errmsg).await;
    return Err(BotError::OrderSendFailed(errmsg));
  }

  // Wait for ORDER_FLY on retry order too
  wait_for_sally_order_confirmation(stone, brain, stuff, to_dr, sucks, &dbgspt).await?;

  let msgmsg = format!
  ( "{} [{}] Haggled order resubmitted (attempt #{})",
    dbgspt, timestamp, brain.haggle_attempt
  );
  let _ = f_report_info(stone, brain, stuff, to_dr, &msgmsg).await;
  Ok(())
}

fn calculate_partial_fill_ratio(
    order: &dsta::Order,
    account: &crate::BotaAccount,
) -> f64 
{
  if order.order_legs.is_empty() {
      return 0.0;
  }

  let mut total_ratio_sum = 0.0;
  let mut leg_count = 0;

  for leg in &order.order_legs {
      let pos = position_for_order_name(account, &leg.buy_what.order_name());
      let required = leg.quantity.abs();

      let filled = if let Some(p) = pos {
          if leg.quantity > 0.0 {
              p.owned.min(required)
          } else {
              (-p.owned).min(required)
          }
      } else {
          0.0
      };

      let leg_ratio = (filled / required).min(1.0).max(0.0);
      total_ratio_sum += leg_ratio;
      leg_count += 1;
  }

  if leg_count == 0 {
      0.0
  } else {
      total_ratio_sum / leg_count as f64
  }
}

mod glossary
{
  pub const ORDER_FLY: &str = "ORDER_FLY";
  pub const ORDER_BAD: &str = "ORDER_BAD";
}

use crate::sbot::HasFriendlyName;

impl HasFriendlyName for dsta::MakeSallyFakes {
    fn friendly_name(&self) -> &str {
        &self.my_accounting.friendly_name
    }
}

impl HasFriendlyName for &dsta::MakeSallyFakes {
    fn friendly_name(&self) -> &str {
        HasFriendlyName::friendly_name(*self)
    }
}

impl HasFriendlyName for &mut dsta::MakeSallyFakes {
    fn friendly_name(&self) -> &str {
        HasFriendlyName::friendly_name(*self)
    }
}