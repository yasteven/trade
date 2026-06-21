//dr_bot0.rs
//This is the ttai order processing bot with cancel order and dry-run support

use crate::*;

// Supporting Structs

#[derive(Debug, Clone)]
pub(crate) struct NewOrderRequest 
{   pub bot_api_order_bot: ( usize, u64 ) // index, order number
  , pub bot_api_named_bot: String // DO i need this?
  , pub tell_bot_it_sucks: tokio::sync::mpsc::Sender<String>
  , pub bot_order_request: dsta::Order
  , pub time_until_cancel: Option<tokio::time::Instant>
}

#[derive(Debug)]
pub(crate) struct CancelOrderRequest 
{   pub bot_api_order_bot: ( usize, u64 ) // index, order number
  , pub bot_api_named_bot: String
  , pub tell_bot_it_sucks: tokio::sync::mpsc::Sender<String>
  , pub order_hash: u64  // The order ID to cancel
  , pub reply_tx: tokio::sync::oneshot::Sender<Result<bool, Box<dyn std::error::Error + Send + Sync>>>,
}

#[derive(Debug)]
pub(crate) struct DryRunRequest 
{   pub bot_api_order_bot: ( usize, u64 ) // index, order number
  , pub bot_api_named_bot: String
  , pub tell_bot_it_sucks: tokio::sync::mpsc::Sender<String>
  , pub bot_order_request: dsta::Order
  , pub reply_tx: tokio::sync::oneshot::Sender<Result<f64, Box<dyn std::error::Error + Send + Sync>>>,
}

#[derive(Debug)]
pub(crate) struct OrderTrackerEntry 
{   pub request: NewOrderRequest
  , pub updates: Vec<dsta::PlacedOrder>
  , pub sum_all: dsta::Order
  , pub start_time: Option<tokio::time::Instant>
  , pub last_update_time: Option<tokio::time::Instant>
  , pub is_cancelled: bool  // NEW: Track if order was cancelled
}

impl OrderTrackerEntry 
{ pub fn new(request: NewOrderRequest, hash: u64) -> Self 
  { log::info!("OrderTrackerEntry created for hash {}", hash);
    
    let order_type = request.bot_order_request.order_type.clone();
    //let order_cost = request.bot_order_request.order_cost.clone();
    Self 
    { updates: Vec::new()
    , sum_all: dsta::Order 
      { order_legs: request.bot_order_request.order_legs.iter().map
        ( |leg| 
          dsta::OrderLeg 
          { buy_what: leg.buy_what.clone()
          , quantity: 0.0 // Start with 0 filled
          , remaining: leg.quantity // All quantity is remaining
          , action: leg.action.clone()
          , price: leg.price
          }
        ).collect()
      , order_type:order_type //request.bot_order_request.order_type
      }
    , start_time: Some(tokio::time::Instant::now())
    , last_update_time: None
    , request: request // .clone()
    , is_cancelled: false  // NEW
    }
  }


}

impl Drop for OrderTrackerEntry 
{ fn drop(&mut self) 
  { log::info!("OrderTrackerEntry deleted for bot {:#?}"
  , self.request.bot_api_order_bot);
  }
}





mod from_ttai
{ 
  use crate::*;
  pub async fn deal_with_done_order
  (   oh : u64
    , order_tracker: &mut std::collections::HashMap< u64, dr_bot0::OrderTrackerEntry>
    , order_history: &mut std::collections::HashMap< u64, dr_bot0::OrderTrackerEntry>
    , our_dr_r_ask_robo: &DrRoboApi
  )
  { let dbgspt = format!("[DB0FTDWDO]x{}", oh);
    log::debug!("[ENTRY] = dr_bot0.rs::from_ttai::deal_with_done_order(x) {}", dbgspt);
    
    log::trace!("Remove the entry from the order tracker and process it...");
    if let Some(entry) = order_tracker.remove(&oh) 
    { let is_full_fill = entry.sum_all.order_legs.iter().zip
      ( entry.request.bot_order_request.order_legs.iter()).all
      ( |(sum_leg, req_leg)| { sum_leg.quantity >= req_leg.quantity }
      );
      
      // NEW: Handle cancelled orders differently
      if entry.is_cancelled 
      { log::info!("{} Order was cancelled, skipping pls_trade notification", dbgspt);
        // Still add to history for record-keeping
        order_history.insert(oh, entry);
        return;
      }
      
      // TODO: we may need to calculate the cash still on a partial order, by checking the total cost somewhere
      // TODO: we need to update the cost function:
      // (1): request from the ttai the actual costs (transaction?)
      // I would say if it's not full order
      //  we need to use the lower cost from (1)
      //  we already waited 1 second to get to this point if it wasn't filled
      //
      if !is_full_fill
      { log::trace!("{} The done order is not full_fill! : {:#?} ",dbgspt, entry);
      }
      else 
      { log::trace!("{} The done order full_fill!",dbgspt);
      }
      
      log::trace!("{} now we extract original bot that ordered, and do a magic pls_trade", dbgspt);
      let (idb,ido) = entry.request.bot_api_order_bot;
      let id0 = ( idb as u64 ) | (ido << 20);
      let _ = our_dr_r_ask_robo.tell_dr_pls_trade.send
      ( ( 0
        , id0 
        , entry.sum_all.clone()
        )
      ).await;
      order_history.insert(oh, entry);
    }
    else
    { if order_history.contains_key(&oh)
      { log::info!("good job, no order to deal with means it was already dealth with!");
      }
      else
      { log::error!("WTF, no order to deal with and not in the history!");
      }
    } 
    log::debug!("EXIT! - {}", dbgspt);
  }

  pub async fn handle_told_order_state
  (   arg: dsta::PlacedOrder
    , order_tracker: &mut std::collections::HashMap< u64, dr_bot0::OrderTrackerEntry>
    , our_own_task_sechuler_deal_with_finished_order : & tokio::sync::mpsc::Sender<u64>
  ) 
  { let dbgspt = "[DB0FTHTOS]. : ";
    log::debug!("ENTRY! dr_bot0::from_ttai::handle_told_order_state - {}order update", dbgspt);
    match order_tracker.get_mut(&arg.order_hash) 
    { Some(entry) => 
      { let ename = format!("{}", entry.request.bot_api_named_bot);
        log::debug!
        ( "{}Order update received for {}'s order hash {}, previous updates length: {}, order = \n{:#?}"
        , dbgspt, ename, arg.order_hash, entry.updates.len(), arg
        );
        entry.updates.push(arg.clone());
        entry.last_update_time = Some(tokio::time::Instant::now());
        log::trace!
        ( "{} Entering loop for Updating sum_order {:#?}"
        , dbgspt                         , entry.sum_all
        );
        for update_leg in &arg.legs_fills // order_legs 
        { log::trace!("Updating sum_all with leg; ");
          if let Some(sum_leg) = entry.sum_all.order_legs.iter_mut().find
          ( |l| l.buy_what.order_name() == update_leg.buy_what.order_name() )
          { let prev_qty = sum_leg.quantity;
            let prev_remaining = sum_leg.remaining;
            let filled = update_leg.quantity; // DONT FUCKING CHANGE THIS. THE QUANITY IS THE FILLED FOR THIS UPDATE
            sum_leg.quantity += filled; // THE SUM ADDS THE UPDATE QUANITY
            let req_qty = if let Some(ordent) = entry.request.bot_order_request.order_legs.iter().find
            ( |l| l.buy_what.order_name() == sum_leg.buy_what.order_name() )
            { ordent.quantity
            }
            else
            { let _ = entry.request.tell_bot_it_sucks.send
              ( format!("log::error(recieved an illegal update leg for an order)")
              ).await; 
              log::debug!("EXIT! - {}", dbgspt);
              return;
            }
            ;

            let epsilon = 1e-10;
            let computed_remaining = req_qty - sum_leg.quantity;
            sum_leg.remaining = computed_remaining;
            if sum_leg.remaining - update_leg.remaining > epsilon
            { log::error!("{}Remaining mismatch for leg {:#?}: computed {}, update {}, hash {}",  
                dbgspt, sum_leg, computed_remaining, update_leg.remaining, arg.order_hash);
              log::warn!("Remainin mismatch error isn't processed by bot0 logic at all, but should idk, set the wrong one right?..");
            }

            log::debug!
            ( "{}Leg {:#?} updated in sum_all: quantity {} -> {}, remaining {} -> {}, bot {:#?}"
              , dbgspt, sum_leg
              , prev_qty, sum_leg.quantity
              , prev_remaining, sum_leg.remaining
              , entry.request.bot_api_order_bot
              )
            ;
            
            if sum_leg.quantity > req_qty 
            { log::error!("{}Overfilled leg {:#?}: filled {}, requested {}, hash {}",  
                dbgspt, sum_leg, sum_leg.quantity, req_qty, arg.order_hash);
              let _ = entry.request.tell_bot_it_sucks.send(
                format!("log::error(Overfilled leg {:#?}, hash {})", sum_leg, arg.order_hash)).await; // FIXED: Added .await
              log::debug!("EXIT! - {}", dbgspt);
              return;
            }
          } 
          else 
          { log::error!("{}Unknown leg {:#?} in update for hash {}", dbgspt, update_leg, arg.order_hash);
            let _ = entry.request.tell_bot_it_sucks.send(
              format!("log::error(Unknown leg {:#?} in update, hash {})", update_leg, arg.order_hash)).await; 
            log::debug!("EXIT! - {}", dbgspt);
            return;
          }
        } // Did all the legs

        // This will tell us what to do for Done orders that aren't full
        let is_full_fill = entry.sum_all.order_legs.iter().zip
        ( entry.request.bot_order_request.order_legs.iter()).all
        ( |(sum_leg, req_leg)| { sum_leg.quantity >= req_leg.quantity }
        );
        if is_full_fill || ( arg.order_fill == dsta::OrderFill::Done )
        { log::info!("Detected a finished order for {}!", ename);
          // If it is done, and it is not full WE MUST WAIT for at most 
          // the timeout before finaizling anything, or at least 1 second
          // , in case there was a done order update bfore all the updates

          // Compute delay before signaling self to handle finished order
          // include the timeout delat in the time delay calculation
          // but if it is marked as done, its probably actually just done.
          let time_delay: u64 = if is_full_fill 
          { 0
          } else 
          { 1 + 
            { if let Some(sinstant) = entry.start_time 
              { if let Some(finstant) = entry.request.time_until_cancel 
                { // Compute duration difference safely, clamp to zero if negative
                  let diff = finstant.saturating_duration_since(sinstant);
                  diff.as_secs()
                } 
                else { 0 }
              } else { 0 }
            }
          };

          { let copydat = arg.order_hash;
            let copymsg = our_own_task_sechuler_deal_with_finished_order.clone(); 
            log::debug!("Spawing task to send signal to self to deal with finished order ");
            tokio::spawn
            ( async move
              { let _ = tokio::time::sleep(std::time::Duration::from_secs(time_delay)).await;
                let _ = copymsg.send(copydat).await;
              }
            );
          }
        }
        log::debug!("EXIT! - {}", dbgspt);
      }
      None => 
      { log::error!("handle_told_order_state {} Unknown order hash {};\n{}"
        , dbgspt
        , arg.order_hash
        , format!("This error is caused by either a non-robo order\n{}{}{:#?}"
          , ", or a older order marked done (canceled?) being removed after"
          , " waiting for 1_s ~ timeout_s for the update we just got: \n"
          , arg 
          )
        );
        // let _ = our_dr_r_ask_robo.tell_bot_it_sucks.send(
        //   format!("log::error(Unknown order hash {})", arg.order_hash)).await; // FIXED: Added .await
        log::debug!("EXIT! - {}", dbgspt);
        return;
      }
    }
  }
  
  pub async fn handle_ttai_account
  (   account: dsta::AccountUpdate
    , call_count: &usize
    , our_dr_r_ask_robo: &DrRoboApi
  ) 
  { let dbgspt = "[DB0FTHTA]. : ";
    log::debug!("ENTRY! dr_bot0::from_ttai::handle_ttai_account - {}Account update", dbgspt);
    log::debug!("{}account.bot_index == 1 => TRUE => Update allocation", dbgspt);
    
    if *call_count == 0
    { log::info!(
        r#"
        // This is the first account update to our bot 0 cash.
        // This is only called once at startup:
        // Dr ROBO makes bot 0 with empty account.
        // then this gets and handle positions gets called
        // So what we do is just use magic to 
        // give ourselves everything in the account update
        "#
      );
      let _ = our_dr_r_ask_robo.tell_dr_swap_info.send
      ( ( 0
        , crate::BotSwaper 
          { source_offering: None
          , target_released: Some
            ( crate::SwapWat
              { who : format!("{}", crate::glossary::MAGIC_BOT0_NAME)
              , wat : dsta::PositionsList { positions: std::collections::HashMap::new() }
              , how : account.long_usd
              }
            )
          }
        )
      ).await;
    }
    if *call_count % 10 == 0
    { log::info!("Another 10 Account updates! {:#?}", account);
    }
    else
    { log::warn!("{}WARNING! handle_ttai_account is not processed after the first update, since we assume DR R will handle future account changes.", dbgspt);
    }
    log::debug!("EXIT! - {}", dbgspt);
  }
}

mod from_robo
{
  use crate::*;
  
  pub async fn handle_told_dryrun_inputs
  (   arg: dr_bot0::DryRunRequest
    , ttai_kernel: &DrTtaiApi
    , bot_api_order_bot: &BotApi
  ) 
  { let dbgspt = "[DB0FBHTDI] : ";
    log::debug!("ENTRY! dr_bot0::from_robo::handle_told_dryrun_inputs - {} Dry run request", dbgspt);
    
    // Make dry run request to TTAI
    let (replywait_tx, replywait_rx) = tokio::sync::oneshot::channel();
    match ttai_kernel.tell_dr_order_dryrun.send
    ( ( arg.bot_order_request
      , replywait_tx
      )
    ).await 
    { Ok(()) =>
      { log::debug!("{} Dry run request sent to TTAI", dbgspt);
      }
      Err(e) =>
      { log::error!("{} Dry run submission FAILED! kernel api tell_dr_order_dryrun channel closed? Err = {:#?}", dbgspt, e);
        let _ = bot_api_order_bot.robot_bota_sender.tell_bot_it_sucks.send
        ( format!("log::error(dry run submission FAILED! kernel api tell_dr_order_dryrun channel closed? Err ={:#?})", e)
        ).await;
        log::debug!("EXIT! - {}", dbgspt);
        return;
      }
    }
    
    let res = match replywait_rx.await 
    { Ok(res) => res,
      Err(e) => 
      { log::error!("{} Dry run FAILED! Oneshot closed! Err={:#?}", dbgspt, e);
        let _ = bot_api_order_bot.robot_bota_sender.tell_bot_it_sucks
          .send(format!("Dry run FAILED! Oneshot closed? Err={:#?}", e))
          .await;
        let _ = arg.reply_tx.send(Err(e.into()));
        return;
      }
    };

    match res 
    { Ok(total_fees) => 
      { log::info!("{} Dry run successful: total fees = ${:.2}", dbgspt, total_fees);
        let _ = bot_api_order_bot.robot_bota_sender.tell_bot_it_sucks
          .send(format!("log::info(Dry run successful, fees: ${:.2})", total_fees))
          .await;

          // send the result back to dr_robo
          let _ = arg.reply_tx.send(Ok(total_fees));
      }
      Err(e) => 
      { log::error!("{} Dry run failed: {:#?}", dbgspt, e);
        let _ = bot_api_order_bot.robot_bota_sender.tell_bot_it_sucks
            .send(format!("log::error(Dry run failed: {:#?})", e))
            .await;
        let _ = arg.reply_tx.send(Err(e.into()));
      }
    }

    log::debug!("EXIT! - {}", dbgspt);
  }
  
  pub async fn handle_told_cancel_inputs
  (   arg: dr_bot0::CancelOrderRequest
    , ttai_kernel: &DrTtaiApi
    , order_tracker: &mut std::collections::HashMap<u64, crate::dr_bot0::OrderTrackerEntry>
    , bot_api_order_bot: &BotApi
  ) 
  { let dbgspt = "[DB0FBHTCI]. : ";
    log::debug!("ENTRY! dr_bot0::from_robo::handle_told_cancel_inputs - {}Cancel order request", dbgspt);
    
    let order_hash = arg.order_hash;
    
    // Check if order exists in tracker
    if !order_tracker.contains_key(&order_hash) 
    { log::error!("{} Cannot cancel unknown order hash {}", dbgspt, order_hash);
      let _ = bot_api_order_bot.robot_bota_sender.tell_bot_it_sucks.send
      ( format!("log::error(Cannot cancel unknown order hash {})", order_hash)
      ).await;
      log::debug!("EXIT! - {}", dbgspt);
      return;
    }
    
    // Make cancel request to TTAI
    let (replywait_tx, replywait_rx) = tokio::sync::oneshot::channel();
    match ttai_kernel.tell_dr_order_cancel.send
    ( ( order_hash
      , replywait_tx
      )
    ).await 
    { Ok(()) =>
      { log::debug!("{} Cancel request sent to TTAI for order {}", dbgspt, order_hash);
      }
      Err(e) =>
      { log::error!("{} Cancel submission FAILED! kernel api tell_dr_order_cancel channel closed? Err = {:#?}", dbgspt, e);
        let _ = bot_api_order_bot.robot_bota_sender.tell_bot_it_sucks.send
        ( format!("log::error(cancel submission FAILED! kernel api tell_dr_order_cancel channel closed? Err ={:#?})", e)
        ).await;
        log::debug!("EXIT! - {}", dbgspt);
        return;
      }
    }
    
    let res = match replywait_rx.await
    { Ok(res) => 
      { log::debug!("{} Cancel oneshot reply received! res={:#?}", dbgspt, res); 
        res
      }
      Err(e) => 
      { log::error!("{} Cancel FAILED! Oneshot channel closed! Err ={:#?})", dbgspt, e);
        let _ = bot_api_order_bot.robot_bota_sender.tell_bot_it_sucks.send
        ( format!("Cancel FAILED! Oneshot channel closed? Err ={:#?})", e)
        ).await;
        log::debug!("EXIT! - {}", dbgspt);
        return;
      }
    };


    match res 
    { Ok(success) => 
      { if success 
        { log::info!("{} Order {} cancelled successfully", dbgspt, order_hash);
          // Mark as cancelled
          if let Some(entry) = order_tracker.get_mut(&order_hash) 
          { entry.is_cancelled = true;
          }
          let _ = bot_api_order_bot.robot_bota_sender.tell_bot_it_sucks.send
          ( format!("log::info(Order {} cancelled successfully)", order_hash)
          ).await;
          let _ = arg.reply_tx.send(Ok(true));
        } 
        else 
        { log::warn!("{} Order {} cancel returned false", dbgspt, order_hash);
          let _ = bot_api_order_bot.robot_bota_sender.tell_bot_it_sucks.send
          ( format!("log::warn(Order {} cancel returned false)", order_hash)
          ).await;
          let _ = arg.reply_tx.send(Ok(false));
        }
      }
      Err(e) => 
      { log::error!("{} Cancel failed: {:#?}", dbgspt, e);
        let _ = bot_api_order_bot.robot_bota_sender.tell_bot_it_sucks.send
        ( format!("log::error(Cancel failed for order {}: {:#?})", order_hash, e)
        ).await;
        let _ = arg.reply_tx.send(Err(e.into()));
      }
    }
    
    log::debug!("EXIT! - {}", dbgspt);
  }
  
  // ROBO / BOTA verified order is legal for the requestee / bot0
  pub async fn handle_told_order_inputs
  (   arg: dr_bot0::NewOrderRequest
    , ttai_kernel: &DrTtaiApi
    , order_tracker: &mut std::collections::HashMap<u64, crate::dr_bot0::OrderTrackerEntry>
    , bot_api_order_bot: &BotApi
  ) 
  { let dbgspt = "[DB0FBHTOI]. : ";
    log::debug!("ENTRY! dr_bot0::from_bota::handle_told_order_inputs - {}New order submission", dbgspt);
    // Bot 0 requires all orders to be limit orders:
    let order = arg.bot_order_request.clone();
    if order.order_type != dsta::MarketOrLimit::Limit 
    { log::debug!("{}is_limit = {} => FALSE => Error", dbgspt, false);
      log::error!("{}Market order discarded for bot/order {:#?}"
      , dbgspt, arg.bot_api_order_bot);
      let _ = bot_api_order_bot.robot_bota_sender.tell_bot_it_sucks.send
      ( format!("Market order discarded, bot/order {:#?}", arg.bot_api_order_bot)
      ).await;
      log::debug!("EXIT! - {}", dbgspt);
      return;
    }
    let is_limit = order.order_type == dsta::MarketOrLimit::Limit;
    let action = order.order_legs.first().map(|leg| leg.action.clone()).unwrap_or(dsta::BuyOrSell::BuyToOpen);
    if !is_limit 
    { log::error!("{}Market order for {:#?} discarded for bot {:#?}", dbgspt, action, arg.bot_api_order_bot);
      let _ = bot_api_order_bot.robot_bota_sender.tell_bot_it_sucks.send(
        format!("log::error(Market order for {:#?} discarded, bot {:#?})", action, arg.bot_api_order_bot)).await;
      log::debug!("EXIT! - {}", dbgspt);
      return;
    }
    else
    { log::debug!("{}is_limit = {} => TRUE => Proceed", dbgspt, is_limit);
    }

    // Uh, this is a NewOrderReuqest
    let (replywait_tx,replywait_rx) = tokio::sync::oneshot::channel();
    match ttai_kernel.tell_dr_order_submit.send
    ( ( order
      , replywait_tx
      )
    ).await 
    { Ok(()) =>
      { log::debug!("order submission sent!");
      }
      Err(e)=>
      { log::error!("order submission FAILED! kernel api tell_dr_order_submit channel closed? Err = {:#?}",e);
        let _ = bot_api_order_bot.robot_bota_sender.tell_bot_it_sucks.send
        ( format!("log::error(order submission FAILED! kernel api tell_dr_order_submit channel closed? Err ={:#?})", e)
        ).await;
        log::debug!("EXIT! - {}", dbgspt);
        return;
      }
    }
    let res = match replywait_rx.await
    { Ok(res) => 
      { log::debug!("{} Oneshot channel replys for tell_dr_order_submit! res={:#?}", dbgspt, res); 
        res
      }
      , Err(e) => 
      { log::error!("order submission FAILED! Oneshot channel closed! Err ={:#?})", e);
        let _ = bot_api_order_bot.robot_bota_sender.tell_bot_it_sucks.send
        ( format!("order submission FAILED! Oneshot channel closed? Err ={:#?})", e)
        ).await;
        log::debug!("EXIT! - {}", dbgspt);
        return;
      }
    };

    match res // TODO: also need a transation cost.
    {  // ok, err, w/e
      Ok(dsta::PushOrderResult::ValidID { new_id }) => 
      { if new_id == 0
        { log::warn!("{}New order submitted: hash {}, bot {:#?}, returned new_id == 0, BITCOIN; bot0 doesn't process yet", dbgspt, new_id, arg.bot_api_order_bot);
          //panic!("NOTE: i think if dr_bot0 new_id is zero, it is also an error ? is zero possible?");
        

        }
        else if order_tracker.contains_key(&new_id) 
        { log::debug!("{}order_tracker.contains_key({}) => TRUE => Error", dbgspt, new_id);
          log::error!("{}Duplicate order hash {}", dbgspt, new_id);
          let _ = bot_api_order_bot.robot_bota_sender.tell_bot_it_sucks.send(
            format!("log::error(Duplicate order hash {})", new_id)).await;
          log::debug!("EXIT! - {}", dbgspt);
          return;
        }
        else
        { log::debug!("{}order_tracker.contains_key({}) => FALSE => Proceed", dbgspt, new_id);
        }
        log::info!("{}New order submitted: hash {}, bot {:#?}", dbgspt, new_id
        , arg.bot_api_order_bot);

        // MOST IMPORTANT part of the thing.
        order_tracker.insert
        ( new_id
        , crate::dr_bot0::OrderTrackerEntry::new(arg, new_id)
        );
        
        let _ = bot_api_order_bot.robot_bota_sender.tell_bot_it_sucks.send(
          format!("log::info(Order submitted: hash {})", new_id)).await;
      }
      Ok(dsta::PushOrderResult::FailedID { new_id }) => 
      { log::error!("{}Order submission failed: hash {}", dbgspt, new_id);
        let _ = bot_api_order_bot.robot_bota_sender.tell_bot_it_sucks.send(
          format!("log::error(Order submission failed: hash {})", new_id)).await;
        log::debug!("EXIT! - {}", dbgspt);
        return;
      }
      Err(e) => 
      { log::error!("{}TTAI kernel error: {:#?}", dbgspt, e);
        let _ = bot_api_order_bot.robot_bota_sender.tell_bot_it_sucks.send(
          format!("log::error(TTAI kernel error: {:#?})", e)).await;
        log::debug!("EXIT! - {}", dbgspt);
        return;
      }
    }
    log::debug!("EXIT! - {}", dbgspt);
  }
}

mod from_self
{ use crate::dr_bot0::*;
  /// Poll TTAI for live orders and update tracked orders accordingly
  pub async fn handle_order_poll_tick
  ( order_tracker: &mut std::collections::HashMap<u64, OrderTrackerEntry>
  , dr_ttai_api: &DrTtaiApi
  , our_own_task_sechuler_deal_with_finished_order : &tokio::sync::mpsc::Sender<u64>
  )
  { let dbgspt = "[DB0FTHOPT]"; // "DR_BOT0_FROM_TTAI_HANDLE_ORDER_POLL_TICK"
    log::debug!("{} [ENTRY] - Polling for live orders ({} tracked)", dbgspt, order_tracker.len());
    
    // Create oneshot channel for the poll request
    let (poll_reply_tx, poll_reply_rx) = tokio::sync::oneshot::channel::
    < Result<Vec<dsta::PlacedOrder>, Box<dyn std::error::Error + Send + Sync>> 
    >
    ();
    

    // Send poll request to TTAI
    if let Err(e) = dr_ttai_api.tell_dr_order_status.send(poll_reply_tx).await 
    { log::error!("{} Failed to send poll request to TTAI: {:?}", dbgspt, e);
      return;
    }
    
    // Wait for response with timeout
    let live_orders = match tokio::time::timeout
    ( std::time::Duration::from_secs(5)
    , poll_reply_rx
    ).await
    { Ok(Ok(Ok(orders))) => orders,
      Ok(Ok(Err(e))) => 
      { log::error!("{} TTAI poll request failed: {:?}", dbgspt, e);
        return;
      }
      Ok(Err(_)) => 
      { log::error!("{} Poll reply channel closed unexpectedly", dbgspt);
        return;
      }
      Err(_) => 
      { log::warn!("{} Poll request timed out", dbgspt);
        return;
      }
    };
    
    log::debug!("{} Received {} live orders from TTAI", dbgspt, live_orders.len());
    
    // Process each live order
    for live_order in live_orders 
    { let order_hash = live_order.order_hash;
      
      // Only process orders we're tracking
      if order_tracker.contains_key(&order_hash) 
      { log::trace!("{} Processing polled order {} (status: {:?})", 
          dbgspt, order_hash, live_order.order_fill);
        
        crate::dr_bot0::from_ttai::handle_told_order_state
        ( live_order
        , order_tracker
        , our_own_task_sechuler_deal_with_finished_order
        ).await;
      } 
      else 
      { log::trace!("{} Ignoring untracked order {} from poll", dbgspt, order_hash);
      }
    }
    
    log::debug!("{} [EXIT] - Poll processing complete", dbgspt);
  }
}



async fn f_run_from_api
( mut told_bot_to_trade_api : tokio::sync::mpsc::Receiver<dsta::Ticker>
, mut told_bot_to_check_api : tokio::sync::mpsc::Receiver<Allocation>
//, mut told_bot_it_sucks_api : tokio::sync::mpsc::Receiver<String>
) -> ()
{ let dbgspt = format!("[DRB0FR] :");
  log::debug!("{} [ENTRY] - dr_bot0.rs::f_run_from_api ~ this is for debug (turn off in release?) ", dbgspt);
  loop
  { tokio::select! 
    { res = told_bot_to_trade_api.recv() => 
      { if let Some(thing) = res 
        { log::error!("BOT ZERO IS NOT SUPPOSED TO GET TICKERS; ticker = {:#?}", thing);
        } 
        else 
        { log::error!("{} told_bot_to_trade_api channel closed", dbgspt);
          break;
        }
      },
      res = told_bot_to_check_api.recv() => 
      { if let Some(thing) = res 
        { if thing.bot_mind == crate::BotMindState::Seppuku
          { log::error!("BOT ZERO FATAL STATE, f_run_from_api told to seppuku");
            break;
          }
        } 
        else 
        { log::error!("{} told_bot_to_trade_api channel closed", dbgspt);
          break;
        }
      },
    }
  }
  log::debug!("{} dr_bot0.rs, f_run_from_api function ending", dbgspt);
}

// Bot 0: Order State Management & TTAI talk
pub(crate) async fn f_run_main_dr_bot0_with_mode
(
    ttai_mode: dr_bot0_ttai_bridge::TtaiMode, // NEW PARAMETER
    
    // All existing parameters
    mut told_order_inputs: tokio::sync::mpsc::Receiver<dr_bot0::NewOrderRequest>,
    mut told_cancel_inputs: tokio::sync::mpsc::Receiver<dr_bot0::CancelOrderRequest>,
    mut told_dryrun_inputs: tokio::sync::mpsc::Receiver<dr_bot0::DryRunRequest>,
    
    our_dr_r_ask_robo: DrRoboApi,
    bot_api_order_bot: BotApi,
    
    mut pipe_dr_ttai_add_ttai_tik: (
        tokio::sync::mpsc::Sender<(String, u64, tokio::sync::mpsc::Sender<dsta::Ticker>, tokio::sync::mpsc::Sender<Option<dsta::PushTickerResult>>)>,
        tokio::sync::mpsc::Receiver<(String, u64, tokio::sync::mpsc::Sender<dsta::Ticker>, tokio::sync::mpsc::Sender<Option<dsta::PushTickerResult>>)>,
    ),
    mut pipe_dr_ttai_pop_ttai_tik: (
        tokio::sync::mpsc::Sender<(String, u64)>,
        tokio::sync::mpsc::Receiver<(String, u64)>,
    ),
    
    told_bot_to_trade_api: tokio::sync::mpsc::Receiver<dsta::Ticker>,
    told_bot_to_check_api: tokio::sync::mpsc::Receiver<crate::Allocation>,
    mut told_bot_it_sucks_api: tokio::sync::mpsc::Receiver<String>,
    
    tell_dr_a_snively: tokio::sync::mpsc::Sender<(usize, dsta::Snively)>,
) 
{ let dbgspt = "[BOTZERO_MAIN]";
  log::info!("{} Starting dr_bot0 with TTAI mode: {:?}", dbgspt, 
  if matches!(ttai_mode, dr_bot0_ttai_bridge::TtaiMode::Mock(_)) { "MOCK" } else { "REAL" });

  tokio::spawn
  ( async move 
    { crate::dr_bot0::f_run_from_api(told_bot_to_trade_api, told_bot_to_check_api).await;
    }
  );

  let reset_ttai = dsta::UsaMarketTimes::ResetTtai;
  let reset_resog = format!("DR BOT ZERO IS RESETTING THE TTAI CONNECTION");

  'bot0_loop_run_ttai: loop 
  { log::info!("{} ENTRY bot0_loop_run_ttai => Spawning TTAI task...", dbgspt);

    // Create all the standard channels
    let (tell_dr_ttai_order_submit, told_dr_ttai_order_submit) = 
        tokio::sync::mpsc::channel::<(dsta::Order, tokio::sync::oneshot::Sender<Result<dsta::PushOrderResult, Box<dyn std::error::Error + Send + Sync>>>)>(100);
    
    let (tell_dr_ttai_order_cancel, told_dr_ttai_order_cancel) = 
        tokio::sync::mpsc::channel::<(u64, tokio::sync::oneshot::Sender<Result<bool, Box<dyn std::error::Error + Send + Sync>>>)>(100);
    
    let (tell_dr_ttai_order_dryrun, told_dr_ttai_order_dryrun) = 
        tokio::sync::mpsc::channel::<(dsta::Order, tokio::sync::oneshot::Sender<Result<f64, Box<dyn std::error::Error + Send + Sync>>>)>(100);
    
    let (tell_dr_ttai_reset, told_dr_ttai_reset) = tokio::sync::mpsc::channel::<String>(10);

    // Bot 0 Public interactions
    let (tell_dr_bot0_final_orders, mut told_dr_bot0_final_orders) = tokio::sync::mpsc::channel::<u64>(10);
    let (tell_dr_bot0_ttai_account, mut told_dr_bot0_ttai_account) = tokio::sync::mpsc::channel::<dsta::AccountUpdate>(100);
    
    // special pipe to the bot0 ( so we can queue when disconnected)
    let pipe_dr_bot0_add_ttai_tik = tokio::sync::mpsc::channel::<(String, u64, tokio::sync::mpsc::Sender<dsta::Ticker>, tokio::sync::mpsc::Sender<Option<dsta::PushTickerResult>>)>(100);
    let pipe_dr_bot0_pop_ttai_tik = tokio::sync::mpsc::channel::<(String, u64)>(100);
    

    let (tell_dr_ttai_order_status, told_dr_ttai_order_status) = 
    tokio::sync::mpsc::channel::<tokio::sync::oneshot::Sender<Result<Vec<dsta::PlacedOrder>, Box<dyn std::error::Error + Send + Sync>>>>(10);


    let dr_ttai_api = crate::DrTtaiApi 
    { tell_dr_order_submit: tell_dr_ttai_order_submit,
      tell_dr_order_cancel: tell_dr_ttai_order_cancel,
      tell_dr_order_dryrun: tell_dr_ttai_order_dryrun,
      tell_dr_order_status: tell_dr_ttai_order_status
    };

    let clone_tell_dr_snively = tell_dr_a_snively.clone();
        
    // ═══════════════════════════════════════════════════════════
    // KEY CHANGE: Use the bridge to spawn either real or mock TTAI
    // ═══════════════════════════════════════════════════════════
    match ttai_mode
    { crate::dr_bot0_ttai_bridge::TtaiMode::Real => 
      { log::info!("[TTAI_BRIDGE] Spawning REAL TastyTrade API connection");
        tokio::spawn
        ( async move 
          { log::info!("[TTAI_BRIDGE] Spawned REAL TastyTrade API connection!");
            crate::dr_ttai::f_run_main_dr_ttai
            ( 

              pipe_dr_bot0_add_ttai_tik.1,
              pipe_dr_bot0_pop_ttai_tik.1,
              
              told_dr_ttai_order_submit,
              told_dr_ttai_order_cancel,
              told_dr_ttai_order_dryrun,
              told_dr_ttai_order_status, 

              tell_dr_bot0_ttai_account.clone(),
              told_dr_ttai_reset,
              clone_tell_dr_snively,
            ).await;
            log::error!("[TTAI_BRIDGE] Real TTAI task ended");
          }
        );
      }

      crate::dr_bot0_ttai_bridge::TtaiMode::Mock(ref config) => 
      { log::info!("[TTAI_BRIDGE] Spawning MOCK TastyTrade API (paper trading mode)");
        log::info!("[TTAI_BRIDGE] Mock config: {:#?}", config);
        let clonecfg = config.clone();
        tokio::spawn
        ( async move 
          { crate::dr_ttai_mock::f_run_main_mock_ttai(
                clonecfg,
                pipe_dr_bot0_add_ttai_tik.1,
                pipe_dr_bot0_pop_ttai_tik.1,
                told_dr_ttai_order_submit,
                told_dr_ttai_order_cancel,
                told_dr_ttai_order_dryrun,
                told_dr_ttai_order_status,

                tell_dr_bot0_ttai_account.clone(),
                told_dr_ttai_reset,
                clone_tell_dr_snively,
            ).await;
            log::error!("[TTAI_BRIDGE] Mock TTAI task ended");
          }
        );
      }
    }


    let mut order_tracker: std::collections::HashMap<u64, crate::dr_bot0::OrderTrackerEntry> = std::collections::HashMap::new();
    let mut order_history: std::collections::HashMap<u64, crate::dr_bot0::OrderTrackerEntry> = std::collections::HashMap::new();
    let mut metrics = crate::helper::LoopMetrics::new();
    let mut call_count_act = 0;

    let mut reset_reson = reset_resog.clone();

    log::info!("[BOTZERO] sending b0su: alerting dr_robo of completed startup; entering tokio select loop");    
    let _ = tell_dr_a_snively.send
    ( ( 0, dsta::Snively::SendLogNote(format!("b0su")) )
    ).await;

    let mut next_reset_time;
    let now = chrono::Utc::now();
    let target_time_today = reset_ttai.to_utc_today();
    next_reset_time = if target_time_today <= now
    { target_time_today + chrono::Duration::days(1)
    }
    else
    { dsta::UsaMarketTimes::ResetTtai.to_utc_today()
    };

    #[cfg(debug_assertions)]
    { log::warn!("Dr Bot 0 in debug setting next TTAI/TTRS TastyTrade API reset time to be in 10 minutes instead of {:#?}",next_reset_time);
      next_reset_time = chrono::Utc::now() + chrono::Duration::minutes(10);
    }
    
    loop 
    { let iteration_start = tokio::time::Instant::now();

      // === Calculate sleep/polling interval BASED ON PENDING ORDERS COUNT ===
      let pending_count = order_tracker.iter().filter(|(_, entry)| {
          !entry.is_cancelled && 
          entry.updates.last().map(|u| u.order_fill == dsta::OrderFill::Live).unwrap_or(true)
      }).count();

      let current_interval = if pending_count > 0 {
          std::time::Duration::from_millis(100)   // fast when orders are pending
      } else {
          std::time::Duration::from_secs(600)     // 10 minutes when idle
      };

      // Reset timer with the freshly calculated interval
      let mut order_poll_timer = tokio::time::interval(current_interval);
      order_poll_timer.reset();

      // Reset our 
      let reset_delay = next_reset_time
        .signed_duration_since(chrono::Utc::now())
        .to_std()
        .unwrap_or(std::time::Duration::ZERO);

      if reset_delay <= std::time::Duration::ZERO
      { // RESET TTRS
        log::info!("Dr BOT ZERO detected a restart flag, This is required to refresh expiring TastyTrade API access tokens");
        log::debug!("I do this reset after 10 minutes as a fuckyou test;");
        break; // inner loop, restart...
      }

      tokio::select! 
      { 
        // ORDER POLLING

        _ = order_poll_timer.tick() =>
        { from_self::handle_order_poll_tick
          ( &mut order_tracker
          , &dr_ttai_api
          , &tell_dr_bot0_final_orders
          ).await;
        }

        // RESET TTRS

        _ = tokio::time::sleep_until(tokio::time::Instant::now() + reset_delay) =>
        { log::info!("Dr BOT ZERO detected a restart flag, This is required to refresh expiring TastyTrade API access tokens");
          log::debug!("I do this reset after 10 minutes as a fuckyou test;");
          break; // inner loop, restart...
        }

        res = told_dr_bot0_final_orders.recv() =>
        { if let Some(uid) = res
          { let _ = from_ttai::deal_with_done_order
            ( uid
            , &mut order_tracker
            , &mut order_history
            , &our_dr_r_ask_robo
            ).await;
          }
        }
        res = told_dr_bot0_ttai_account.recv() => 
        { if let Some(account) = res 
          { let _ = from_ttai::handle_ttai_account
            ( account
            , &call_count_act
            , &our_dr_r_ask_robo
            ).await;
            call_count_act += 1;
          } 
          else 
          { log::error!("{}told_dr_bot0_ttai_account channel closed", dbgspt);
            reset_reson = format!("DR BOT ZERO told_dr_bot0_ttai_account channel closed ");
            break;
          }
        },

        // Our API in the system 
        res = told_order_inputs.recv() => 
        { if let Some(input) = res 
          { let _ = from_robo::handle_told_order_inputs
            ( input
            , &dr_ttai_api
            , &mut order_tracker
            , &bot_api_order_bot
            ).await;
          } 
          else 
          { log::error!("{} FATAL ERROR! BOT ZERO told_order_inputs channel closed", dbgspt);
            break 'bot0_loop_run_ttai;
          }
        },
        // NEW: Handle cancel order requests
        res = told_cancel_inputs.recv() => 
        { if let Some(input) = res 
          { let _ = from_robo::handle_told_cancel_inputs
            ( input
            , &dr_ttai_api
            , &mut order_tracker
            , &bot_api_order_bot
            ).await;
          } 
          else 
          { log::error!("{} FATAL ERROR! BOT ZERO told_cancel_inputs channel closed", dbgspt);
            break 'bot0_loop_run_ttai;
          }
        },
        // NEW: Handle dry run requests
        res = told_dryrun_inputs.recv() => 
        { if let Some(input) = res 
          { let _ = from_robo::handle_told_dryrun_inputs
            ( input
            , &dr_ttai_api
            , &bot_api_order_bot
            ).await;
          } 
          else 
          { log::error!("{} FATAL ERROR! BOT ZERO told_dryrun_inputs channel closed", dbgspt);
            break 'bot0_loop_run_ttai;
          }
        },
        res = told_bot_it_sucks_api.recv() => 
        { if let Some(thing) = res 
          { log::warn!("BOT ZERO IS NOT SUPPOSED TO GET TOLD IT SUCKS; thing = {:#?}", thing);
          } 
          else 
          { log::error!("{} ERROR! told_bot_it_sucks_api channel closed", dbgspt);
            break 'bot0_loop_run_ttai;
          }
        },
        res = pipe_dr_ttai_add_ttai_tik.1.recv() =>
        { if let Some(req) = res
          { if let Err(e) = pipe_dr_bot0_add_ttai_tik.0.send(req).await  // ✅ CORRECT
            { log::error!("Bot0 pipe to add tick at TTAI controller is borked!, {:#?}",e);
              break;
            }
          }
          else
          { log::error!("BOT ZERO FATIAL ERROR: tick add piper is closed!");
            break 'bot0_loop_run_ttai;
          }
        }
        res = pipe_dr_ttai_pop_ttai_tik.1.recv() =>
        { if let Some(req) = res
          { if let Err(e) =  pipe_dr_bot0_pop_ttai_tik.0.send(req).await
            { log::error!("Bot0 pipe to pop tick at TTAI controller is borked! - {:#?}",e);
              break;
            }
          }
          else
          { log::error!("BOT ZERO FATIAL ERROR: tick pop piper is closed!");
            break 'bot0_loop_run_ttai;
          }
        }
      }
      let elapsed_ms = iteration_start.elapsed().as_secs_f64() * 1000.0;
      metrics.record(elapsed_ms);
    }

    if reset_reson != reset_resog
    { let _ = bot_api_order_bot.robot_bota_sender.tell_bot_it_sucks.send
      ( format!("Dr Bot 0: Failed to live!, trying to restart")
      ).await; // 
    }
    
    let _ = tell_dr_ttai_reset.send(reset_reson).await;

    log::info!("Before DR BOT Zero resets, we wait 1 seconds for ttai shutdown and reset_resog");
    _ = tokio::time::sleep(std::time::Duration::from_secs(1)).await;
  }

  log::error!("BOT ZERO IS NOT SUPPOSED TO DIE!");

}

