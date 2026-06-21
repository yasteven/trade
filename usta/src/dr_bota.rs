// usta/src/dr_bota.rs
use crate::*;

// The API bridge between bots and dr_robo

// TODO: The dr_bota should be in charge of running the bot thread,
// so when its main thread ends, the bota can end which closes the
// pipes to the other components.

// How the messages are processed 
mod from_robo
{ // Commands that come in from Dr Robo

  pub async fn process_a_told_to_start
  (   bot: &mut crate::BotApi
    , name_of_bot: String
    , dr_r: &crate::DrRoboApi
    , spot: usize
  ) 
  { let dbgspt = format!("[DBFRPATTS].{} : ", spot);
    log::debug!("ENTRY! dr_bota::from_robo::process_a_told_to_start - {} - start running the base bot api interface.", dbgspt);
    bot.robot_brain_state.bot_mind = crate::BotMindState::Running;
    let _ = bot.robot_bota_sender.tell_bot_it_sucks.send(format!("{} - Bot Name @ {} is `{}", crate::FixFails::Start.to_string(), dbgspt, name_of_bot));
    let _ = dr_r.tell_dr_new_brain.send((spot,crate::BotMindState::Running)).await;
  }

  pub async fn process_a_told_to_pause // DEPRECIATED
  (   bot: &mut crate::BotApi
    , dr_r: &crate::DrRoboApi
    , spot: usize
  ) 
  { let dbgspt = format!("[DBFRPATTP].{} : ", spot);
    log::debug!("ENTRY! dr_bota::from_robo::process_a_told_to_pause - {} - pause the bot.", dbgspt);
    bot.robot_brain_state.bot_mind = crate::BotMindState::Stopped;
    let _ = dr_r.tell_dr_new_brain.send((spot, crate::BotMindState::Stopped)).await;
    let _ = bot.robot_bota_sender.tell_bot_it_sucks.send(format!("{} pausing tickers or account updates !", crate::FixFails::Pause.to_string())).await;
  }

  pub async fn process_a_told_to_death // DEPRECIATED
  (   bot: &mut crate::BotApi
    , msg: String
    , dr_r: &crate::DrRoboApi
    , spot: usize
    , got_killed: &mut bool
  ) 
  { let dbgspt = format!("[DBFRPATTDE].{} : ", spot);
    log::debug!("ENTRY! dr_bota::from_robo::process_a_told_to_death - {} - transition bot to Seppuku state.", dbgspt);
    bot.robot_brain_state.bot_mind = crate::BotMindState::Seppuku;
    let _ = dr_r.tell_dr_new_brain.send((spot, crate::BotMindState::Seppuku)).await;
    let _ = bot.robot_bota_sender.tell_bot_it_sucks.send
    ( format!
      ( "{} And the msg = {}"
        , crate::FixFails::Death.to_string()
        , msg
      )
    ).await;
    *got_killed = true;
  }

  pub async fn process_a_told_it_sucks // forward directly to bot
  (   msg: String
    , tell_bot_it_sucks_api: &tokio::sync::mpsc::Sender<String>
  ) 
  { let dbgspt = format!("[DBFRPATIS] : ");
    log::debug!("ENTRY! dr_bota::from_robo::process_a_told_it_sucks - {} - forwarding error message to bot...", dbgspt);
    match tell_bot_it_sucks_api.send(msg).await 
    { Ok(()) => 
      {
        
      }
      Err(e) => 
      { log::error!("{}Failed to send error message to tell_bot_it_sucks_api: {}; TODO: return error to caller to exit the loop due to closed pipe", dbgspt, e);
      }
    }
  }

  pub async fn process_a_told_bot_to_check
  (   msg: crate::Allocation
    , bot: &crate::BotApi
    , tell_bot_to_check_api: &tokio::sync::mpsc::Sender<crate::Allocation>
  ) 
  { let dbgspt = format!("[DBFRPABTC] : ");
    log::debug!("ENTRY! dr_bota::from_robo::process_a_told_bot_to_check - {} - forward check allocation to bot.", dbgspt);
    if bot.robot_brain_state.bot_mind == crate::BotMindState::Running || // tell us
      msg.bot_name == bot.robot_brain_state.bot_name // always tell us about us.
    { let _ = tell_bot_to_check_api.send(msg).await;
    }
  }
}

// Commands that come in from Bot Api, to forward to robo 
// FUTURE: maybe forward some not to robo, but directly to base or ttai 
mod from_apis
{ 
  use crate::*;

  pub async fn process_a_told_dr_pls_trade
  (   tell_bot_it_sucks: &tokio::sync::mpsc::Sender::<String>
    , order: dsta::Order
    , bot: &crate::BotApi
    , dr_r: &crate::DrRoboApi
    , spot: usize
  ) 
  { let dbgspt = format!("[DBFAPATT].{} : ", spot);
    log::debug!("ENTRY! dr_bota::from_apis::process_a_told_dr_pls_trade - {} - validate and forward trade request.", dbgspt);
    let num = bot.robot_brain_state.num_swap + 1;
    let _ = dr_r.tell_dr_pls_trade.send((spot, num, order)).await;
    let _ = tell_bot_it_sucks.send(format!("dr_bota bot api forwareded order number {} to dr_robo!",num)).await;
  }

  pub async fn process_a_told_dr_new_brain
  (   tell_bot_it_sucks: &tokio::sync::mpsc::Sender::<String>
    , repbrain: crate::Allocation
    , bot: &crate::BotApi
  ) 
  { let dbgspt = format!("[DBFAPATNB] : ");
    log::debug!("ENTRY! dr_bota::from_apis::process_a_told_dr_new_brain - {} - handle bot state update.", dbgspt);
    let ourbrain = &bot.robot_brain_state;
    if cfg!(debug_assertions) 
    { if repbrain.bot_name != ourbrain.bot_name 
      { log::error!("{}Bot name mismatch: reported {} vs our {}", dbgspt, repbrain.bot_name, ourbrain.bot_name);
        return;
      }
    }
    match repbrain.bot_mind 
    { BotMindState::Running => 
      { log::debug!("{}Bot {} mind state changed to Running", dbgspt, ourbrain.bot_name);
        log::warn!("{}Dealing with bot api implementation requesting a state changed to Running..., by doing nothing...", dbgspt);
      },
      BotMindState::Stopped => 
      { log::debug!("{}Bot {} mind state changed to Stopped", dbgspt, ourbrain.bot_name);
        log::warn!("{}Dealing with bot api implementation requesting a state changed to Stopped..., by doing nothing...", dbgspt);
      },
      BotMindState::Seppuku => 
      { log::debug!("{}Bot {} mind state changed to Seppuku", dbgspt, ourbrain.bot_name);
        log::warn!("{}Dealing with bot api implementation requesting a state changed to Seppuku..., by doing nothing...", dbgspt);
      },
      BotMindState::Zombied => 
      { log::debug!("{}Bot {} mind state changed to Zombied", dbgspt, ourbrain.bot_name);
        log::warn!("{}Dealing with bot api implementation requesting a state changed to Zombied..., by doing nothing...", dbgspt);
      },
      BotMindState::Trading => 
      { log::debug!("{}Bot {} mind state changed to Trading", dbgspt, ourbrain.bot_name);
        log::warn!("{}Dealing with bot api implementation requesting a state changed to Trading..., by doing nothing...", dbgspt);
      },
      
    }
    let _ = tell_bot_it_sucks.send(format!("TODO: deal with new brain")).await;
  }

  pub async fn process_a_told_dr_pls_ticks
  (   tell_bot_it_sucks: &tokio::sync::mpsc::Sender::<String>
    , msg : String
    , dr_r : &DrRoboApi
    , spot : usize
    , info : tokio::sync::mpsc::Sender<Option<dsta::PushTickerResult>>
  ) 
  { let dbgspt = format!("[DBFAPAPT].{} : ", spot);
    log::debug!("ENTRY! dr_bota::from_apis::process_a_told_dr_pls_ticks - {} - forward ticker subscription request.", dbgspt);
    
    let _ = dr_r.tell_dr_pls_ticks.send((spot, msg, info)).await;


    // TODO: get the asset deetails here too


    let _ = tell_bot_it_sucks.send
    ( format!
      ( "{} - yo Bot @ {} [{}]"
        , crate::FixFails::Sucks
         ( format!(", dr_bota done with tick request...")
         ).to_string()
        , spot
        , dbgspt
        , 
      )
    ).await;
  }

  pub async fn process_a_told_dr_pls_check
  (   tell_bot_it_sucks: &tokio::sync::mpsc::Sender::<String>
    , msg: String
    , dr_r: &DrRoboApi
    , spot: usize
  ) 
  { let dbgspt = format!("[DBFAPAPC].{} : ", spot);
    log::debug!("ENTRY! dr_bota::from_apis::process_a_told_dr_pls_check - {} - forward check request.", dbgspt);
    let _ = dr_r.tell_dr_pls_check.send((spot, msg)).await;
    let _ = tell_bot_it_sucks.send
    ( format!
      ( "{} - yo Bot @ {} [{}]"
        , crate::FixFails::Sucks
         ( format!(", dr_bota done with tell_dr_pls_check bot account request...")
         ).to_string()
        , spot
        , dbgspt
        , 
      )
    ).await;
  }

  pub async fn process_a_told_dr_stop_tick
  (   tell_bot_it_sucks: &tokio::sync::mpsc::Sender::<String>
    , msg: String
    , dr_r: &DrRoboApi
    , spot: usize
  ) 
  { let dbgspt = format!("[DBFAPAST].{} : ", spot);
    log::debug!("ENTRY! dr_bota::from_apis::process_a_told_dr_stop_tick - {} - forward stop ticker request.", dbgspt);
    let _ = dr_r.tell_dr_stop_tick.send((spot, msg)).await;
    let _ = tell_bot_it_sucks.send
    ( format!
      ( "{} - yo Bot @ {} [{}]"
        , crate::FixFails::Sucks
         ( format!(", dr_bota done with process_a_told_dr_stop_tick request...")
         ).to_string()
        , spot
        , dbgspt
        , 
      )
    ).await;
  }

  pub async fn process_a_told_dr_stop_info
  (   tell_bot_it_sucks: &tokio::sync::mpsc::Sender::<String>
    , msg: String
    , dr_r: &DrRoboApi
    , spot: usize
  ) 
  { let dbgspt = format!("[DBFAPASI].{} : ", spot);
    log::debug!("ENTRY! dr_bota::from_apis::process_a_told_dr_stop_info - {} - forward stop info request.", dbgspt);
    let _ = dr_r.tell_dr_stop_info.send((spot, msg)).await;
    let _ = tell_bot_it_sucks.send
    ( format!
      ( "{} - yo Bot @ {} [{}]"
        , crate::FixFails::Sucks
         ( format!(", dr_bota done with process_a_told_dr_stop_info bot account request...")
         ).to_string()
        , spot
        , dbgspt
        , 
      )
    ).await;
  }
  pub async fn process_a_told_dr_swap_info
  (   _tell_bot_it_sucks: &tokio::sync::mpsc::Sender::<String>
    , msg: BotSwaper
    , dr_r: &DrRoboApi
    , spot: usize
  ) 
  { let dbgspt = format!("[DBFAPASI].{} : ", spot);
    log::debug!("ENTRY! dr_bota::from_apis::process_a_told_dr_swap_info - {} - forward swap info request.", dbgspt);
    // TODO: Initial legality check ( check bot state, have, etc. )
    // MORE: legality checks are great for defense agaisnt the dark arts
    log::debug!("{}Forwarding swap info to dr_robo.", dbgspt);
    let _ = dr_r.tell_dr_swap_info.send((spot, msg)).await;
  }
  pub async fn process_a_told_dr_a_snively
  (   _tell_bot_it_sucks: &tokio::sync::mpsc::Sender::<String>
    , msg: dsta::Snively
    , dr_r: &DrRoboApi
    , spot: usize
  ) 
  { let dbgspt = format!("[DBFAPAsI].{} : ", spot);
    log::debug!("ENTRY! dr_bota::from_apis::process_a_told_dr_a_snively - {} - forward to robo.", dbgspt);
    // TODO: Initial legality check ( log note, etc. )
    log::debug!("{}Forwarding swap info to dr_robo.", dbgspt);
    let _ = dr_r.tell_dr_a_snively.send((spot, msg)).await;
  }
}

mod from_ttai
{ 
  use crate::*;

  pub async fn process_a_told_bot_to_trade
  (   msg: dsta::Ticker
    , bot: &BotApi
    , tell_bot_to_trade_api: &tokio::sync::mpsc::Sender<dsta::Ticker>
  ) 
  { let dbgspt = format!("[DBFTPABTT] : ");
    slog::debug!
    ( crate::glossary::TICK_SLOG
    , "{} ENTRY! dr_bota::from_ttai::process_a_told_bot_to_trade"
    , dbgspt
    );
    if bot.robot_brain_state.bot_mind == BotMindState::Running
    { let _ = tell_bot_to_trade_api.send(msg).await;
    }
  }
} 

// Base Bot API Task (bridge between impl and dr robo)
pub(crate) async fn f_run_main_dr_bota
(   dr_r: DrRoboApi
  , ctor: CtorBot
  , spot: usize
  , hand: DrBotaApiHand
  , foot: DrBotaApiFoot
) 
{ let dbgspt = format!("[BOTA].{} :", spot);
  log::debug!
  ( "\n  {} ENTRY! dr_bota::f_run_main_dr_bota \n\
    Performs the base logic for running a bot, allowing the API\n\
    user to implement simple async select functions passed as \n\
    arguments.\n\
    // ( when dr_robo launches a bot, it launches this ) => NEW TASK",
    dbgspt
  );
  log::debug!("{} initialize API memory...", dbgspt);
  let tell_bot_to_trade_api = ctor.tell_bot_to_trade_api;
  let tell_bot_to_check_api = ctor.tell_bot_to_check_api;
  let tell_bot_it_sucks_api = ctor.tell_bot_it_sucks_api;
  let told_the_bot_requests = ctor.told_the_bot_requests;
  log::debug!("{} creating channels for self...", dbgspt);
  let mut told_bot_to_start = foot.told_bot_to_start;
  let mut told_bot_to_pause = foot.told_bot_to_pause;
  let mut told_bot_to_death = foot.told_bot_to_death;
  let mut told_bot_to_trade = foot.told_bot_to_trade;
  let mut told_bot_to_check = foot.told_bot_to_check;
  let mut told_bot_it_sucks = foot.told_bot_it_sucks;
  log::debug!("{} creating channels for user...", dbgspt);
  let mut told_dr_pls_trade = told_the_bot_requests.told_dr_pls_trade;
  let mut told_dr_new_brain = told_the_bot_requests.told_dr_new_brain;
  let mut told_dr_pls_ticks = told_the_bot_requests.told_dr_pls_ticks;
  let mut told_dr_pls_check = told_the_bot_requests.told_dr_pls_check;
  let mut told_dr_stop_tick = told_the_bot_requests.told_dr_stop_tick;
  let mut told_dr_stop_info = told_the_bot_requests.told_dr_stop_info;
  let mut told_dr_swap_info = told_the_bot_requests.told_dr_swap_info;
  let mut told_dr_a_snively = told_the_bot_requests.told_dr_a_snively;
  log::debug!("{} moving io channels to BotApi struct...", dbgspt);
  
  let mut baby = crate::BotApi
  {   robot_brain_state : ctor.robot_api_brain_state
    , robot_bota_sender : hand
  };

  let baby_name = baby.robot_brain_state.bot_name.clone();
  let lbisc = baby.robot_bota_sender.tell_bot_it_sucks.clone();
  let mut got_killed = false;


  



  // Main Base Bot API Task Handler:
  loop 
  { tokio::select!
    { 
      // FROM ROBO =
      res = told_bot_to_start.recv() => 
      { match res 
        { Some(msg) => 
          { log::info!("{} Received told_bot_to_start: {:?}", dbgspt, msg);
            let _ = from_robo::process_a_told_to_start
            ( &mut baby
            , msg
            , &dr_r
            , spot
            ).await;
          },
          None => 
          { log::error!("{} Channel 'told_bot_to_start' for {} closed! Breaking from loop!", dbgspt, baby_name);
            break;
          },
        }
      },
      res = told_bot_to_pause.recv() => 
      { match res 
        { Some(()) => 
          { log::info!("{}Received told_bot_to_pause", dbgspt);
            let _ = from_robo::process_a_told_to_pause
            ( &mut baby
            , &dr_r
            , spot
            ).await;
          },
          None => 
          { log::error!("{} Channel 'told_bot_to_pause' for {} closed! Breaking from loop!", dbgspt, baby_name);
            break;
          },
        }
      },
      res = told_bot_to_death.recv() => 
      { match res 
        { Some(msg) => 
          { log::info!("{}Received told_bot_to_death: {:?}", dbgspt, msg);
            let _ = from_robo::process_a_told_to_death
            ( &mut baby
            , msg
            , &dr_r
            , spot
            , &mut got_killed
            ).await;
            break;
          },
          None => 
          { log::error!("{} Channel 'told_bot_to_death' for {} closed! Breaking from loop!", dbgspt, baby_name);
            break;
          },
        }
      },
      res = told_bot_it_sucks.recv() => 
      { match res 
        { Some(msg) => 
          { log::info!("{}Processing bot error message ( = {} )", dbgspt, msg);
            let _ = from_robo::process_a_told_it_sucks
            ( msg
            , &tell_bot_it_sucks_api
            ).await;
          },
          None => 
          { log::error!("{} Channel 'told_bot_it_sucks' for {} closed! Breaking from loop!", dbgspt, baby_name);
            break;
          },
        }
      },
      res = told_bot_to_check.recv() => // 
      { match res 
        { Some(msg) => 
          { log::info!("{}Processing bot check allocation", dbgspt);
            let _ = from_robo::process_a_told_bot_to_check
            ( msg
            , &baby
            , &tell_bot_to_check_api
            ).await;
          },
          None => 
          { log::error!("{} Channel 'told_bot_to_check' for {} closed! Breaking from loop!", dbgspt, baby_name);
            break;
          },
        }
      },

      // API BASE DEF CAN DO, e.g, FROM APIS
      res = told_dr_pls_trade.recv() => 
      { match res 
        { Some(order) => 
          { log::info!("{}Processing trade request from implementer", dbgspt);
            let _ = from_apis::process_a_told_dr_pls_trade
            ( &tell_bot_it_sucks_api
            , order
            , &baby
            , &dr_r
            , spot
            ).await;
          },
          None => 
          { log::error!("{} Channel 'told_dr_pls_trade' for {} closed! Breaking from loop!", dbgspt, baby_name);
            break;
          },
        }
      },
      res = told_dr_new_brain.recv() => 
      { match res 
        { Some(repbrain) => 
          { log::info!("{}Processing new brain state update", dbgspt);
            let _ = from_apis::process_a_told_dr_new_brain
            ( &tell_bot_it_sucks_api
            , repbrain
            , &baby
            ).await;
          },
          None => 
          { log::info!("{}Bot update channel closed", dbgspt);
          },
        }
      },
      res = told_dr_pls_ticks.recv() => 
      { match res 
        { Some(msg) => 
          { log::info!("{}Processing ticker subscription request", dbgspt);
            
            let mut inject = tokio::sync::mpsc::channel::<Option<dsta::PushTickerResult>>(2);
            let _ = from_apis::process_a_told_dr_pls_ticks
            ( &tell_bot_it_sucks_api
            , msg.0
            , &dr_r
            , spot
            , inject.0 // msg.1
            ).await;
 
            // FUTURE: now  we can also store the ticker info.
            if let Some(res) = inject.1.recv().await
            { let _ = msg.1.send( res ).await;
            }
            else
            { panic!("this would be a horrible error");
            }
          },
          None => 
          { log::error!("{} Channel 'told_dr_pls_ticks' for {} closed! Breaking from loop!", dbgspt, baby_name);
            break;
          },
        }
      },
      res = told_dr_pls_check.recv() => 
      { match res 
        { Some(msg) => 
          { log::info!("{}Processing check request", dbgspt);
            let _ = from_apis::process_a_told_dr_pls_check
            ( &tell_bot_it_sucks_api
            , msg
            , &dr_r
            , spot
            ).await;
          },
          None => 
          { log::error!("{} Channel 'told_dr_pls_check' for {} closed! Breaking from loop!", dbgspt, baby_name);
            break;
          },
        }
      },
      res = told_dr_stop_tick.recv() => 
      { match res 
        { Some(msg) => 
          { log::info!("{}Processing stop ticker request", dbgspt);
            let _ = from_apis::process_a_told_dr_stop_tick
            ( &tell_bot_it_sucks_api
            , msg
            , &dr_r
            , spot
            ).await;
          },
          None => 
          { log::error!("{} Channel 'told_dr_stop_tick' for {} closed! Breaking from loop!", dbgspt, baby_name);
            break;
          },
        }
      },
      res = told_dr_stop_info.recv() => 
      { match res 
        { Some(msg) => 
          { log::debug!("{}Processing stop info request", dbgspt);
            let _ = from_apis::process_a_told_dr_stop_info
            ( &tell_bot_it_sucks_api
            , msg
            , &dr_r
            , spot
            ).await;
          },
          None => 
          { log::error!("{} Channel 'told_dr_stop_info' for {} closed! Breaking from loop!", dbgspt, baby_name);
            break;
          },
        }
      },
      res = told_dr_swap_info.recv() => 
      { match res 
        { Some(msg) => 
          { log::debug!("{}Processing swap info request", dbgspt);
            let _ = from_apis::process_a_told_dr_swap_info
            ( &tell_bot_it_sucks_api
            , msg
            , &dr_r
            , spot
            ).await;
          },
          None => 
          { log::error!("{} Channel 'told_dr_stop_info' for {} closed! Breaking from loop!", dbgspt, baby_name);
            break;
          },
        }
      },
      res = told_dr_a_snively.recv() => 
      { match res 
        { Some(msg) => 
          { log::info!("{}Processing swap info request", dbgspt);
            let _ = from_apis::process_a_told_dr_a_snively
            ( &tell_bot_it_sucks_api
            , msg
            , &dr_r
            , spot
            ).await;
          },
          None => 
          { log::error!("{} Channel 'told_dr_stop_info' for {} closed! Breaking from loop!", dbgspt, baby_name);
            break;
          },
        }
      },

      // FROM TTAI:
      res = told_bot_to_trade.recv() => 
      { match res 
        { Some(msg) => 
          { let _ = from_ttai::process_a_told_bot_to_trade
            ( msg
            , &baby
            , &tell_bot_to_trade_api
            ).await;
          },
          None => 
          { log::error!("{} Channel 'told_bot_to_trade' for {} closed! Breaking from loop!", dbgspt, baby_name);
            break;
          },
        }
      }
    } // end of tokio::select
  }
  let _ = lbisc.send
  ( format!
    ( "{} The f_dr_bot_a function {:#?} is exiting."
    , dbgspt
    , FixFails::Sucks(format!("Goodbye World!")).to_string()
    )
  ).await;
  if got_killed 
  { let _ = lbisc.send(FixFails::Ghost.to_string()).await;
  } 
  else 
  { let _ = lbisc.send(format!("{} Bot {} has reached Death state.", dbgspt, FixFails::Death.to_string())).await;
  }

  if !dr_r.tell_dr_new_brain.send((spot, crate::BotMindState::Zombied)).await.is_ok()
  { log::error!("BOTA COULDN'T ALERT ROBO OF ZOMBIE! this could be very bad");
  }
  log::debug!("{} Ghosted or dead ..., we end the thread", dbgspt);
}

// Base Bot API Task (bridge between impl and dr robo)
pub(crate) async fn f_run_main_dr_bota_2
(   dr_r: DrRoboApi
  , ctor: CtorBot
  , spot: usize
  , hand: DrBotaApiHand
  , foot: DrBotaApiFoot
  , mind: tokio::task::JoinHandle<()>
) 
{ let dbgspt = format!("[BOTA2].{} :", spot);
  log::debug!
  ( "\n  {} ENTRY! dr_bota::f_run_main_dr_bota_2 \n\
    Performs the base logic for running a bot, allowing the API\n\
    user to implement simple async select functions passed as \n\
    arguments.\n\
    // ( when dr_robo launches a bot, it launches this ) => NEW TASK",
    dbgspt
  );
  let kclone = hand.tell_bot_to_death.clone();
  log::debug!("{} initialize API memory...", dbgspt);
  let tell_bot_to_trade_api = ctor.tell_bot_to_trade_api;
  let tell_bot_to_check_api = ctor.tell_bot_to_check_api;
  let tell_bot_it_sucks_api = ctor.tell_bot_it_sucks_api;
  let told_the_bot_requests = ctor.told_the_bot_requests;
  log::debug!("{} creating channels for self...", dbgspt);
  let mut told_bot_to_start = foot.told_bot_to_start;
  let mut told_bot_to_pause = foot.told_bot_to_pause;
  let mut told_bot_to_death = foot.told_bot_to_death;
  let mut told_bot_to_trade = foot.told_bot_to_trade;
  let mut told_bot_to_check = foot.told_bot_to_check;
  let mut told_bot_it_sucks = foot.told_bot_it_sucks;
  log::debug!("{} creating channels for user...", dbgspt);
  let mut told_dr_pls_trade = told_the_bot_requests.told_dr_pls_trade;
  let mut told_dr_new_brain = told_the_bot_requests.told_dr_new_brain;
  let mut told_dr_pls_ticks = told_the_bot_requests.told_dr_pls_ticks;
  let mut told_dr_pls_check = told_the_bot_requests.told_dr_pls_check;
  let mut told_dr_stop_tick = told_the_bot_requests.told_dr_stop_tick;
  let mut told_dr_stop_info = told_the_bot_requests.told_dr_stop_info;
  let mut told_dr_swap_info = told_the_bot_requests.told_dr_swap_info;
  let mut told_dr_a_snively = told_the_bot_requests.told_dr_a_snively;
  log::debug!("{} moving io channels to BotApi struct...", dbgspt);
  
  let mut baby = crate::BotApi
  {   robot_brain_state : ctor.robot_api_brain_state
    , robot_bota_sender : hand
  };

  let baby_name = baby.robot_brain_state.bot_name.clone();
  let lbisc = baby.robot_bota_sender.tell_bot_it_sucks.clone();
  let mut got_killed = false;

  tokio::spawn
  ( async move
    { log::debug!("DR_BOTA is making a task to await the Mind Handle");
      let _ = mind.await; 
      let _ = kclone.send(format!("Mind Handle Finished!")).await;
      log::debug!("DR_BOTA is done with the task to await the Mind Handle");
    }
  );
  
  // Main Base Bot API Task Handler:
  loop 
  { tokio::select!
    { 
      res = told_bot_to_start.recv() => 
      { match res 
        { Some(msg) => 
          { log::info!("{} Received told_bot_to_start: {:?}", dbgspt, msg);
            let _ = from_robo::process_a_told_to_start
            ( &mut baby
            , msg
            , &dr_r
            , spot
            ).await;
          },
          None => 
          { log::error!("{} Channel 'told_bot_to_start' for {} closed! Breaking from loop!", dbgspt, baby_name);
            break;
          },
        }
      },
      res = told_bot_to_pause.recv() => 
      { match res 
        { Some(()) => 
          { log::info!("{}Received told_bot_to_pause", dbgspt);
            let _ = from_robo::process_a_told_to_pause
            ( &mut baby
            , &dr_r
            , spot
            ).await;
          },
          None => 
          { log::error!("{} Channel 'told_bot_to_pause' for {} closed! Breaking from loop!", dbgspt, baby_name);
            break;
          },
        }
      },
      res = told_bot_to_death.recv() => 
      { match res 
        { Some(msg) => 
          { log::info!("{}Received told_bot_to_death: {:?}", dbgspt, msg);
            let _ = from_robo::process_a_told_to_death
            ( &mut baby
            , msg
            , &dr_r
            , spot
            , &mut got_killed
            ).await;
            break;
          },
          None => 
          { log::error!("{} Channel 'told_bot_to_death' for {} closed! Breaking from loop!", dbgspt, baby_name);
            break;
          },
        }
      },
      res = told_bot_it_sucks.recv() => 
      { match res 
        { Some(msg) => 
          { log::info!("{}Processing bot error message ( = {} )", dbgspt, msg);
            let _ = from_robo::process_a_told_it_sucks
            ( msg
            , &tell_bot_it_sucks_api
            ).await;
          },
          None => 
          { log::error!("{} Channel 'told_bot_it_sucks' for {} closed! Breaking from loop!", dbgspt, baby_name);
            break;
          },
        }
      },
      res = told_bot_to_check.recv() => // 
      { match res 
        { Some(msg) => 
          { log::info!("{}Processing bot check allocation", dbgspt);
            let _ = from_robo::process_a_told_bot_to_check
            ( msg
            , &baby
            , &tell_bot_to_check_api
            ).await;
          },
          None => 
          { log::error!("{} Channel 'told_bot_to_check' for {} closed! Breaking from loop!", dbgspt, baby_name);
            break;
          },
        }
      },

      // API BASE DEF CAN DO, e.g, FROM APIS
      res = told_dr_pls_trade.recv() => 
      { match res 
        { Some(order) => 
          { log::info!("{}Processing trade request from implementer", dbgspt);
            let _ = from_apis::process_a_told_dr_pls_trade
            ( &tell_bot_it_sucks_api
            , order
            , &baby
            , &dr_r
            , spot
            ).await;
          },
          None => 
          { log::error!("{} Channel 'told_dr_pls_trade' for {} closed! Breaking from loop!", dbgspt, baby_name);
            break;
          },
        }
      },
      res = told_dr_new_brain.recv() => 
      { match res 
        { Some(repbrain) => 
          { log::info!("{}Processing new brain state update", dbgspt);
            let _ = from_apis::process_a_told_dr_new_brain
            ( &tell_bot_it_sucks_api
            , repbrain
            , &baby
            ).await;
          },
          None => 
          { log::info!("{}Bot update channel closed", dbgspt);
          },
        }
      },
      res = told_dr_pls_ticks.recv() => 
      { match res 
        { Some(msg) => 
          { log::info!("{}Processing ticker subscription request", dbgspt);
            
            let mut inject = tokio::sync::mpsc::channel::<Option<dsta::PushTickerResult>>(2);
            let _ = from_apis::process_a_told_dr_pls_ticks
            ( &tell_bot_it_sucks_api
            , msg.0
            , &dr_r
            , spot
            , inject.0 // msg.1
            ).await;
 
            // FUTURE: now  we can also store the ticker info.
            if let Some(res) = inject.1.recv().await
            { let _ = msg.1.send( res ).await;
            }
            else
            { panic!("this would be a horrible error");
            }
          },
          None => 
          { log::error!("{} Channel 'told_dr_pls_ticks' for {} closed! Breaking from loop!", dbgspt, baby_name);
            break;
          },
        }
      },
      res = told_dr_pls_check.recv() => 
      { match res 
        { Some(msg) => 
          { log::info!("{}Processing check request", dbgspt);
            let _ = from_apis::process_a_told_dr_pls_check
            ( &tell_bot_it_sucks_api
            , msg
            , &dr_r
            , spot
            ).await;
          },
          None => 
          { log::error!("{} Channel 'told_dr_pls_check' for {} closed! Breaking from loop!", dbgspt, baby_name);
            break;
          },
        }
      },
      res = told_dr_stop_tick.recv() => 
      { match res 
        { Some(msg) => 
          { log::info!("{}Processing stop ticker request", dbgspt);
            let _ = from_apis::process_a_told_dr_stop_tick
            ( &tell_bot_it_sucks_api
            , msg
            , &dr_r
            , spot
            ).await;
          },
          None => 
          { log::error!("{} Channel 'told_dr_stop_tick' for {} closed! Breaking from loop!", dbgspt, baby_name);
            break;
          },
        }
      },
      res = told_dr_stop_info.recv() => 
      { match res 
        { Some(msg) => 
          { log::info!("{}Processing stop info request", dbgspt);
            let _ = from_apis::process_a_told_dr_stop_info
            ( &tell_bot_it_sucks_api
            , msg
            , &dr_r
            , spot
            ).await;
          },
          None => 
          { log::error!("{} Channel 'told_dr_stop_info' for {} closed! Breaking from loop!", dbgspt, baby_name);
            break;
          },
        }
      },
      res = told_dr_swap_info.recv() => 
      { match res 
        { Some(msg) => 
          { log::debug!("{} Processing swap info request", dbgspt);
            let _ = from_apis::process_a_told_dr_swap_info
            ( &tell_bot_it_sucks_api
            , msg
            , &dr_r
            , spot
            ).await;
          },
          None => 
          { log::error!("{} Channel 'told_dr_stop_info' for {} closed! Breaking from loop!", dbgspt, baby_name);
            break;
          },
        }
      },
      res = told_dr_a_snively.recv() => 
      { match res 
        { Some(msg) => 
          { log::info!("{}Processing swap info request", dbgspt);
            let _ = from_apis::process_a_told_dr_a_snively
            ( &tell_bot_it_sucks_api
            , msg
            , &dr_r
            , spot
            ).await;
          },
          None => 
          { log::error!("{} Channel 'told_dr_stop_info' for {} closed! Breaking from loop!", dbgspt, baby_name);
            break;
          },
        }
      },

      // FROM TTAI:
      res = told_bot_to_trade.recv() => 
      { match res 
        { Some(msg) => 
          { slog::trace!
            ( crate::glossary::TICK_SLOG
            , "{} Processing bot trade ticker (where the logic goes)"
            , dbgspt
            );
            let _ = from_ttai::process_a_told_bot_to_trade
            ( msg
            , &baby
            , &tell_bot_to_trade_api
            ).await;
          },
          None => 
          { log::error!("{} Channel 'told_bot_to_trade' for {} closed! Breaking from loop!", dbgspt, baby_name);
            break;
          },
        }
      }
    } // end of tokio::select
  }
  let _ = lbisc.send
  ( format!
    ( "{} The f_dr_bot_a function {:#?} is exiting."
    , dbgspt
    , FixFails::Sucks(format!("Goodbye World!")).to_string()
    )
  ).await;
  if got_killed 
  { let _ = lbisc.send(FixFails::Ghost.to_string()).await;
  } 
  else 
  { let _ = lbisc.send(format!("{} Bot {} has reached Death state.", dbgspt, FixFails::Death.to_string())).await;
  }
  log::debug!("{} Ghosted or dead ..., we end the thread", dbgspt);
}

// END OF DR_BOTA.rs