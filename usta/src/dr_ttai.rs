
//dr_ttai.rs - With restored ticker distribution + Cancel Orders + Dry Run

pub enum TtaiTickCommand
{ 
  NewSub
  ( ( String // Ticker name
    , u64 // sub ID
    , tokio::sync::mpsc::Sender<dsta::Ticker> // sub sender
    )
  )
, DieSub
  ( ( String // Ticker name
    , u64 // Sub ID
    )
  ) 
}

// Ticker distribution task - converts StreamData to Ticker and distributes to subscribers
async fn f_run_tick
( mut ttai_channel_tick: tokio::sync::mpsc::Receiver<ttrs::StreamData>
, mut ttai_channel_info: tokio::sync::mpsc::Receiver<TtaiTickCommand>
, mut told_dr_ttai_reset: tokio::sync::mpsc::Receiver<String>
, tell_dr_ttai_reset: tokio::sync::mpsc::Sender<String>
)
{ let dbgspt = format!("[TTAI-TICK]");
  
  // Map of ticker name -> list of (subscriber_id, sender)
  let mut subz: std::collections::HashMap<String, Vec<(u64, tokio::sync::mpsc::Sender<dsta::Ticker>)>> 
    = std::collections::HashMap::new();
  let mut new_stream_data = Vec::new();
  
  loop 
  { tokio::select! 
    { // Receive StreamData from ttrs and distribute as Tickers
      res = ttai_channel_tick.recv_many(&mut new_stream_data, 100) => 
      { if res == 0 
        { log::error!("{} Tick channel closed, initiating shutdown", dbgspt);
          let _ = tell_dr_ttai_reset.send("Tick loop: Tick channel closed.".to_string()).await;
          break;
        }
        slog::trace!
        ( crate::glossary::TICK_SLOG
        , "{} Received {} stream data items for distribution"
        , dbgspt, res
        );
        for stream_data in new_stream_data.drain(..) 
        { // Convert StreamData to Ticker based on type
          let ticker_opt = match &stream_data 
          { ttrs::StreamData::Quote(q) => 
            { Some(dsta::convert::stream_quote_to_ticker(q))
            }
            ttrs::StreamData::Trade(t) => 
            { Some(dsta::convert::stream_trade_to_ticker(t))
            }
            ttrs::StreamData::Summary(s) => 
            { log::info!("{} dr_ttai Received Summary update, skipping ticker distribution {:#?}", dbgspt, s);
              None
            }
            ttrs::StreamData::Profile(p) => 
            { log::info!("{} dr_ttai Received profile update, skipping ticker distribution {:#?}", dbgspt, p);
              None
            }
            ttrs::StreamData::Greeks(p) => 
            { if p.theo_price.is_none()
              { log::warn!("{} dr_ttai Received greeks update without theo_price, skipping ticker distribution TODO: pass theo price for options! {:#?} ", dbgspt, p);
                None
              }
              else
              { log::info!("{} dr_ttai Received greeks update with theo_price ={:#?} ", dbgspt, p);
                Some(dsta::convert::stream_greek_to_ticker(p))
              }
            }
          };
          if let Some(ticker) = ticker_opt 
          { if let Some(subscribers) = subz.get_mut(&ticker.name) 
            { slog::trace!
              ( crate::glossary::TICK_SLOG
              , "{} dr_ttai Distributing ticker {} to {} subscribers",
                dbgspt,
                ticker.name,
                subscribers.len()
              );
              // We'll collect indices to remove in reverse order so we can safely remove them
              let mut to_remove = Vec::new();
              for (idx, (sub_id, sender)) in subscribers.iter_mut().enumerate() 
              { if let Err(e) = sender.send(ticker.clone()).await 
                { log::warn!
                  ( "{} Failed to send ticker {} to subscriber {} → removing it (error: {})",
                     dbgspt, ticker.name, sub_id, e
                  );
                  to_remove.push(idx);
                }
              }

              // Remove failed subscribers (in reverse order to preserve indices)
              for &idx in to_remove.iter().rev() 
              { subscribers.remove(idx);
              }

              // Clean up the entry if no subscribers remain
              if subscribers.is_empty() 
              { subz.remove(&ticker.name);
                log::debug!
                ( "{} No more live subscribers for {}, removed from map",
                  dbgspt,
                  ticker.name
                );
              }
            } 
            else 
            { log::trace!
              ("{} No subscribers for ticker {}, discarding",
                dbgspt,
                ticker.name
              );
              // TODO: remove the actual ttrs subscription when implemented  
            }
          }
        }
      }
      
      // Handle subscription commands
      res = ttai_channel_info.recv() => 
      { match res 
        { Some(TtaiTickCommand::NewSub((symbol, sub_id, sender))) => 
          { log::debug!("{} Adding/Updating subscriber {} for ticker {}", dbgspt, sub_id, symbol);
            let subscribers = subz.entry(symbol.clone()).or_insert_with(Vec::new);
            // Remove any previous entry with the same sub_id
            subscribers.retain(|(id, _)| *id != sub_id);
            // Always add the new (or updated) one
            subscribers.push((sub_id, sender));
            // log depending on whether it was an update or new
            if subscribers.len() > 1 || !subscribers.is_empty() && subscribers[0].0 == sub_id 
            { log::debug!("{} Updated subscriber {} for {}", dbgspt, sub_id, symbol);
            } else 
            { log::debug!("{} Added new subscriber {} for {}", dbgspt, sub_id, symbol);
            }
          }
          ,
          Some(TtaiTickCommand::DieSub((symbol, sub_id))) => 
          { if let Some(subscribers) = subz.get_mut(&symbol) 
            { subscribers.retain(|(id, _)| *id != sub_id);
              log::debug!("{} Removed subscriber {} for ticker {}", dbgspt, sub_id, symbol);
              if subscribers.is_empty() 
              { subz.remove(&symbol);
                log::debug!("{} No more subscribers for {}, removed from map", dbgspt, symbol);
              }
            } 
            else 
            { log::warn!("{} Attempted to remove subscriber {} for non-subscribed ticker {}", dbgspt, sub_id, symbol);
            }
          }
          ,
          None => 
          { log::debug!("{} Info channel closed, initiating shutdown", dbgspt);
            let _ = tell_dr_ttai_reset.send("Tick loop: Info channel closed.".to_string()).await;
            break;
          }
        }
      }
      
      // Handle reset signals
      res = told_dr_ttai_reset.recv() => 
      { match res 
        { Some(reason) => 
          { log::warn!("{} dr_ttai Received reset signal: {}", dbgspt, reason);
            let _ = tell_dr_ttai_reset.send(format!("Tick loop: Propagate reset - {}", reason)).await;
            break;
          }
          None => 
          { log::error!("{} Reset channel closed without reason, initiating shutdown", dbgspt);
            let _ = tell_dr_ttai_reset.send("Tick loop: Reset channel closed.".to_string()).await;
            break;
          }
        }
      }
    }
  }
  slog::debug!
  ( crate::glossary::TICK_SLOG
  , "{} Exiting f_run_tick, cleaning up resources"
  , dbgspt
  );
}

async fn _request_positions
( kernel : &ttrs::Thand
, acct_num : &String
, future_output: tokio::sync::mpsc::Sender<Option<dsta::PositionsList>>
, error_output: tokio::sync::mpsc::Sender<String>
) 
{ 
  tokio::select! 
  { res =  kernel.tell_tastytrade_to_get_positions(acct_num.clone()) =>
    { match res 
      { Ok(vec_ans) => 
        { log::trace!("dr_ttai request_positions returned successfully");
          let ans = dsta::convert::positions_to_positions_list(&vec_ans);
          let _ = future_output.send(Some(ans)).await;
        }
        Err(e) => 
        { let err_msg = format!("TTAII, request_positions: Error fetching positions: {:?}", e);
          log::error!("{}", err_msg);
          let _ = error_output.send(err_msg).await;
        }
      }
    }
    _ = tokio::time::sleep(std::time::Duration::from_secs(15)) => 
    { let err_msg = "TTAII, request_positions: timeout".to_string();
      log::info!("{}", err_msg);
      let _ = error_output.send(err_msg).await;
      let _ = future_output.send(None).await;
    }
  }
}

pub async fn request_todays_orders
( kernel : &ttrs::Thand
, acct_num : &String
, future_output: tokio::sync::mpsc::Sender<Option<Vec<dsta::PlacedOrder>>>
, error_output: tokio::sync::mpsc::Sender<String>
) 
{ tokio::select! 
  { res = kernel.tell_tastytrade_to_get_live_orders(acct_num.clone()) => 
    { match res 
      { Ok(vec_ans) => 
        { log::trace!("dr_ttai request_todays_orders returned successfully");
          let ans = dsta::convert::order_responses_to_placed_orders(&vec_ans);
          let _ = future_output.send(Some(ans)).await;
        }
        Err(e) => 
        { let err_msg = format!("TTAII, request_todays_orders: Error fetching orders: {:?}", e);
          log::error!("{}", err_msg);
          let _ = error_output.send(err_msg).await;
        }
      }
    }
    _ = tokio::time::sleep(std::time::Duration::from_secs(3)) => 
    { let err_msg = "TTAII, request_todays_orders: timeout".to_string();
      log::info!("{}", err_msg);
      let _ = error_output.send(err_msg).await;
      let _ = future_output.send(None).await;
    }
  }
}

fn load_creds() -> serde_json::Value 
{ let creds_str = include_str!("../../ttrs/tests/test-creds.json");
  let res: serde_json::Value = serde_json::from_str(creds_str).expect("Invalid test_creds.json");
  log::info!("Loaded credentials!");
  
  if res["client_id"].as_str().map(|s| s.is_empty()).unwrap_or(true) {
    panic!("client_id is missing or empty in test-creds.json");
  }
  if res["client_secret"].as_str().map(|s| s.is_empty()).unwrap_or(true) {
    panic!("client_secret is missing or empty in test-creds.json");
  }
  if res["refresh_token"].as_str().map(|s| s.is_empty()).unwrap_or(true) {
    panic!("refresh_token is missing or empty in test-creds.json");
  }

  log::info!("All required credentials present");
  res
}

pub async fn f_run_main_dr_ttai
( 
    // From dr_robo via dr_bot0
    mut told_dr_add_ttai_tik : tokio::sync::mpsc::Receiver<
      ( String
      , u64
      , tokio::sync::mpsc::Sender<dsta::Ticker>
      , tokio::sync::mpsc::Sender<Option<dsta::PushTickerResult>>
      )>
  , mut told_dr_pop_ttai_tik : tokio::sync::mpsc::Receiver<(String, u64)>
  
    // From dr_bot0 - order submission
  , mut told_dr_order_submit : tokio::sync::mpsc::Receiver<
      ( dsta::Order
      , tokio::sync::oneshot::Sender<Result<dsta::PushOrderResult, Box<dyn std::error::Error + Send + Sync>>>
      )>
  
    // From dr_bot0 - : order cancellation
  , mut told_dr_order_cancel : tokio::sync::mpsc::Receiver<
      ( u64  // order_hash to cancel
      , tokio::sync::oneshot::Sender<Result<bool, Box<dyn std::error::Error + Send + Sync>>>
      )>
  
    // From dr_bot0 - : dry run order
  , mut told_dr_order_dryrun : tokio::sync::mpsc::Receiver<
      ( dsta::Order
      , tokio::sync::oneshot::Sender<Result<f64, Box<dyn std::error::Error + Send + Sync>>>  // Returns total fees
      )>

    // From dr_bot0 - order status polling
    , mut told_dr_order_status : tokio::sync::mpsc::Receiver
      <tokio::sync::oneshot::Sender<Result<Vec<dsta::PlacedOrder>, Box<dyn std::error::Error + Send + Sync>>>
    >
  
  
    // To dr_bot0
  , tell_dr_bot0_account_update: tokio::sync::mpsc::Sender<dsta::AccountUpdate>
  
    // Reset control
  , mut told_dr_ttai_reset: tokio::sync::mpsc::Receiver<String>
  , tell_dr_a_snively : tokio::sync::mpsc::Sender<(usize, dsta::Snively)>
) 
{
  let dbgspt = format!("[DR_TTAI_MAIN] :");
  log::debug!("{} [ENTRY] - f_run_main_dr_ttai", dbgspt);
  
  // Create channels for ticker distribution
  let (ttai_channel_tick_tx, ttai_channel_tick_rx) = tokio::sync::mpsc::channel::<ttrs::StreamData>(1000);
  let (ttai_channel_info_tx, ttai_channel_info_rx) = tokio::sync::mpsc::channel::<TtaiTickCommand>(100);
  
  let (ttai_tick_reset_tx, ttai_tick_reset_rx) = tokio::sync::mpsc::channel::<String>(10);

  log::debug!("{} Loading credentials...", dbgspt);
  let creds = load_creds();
  let conn_info = ttrs::ConnectionInfo {
      base_url: "https://api.tastyworks.com".to_string(),
      oauth: (
          creds["client_id"].as_str().unwrap_or("").to_string(),
          creds["client_secret"].as_str().unwrap_or("").to_string(),
          creds["refresh_token"].as_str().unwrap_or("").to_string(),
      ),
  };

  log::debug!("{} Creating ttrs core...", dbgspt);
  let (error_tx, mut from_ttrs_error_rx) = tokio::sync::mpsc::unbounded_channel();
  let (tell_ttrs_to_shutdown_tx, shutdown_rx) = tokio::sync::mpsc::channel::<String>(1);
  let config = ttrs::CoreConfig { error_tx, shutdown_rx };
  
  let (thand, tfoot) = ttrs::make_core_api(10);
  let _conn_handle = tokio::spawn(ttrs::fn_run_core(tfoot, conn_info.clone(), config));
  
  // Wait for core to initialize
  tokio::time::sleep(std::time::Duration::from_millis(200)).await;

  log::debug!("{} Getting account info...", dbgspt);
  let acct = match thand.tell_tastytrade_to_get_accounts_info().await {
    Ok(ok) => {
      if !ok.items.is_empty() {
        ok.items[1].account.clone()
      } else {
        log::error!("{} couldn't get accounts info - empty items", dbgspt);
        let _ = tell_dr_a_snively.send((0, dsta::Snively::SendLogNote(format!("dtsd ERROR")))).await;
        return; 
      }
    }
    Err(e) => {
      log::error!("{} couldn't get accounts info - error {:#?}", e, dbgspt);
      let _ = tell_dr_a_snively.send((0, dsta::Snively::SendLogNote(format!("dtsd ERROR")))).await;
      return; 
    }
  };

  log::debug!("{} STARTUP Account #: {}", dbgspt, acct.account_number);
  let acct_num = acct.account_number;
  
  log::trace!("{} Fetching initial balance...", dbgspt);
  let balance = thand.tell_tastytrade_to_get_balances(acct_num.clone()).await
      .expect("Failed to get balance");
  let account_update = dsta::convert::balance_to_account_update(&balance);
  log::debug!("{} FIRST Account update (sending to bot0): {:#?}", dbgspt, account_update);
  if tell_dr_bot0_account_update.send(account_update).await.is_err() 
  { let _ = tell_dr_a_snively.send((0, dsta::Snively::SendLogNote(format!("FAILED {}", crate::glossary::MAGIC_DTTAI_STARTUP_OK)))).await;
  }
  
  log::info!("{} Spawning tick distribution task...", dbgspt);
  let clone_ttai_tick_reset_tx = ttai_tick_reset_tx.clone();
  tokio::spawn(async move {
    f_run_tick(
      ttai_channel_tick_rx,
      ttai_channel_info_rx,
      ttai_tick_reset_rx,
      clone_ttai_tick_reset_tx,
    ).await;
    log::error!("[TTAI-TICK] Tick distribution task ended!");
  });


  log::info!("{} Alerting dr_robo of completed startup", dbgspt);

  let _ = tell_dr_a_snively.send((0, dsta::Snively::SendLogNote(format!("{}",crate::glossary::MAGIC_DTTAI_STARTUP_OK)))).await;

  #[derive(PartialEq)]
  enum TtaiDr
  { NiceTits
  , TtrsDown // TODO: make sure we buffer requests or return errors when this happens.
  }
  struct DrTtai
  { ttai : TtaiDr,
  }
  let mut dr = DrTtai
  { ttai : TtaiDr::NiceTits
  };
  loop 
  {
    tokio::select! 
    { // Monitor ttrs core errors
      res = from_ttrs_error_rx.recv() => 
      { match res 
        { Some(ttrs_core_error) => 
          { let tcestr = format!("{:#?}", ttrs_core_error);
            let mut pe = true;
            match ttrs_core_error 
            { ttrs::CoreError::OAuth(_) => log::warn!("TODO: handle ttrs oauth error - "),
              ttrs::CoreError::WebSocket(_) => 
              { log::warn!("Dr TTAI is chillin, ttrs resets on a websocket error and wait for a parseworking.");
                dr.ttai = TtaiDr::NiceTits;
                pe = false; 
              },
              ttrs::CoreError::TaskPanic(_) => log::warn!("TODO: handle ttrs taskpanic error"),
              ttrs::CoreError::Unrecoverable(_) => log::warn!("TODO: handle ttrs Unrecoverable error"),
              ttrs::CoreError::ParseFailure(_) => log::warn!("TODO: handle ttrs ParseFailure error"),
              ttrs::CoreError::ParseWarning(_) => 
              { /* TTRS warns us, usually non-fatal
                   but we should put some counters in dr and track #/rate and 
                   then try restarting ttrs if it gets a bad threshold
                */ 
                pe = false;
              } 
              ttrs::CoreError::ValidStartup(count) => 
              { pe = false; 
                if dr.ttai != TtaiDr::NiceTits
                { dr.ttai = TtaiDr::NiceTits;
                }
                log::info!("{} Valid TTRS Startup!, count = {}",dbgspt, count);
              },
              ttrs::CoreError::ReplyWarning(info) => 
              { log::warn!("{} TODO: we probably sent a bad request since we got no  TTRS reply - {:#?}", dbgspt, info);
                pe = false;
              }
            };
            if pe 
            { log::error!("{} !!! FATAL !!! UNHANDLED DR_TTAI ERROR FROM TTRS, intializing restart sequence: {:#?}", dbgspt, tcestr);
              break;
            }
          }, 
          None => {
            log::debug!("{} ttrs error channel closed, initiating restart sequence", dbgspt);
            let _ = ttai_tick_reset_tx.send("Main loop: ttrs error channel closed.".to_string()).await;
            break;
          }
        }
      }

      // Handle ticker subscription requests
      res = told_dr_add_ttai_tik.recv() => 
      { match res 
        { Some
          ( (symbol, sub_id, sender, resulter)
          ) => 
          { log::debug!("{} New ticker subscription request for {} with ID {}", dbgspt, symbol, sub_id);
            
            // Register with tick distributor
            if ttai_channel_info_tx.send
            ( TtaiTickCommand::NewSub((symbol.clone(), sub_id, sender.clone()))
            ).await.is_ok() 
            { log::debug!("{} Registered subscriber {} with tick distributor", dbgspt, sub_id);

              // Subscribe to ttrs stream - this sends StreamData to ttai_channel_tick_tx
              thand.tell_tastytrade_to_get_ticker_stream
              ( symbol.clone(), ttai_channel_tick_tx.clone()
              ).await;
              
              // Get option chain details
              let finass = if symbol.starts_with('.') // equity optin
              { dsta::FinAss::OptDeets
                ( dsta::Opt
                  {   ticker_name : format!("{}", symbol) // String, (we should be able to deduce it )
                    , option_name : format!("") // String,
                    , option_rite : dsta::OptRite::Put
                    , strike_spot : 0.0 // f64,
                    , expire_date : chrono::DateTime::<chrono::Utc>::default() // chrono::DateTime<chrono::Utc>,
                    , option_type : dsta::OptStyle::Usa
                    , tick_rulers : None
                    , last_ticker : None
                  }
                )
              }
              else
              { dsta::FinAss::StkDeets
                ( dsta::Stk
                  {   ticker_name : format!("{}", symbol) // String,
                    , ticker_info : format!("")
                    , last_ticker : None
                    , tick_rulers : None
                  }
                )
              };

              if !symbol.starts_with('.')
              { if let Ok(ocd) = thand.tell_tastytrade_to_get_option_chain(symbol.clone()).await 
                { let info = dsta::PushTickerResult 
                  { ass_deets: finass
                  , opt_chain: dsta::convert::option_chain_data_to_derivatives(&ocd),
                  };
                  let _ = resulter.send(Some(info)).await;
                }
                else
                { log::warn!("{} Couldn't get option details for {}", dbgspt, symbol);
                  let info = dsta::PushTickerResult 
                  { ass_deets: finass
                  , opt_chain: Vec::new()
                  };
                  let _ = resulter.send(Some(info)).await;
                } 
              }
              else
              { log::warn!("{} Couldn't get option details for {} - assuming its an option", dbgspt, symbol);
                let info = dsta::PushTickerResult 
                { ass_deets: finass
                , opt_chain: Vec::new()
                };
                let _ = resulter.send(Some(info)).await;
              }
            } 
            else 
            { log::error!("{} Failed to register subscriber with tick distributor", dbgspt);
              let _ = resulter.send(None).await;
            }
          }
          None => {
            log::debug!("{} Ticker subscription channel closed, initiating shutdown", dbgspt);
            let _ = ttai_tick_reset_tx.send("Main loop: Subscription channel closed.".to_string()).await;
            break;
          }
        }
      }

      // Handle ticker unsubscription requests
      res = told_dr_pop_ttai_tik.recv() => {
        match res {
          Some((symbol, sub_id)) => {
            log::debug!("{} Unsubscribe request for {} ID {}", dbgspt, symbol, sub_id);
            if ttai_channel_info_tx.send(TtaiTickCommand::DieSub((symbol.clone(), sub_id))).await.is_ok() {
              log::debug!("{} Sent unsubscribe command to tick distributor", dbgspt);
            } else {
              log::error!("{} Failed to send unsubscribe command", dbgspt);
            }
          }
          None => {
            log::debug!("{} Unsubscription channel closed, initiating shutdown", dbgspt);
            let _ = ttai_tick_reset_tx.send("Main loop: Unsubscription channel closed.".to_string()).await;
            break;
          }
        }
      }

      // NEW: Handle dry run requests
      res = told_dr_order_dryrun.recv() => 
      { match res 
        { Some((order, replyer)) => 
          { log::debug!("{} Received dry run request: {:?}", dbgspt, order);
            let orderreq = dsta::convert::order_to_order_request(&order);
            
            match thand.tell_tastytrade_to_dry_run_order
            ( acct_num.clone(), orderreq.clone()
            ).await
            { Ok(dry_run_result) =>
              { let total_fees = 
                  dry_run_result.fee_calculation.total_fees +
                  dry_run_result.fee_calculation.commission.unwrap_or(0.0)
                ;
                log::info!("{} Dry run completed: total fees = ${:.2}", dbgspt, total_fees);
                let _ = replyer.send(Ok(total_fees));
              }
              Err(e) =>
              { log::error!("{} Dry run order failed! {:#?}", dbgspt, e);
                let _ = replyer.send(Err(e.into()));
              }
            };
          }
          None => {
            log::debug!("{} Dry run channel closed, initiating shutdown", dbgspt);
            let _ = ttai_tick_reset_tx.send("Main loop: Dry run channel closed.".to_string()).await;
            break;
          }
        }
      }

      // Handle order submissions
      res = told_dr_order_submit.recv() => 
      { match res 
        { Some((order, replyer)) => 
          {
            log::debug!("{} Received order submission: {:?}", dbgspt, order);
            let orderreq = dsta::convert::order_to_order_request(&order);
            
            match thand.tell_tastytrade_to_dry_run_order
            ( acct_num.clone(), orderreq.clone()
            ).await
            { Ok(ok) =>
              { let total_cost = 
                  ok.fee_calculation.total_fees +
                  ok.fee_calculation.commission.unwrap_or(0.0)
                ;
                log::info!("{} Order dry run passed, total fees: ${:.2}", dbgspt, total_cost);
                // NOTE: Fees are now handled via dry_run interface before order submission
              }
              Err(e) =>
              { log::error!("{} Dry run order failed! {:#?}", dbgspt, e);
                let _ = replyer.send(Err(e.into()));
                continue;
              }
            };

            match thand.tell_tastytrade_to_place_order(acct_num.clone(), orderreq).await 
            { // TODO: look at the result better, we need to detect from the actual 
              // result (the data in it), if it was okay if bitcoin sent because it
              // has no order id's for bitcoin orders. so check the reply different
              // somewhere
              // Or we can just say new_id 0 with valid id is a bitcoin thing.
              Ok(ok) => 
              { let por: dsta::PushOrderResult = if let Some(x) = ok.id 
                { if x != 0
                  { dsta::PushOrderResult::ValidID { new_id: x }
                  }
                  else 
                  { dsta::PushOrderResult::FailedID { new_id: 0 }
                  }
                } 
                else 
                { dsta::PushOrderResult::ValidID { new_id: 0 }
                };
                log::info!("{} Order placed successfully: {:#?}", dbgspt, por);
                let _ = replyer.send(Ok(por));
              }
              Err(e) => 
              { log::error!("{} Order placement failed: {:#?}", dbgspt, e);
                let _ = replyer.send(Err(e.into()));
              }
            }
          }
          None => {
            log::debug!("{} Order submission channel closed, initiating shutdown", dbgspt);
            let _ = ttai_tick_reset_tx.send("Main loop: Order submission channel closed.".to_string()).await;
            break;
          }
        }
      }

      // NEW: Handle order cancellation requests
      res = told_dr_order_cancel.recv() => 
      { match res 
        { Some((order_hash, replyer)) => 
          { log::info!("{} Received order cancellation request for order hash {}", dbgspt, order_hash);
            
            // Convert order_hash (u64) to string for ttrs API
            //let order_id_str = order_hash.to_string();
            
            match thand.tell_tastytrade_to_cancel_order
            ( acct_num.clone(),
              order_hash,           // ← use the original u64
            ).await 
            { Ok(_cancel_response) =>
              { log::info!("{} Order {} cancelled successfully", dbgspt, order_hash);
                let _ = replyer.send(Ok(true));
                
                // TODO: We should get a confirmation update via order state channel
                // For now, we just log success
              }
              Err(e) =>
              { log::error!("{} Failed to cancel order {}: {:#?}", dbgspt, order_hash, e);
                let _ = replyer.send(Err(e.into()));
              }
            }
            
          }
          None => {
            log::debug!("{} Order cancel channel closed, initiating shutdown", dbgspt);
            let _ = ttai_tick_reset_tx.send("Main loop: Order cancel channel closed.".to_string()).await;
            break;
          }
        }
      }

      // Handle reset signals
      res = told_dr_ttai_reset.recv() => {
        match res {
          Some(reason) => {
            log::debug!("{} Received reset signal: {}", dbgspt, reason);
            let _ = ttai_tick_reset_tx.send(format!("Main loop: {}", reason)).await;
            break;
          }
          None => {
            log::debug!("{} Reset channel closed, initiating shutdown", dbgspt);
            let _ = ttai_tick_reset_tx.send("Main loop: Reset channel closed.".to_string()).await;
            break;
          }
        }
      }

      // Forward order states to dr_bot0
      res = told_dr_order_status.recv() => 
      { match res 
        { Some(replyer) => 
          { log::debug!("{} Received order status poll request", dbgspt);
            
            match thand.tell_tastytrade_to_get_live_orders(acct_num.clone()).await 
            { Ok(order_responses) => 
              { let placed_orders = dsta::convert::order_responses_to_placed_orders(&order_responses);
                log::info!("{} Polled {} live orders", dbgspt, placed_orders.len());
                let _ = replyer.send(Ok(placed_orders));
              }
              Err(e) => 
              { log::error!("{} Failed to poll live orders: {:?}", dbgspt, e);
                let _ = replyer.send(Err(e.into()));
              }
            }
          }
          None => {
            log::debug!("{} Order status channel closed, initiating shutdown", dbgspt);
            let _ = ttai_tick_reset_tx.send("Main loop: Order status channel closed.".to_string()).await;
            break;
          }
        }
      }
    }
  }

  log::debug!("{} Exiting dr_ttai main loop, cleaning up", dbgspt);
  let _ = tell_ttrs_to_shutdown_tx.send("Normal shutdown".to_string()).await;
  let _ = tell_dr_a_snively.send((0, dsta::Snively::SendLogNote(format!("dtsd")))).await;
}
