//dr_robo.rs

use crate::*;
use crate::core::*; // for helper functions that check assets accounts and leins and stuff


pub(crate) mod from_dr_r // main/base
{ 
  use crate::*;
  pub async fn process_new_robot
  (   new_bot: CtorBot
    , all_bot_api: &mut Vec<Option<BotApi>>        
    , map_bots_index: &mut std::collections::HashMap<String, usize>
    , all_bot_checks: &mut Vec<Vec<usize>>
    , dr_seek_api: &DrSeekApi
    , dr_robo_api: &DrRoboApi
  ) -> ()
  { let dbgspt = format!("[{}]", crate::glossary::LOG_PREFIX_DR_ROBO_NEW_BOT);
    let eeemsg = format!("dr_robo::from_dr_r::process_new_robot() == {} - checks validity, HUMU: will save data for loading later probably?", dbgspt);
    log::debug!("\n  {} [ENTRY] {}",dbgspt, eeemsg);
    let bot_name = new_bot.robot_api_brain_state.bot_name.clone();
   
    log::trace!("{}  - // Add new bot to dr_robo(ticizer) (brains, map-index, checks-map",dbgspt );
    let (hand, foot) = fn_make_bot_api_hand_and_foot();
    let new_index = all_bot_api.len();
    all_bot_api.push
    ( Some
      ( BotApi
        {   robot_brain_state : new_bot.robot_api_brain_state.clone()
          , robot_bota_sender : hand.clone()
        }
      )
    );
    map_bots_index.insert(bot_name.clone(), new_index);
    all_bot_checks.push(vec![new_index]);

    log::trace!("{}  - // spawn crate::dr_bota::f_run_main_dr_bota api thread",dbgspt );
    let clone_dr_r_api = dr_robo_api.clone();
    let clone_hand = hand.clone();
    tokio::spawn
    ( async move
      { crate::dr_bota::f_run_main_dr_bota
        ( clone_dr_r_api // DrRoboApi // dr_r: DrRoboApi
        , new_bot // CtorBot // ctor: CtorBot
        , new_index // usize // spot: usize
        , clone_hand
        , foot
        ).await;
      }
    );

    log::info!("{}  - // informing GUI of new BOT - dr_seek_api.tell_dr_made_bot",dbgspt );
    match dr_seek_api.tell_dr_made_bot.try_send
    ( dsta::MadeBot::Made( format!("{}", bot_name) )
    )
    { Ok(()) => 
      { log::info!
        ( "{} dr_robo told seek of OK MadeBot creation!"
          , dbgspt
        );
      }
      Err(tokio::sync::mpsc::error::TrySendError::Full(_)) => 
      { log::warn!
        ( "{} dr_seek_api.tell_dr_made_bot channel full, buzz default not sent to fronted: (make sure in the future we buffer this for the frontend )"
        , dbgspt
        );
      }
      Err(tokio::sync::mpsc::error::TrySendError::Closed(_)) => 
      { log::error!
        ( "{} dr_seek_api.tell_dr_made_bot channel closed, critical error (HUMU: dont exit from the loop but do admin stuff with the thingy) make sure in the future we buffer this for the frontend reconnect "
        , dbgspt
        );
        // Since process_new_robot returns (), we can't return Err; just log and continue
      }
    };

    log::trace!("{}  - // NOT informing bota it should start in process new robot!, wait for user request",dbgspt );
    // match clone_hand.tell_bot_to_start.try_send 
    // ( format!("{}", bot_name)
    // ).await
    // { Ok(())=>
    //   { log::info!
    //     ( "{} dr_robo told underlying bota to start!"
    //       , dbgspt
    //     );
    //   }, 
    //   Err(e) =>
    //   { // The issue here is probably an out-of-memory
    //     // But this really shouldnt be possible with rust
    //     // 2DO: report bad bot to deal with
    //   }
    // }
    log::debug!("{} [EXIT] - New robot {} added at index {}", dbgspt, bot_name, new_index);
  }
}

pub(crate) mod from_seek
{ use crate::*;

  pub async fn process_a_snively
  (   sniv: dsta::Snively
    , all_bot_api: &mut Vec<Option<BotApi>>
    , dr_seek_api: &DrSeekApi
    , tell_dr_new_robot : &tokio::sync::mpsc::Sender<crate::CtorBot>
    , map_bots_index: &mut std::collections::HashMap<String, usize>
    , dr_robo_api: &DrRoboApi
    , all_bot_checks: &mut Vec<Vec<usize>>
  ) -> () // all processing here happens here, dont care about emergency exit Result<(), Box<dyn std::error::Error>> 
  { let dbgspt = format!("[{}]", crate::glossary::LOG_PREFIX_DR_ROBO_DO_SNIV);
    log::debug!("ENTRY! - dr_robo.rs::from_seek::process_a_snively = {}", dbgspt);
    match sniv 
    { dsta::Snively::LiveBotName(bot_name) => 
      { log::debug!("{} dr_seek got a snively::LiveBotName = {:#?}", dbgspt, bot_name);

        // find bot index by name
        let bot_index = match map_bots_index.get(&bot_name) 
        {   Some(idx) => *idx
          , None => 
            { log::error!("{} Could not find bot by name: {}", dbgspt, bot_name);
              return;
            }
        };

        let hand = if let Some(Some(bot_api)) = all_bot_api.get(bot_index) 
        { Some(bot_api.robot_bota_sender.clone())
        } else 
        { None
        };
        
        if let Some(hand) = hand
        { log::trace!("{} - telling bot to start due to sniely!", dbgspt);
          match hand.tell_bot_to_start.try_send 
          ( format!("{}", bot_name)
          )
          { Ok(())=>
            { log::info!
              ( "{} dr_robo told underlying bota to start due to Snively!"
                , dbgspt
              );
            }, 
            Err(e) =>
            { log::error!("{} hand couldnt tell bot to start!, error = {:#?}", dbgspt, e);
              log::debug!("{} [EXIT] - bad snvielly", dbgspt);
              return;
              // The issue here is possibly an out-of-memory, or the channel is clogged somehow
              // But this really shouldnt be possible with rust
              // 2DO: report bad bot to deal with
            }
          }
        }
        else // if hand.is_none()
        { log::error!("{} {}", dbgspt, crate::glossary::ERROR_NO_BOT_HAND);
          log::debug!("{} [EXIT] - bad snvielly", dbgspt);
          return;
        }
        log::debug!("{} [EXIT] - New robot {} added at index {}", dbgspt, bot_name, bot_index);
      }
      ,

      dsta::Snively::KillBotName(bot_name) =>
      { log::debug!("{} dr_seek got a snively::KillBotName = {:#?}", dbgspt, bot_name);
        let bot_index = match map_bots_index.get(&bot_name)
        { Some(idx) => *idx,
          None =>
          { log::error!("{} Could not find bot by name: {}", dbgspt, bot_name);
            return;
          }
        };
        let hand = if let Some(Some(bot_api)) = all_bot_api.get(bot_index)
        { Some(bot_api.robot_bota_sender.clone())
        } else 
        { None
        };
        if let Some(hand) = hand
        { log::info!("{} Telling bot {} to perform seppuku", dbgspt, bot_name);
          if hand.tell_bot_to_death.try_send(format!("Frontend says time to die!")).is_ok()
          { log::info!("{} Sent death signal to bot {}", dbgspt, bot_name);
          }
          else
          { log::error!("{} Failed to send death signal to bot {}", dbgspt, bot_name);
          }
        }
        else
        { log::error!("{} No hand for bot {}", dbgspt, bot_name);
        }
        log::debug!("{} [EXIT] - snively robot death request", dbgspt);
      }
      ,
      dsta::Snively::StopBotName(bot_name) => 
      { log::debug!("{} dr_seek got a snively::StopBotName = {:#?}", dbgspt, bot_name);
        let bot_index = match map_bots_index.get(&bot_name) 
        {   Some(idx) => *idx
          , None => 
            { log::error!("{} Could not find bot by name: {}", dbgspt, bot_name);
              return;
            }
        };

        let hand = if let Some(Some(bot_api)) = all_bot_api.get(bot_index) 
        { Some(bot_api.robot_bota_sender.clone())
        } else 
        { None
        };

        if let Some(hand) = hand
        { log::trace!("{} - telling bot to pause/stop due to snively!", dbgspt);
          match hand.tell_bot_to_pause.try_send 
          ( ()//format!("{}", bot_name)
          )
          { Ok(())=>
            { log::info!
              ( "{} dr_robo told underlying bota to pause/stop due to Snively !"
                , dbgspt
              );
            }, 
            Err(e) =>
            { log::error!("{} hand couldnt tell bot to death!, error = {:#?}", dbgspt,e);
              log::debug!("{} [EXIT] - bad snvielly", dbgspt);
              return;
              // The issue here is possibly an out-of-memory, or the channel is clogged somehow
              // But this really shouldnt be possible with rust
              // 2DO: report bad bot to deal with
            }
          }
          log::debug!("{} [EXIT] - snively robot death request", dbgspt);
          return;
        }
        else
        { log::error!("{} no bot hand at index!", dbgspt);
          log::debug!("{} [EXIT] - bad snvielly", dbgspt);
          return;
        }
      }
      ,
      dsta::Snively::SendLogNote(note) => 
      { log::debug!("{} dr_robo got a snively::SendLogNote = {:#?}", dbgspt, note);
        let msg = format!("LOG NOTE: dr robo doesn't listen to your human thoughts, snively!");
        // in the future, after comms are encrypted, we can put secret commands
        // here for fun.
        match dr_seek_api.tell_dr_snively.try_send
        ( dsta::Snively::SendLogNote
          ( msg
          )
        ) 
        { Ok(()) => 
          { log::info!
            ( "{}Told snively to log note remote"
              , dbgspt
            );
          }
          Err(tokio::sync::mpsc::error::TrySendError::Full(e)) => 
          { log::warn!
            ( "{} {} dr_seek_api.tell_dr_snively {}"
            , dbgspt
            , format!("{:#?}",e)
            , crate::glossary::ERROR_CHANNEL_FULL
            );
          }
          Err(tokio::sync::mpsc::error::TrySendError::Closed(e)) => 
          { log::error!
            ( "{}dr_seek_api.tell_dr_snively channel closed, critical error: {}"
            , dbgspt
            , format!("{:#?}",e)
            );
          }
        }
      }
      ,
      dsta::Snively::MakeBotBuzz(s) =>
      {// New Method to make bots (skips process new bot:)
        let ( mind,body ) = crate::sbot::buzz_bomber::f_make_bot_buzz_2
        ( s
        ).await;
        let new_bot = body;
        log::trace!("{}  - // Add new bot to dr_robo(ticizer) (brains, map-index, checks-map",dbgspt );
        let (hand, foot) = fn_make_bot_api_hand_and_foot();
        let new_index = all_bot_api.len();
        all_bot_api.push
        ( Some
          ( BotApi
            {   robot_brain_state : new_bot.robot_api_brain_state.clone()
              , robot_bota_sender : hand.clone()
            }
          )
        );
        map_bots_index.insert(new_bot.robot_api_brain_state.bot_name.clone(), new_index);
        all_bot_checks.push(vec![new_index]);
        log::trace!("{}  - // spawn crate::dr_bota::f_run_main_dr_bota api thread",dbgspt );
        let clone_dr_r_api = dr_robo_api.clone();
        let clone_hand = hand.clone();
        tokio::spawn
        ( async move
          { crate::dr_bota::f_run_main_dr_bota_2
            ( clone_dr_r_api // DrRoboApi // dr_r: DrRoboApi
            , new_bot // CtorBot // ctor: CtorBot
            , new_index // usize // spot: usize
            , clone_hand
            , foot
            , mind
            ).await;
          }
        );
        let _ = dr_seek_api.tell_dr_snively.try_send
        ( dsta::Snively::SendLogNote
          ( format!
            ( "Lanching stealth bot nominal from dr_robo perspective"
            )
          )
        );
      }
      ,
      dsta::Snively::MakeSallyBs(s) =>
      { log::trace!("{} => dr_robo handling dsta::Snively::MakeSallyBs = \n{:#?}", dbgspt, s);
        if let Err(e) = enorce_no_duplicate_bot_names // last check before creation
        ( &s.my_accounting.friendly_name
        , &map_bots_index
        )
        { let errmsg = format!("Couldn't make a SallyBs: {}", e);
          log::error!("{errmsg}");
          match dr_seek_api.tell_dr_snively.try_send
          ( dsta::Snively::SendLogNote
            ( format!
              ( "ERROR! dr_robo::from_dr_seek::MakeSallyBs failed : {}"
              , errmsg
              )
            )
          ) 
          { Ok(()) => 
            { log::info!
              ( "{}Told snively to log note the error"
                , dbgspt
              );
            }
            Err(tokio::sync::mpsc::error::TrySendError::Full(_)) => 
            { log::warn!
              ( "{} {} dr_seek_api.tell_dr_snively {}"
              , dbgspt
              , errmsg
              , crate::glossary::ERROR_CHANNEL_FULL
              );
            }
            Err(tokio::sync::mpsc::error::TrySendError::Closed(_)) => 
            { log::error!
              ( "{}dr_seek_api.tell_dr_snively channel closed, critical error: {}"
              , dbgspt
              , errmsg
              );
              // just log and continue, no returns/breaks/etc 
            }
          }
        }
        else
        { // New Method to make bots (skips process new bot:)
          let ( mind,body ) = crate::sbot::sally_bot::f_make_bot_sally_2
          ( s
          ).await;
          let new_bot = body;
          log::trace!("{}  - // Add new bot to dr_robo(ticizer) (brains, map-index, checks-map",dbgspt );
          let (hand, foot) = fn_make_bot_api_hand_and_foot();
          let new_index = all_bot_api.len();
          all_bot_api.push
          ( Some
            ( BotApi
              {   robot_brain_state : new_bot.robot_api_brain_state.clone()
                , robot_bota_sender : hand.clone()
              }
            )
          );
          map_bots_index.insert(new_bot.robot_api_brain_state.bot_name.clone(), new_index);
          all_bot_checks.push(vec![new_index]);
          log::trace!("{}  - // spawn crate::dr_bota::f_run_main_dr_bota api thread",dbgspt );
          let clone_dr_r_api = dr_robo_api.clone();
          let clone_hand = hand.clone();
          tokio::spawn
          ( async move
            { crate::dr_bota::f_run_main_dr_bota_2
              ( clone_dr_r_api // DrRoboApi // dr_r: DrRoboApi
              , new_bot // CtorBot // ctor: CtorBot
              , new_index // usize // spot: usize
              , clone_hand
              , foot
              , mind
              ).await;
            }
          );
          let _ = dr_seek_api.tell_dr_snively.try_send
          ( dsta::Snively::SendLogNote
            ( format!
              ( "Lanching sally bot nominal from dr_robo perspective"
              )
            )
          );
        }
      } 
      ,
      dsta::Snively::MakeStealth(s) => 
      { log::trace!("{} => dr_robo handling dsta::Snively::MakeStealth = \n{:#?}", dbgspt, s);
        if let Err(e) = enorce_no_duplicate_bot_names // last check before creation
        ( &s.my_accounting.friendly_name
        , &map_bots_index
        )
        { let errmsg = format!("Couldn't make a Stealth: {}", e);
          log::error!("{errmsg}");
          match dr_seek_api.tell_dr_snively.try_send
          ( dsta::Snively::SendLogNote
            ( format!
              ( "ERROR! dr_robo::from_dr_seek::MakeStealth failed : {}"
              , errmsg
              )
            )
          ) 
          { Ok(()) => 
            { log::info!
              ( "{}Told snively to log note the error"
                , dbgspt
              );
            }
            Err(tokio::sync::mpsc::error::TrySendError::Full(_)) => 
            { log::warn!
              ( "{} {} dr_seek_api.tell_dr_snively {}"
              , dbgspt
              , errmsg
              , crate::glossary::ERROR_CHANNEL_FULL
              );
            }
            Err(tokio::sync::mpsc::error::TrySendError::Closed(_)) => 
            { log::error!
              ( "{}dr_seek_api.tell_dr_snively channel closed, critical error: {}"
              , dbgspt
              , errmsg
              );
              // just log and continue, no returns/breaks/etc 
            }
          }
        }
        else
        { // New Method to make bots (skips process new bot:)
          let ( mind,body ) = crate::sbot::stealth_bot::f_make_bot_stealth_2
          ( s
          ).await;
          let new_bot = body;
          log::trace!("{}  - // Add new bot to dr_robo(ticizer) (brains, map-index, checks-map",dbgspt );
          let (hand, foot) = fn_make_bot_api_hand_and_foot();
          let new_index = all_bot_api.len();
          all_bot_api.push
          ( Some
            ( BotApi
              {   robot_brain_state : new_bot.robot_api_brain_state.clone()
                , robot_bota_sender : hand.clone()
              }
            )
          );
          map_bots_index.insert(new_bot.robot_api_brain_state.bot_name.clone(), new_index);
          all_bot_checks.push(vec![new_index]);
          log::trace!("{}  - // spawn crate::dr_bota::f_run_main_dr_bota api thread",dbgspt );
          let clone_dr_r_api = dr_robo_api.clone();
          let clone_hand = hand.clone();
          tokio::spawn
          ( async move
            { crate::dr_bota::f_run_main_dr_bota_2
              ( clone_dr_r_api // DrRoboApi // dr_r: DrRoboApi
              , new_bot // CtorBot // ctor: CtorBot
              , new_index // usize // spot: usize
              , clone_hand
              , foot
              , mind
              ).await;
            }
          );
          let _ = dr_seek_api.tell_dr_snively.try_send
          ( dsta::Snively::SendLogNote
            ( format!
              ( "Lanching stealth bot nominal from dr_robo perspective"
              )
            )
          );
        }
      }
      , 
      dsta::Snively::MakeSwatBot(s) =>
      { log::trace!("{} => dsta::Snively::MakeSwatBot = \n{:#?}", dbgspt, s);
        if let Err(e) = enorce_no_duplicate_bot_names // last check before creation
        ( &s.my_accounting.friendly_name
        , &map_bots_index
        )
        { let errmsg = format!("Couldn't make a MakeSwatBot: {}", e);
          log::error!("{errmsg}");
          match dr_seek_api.tell_dr_snively.try_send
          ( dsta::Snively::SendLogNote
            ( format!
              ( "ERROR! dr_robo::from_dr_seek::MakeSwatBot failed : {}"
              , errmsg
              )
            )
          ) 
          { Ok(()) => 
            { log::info!
              ( "{}Told snively to log note the error"
                , dbgspt
              );
            }
            Err(tokio::sync::mpsc::error::TrySendError::Full(_)) => 
            { log::warn!
              ( "{} {} dr_seek_api.tell_dr_snively {}"
              , dbgspt
              , errmsg
              , crate::glossary::ERROR_CHANNEL_FULL
              );
            }
            Err(tokio::sync::mpsc::error::TrySendError::Closed(_)) => 
            { log::error!
              ( "{}dr_seek_api.tell_dr_snively channel closed, critical error: {}"
              , dbgspt
              , errmsg
              );
            }
          }
        }
        else
        { // New Method to make bots (skips process new bot:)
            let ( mind,body ) = crate::sbot::swat_bot::f_make_bot_swat_2
            ( s
            , dr_robo_api // &DrRoboApi
            , None // raw bot from frontend has no swap
            , dr_seek_api.tell_dr_got_tick00.clone()
            ).await;
            let new_bot = body;
            log::trace!("{}  - // Add new bot to dr_robo(ticizer) (brains, map-index, checks-map",dbgspt );
            let (hand, foot) = fn_make_bot_api_hand_and_foot();
            let new_index = all_bot_api.len();
            all_bot_api.push
            ( Some
              ( BotApi
                {   robot_brain_state : new_bot.robot_api_brain_state.clone()
                  , robot_bota_sender : hand.clone()
                }
              )
            );
            map_bots_index.insert(new_bot.robot_api_brain_state.bot_name.clone(), new_index);
            all_bot_checks.push(vec![new_index]);
            log::trace!("{}  - // spawn crate::dr_bota::f_run_main_dr_bota api thread",dbgspt );
            let clone_dr_r_api = dr_robo_api.clone();
            let clone_hand = hand.clone();
            tokio::spawn
            ( async move
              { crate::dr_bota::f_run_main_dr_bota_2
                ( clone_dr_r_api // DrRoboApi // dr_r: DrRoboApi
                , new_bot // CtorBot // ctor: CtorBot
                , new_index // usize // spot: usize
                , clone_hand
                , foot
                , mind
                ).await;
              }
            );
            let _ = dr_seek_api.tell_dr_snively.try_send
            ( dsta::Snively::SendLogNote
              ( format!
                ( "Lanching user's SWAT bot nominal from dr_robo perspective"
                )
              )
            );
          // Note, we should make all bots start like this
        }
      } 
      _ =>
      { log::error!("{} Bad format", dbgspt);
        log::debug!("{} [EXIT] - bad snively", dbgspt);
        return;
      }
    }
    log::debug!("{} [EXIT OK] process_a_snively done", dbgspt);
  }

  // basic checks are enforced at dr_seek 
  fn enorce_no_duplicate_bot_names
  ( bot_name : &str
  , map_bots_index: &std::collections::HashMap<String, usize>
  ) -> Result<(),String>
  { let dbgspt = format!("[{}]", crate::glossary::LOG_PREFIX_DR_ROBO_NAME_OK ) ;
    log::debug!("{}  - // Enorce no duplicate bot names",dbgspt );
    log::trace!("{}  - Enforce short bot name",dbgspt );
    if bot_name.len() < crate::glossary::MIN_BOT_NAME_LENGTH
    { let errmsg = format!
      ( "{} {}, {} letters minimum"
      , dbgspt
      , crate::glossary::ERROR_ROBOT_NAME_TOO_SMALL
      , crate::glossary::MIN_BOT_NAME_LENGTH
      );
      Err(errmsg)
    }
    else if bot_name.len() > crate::glossary::MAX_BOT_NAME_LENGTH
    { let errmsg = format!
      ( "{} {}, {} bytes max"
      , dbgspt
      , crate::glossary::ERROR_ROBOT_NAME_TOO_BIG
      , crate::glossary::MAX_BOT_NAME_LENGTH
      );
      Err(errmsg)
    }
    else if let Some(idx) = map_bots_index.get(bot_name) 
    { let errmsg = format!("{} Robot with name {} already exists at index {}", dbgspt, bot_name, idx);
      Err(errmsg)
    } 
    else
    { Ok(())
    }
  }
}

pub(crate) mod from_bota 
{ use crate::*;

  fn delete_bot_completely(
    bot_index: usize,
    all_bot_api: &mut Vec<Option<BotApi>>,
    map_bots_index: &mut std::collections::HashMap<String, usize>,
    all_bot_checks: &mut Vec<Vec<usize>>,
  )
  { let dbgspt = format!("[DR_ROBO_DELETE].{}", bot_index);
    log::debug!("{} [ENTRY] delete_bot_completely()", dbgspt);

    log::trace!("{} marking bot as None in all_bot_api (if in bounds)", dbgspt);
    let bot_name = if bot_index < all_bot_api.len()
    {
      match &all_bot_api[bot_index]
      {
        Some(bot) =>
        {
          let name = bot.robot_brain_state.bot_name.clone();
          log::info!("{} deleting bot: {}", dbgspt, name);
          all_bot_api[bot_index] = None;
          Some(name)
        }
        None =>
        {
          log::warn!("{} no bot at index {} (already deleted or never existed)", dbgspt, bot_index);
          None
        }
      }
    }
    else
    {
      log::warn!("{} bot_index {} out of bounds for all_bot_api (len={})", dbgspt, bot_index, all_bot_api.len());
      None
    };

    log::trace!("{} removing from name→index map (if bot exists)", dbgspt);
    if let Some(name) = &bot_name
    {
      map_bots_index.remove(name);
    }

    log::trace!("{} clearing subscribers for this bot (if in bounds)", dbgspt);
    if bot_index < all_bot_checks.len()
    {
      if let Some(check_vec) = all_bot_checks.get_mut(bot_index)
      {
        check_vec.clear();
      }
    }
    else
    {
      log::warn!("{} bot_index {} out of bounds for all_bot_checks (len={})", dbgspt, bot_index, all_bot_checks.len());
    }

    log::trace!("{} removing this bot from all other subscriber lists", dbgspt);
    for check_vec in all_bot_checks.iter_mut()
    {
      check_vec.retain(|&id| id != bot_index);
    }

    if let Some(name) = bot_name
    {
      log::info!("{} bot {} fully deleted (name, index, checks)", dbgspt, name);
    }

    log::debug!("{} [EXIT] delete_bot_completely()", dbgspt);
  }

  /// This will create a new trade over at the ttai bot
  /// and it will first cancel any existing orders; and
  /// it cancels current order if the order has no legs
  pub async fn process_pls_trade
  (
      bot_ind_ord: (usize, u64),
      order: dsta::Order,
      all_bot_api: &mut Vec<Option<BotApi>>,
      dr_bot0_com: &tokio::sync::mpsc::Sender<dr_bot0::NewOrderRequest>,
      dr_bot0_dryrun: &tokio::sync::mpsc::Sender<dr_bot0::DryRunRequest>,     // NEW: added
      dr_bot0_cancel: &tokio::sync::mpsc::Sender<dr_bot0::CancelOrderRequest>, // NEW: added
      dr_robo_api: &DrRoboApi,
      asset_infos: &std::collections::HashMap<String, dsta::FinAss>,
      map_bots_index: &std::collections::HashMap<String, usize>,               // NEW: needed for CASH lookup
  ) -> () {
      let dbgspt = format!("[{}.{:?}] : ", crate::glossary::LOG_PREFIX_DR_ROBO_ORD_REQ, bot_ind_ord);
      let bot_index = bot_ind_ord.0;

      log::debug!("ENTRY! {} - dr_robo::from_bota::process_pls_trade, order: {:#?}", dbgspt, order);
      log::info!(
          "{} dr_robo::from_ttai impl of mpsc rx told_dr_pls_trade recvd from bot {:#?}, \n the order = {:#?}",
          dbgspt, bot_index, order
      );

      // ────────────────────────────────────────────────────────────────
      // CASE 1: This is a DONE trade reported back from bot0 (index 0)
      // ────────────────────────────────────────────────────────────────
      if bot_index == 0 {
          // Your existing logic for handling completed/filled orders from bot0
          // (unchanged — I'm keeping it exactly as you had it)
          log::debug!("DR ROBO processing a done trade");
          let biohash = bot_ind_ord.1;
          let bot_index = biohash & crate::glossary::MAX_BOT_INDEX;
          let order_num = (biohash >> crate::glossary::DEFAULT_BIOHASH_ORDER_NUM_SHIFT) & crate::glossary::MAX_ORDER_NUMBER;

          log::debug!("ALERT! {} - bot_ind_ord decomposes to index {} order_num {}", dbgspt, bot_index, order_num);

          let the_bot_trading = match all_bot_api.get_mut(bot_index as usize) {
              Some(Some(bot)) => bot,
              Some(None) => {
                  log::warn!("{}Inactive bot index {} for pls_trade", dbgspt, bot_index);
                  return;
              }
              None => {
                  log::warn!("{}Invalid bot index {} for pls_trade", dbgspt, bot_index);
                  return;
              }
          };

          let storage_safe = &mut the_bot_trading.robot_brain_state.bot_safe;
          let ass = match crate::dr_robo::fn_get_account_shennanagans(&order, &storage_safe, &asset_infos) {
              Ok(acc) => acc,
              Err(e) => {
                  log::error!("FATAL ERROR! account shennangians failed!, error={:#?}", e);
                  return;
              }
          };

          if let Err(wat) = crate::dr_robo::apply_order_to_account(storage_safe, &order, &asset_infos) {
              log::error!("FATAL ERROR! order apply to account failed! - {:#?}", wat);
              return;
          }

          crate::dr_robo::apply_account_cash_and_collateral_changes(storage_safe, &ass);

          if the_bot_trading.robot_brain_state.num_swap == order_num {
              let new_state_reporting = BotMindState::Running;
              the_bot_trading.robot_brain_state.bot_mind = new_state_reporting;
              let _ = dr_robo_api.tell_dr_new_brain.send((bot_index as usize, new_state_reporting)).await;
          } else {
              log::warn!("BOT REMAINS IN TRADING mind state due to detected new order number");
          }

          return;
      }

      // ────────────────────────────────────────────────────────────────
      // CASE 2: This is a request from a real bot (not bot0)
      // ────────────────────────────────────────────────────────────────
      log::debug!("DR ROBO processing a new trade request from bot {}", bot_index);

      // Find CASH bot index
      let cash_name = crate::glossary::USER_CASH_BOT_NAME;
      let cash_idx = match map_bots_index.get(cash_name) {
          Some(&idx) => idx,
          None => {
              log::error!("{} CASH bot '{}' not found in map_bots_index", dbgspt, cash_name);
              return;
          }
      };

      // We need two mutable references — use split_at_mut to satisfy the borrow checker
      let (the_bot_trading, cash_bot) = if bot_index == cash_idx {
          // Should never happen — log and bail
          log::error!("{} BUG: bot_index {} == cash_idx {} — cannot proceed", dbgspt, bot_index, cash_idx);
          return;
      } else if bot_index < cash_idx {
          // bot_index is before cash_idx → split at cash_idx
          let (left, right) = all_bot_api.split_at_mut(cash_idx);
          
          match (left.get_mut(bot_index), right.get_mut(0)) {
              (Some(Some(trading)), Some(Some(cash))) => (trading, cash),
              _ => {
                  log::error!("{} One or both bots missing/inactive (trading @ {}, cash @ {})", 
                              dbgspt, bot_index, cash_idx);
                  return;
              }
          }
      } else {
          // cash_idx is before bot_index → split at bot_index
          let (left, right) = all_bot_api.split_at_mut(bot_index);
          
          match (right.get_mut(0), left.get_mut(cash_idx)) {
              (Some(Some(trading)), Some(Some(cash))) => (trading, cash),
              _ => {
                  log::error!("{} One or both bots missing/inactive (trading @ {}, cash @ {})", 
                              dbgspt, bot_index, cash_idx);
                  return;
              }
          }
      };

      // If already trading → cancel current order first
      if the_bot_trading.robot_brain_state.bot_mind == BotMindState::Trading 
      {
          log::warn!(
              "{} Bot {} is already Trading — sending cancel before new order",
              dbgspt, bot_index
          );

          let current_order_num = the_bot_trading.robot_brain_state.num_swap;

          let (cancel_reply_tx, cancel_reply_rx) = tokio::sync::oneshot::channel::<Result<bool, Box<dyn std::error::Error + Send + Sync>>>();

          let cancel_req = dr_bot0::CancelOrderRequest {
              bot_api_order_bot: (bot_index, current_order_num),
              bot_api_named_bot: the_bot_trading.robot_brain_state.bot_name.clone(),
              tell_bot_it_sucks: the_bot_trading.robot_bota_sender.tell_bot_it_sucks.clone(),
              order_hash: current_order_num, // assuming order_hash matches num_swap
              reply_tx : cancel_reply_tx
          };

          if let Err(e) = dr_bot0_cancel.send(cancel_req).await {
              log::error!("{} Failed to send cancel request to bot0: {}", dbgspt, e);
              let _ = the_bot_trading.robot_bota_sender.tell_bot_it_sucks
                  .send("`Failed to send cancel request`".to_string())
                  .await;
              return;
          }

          // Wait for cancel confirmation (with timeout)
          match tokio::time::timeout(std::time::Duration::from_secs(8), cancel_reply_rx).await {
              Ok(Ok(Ok(true))) => {
                  log::info!("{} Cancel confirmed for bot {} order {}", dbgspt, bot_index, current_order_num);
                  // Now safe to exit Trading state
                  the_bot_trading.robot_brain_state.bot_mind = BotMindState::Running;
                  let _ = dr_robo_api.tell_dr_new_brain.send((bot_index, BotMindState::Running)).await;
              }
              _ => {
                  log::warn!("{} Cancel not confirmed in time for bot {} — proceeding anyway", dbgspt, bot_index);
                  // You can choose: block, retry, or proceed with risk
              }
          }
      }

    // If the incoming order is empty → it's just a cancel request (already handled above)
    if order.order_legs.is_empty() {
        log::debug!("{} Received explicit empty order → cancel only, done.", dbgspt);
        return;
    }

    // ────────────────────────────────────────────────────────────────
    // Real order: dry-run → check cash → deduct fees → submit
    // ────────────────────────────────────────────────────────────────
    log::info!("{} Starting dry-run for real order from bot {}", dbgspt, bot_index);

    let (dry_reply_tx, dry_reply_rx) = tokio::sync::oneshot::channel::<Result<f64, Box<dyn std::error::Error + Send + Sync>>>();

    let dry_req = dr_bot0::DryRunRequest 
    {
        bot_api_order_bot: (bot_index, the_bot_trading.robot_brain_state.num_swap + 1),
        bot_api_named_bot: the_bot_trading.robot_brain_state.bot_name.clone(),
        tell_bot_it_sucks: the_bot_trading.robot_bota_sender.tell_bot_it_sucks.clone(),
        bot_order_request: order.clone(),
        reply_tx : dry_reply_tx
    };

    if let Err(e) = dr_bot0_dryrun.send(dry_req).await 
    { log::error!("{} Failed to send dry-run request: {}", dbgspt, e);
      let _ = the_bot_trading.robot_bota_sender.tell_bot_it_sucks
      .send(format!("{}", crate::glossary::ORDER_BAD))
      .await;
        return;
    }

    let total_fees = match tokio::time::timeout
    ( std::time::Duration::from_secs(7)
    , dry_reply_rx
    ).await 
    { Ok(Ok(Ok(fees))) if fees >= 0.0 => 
      { log::info!("{} OK DRY RUN - fees = {}", dbgspt, fees);
        fees
      },
      _ => 
      { if let Err(e) = the_bot_trading.robot_bota_sender.tell_bot_it_sucks
          .send(format!("{}", crate::glossary::ORDER_BAD))
          .await
        { log::error!("{} Failed to notify bot {} that dry-run failed: {}", dbgspt, bot_index, e);
        }
        else
        { log::error!("{} Dry-run failed — cannot proceed with order", dbgspt);
        }
        return;
      }
    };

    log::info!("{} Dry-run successful — estimated total fees: ${:.2}", dbgspt, total_fees);

    if cash_bot.robot_brain_state.bot_safe.m_cash < total_fees 
    { log::warn!
      ( "{} Insufficient cash in CASH bot: {:.2} < {:.2} required",
        dbgspt, cash_bot.robot_brain_state.bot_safe.m_cash, total_fees
      );
      if let Err(e) = the_bot_trading.robot_bota_sender.tell_bot_it_sucks
        .send(format!("{}", crate::glossary::ORDER_BAD))
        .await
      { log::error!("{} Failed to notify bot {} that dry-run failed: {}", dbgspt, bot_index, e);
      }
      return;
    }

    // Deduct fees
    cash_bot.robot_brain_state.bot_safe.m_cash -= total_fees;
    log::info!(
        "{} Deducted ${:.2} fees from CASH bot (remaining: ${:.2})",
        dbgspt, total_fees, cash_bot.robot_brain_state.bot_safe.m_cash
    );

    // ────────────────────────────────────────────────────────────────
    // All checks passed → proceed with real order submission
    // ────────────────────────────────────────────────────────────────
    let new_state_reporting = BotMindState::Trading;
    the_bot_trading.robot_brain_state.bot_mind = new_state_reporting;
    the_bot_trading.robot_brain_state.num_swap += 1;
    let order_num = the_bot_trading.robot_brain_state.num_swap;
    log::info!
    ( "NOTE: order ID consistency check: input={}, calculated={};  Bot {} Trade {} Commencing!",
      bot_ind_ord.1, order_num, bot_index, order_num
    );

    // Your existing account check (shenanigans)
    let _ass = match crate::dr_robo::fn_get_account_shennanagans
    ( &order,
      &the_bot_trading.robot_brain_state.bot_safe,
      &asset_infos,
    ) 
    { Ok(acc) => acc,
      Err(e) => 
      { let errmsg = format!("{} fn_get_account_shennanagans FAILED: {}", dbgspt, e);
        let new_state_reporting = BotMindState::Stopped;
        the_bot_trading.robot_brain_state.bot_mind = new_state_reporting;
        log::error!("{}", errmsg);
        let _ = dr_robo_api.tell_dr_new_brain.send((bot_index, new_state_reporting)).await;
        if let Err(e) = the_bot_trading.robot_bota_sender.tell_bot_it_sucks
          .send(format!("{}", crate::glossary::ORDER_BAD))
          .await
        { log::error!("{} Failed to notify bot {} that order is in flight: {}", dbgspt, bot_index, e);
        }
        return;
      }
    };

    if let Err(e) = the_bot_trading.robot_bota_sender.tell_bot_it_sucks
      .send(format!("{}", crate::glossary::ORDER_FLY))
      .await
    { log::error!("{} Failed to notify bot {} that order is in flight: {}", dbgspt, bot_index, e);
      return;
    }

    // Send real order to bot0
    let mut has_err = false;
    match dr_bot0_com.try_send
    ( dr_bot0::NewOrderRequest 
      { bot_api_order_bot: (bot_index, order_num),
        bot_api_named_bot: the_bot_trading.robot_brain_state.bot_name.clone(),
        tell_bot_it_sucks: the_bot_trading.robot_bota_sender.tell_bot_it_sucks.clone(),
        bot_order_request: order,
        time_until_cancel: None,
      }
    ) 
    { Ok(()) => 
      { log::info!("{} Successfully sent real order to bot0", dbgspt);
      }
      Err(tokio::sync::mpsc::error::TrySendError::Full(_)) => 
      { log::warn!("{} dr_bot0_com channel full", dbgspt);
        has_err = true;
      }
      Err(tokio::sync::mpsc::error::TrySendError::Closed(_)) => 
      { log::error!("{} dr_bot0_com channel closed", dbgspt);
        has_err = true;
      }
    }

    let final_state = if has_err 
    { if let Err(e) = the_bot_trading.robot_bota_sender.tell_bot_it_sucks
        .send(format!("{}", crate::glossary::ORDER_BAD))
        .await
      { log::error!("{} Failed to notify bot {} that order is in flight: {}", dbgspt, bot_index, e);
        BotMindState::Stopped
      }
      else 
      { BotMindState::Running 
      }
    } 
    else 
    { BotMindState::Trading
    };
    let _ = dr_robo_api.tell_dr_new_brain.send((bot_index, final_state)).await;
    log::debug!("EXIT! {} - Order processing completed for bot {}", dbgspt, bot_index);
  }

  pub async fn process_new_brain
  (
    bot_index: usize,
    mind_state: BotMindState,
    all_bot_api: &mut Vec<Option<BotApi>>,
    all_bot_checks: &mut Vec<Vec<usize>>,
    map_bots_index: &mut std::collections::HashMap<String, usize>,
    dr_robo_api : &DrRoboApi
  ) -> ()
  { let dbgspt = format!("[{}.{}] : "
    , crate::glossary::LOG_PREFIX_DR_ROBO_N_BRAIN
    , bot_index
    );
    log::debug!
    ( "{} ENTRY! - dr_robo::from_bota::process_new_brain, mind_state: {:#?}"
    , dbgspt
    , mind_state
    );
    
    // Handle death states: Seppuku or Zombied
    if mind_state == BotMindState::Seppuku || mind_state == BotMindState::Zombied
    { log::trace!("{} mind state is a near death state!", dbgspt);
      if bot_index == 0
      { unimplemented!("{} dr_bot0 should never die", dbgspt); // TODO: warn in release, throw in debug
      }
      let bot = match all_bot_api.get(bot_index)
      { Some(Some(b)) => b,
        _ =>
        { log::warn!("{} Bot already gone when reporting death state", dbgspt);
          return;
        }
      };

      let account = &bot.robot_brain_state.bot_safe;
      let bot_name = bot.robot_brain_state.bot_name.clone();

      // Check if assets are truly empty (ignore tiny floating point dust)
      let assets_empty = account.m_stock.is_none()
        && account.m_long_calls.is_none()
        && account.m_long_puts.is_none()
        && account.m_short_calls.is_none()
        && account.m_short_puts.is_none()
        && account.m_call_credit_spread_lein.abs() < 0.05
        && account.m_put_credit_spread_lein.abs() < 0.05
        && account.m_short_iron_condor_lein.abs() < 0.05
        && account.m_short_put_lein.abs() < 0.05
      ;
      let cash_negligible = account.m_cash.abs() < 0.05;
      if assets_empty // empty cash + delete
      { log::trace!("{} Bot Account Completely empty → transfer any leftover cash and delete",dbgspt);
        if !cash_negligible
        { log::info!("{} Transferring remaining cash {:.2} from {} to {}"
          , dbgspt, account.m_cash, bot_name, crate::glossary::USER_CASH_BOT_NAME
          );
          let cash_swap = BotSwaper
          { source_offering: Some
            ( SwapWat
              { who: bot_name.clone(),
                wat: dsta::PositionsList::default(),
                how: account.m_cash,
              }
            )
            ,
            target_released: Some
            ( SwapWat
              { who: format!("{}", crate::glossary::USER_CASH_BOT_NAME),
                wat: dsta::PositionsList::default(),
                how: crate::glossary::TOUCH_NO_HOW_WHO_IN_A_SWAP,
              }
            ),
          };
          let _ = dr_robo_api.tell_dr_swap_info.send((0, cash_swap)).await;
        }

        log::info!("{} Bot {} is completely empty – deleting permanently", dbgspt, bot_name);
        delete_bot_completely(bot_index, all_bot_api, map_bots_index, all_bot_checks);
        return;
      }

      // Still has assets and reported Zombied → disperse everything to sink bots
      if mind_state == BotMindState::Zombied
      {
          log::info!("{} Bot {} reported Zombied — initiating asset dispersal", dbgspt, bot_name);

          let account = &bot.robot_brain_state.bot_safe;
          let mut swaps = Vec::new();

          // Helper to queue a one-way transfer to a sink bot
          let mut queue_transfer = 
          | target_name: String
          , positions: std::collections::HashMap<String, dsta::OwnedPosition>
          , cash: f64
          | 
          { swaps.push
            ( BotSwaper 
              { source_offering: Some(SwapWat
                { who: bot_name.clone(),
                  wat: dsta::PositionsList { positions },
                  how: cash,
                  }
                ),
                target_released: Some(SwapWat 
                { who: target_name,
                  wat: dsta::PositionsList::default(),
                  how: crate::glossary::TOUCH_NO_HOW_WHO_IN_A_SWAP,
                }),
              }
            );
          };

          // 1. Cash → USER_CASH
          if account.m_cash.abs() > 0.05 
          { let pos_map = std::collections::HashMap::new();
            queue_transfer
            ( format!("{}", crate::glossary::USER_CASH_BOT_NAME)
            , pos_map // we don't care about positions for cash bot
            , account.m_cash
            );
          }

          // 2. Build position map for all owned positions
          let mut all_positions = std::collections::HashMap::new();

          macro_rules! insert_pos 
          { ($opt:expr) => 
            { if let Some(pos) = $opt 
              { all_positions.insert
                ( pos.asset.order_name() 
                , pos.clone()
                );
              }
            };
          }

          insert_pos!(&account.m_stock);
          insert_pos!(&account.m_long_calls);
          insert_pos!(&account.m_long_puts);
          insert_pos!(&account.m_short_calls);
          insert_pos!(&account.m_short_puts);

          // 3. Decide dispersal strategy
          let has_leins = !crate::core::can_safely_disperse(&account);
          if has_leins || !all_positions.is_empty() && account.m_stock.is_some() {
              // Complex/entangled positions → dump everything into a tomb bot
              let tomb_name = format!("USER_DEAD_{}", bot_name);
              queue_transfer(tomb_name, all_positions, 0.0);
              log::warn!("{} Complex account state → dumping to tomb bot USER_DEAD_{}", dbgspt, bot_name);
          } else {
              // Clean positions → disperse individually
              for (ticker, owned_position) in all_positions {
                  let target = format!("USER_{}", ticker);
                  let mut pos_map = std::collections::HashMap::new();
                  pos_map.insert(ticker, owned_position);
                  queue_transfer(target, pos_map, 0.0);
              }
          }

          // 4. Execute all queued magic swaps
          for swap in &swaps {
              let _ = dr_robo_api.tell_dr_swap_info.send((0, swap.clone())).await;
          }

          // 5. Tell frontend
          let _ = dr_robo_api.tell_dr_a_snively.send((
              0,
              dsta::Snively::SendLogNote(format!(
                  "Zombied bot '{}' dispersed assets ({} transfers)",
                  bot_name,
                  swaps.len()
              )),
          )).await;

          // 6. Encourage final cleanup (in case swaps fail or take time)
          if let Some(Some(b)) = all_bot_api.get(bot_index) {
              let _ = b.robot_bota_sender.tell_bot_to_death.try_send("Cleanup after dispersal".to_string());
          }

          return;
      }
    }
 
    if bot_index == 0
    { log::warn!("{} Bot Zero brain update - usually something special",dbgspt);
        match all_bot_api.get_mut(0) 
        { Some(Some(sub_bot)) => 
          { sub_bot.robot_brain_state.bot_mind = mind_state;
            match sub_bot.robot_bota_sender.tell_bot_to_check.try_send
            ( sub_bot.robot_brain_state.clone()
            )
            { Ok(()) => 
              { log::info!
                ( "{} Told sub_bot ZERO to check the new robot brain for {}!"
                  , dbgspt, 0
                );
              }
              Err(tokio::sync::mpsc::error::TrySendError::Full(_)) => 
              { log::warn!
                ( "{}sub_bot ZERO.tell_bot_to_check channel full, error log not sent for {}"
                , dbgspt, 0
                );
              }

              Err(tokio::sync::mpsc::error::TrySendError::Closed(_)) => 
              { log::error!
                ( "{}sub_bot ZERO.tell_bot_to_check channel closed, critical error: (HUMU: dont exit from the loop but do admin stuff with the thingy), for {}"
                , dbgspt, 0
                );
              }
            }
          }
          Some(None) => 
          { // TODO: warn in release, unreachable! in debug:
            log::warn!("{} Impossible code for bot ZERO {}", dbgspt, 0);
          }
          None => 
          { log::warn!("{} Impossible CODE for bot zero {}", dbgspt, 0);
          }
        }
        log::debug!("EXIT! {} - Awkward bot index 0, we alerted bot0 about itself though.", dbgspt);
        return;
      }
      log::trace!("{} Fetch and update bot (first mutable borrow)", dbgspt);
      let (bot_name, bot_safe) = match all_bot_api.get_mut(bot_index) 
      { Some(Some(bot)) => 
        { // Send message to bot
          let _ = bot.robot_bota_sender.tell_bot_it_sucks
            .send(format!("`{}Got ya brain!", dbgspt))
            .await;

          // Perform debug validity checks
          #[cfg(debug_assertions)]
          { let dbgspt2 = format!("{} DVCTMBUL", dbgspt);
            if map_bots_index.get(&bot.robot_brain_state.bot_name) != Some(&bot_index) 
            {
              let errmsg = format!
              ( "{}Index mismatch for bot {}: expected {}, found {:?}"
                , dbgspt2
                , bot.robot_brain_state.bot_name
                , bot_index
                , map_bots_index.get(&bot.robot_brain_state.bot_name)
              );
              log::error!("{}", errmsg);
              log::warn!("EARLY EXIT! {} - {}", dbgspt2, errmsg);
              return;
            }
          }

          // Update bot mind
          log::info!(
              "{}Updating bot_mind at index {} from {:?} to {:?}",
              dbgspt,
              bot_index,
              bot.robot_brain_state.bot_mind,
              mind_state
          );
          bot.robot_brain_state.bot_mind = mind_state;

          // get clone of data for updating subscribers
          let bot_name = bot.robot_brain_state.bot_name.clone();
          let bot_safe = bot.robot_brain_state.clone();
          (bot_name, bot_safe)
        }

        Some(None) => 
        { log::warn!("{}Inactive bot index {} for new_brain", dbgspt, bot_index);
          log::debug!("EXIT! {} - Inactive bot index {}", dbgspt, bot_index);
          return;
        }

        None => 
        { log::warn!("{}Invalid bot index {} for new_brain", dbgspt, bot_index);
          log::debug!("EXIT! {} - Invalid bot index {}", dbgspt, bot_index);
          return;
        }
      };

      log::trace!("{} // Notify subscribers (second mutable borrow)", dbgspt);
      match all_bot_checks.get(bot_index) 
      { Some(check_vec) => 
        { for &sub_bot_index in check_vec.iter() 
            { match all_bot_api.get_mut(sub_bot_index) 
              { Some(Some(sub_bot)) => 
                { match sub_bot.robot_bota_sender.tell_bot_to_check.try_send(bot_safe.clone()) 
                  { Ok(()) => 
                    { log::info!
                      ( "{} Told sub_bot to check the new robot brain for {}!"
                        , dbgspt, bot_name
                      );
                    }
                    Err(tokio::sync::mpsc::error::TrySendError::Full(_)) => 
                    { log::warn!
                      ( "{}sub_bot.tell_bot_to_check channel full, error log not sent for {}"
                      , dbgspt, bot_name
                      );
                    }

                    Err(tokio::sync::mpsc::error::TrySendError::Closed(_)) => 
                    { log::error!
                      ( "{}sub_bot.tell_bot_to_check channel closed, critical error: (HUMU: dont exit from the loop but do admin stuff with the thingy), for {}"
                      , dbgspt, bot_name
                      );
                    }
                  }
                }
                Some(None) => 
                { log::warn!("{}Inactive subscriber bot index {}", dbgspt, sub_bot_index);
                }
                None => 
                { log::warn!("{}Invalid subscriber bot index {}", dbgspt, sub_bot_index);
                }
              }
            }
        }
        None => 
        { let errmsg = format!("{} {} {}", dbgspt, crate::glossary::ERROR_NO_TICK_VEC, bot_index);
          log::error!("{}", errmsg);
          log::debug!("EXIT! {} - {}", dbgspt, errmsg);
          return;
        }
      }
      log::debug!("EXIT! {} - Bot mind updated for index {}", dbgspt, bot_index);
    }

  pub async fn process_pls_ticks
  (   bot_index: usize
    , ticker: String
    , all_bot_api: &Vec<Option<BotApi>>
    , map_tick_looks: &mut std::collections::HashMap<String, Vec<usize>>
    , tell_dr_add_ttai_tik : &tokio::sync::mpsc::Sender
      < ( String // tick name
          , u64 // subscriber ID
          , tokio::sync::mpsc::Sender<dsta::Ticker> 
          , tokio::sync::mpsc::Sender<Option<dsta::PushTickerResult>>
        )
      >
    , tell_tick_result : tokio::sync::mpsc::Sender<Option<dsta::PushTickerResult>>
    , asset_infos: &mut std::collections::HashMap<String, dsta::FinAss>
    , dr_seek_api: &crate::DrSeekApi
  ) -> ()
  { let dbgspt = format!("[{}.{}] : "
    , crate::glossary::LOG_PREFIX_DR_ROBO_REQ_TIK
    , bot_index);
    log::debug!
    ( "ENTRY! dr_robo::from_bota::process_pls_ticks == {} = impl of mpsc rx told_dr_pls_ticks recvd from bot {:#?}, \n the sub = {:#?}"
    , dbgspt
    , bot_index
    , ticker
    );
    log::trace!("{} - Fetching Bot...", dbgspt);
    let the_bot = match all_bot_api.get(bot_index) 
    {   Some(Some(bot)) => bot
      , Some(None) => 
      { log::warn!("{}Inactive bot index {} for pls_ticks", dbgspt, bot_index);
        log::debug!("EXIT! {} - Inactive bot index {}", dbgspt, bot_index);
        return;
      }
      None => 
      { log::warn!("{}Invalid bot index {} for pls_ticks", dbgspt, bot_index);
        log::debug!("EXIT! {} - Invalid bot index {}", dbgspt, bot_index);
        return;
      }
    };

    log::trace!("{} - Alerting bot the ticker is subscribing...", dbgspt);
    let _ = the_bot.robot_bota_sender.tell_bot_it_sucks.send
    ( format!("{}Tick Subscribing!", dbgspt)
    );
    
    log::trace!("{} - Adding ticker+bot to robo's subscribe list...", dbgspt);
    let tick_vec = map_tick_looks.entry(ticker.clone()).or_insert_with(Vec::new);
    if !tick_vec.contains(&bot_index) 
    { tick_vec.push(bot_index);
    }
    else
    { log::warn!("Bot is already subscribed to tick@");
    }

    log::trace!("{} - Adding ticker+bot to ttai's subscribe list...", dbgspt);
    let mut hijack = tokio::sync::mpsc::channel::<Option<dsta::PushTickerResult>>
    ( crate::glossary::CHANNEL_BUFFER_SIZE_TICK_RESULT
    );
    match tell_dr_add_ttai_tik.try_send
    ( ( ticker // symbol
      , bot_index as u64 //sub id
      , the_bot.robot_bota_sender.tell_bot_to_trade.clone() // sender
      , hijack.0 // tell_tick_result //the_bot.robot_bota_sender. tell_dr_add_ttai_tik.2
      )
    )
    { Ok(()) => 
      { log::info!
        ( "{} successs tell_dr_add_ttai_tik !"
          , dbgspt
        );
      }
      Err(tokio::sync::mpsc::error::TrySendError::Full(_)) => 
      { log::warn!
        ( "{}tell_dr_add_ttai_tik channel full, error log not sent"
        , dbgspt
        );
        return;
      }
      Err(tokio::sync::mpsc::error::TrySendError::Closed(_)) => 
      { log::error!
        ( "{}tell_dr_add_ttai_tik channel closed, critical error: (HUMU: dont exit from the loop but do admin stuff with the thingy)"
        , dbgspt
        );
        return;
      }
    };

    // get clone res, pass to the hijack
    let info : dsta::PushTickerResult = 
    match hijack.1.recv().await
    { Some(res) =>
      { let ans = if let Some(ref info) = res
        { info.clone()
        }
        else // bad reply
        { let _ = tell_tick_result.send(res).await;
          return;
        };
        
        let _ = tell_tick_result.send(res).await;
        ans
      }
    , _ =>
      { log::error!("{} dr_robo failed to recieve ttai push ticker result...", dbgspt);
        // 2DO: report a ttai error
        return;
      }
    };


    log::debug!("{} Dr Robo sending snively the result clone - {:#?}", dbgspt, info);
    let snvmsg = dsta::Snively::EmeraldInfo(info.clone());
    match dr_seek_api.tell_dr_snively.try_send
    ( snvmsg
    ) 
    { Ok(()) => 
      { log::info!
        ( "{} dr_robo sent dr_seek a snively to send "
          , dbgspt
        );
      }
      Err(tokio::sync::mpsc::error::TrySendError::Full(_)) => 
      { log::warn!
        ( "{} FAILED! [{}] - dr_robo try send dr_seek a snively: {:#?}"
        , dbgspt
        , crate::glossary::ERROR_CHANNEL_FULL
        , dsta::Snively::EmeraldInfo(info.clone())
        );
      }
      Err(tokio::sync::mpsc::error::TrySendError::Closed(_)) => 
      { log::error!
        ( "{} FAILED! [Channel Closed] - dr_robo try send dr_seek a snively: {:#?}"
        , dbgspt
        , dsta::Snively::EmeraldInfo(info.clone())
        );
      }
      // just log and continue if we have comms errors
    }

    // Add the ticker info to the asset_infos
    if let Some(asset_info) = asset_infos.get(&info.ass_deets.order_name())
    { log::debug!("{} ticker asset info is pre-existing! = {:#?}", dbgspt, asset_info);
      // though we may want to update it...
    }
    else
    { log::debug!("{} Inserting new asset infos in Dr Robo!",dbgspt);
      asset_infos.insert
      ( info.ass_deets.order_name()
      , info.ass_deets.clone()
      );
      for elem in info.opt_chain.into_iter()
      { for put in elem.option_puts.into_iter()
        { asset_infos.insert
          ( put.option_name.clone() // order_name() for option
          , dsta::FinAss::OptDeets(put)
          );
        }
        for cal in elem.option_calls.into_iter()
        { asset_infos.insert
          ( cal.option_name.clone() // order_name() for option
          , dsta::FinAss::OptDeets(cal)
          );
        }
      }
    }
    
    log::debug!("EXIT! {} - Ticker subscription added for bot index {}", dbgspt, bot_index);
  }

  pub async fn process_a_pls_check
  (   bot_index: usize
    , the_sub: String
    , all_bot_api: &Vec<Option<BotApi>>
    , all_bot_checks: &mut Vec<Vec<usize>>
    , map_bots_index: &std::collections::HashMap<String, usize>
  ) -> ()
  { let dbgspt = format!("[{}.{}] : "
    , crate::glossary::LOG_PREFIX_DR_ROBO_REQ_ACT, bot_index);
    log::debug!
    ( "ENTRY! dr_robo::from_bota::process_a_pls_check == {} impl of mpsc rx told_dr_pls_check recvd from bot {:#?}, \n the sub = {:#?}"
      , dbgspt
      , bot_index
      , the_sub
    );

    log::trace!("{} Fetch bot", dbgspt );
    let the_bot = match all_bot_api.get(bot_index) 
    {   Some(Some(bot)) => bot
      , Some(None) => 
        { log::warn!("{} EXIT! IMPOSSIBLE Inactive bot index {} for pls_check", dbgspt, bot_index);
          return;
        }
        None => 
        { log::warn!("{} EXIT! IMPOSSIBLE Invalid bot index {} for pls_check", dbgspt, bot_index);
          return;
        }
    };

    log::trace!("{} Inform bot of check hooking", dbgspt);
    let _ = the_bot.robot_bota_sender.tell_bot_it_sucks.send
    ( format!("`{}Subin' ya to the_sub!", dbgspt)
    ).await;
    
    // Find or create check vector and add bot
    let sub_index = map_bots_index.get(&the_sub).copied();
    let check_vec = match sub_index 
    {   Some(idx) => all_bot_checks.get_mut(idx)
      , None => None
    };
    match check_vec 
    { Some(vec) => 
      { if !vec.contains(&bot_index) 
        { vec.push(bot_index);
        }
      }
      None => 
      { let errmsg = format!("{}ERROR! told_dr_pls_check No check vector found for bot {}", dbgspt, the_sub);
        log::error!("{}", errmsg);
        let _ = the_bot.robot_bota_sender.tell_bot_it_sucks.send
        ( format!("`{} Uh oh! - {}", dbgspt, errmsg)
        ).await;
        log::debug!("EXIT! {} - {}", dbgspt, errmsg);
        return;
      }
    }
    log::debug!("EXIT! {} - Check subscription added for bot index {}", dbgspt, bot_index);
  }

  pub async fn process_a_stop_tick
  ( bot_index: usize
  , the_tik: String
  , all_bot_api: &Vec<Option<BotApi>>
  , map_tick_looks: &mut std::collections::HashMap<String, Vec<usize>>
  , tell_dr_pop_ttai_tik : &tokio::sync::mpsc::Sender<(String, u64)>
  ) -> ()
  { let dbgspt = format!("[{}.{}] : "
    , crate::glossary::LOG_PREFIX_DR_ROBO_BYE_TIK
    , bot_index);
    let dbgtik = format!("{}", the_tik);

    log::debug!("ENTRY! {} - dr_robo::from_bota::process_a_stop_tick, ticker: {}", dbgspt, the_tik);
    log::info!
    ( "EXIT {} impl of mpsc rx told_dr_stop_tick recvd from bot {}, \n the ticker = {}"
      , dbgspt
      , bot_index
      , the_tik
    );

    // Fetch bot
    let the_bot = match all_bot_api.get(bot_index) 
    {   Some(Some(bot)) => bot
      , Some(None) => 
        {
          log::warn!("{}Inactive bot index {} for stop_tick", dbgspt, bot_index);
          log::debug!("EXIT! {} - Inactive bot index {}", dbgspt, bot_index);
          return;
        }
      None =>
        { log::warn!("{}Invalid bot index {} for stop_tick", dbgspt, bot_index);
          log::debug!("EXIT! {} - Invalid bot index {}", dbgspt, bot_index);
          return;
        }
    };

    log::info!
    ( "{} bot_data = {:#?}"
      , dbgspt
      , the_bot
    );

    let _ = the_bot.robot_bota_sender.tell_bot_it_sucks.send
    ( format!("{}Tick Stoping!", dbgspt)
    ).await;

    log::debug!("{}Removing bot from ticker's subscriber list {}", dbgspt, the_tik);
    if let Some(tick_vec) = map_tick_looks.get_mut(&the_tik) 
    { if let Some(idx) = tick_vec.iter().position(|&id| id == bot_index) 
      { tick_vec.swap_remove(idx);
      } 
      if tick_vec.is_empty() 
      { map_tick_looks.remove(&the_tik);
      }
    }

    log::debug!("{} -  tell_dr_pop_ttai_tik.try_send being called - matchs errors",dbgspt);
    let mut has_err = false;
    match tell_dr_pop_ttai_tik.try_send 
    ( ( the_tik
       , bot_index as u64 //the_bot.tell_bot_to_trade.clone() 
       )
    )
    { Ok(()) => 
      { log::info!
        ( "{} Told tell_dr_pop_ttai_tik had a good request deployed!"
          , dbgspt
        );
      }
      Err(tokio::sync::mpsc::error::TrySendError::Full(_)) => 
      { log::warn!
        ( "{} tell_dr_pop_ttai_tik channel full, error log not sent"
        , dbgspt
        );
        has_err = true;
      }
      Err(tokio::sync::mpsc::error::TrySendError::Closed(_)) => 
      { log::error!
        ( "{} tell_dr_pop_ttai_tik channel closed, critical error: (HUMU: dont exit from the loop but do admin stuff with the thingy)"
        , dbgspt
        );
        has_err = true;
        // Since process_new_robot returns (), we can't return Err; just log and continue
      }
    }

    if has_err
    { match the_bot.robot_bota_sender.tell_bot_it_sucks.try_send(format!("`{} tell_dr_pop_ttai_tik ERROR!", dbgspt))
      { Ok(()) => 
        { log::info!
          ( "{} Told robot_bota_sender had a good request deployed!"
            , dbgspt
          );
        }
        Err(tokio::sync::mpsc::error::TrySendError::Full(_)) => 
        { log::warn!
          ( "{} robot_bota_sender {}"
          , dbgspt
          , crate::glossary::ERROR_CHANNEL_FULL
          );
        }
        Err(tokio::sync::mpsc::error::TrySendError::Closed(_)) => 
        { log::error!
          ( "{} robot_bota_sender channel closed, critical error for bot "
          , dbgspt
          );
        }
      }
    }

    log::debug!("EXIT! {} - Ticker {} subscription stopped for bot index {}", dbgspt, dbgtik, bot_index);
  }

  pub async fn process_a_stop_info
  (   bot_index: usize
    , the_sub: String
    , all_bot_api: &Vec<Option<BotApi>>
    , all_bot_checks: &mut Vec<Vec<usize>>
    , map_bots_index: &std::collections::HashMap<String, usize>
  ) -> ()
  { let dbgspt = format!("[{}.{}] : "
    , crate::glossary::LOG_PREFIX_DR_ROBO_BYE_ACT
    , bot_index);
    log::debug!("{} - dr{} - dr_roboto::from_bota::process_a_stop_info, sub: {}", dbgspt, bot_index, the_sub);
    log::info!
    ( "{}Dr robo f_loop_dr_r_robotizizer impl of mpsc rx told_dr_stop_info recvd from bot {:#?}, \n the order = {:#?}"
    , dbgspt
    , bot_index
    , the_sub
    );
    log::debug!("{} - Fetch bot that sent request...", dbgspt);
    let the_bot = match all_bot_api.get(bot_index) 
    {   Some(Some(bot)) => bot
      , Some(None) => 
        { log::warn!("{}Inactive bot index {} for stop_info", dbgspt, bot_index);
          log::debug!("EXIT! {} - Inactive bot index {}", dbgspt, bot_index);
          return;
        }
      , None => 
        { log::warn!("{}Invalid bot index {} for stop_info", dbgspt, bot_index);
          log::debug!("EXIT! {} - Invalid bot index {}", dbgspt, bot_index);
          return;
        }
    };

    let _ = the_bot.robot_bota_sender.tell_bot_it_sucks.send
    ( format!("`{}Don't listen to the sub!", dbgspt)
    ).await
    ;
    log::debug!("{} - // Find and remove bot from check vector (unless it's self)...", dbgspt);
    if let Some(sub_index) = map_bots_index.get(&the_sub).copied()
    { if bot_index == sub_index 
      { let errmsg = format!("{}Bad bot! don't remove own sub to self!", dbgspt);
        log::error!("{}", errmsg);
        log::debug!("EXIT! {} - {}", dbgspt, errmsg);
        return;
      }
      match all_bot_checks.get_mut(sub_index) 
      { Some(check_vec) => 
        { if let Some(idx) = check_vec.iter().position(|&id| id == bot_index) 
          { check_vec.swap_remove(idx);
            log::debug!("removed a bot check!");
          }
        }
        None => 
        { let errmsg = format!("{} Bad bot! no ", dbgspt);
          log::error!("{}", errmsg);
          log::debug!("EXIT! {} - {}", dbgspt, errmsg);
          return;
        }
      }
    }
    else
    { let errmsg = format!("{}Bad bot! no bot at mapped index!", dbgspt);
      log::error!("{}", errmsg);
      log::debug!("EXIT! {} - {}", dbgspt, errmsg);
      return;
    }
    log::debug!("EXIT! {} - Stop info subscription processed OK! for bot index {}", dbgspt, bot_index);
  }
  
  pub async fn process_swap_info
  ( bot_index: usize
  , botswap: BotSwaper
  , all_bot_api: &mut Vec<Option<BotApi>>
  , map_bots_index: &mut std::collections::HashMap<String, usize>
  , all_bot_checks: &mut Vec<Vec<usize>>
  , dr_r: &crate::DrRoboApi
  , tell_dr_new_robot: &tokio::sync::mpsc::Sender<crate::CtorBot>  // ADD THIS LINE
  , stuff : &std::collections::HashMap< String, dsta::FinAss>
  , dr_robo_api : &DrRoboApi
  , dr_seek_api : &DrSeekApi
  ) -> ()
  { 
    let dbgspt = format!("[{}.{}] : "
    , crate::glossary::LOG_PREFIX_DR_ROBO_SWAPPER
    , bot_index);
    log::debug!("ENTRY! {} - dr_robo::from_bota::process_swap_info, botindex={} botswap= {:#?}", dbgspt, bot_index, botswap);
    let suck_sender = match all_bot_api.get(bot_index) 
    { Some(Some(bot)) => bot.robot_bota_sender.tell_bot_it_sucks.clone(),
      Some(None) | None => 
      { log::warn!("{} Invalid or inactive requesting bot index {}", dbgspt, bot_index);
        return;
      }
    };
    // Helper macro to always notify the requesting bot via sucks channel
    // and return from the function.
    //
    // Usage:
    //   do_return!!(true);  // sends SWAP_SUCC and returns
    //   do_return!!(false); // sends SWAP_FAIL and returns
    macro_rules! do_return {
        (true) => {{
            let _ = suck_sender.send(crate::glossary::SWAP_SUCC.to_string()).await;
            return;
        }};
        (false) => {{
            let _ = suck_sender.send(crate::glossary::SWAP_FAIL.to_string()).await;
            return;
        }};
        ($success:expr) => {{
            if $success {
                let _ = suck_sender.send(crate::glossary::SWAP_SUCC.to_string()).await;
            } else {
                let _ = suck_sender.send(crate::glossary::SWAP_FAIL.to_string()).await;
            }
            return;
        }};
    }
    if bot_index == 0
    {
      // ------------------------------------------------------------------
      // Magic bot0 swap – special cases for direct cash injection/withdrawal
      // ------------------------------------------------------------------
      // Check for cash transfer to/from bot0
      if botswap.source_offering.is_none()
      { if let Some(trel) = botswap.target_released.as_ref() 
        { if trel.who == crate::glossary::MAGIC_BOT0_NAME
          { let sink_name = format!("{}", crate::glossary::USER_CASH_BOT_NAME);
            let target_idx = if let Some(&idx) = map_bots_index.get(&sink_name) 
            { idx
            } 
            else 
            { // Create USER_CASH sink on first injection
              log::info!("{} Creating USER_CASH sink bot for initial capital", dbgspt);
              let make_swat = dsta::MakeSwatBotsGo 
              { my_accounting : 
                { let mut info = dsta::CommonBotInfo::default();
                  info.friendly_name = sink_name.clone();
                  info.tracking_tick = format!("VTEB"); // Tax-advantage thingy i guess
                  info
                },
              };
              // Make the swatbot
              let ( mind,body ) = 
              crate::sbot::swat_bot::f_make_bot_swat_2
              ( make_swat
              , dr_robo_api // &DrRoboApi
              , Some((0, botswap.clone())) // resend again
              , dr_seek_api.tell_dr_got_tick00.clone()
              ).await;
              let new_bot = body;
              log::trace!("{}  - // Add new bot to dr_robo(ticizer) (brains, map-index, checks-map",dbgspt );
              let (hand, foot) = fn_make_bot_api_hand_and_foot();
              let new_index = all_bot_api.len();
              all_bot_api.push
              ( Some
                ( BotApi
                  {   robot_brain_state : new_bot.robot_api_brain_state.clone()
                    , robot_bota_sender : hand.clone()
                  }
                )
              );
              map_bots_index.insert(new_bot.robot_api_brain_state.bot_name.clone(), new_index);
              all_bot_checks.push(vec![new_index]);
              log::trace!("{}  - // spawn crate::dr_bota::f_run_main_dr_bota api thread",dbgspt );
              let clone_dr_r_api = dr_robo_api.clone();
              let clone_hand = hand.clone();
              tokio::spawn
              ( async move
                { crate::dr_bota::f_run_main_dr_bota_2
                  ( clone_dr_r_api // DrRoboApi // dr_r: DrRoboApi
                  , new_bot // CtorBot // ctor: CtorBot
                  , new_index // usize // spot: usize
                  , clone_hand
                  , foot
                  , mind
                  ).await;
                }
              );
              return; // no OK or fail on a resend
            };
            if let Some(Some(cash_bot)) = all_bot_api.get_mut(target_idx) 
            { cash_bot.robot_brain_state.bot_safe.m_cash += trel.how;
              log::info!("{} {} {}: +{:.2}"
              , dbgspt
              , crate::glossary::MAGIC_BOT0_CASH_INJECT
              , crate::glossary::USER_CASH_BOT_NAME
              , trel.how
              );
              let _ = dr_r.tell_dr_new_brain.send((0, BotMindState::Running)).await;
              let _ = dr_r.tell_dr_new_brain.send((target_idx, BotMindState::Running)).await;
            }

            let cash_suck_sender = match all_bot_api.get(target_idx) 
            { Some(Some(bot)) => bot.robot_bota_sender.tell_bot_it_sucks.clone(),
              Some(None) | None => 
              { log::warn!("{} Invalid or inactive requesting bot index {}", dbgspt, bot_index);
                return;
              }
            };
            let _ = cash_suck_sender.send(crate::glossary::SWAP_SUCC.to_string()).await;

            do_return!(true);
          }
          else
          { unreachable!("none source from 0 => must be magic"); 
          }
        }
        else
        { unreachable!("none source from 0 => never empty swap");
        } 
      } 
      else if let Some(_soff) = &botswap.source_offering // source is 0 only now
      { // Dispersal to sink bots (USER_*, DEAD_*)
        if let Some(trel) = botswap.target_released.as_ref() 
        { if trel.who.starts_with("USER_") || trel.who.starts_with("DEAD_") 
          { let target_idx = 
            { if let Some(idx) = map_bots_index.get(&trel.who)
              { idx
              }
              else
              { log::error!("target bot must exist for dispersal");
                do_return!(false);
              }
            };

            let sink_name = trel.who.clone();
            // If sink bot doesn't exist → create it via proper swatbot factory
            if map_bots_index.get(&sink_name).is_none() 
            { log::info!("{} Creating new swatbot sink: {}", dbgspt, sink_name);
              let tracking_ticker = if sink_name.starts_with("USER_") 
              { sink_name.strip_prefix("USER_").map(|s| s.to_string())
              } 
              else if sink_name.starts_with("DEAD_") 
              { //soff.wat.positions.values().find_map(|position| crate::core::get_underlying_from_position(position))
                None
              }
              else 
              { None
              };

              let make_swat = dsta::MakeSwatBotsGo 
              { my_accounting : 
                { let mut info = dsta::CommonBotInfo::default();
                  info.friendly_name = sink_name.clone();
                  if let Some(tt) = tracking_ticker
                  { info.tracking_tick = tt;
                  }
                  info
                },
              };

              // pass the same swap to the make_bot_swat instead of here so it will re-queue after it starts.
              // the call stack is verified to tell_dr_new_robot with a properly configured CtorBot and also,
              // re-submit the Some((0, botswap.clone()))

              // New Method to make bots (skips process new bot:)
                let ( mind,body ) = crate::sbot::swat_bot::f_make_bot_swat_2
                ( make_swat
                , dr_robo_api // &DrRoboApi
                , Some((0, botswap.clone()))
                , dr_seek_api.tell_dr_got_tick00.clone()
                ).await;
                let new_bot = body;
                log::trace!("{}  - // Add new bot to dr_robo(ticizer) (brains, map-index, checks-map",dbgspt );
                let (hand, foot) = fn_make_bot_api_hand_and_foot();
                let new_index = all_bot_api.len();
                all_bot_api.push
                ( Some
                  ( BotApi
                    {   robot_brain_state : new_bot.robot_api_brain_state.clone()
                      , robot_bota_sender : hand.clone()
                    }
                  )
                );
                map_bots_index.insert(new_bot.robot_api_brain_state.bot_name.clone(), new_index);
                all_bot_checks.push(vec![new_index]);
                log::trace!("{}  - // spawn crate::dr_bota::f_run_main_dr_bota api thread",dbgspt );
                let clone_dr_r_api = dr_robo_api.clone();
                let clone_hand = hand.clone();
                tokio::spawn
                ( async move
                  { crate::dr_bota::f_run_main_dr_bota_2
                    ( clone_dr_r_api // DrRoboApi // dr_r: DrRoboApi
                    , new_bot // CtorBot // ctor: CtorBot
                    , new_index // usize // spot: usize
                    , clone_hand
                    , foot
                    , mind
                    ).await;
                  }
                );
              // Note, we should make all bots start like this

              return ; // Redo, no do_return!
            }

            // Sink now exists → apply the transfer directly to its account
            // After confirming sink exists
            if let Some(source_offering) = &botswap.source_offering 
            { let source_idx = 
              { if let Some(idx) = map_bots_index.get(&source_offering.who)
                { idx
                }
                else
                { log::error!("Source bot must exist for dispersal");
                  do_return!(false);
                }
              };

              // Process source_bot first
              if let Some(Some(source_bot)) = all_bot_api.get_mut(*source_idx) 
              { let src_acc = &mut source_bot.robot_brain_state.bot_safe;

                // Transfer cash from source_bot
                src_acc.m_cash -= source_offering.how;

                // Apply orders to source_bot
                for (_hash_name, owned_position) in &source_offering.wat.positions 
                {
                  let qty_abs = owned_position.owned.abs();
                  let src_action = if owned_position.owned > 0.0 
                  { dsta::BuyOrSell::SellToClose   // source gives away long
                  } else 
                  { dsta::BuyOrSell::BuyToClose    // source closes short
                  };

                  let order_src = dsta::Order 
                  { order_legs: vec!
                    [ dsta::OrderLeg 
                      {   buy_what: owned_position.asset.clone()
                        , quantity: qty_abs
                        , remaining: qty_abs
                        , action: src_action
                        , price: 0.0,
                      }
                    ]
                    , order_type: dsta::MarketOrLimit::Market
                    //, order_cost: None
                  };

                  let _ = crate::dr_robo::apply_order_to_account(src_acc, &order_src, stuff);
                }

                  // Notify source_bot
                  let _ = dr_robo_api.tell_dr_new_brain.send((*source_idx, source_bot.robot_brain_state.bot_mind.clone())).await;
              }

              // Process target_bot next
              if let Some(Some(target_bot)) = all_bot_api.get_mut(*target_idx) 
              { let tgt_acc = &mut target_bot.robot_brain_state.bot_safe;
                // Transfer cash to target_bot
                tgt_acc.m_cash += source_offering.how;
                // Apply orders to target_bot
                for (_hash_name, owned_position) in &source_offering.wat.positions 
                { let qty_abs = owned_position.owned.abs();
                  let tgt_action = if owned_position.owned > 0.0 
                  { dsta::BuyOrSell::BuyToOpen
                  } else 
                  { dsta::BuyOrSell::SellToOpen
                  };
                  let order_tgt = dsta::Order 
                  {   order_legs: vec!
                      [ dsta::OrderLeg 
                        {   buy_what: owned_position.asset.clone()
                          , quantity: qty_abs
                          , remaining: qty_abs
                          , action: tgt_action
                          , price: 0.0
                        }
                      ]
                    , order_type: dsta::MarketOrLimit::Market
                    //, order_cost: None,
                  };
                  let _ = crate::dr_robo::apply_order_to_account(tgt_acc, &order_tgt, stuff);
                }
                // Notify target_bot
                let _ = dr_robo_api.tell_dr_new_brain.send((*target_idx, target_bot.robot_brain_state.bot_mind.clone())).await;
              }

              log::info!("{} Dispersal from {} to sink {} completed", dbgspt, source_offering.who, sink_name);
              do_return!(true);
            }
          }
        }
      }

      log::error!("Bad bot 0 swapping (FALLTRHU) ... {:#?}", botswap);
      do_return!(false);
    }
    else // THESE are swaps requested by untrustworthy user generated robots
    {
      // ------------------------------------------------------------------
      // 0. Find source and target bot indices
      // ------------------------------------------------------------------
      let source_index = botswap.source_offering.as_ref()
          .and_then(|soff| map_bots_index.get(&soff.who));
      let target_index = botswap.target_released.as_ref()
          .and_then(|trel| map_bots_index.get(&trel.who));

      let involved_indices: Vec<usize> = std::iter::once(bot_index)
          .chain(source_index.copied())
          .chain(target_index.copied())
          .collect::<std::collections::HashSet<_>>()
          .into_iter()
          .collect();

      // 1: // TODO: if its a USER_ at source or target, ensure the offerings/releases match the USER_[THIS PART] 

      // ------------------------------------------------------------------
      // 2. Ensure NONE of the involved bots are currently Trading
      // ------------------------------------------------------------------
      for &idx in &involved_indices 
      { if let Some(Some(bot)) = all_bot_api.get(idx) 
        { if bot.robot_brain_state.bot_mind != BotMindState::Running 
          { log::warn!("{} Swap rejected: bot {} (index {}) is in {:#?}; non-magic swaps only allowed when bots are in running state."
            , dbgspt
            , bot.robot_brain_state.bot_name
            , idx
            , bot.robot_brain_state.bot_mind
            );
            do_return!(false)
          }
        }
      }

      // ------------------------------------------------------------------
      // 3. Detect cash-only swap
      // ------------------------------------------------------------------
      let is_cash_only = botswap.source_offering.as_ref()
          .map(|soff| soff.wat.positions.is_empty())
          .unwrap_or(true)
          && botswap.target_released.as_ref()
          .map(|trel| trel.wat.positions.is_empty())
          .unwrap_or(true);

      if is_cash_only {
          // --------------------------------------------------------------
          // Cash-only fast path – correct bidirectional transfer
          // --------------------------------------------------------------
          let source_gives = botswap.source_offering
              .as_ref()
              .map(|soff| soff.how)
              .unwrap_or(0.0);  // positive = source gives this much

          let target_gives = botswap.target_released
              .as_ref()
              .map(|trel| trel.how)
              .unwrap_or(0.0);  // positive = target gives this much

          // Net change:
          // source:  -source_gives + target_gives
          // target:  -target_gives + source_gives

          let source_net = -source_gives + target_gives;
          let target_net = -target_gives + source_gives;

          let mut source_new_cash: Option<f64> = None;
          let mut target_new_cash: Option<f64> = None;

          // Validate source
          if let Some(si) = source_index {
              if let Some(Some(src_bot)) = all_bot_api.get(*si) {
                  let proposed = src_bot.robot_brain_state.bot_safe.m_cash + source_net;
                  if proposed < 0.0 {
                      log::warn!("{} Cash-only swap rejected: source bot insufficient cash (net Δ{:.2})", dbgspt, source_net);
                      do_return!(false);
                  }
                  source_new_cash = Some(proposed);
              }
          }

          // Validate target
          if let Some(ti) = target_index {
              if let Some(Some(tgt_bot)) = all_bot_api.get(*ti) {
                  let proposed = tgt_bot.robot_brain_state.bot_safe.m_cash + target_net;
                  if proposed < 0.0 {
                      log::warn!("{} Cash-only swap rejected: target bot insufficient cash (net Δ{:.2})", dbgspt, target_net);
                      do_return!(false);
                  }
                  target_new_cash = Some(proposed);
              }
          }

          // Commit both sides
          if let Some(si) = source_index {
              if let Some(Some(src_bot)) = all_bot_api.get_mut(*si) {
                  src_bot.robot_brain_state.bot_safe.m_cash = source_new_cash.unwrap();
              }
          }
          if let Some(ti) = target_index {
              if let Some(Some(tgt_bot)) = all_bot_api.get_mut(*ti) {
                  tgt_bot.robot_brain_state.bot_safe.m_cash = target_new_cash.unwrap();
              }
          }

          log::info!(
              "{} Cash-only swap [by {} from {:#?} to {:#?}] SUCCESS: source gives {:.2}, receives {:.2} (net Δ{:.2}); target gives {:.2}, receives {:.2} (net Δ{:.2})",
              dbgspt,
              bot_index, source_index, target_index,
              source_gives, target_gives, source_net,
              target_gives, source_gives, target_net
          );

          // Trigger preserved mind state updates
          let mut mind_updates = Vec::new();
          for &idx in &involved_indices {
              if let Some(Some(bot)) = all_bot_api.get(idx) {
                  mind_updates.push((idx, bot.robot_brain_state.bot_mind.clone()));
              }
          }
          for (idx, mind) in mind_updates {
              let _ = dr_r.tell_dr_new_brain.send((idx, mind)).await;
          }

          do_return!(true);
      }

      // ------------------------------------------------------------------
      // Existing full swap logic (positions + cash) – unchanged except for early Trading check
      // ------------------------------------------------------------------
      let the_bot = match all_bot_api.get(bot_index)
      {   Some(Some(bot)) =>
          { log::trace!("{} Requesting Bot: {:#?}", dbgspt, bot);
            bot
          }
        , Some(None) =>
          { log::warn!("{} Requesting Bot is Inactive bot index {} for swap_info", dbgspt, bot_index);
            log::debug!("EXIT! {} - Inactive bot index {}", dbgspt, bot_index);
            do_return!(false);
          }
        , None =>
          { log::warn!("{} Requesting Bot is Invalid bot index {} for swap_info", dbgspt, bot_index);
            log::debug!("EXIT! {} - Invalid bot index {}", dbgspt, bot_index);
            do_return!(false);
          }
      };
      let _ = the_bot.robot_bota_sender.tell_bot_it_sucks.send
      ( format!("`{}: Bot Direct Swap Trade Commencing!", dbgspt)
      ).await;

      log::trace!( "{}// Find bot_a by matching source_offering bot_name, validate positions and cash", dbgspt);
      let mut source_index : Option<usize> = None;
      let mut target_index : Option<usize> = None;
      let mut fake_o1_flip : Option<dsta::Order> = None;
      let mut fake_o2_flip : Option<dsta::Order> = None;

      let fake_o1 =
      { if let Some(soff) = botswap.source_offering
        {
          let mut legs = Vec::new();
          let mut flegs = Vec::new();

          for (_key, val) in soff.wat.positions.iter()
          { let is_short = val.owned < 0.0;
            legs.push
            ( dsta::OrderLeg
              { buy_what: val.asset.clone()
              , quantity: val.owned 
              , remaining: val.owned
              , action:
                { if is_short { dsta::BuyOrSell::BuyToClose }
                  else { dsta::BuyOrSell::SellToClose }
                }
                , price: 0.0
              }
            );
            flegs.push
            ( dsta::OrderLeg
              { buy_what: val.asset.clone()
              , quantity: val.owned  
              , remaining: val.owned 
              , action:
                { if is_short { dsta::BuyOrSell::SellToOpen }
                  else { dsta::BuyOrSell::BuyToOpen }
                }
              , price: 0.0
              }
            );
          }
          source_index = map_bots_index.get(&soff.who).copied();
          fake_o1_flip =
          Some
          ( dsta::Order
            { order_legs: flegs
            , order_type: dsta::MarketOrLimit::Limit
            }
          );
          Some
          ( dsta::Order
            { order_legs: legs
            , order_type: dsta::MarketOrLimit::Limit
            }
          )
        }
        else
        { None
        }
      };

      let fake_o2 =
      { if let Some(trel) = botswap.target_released
        {
          let mut legs = Vec::new();
          let mut flegs = Vec::new();

          for (_key, val) in trel.wat.positions.iter()
          { let is_short = val.owned < 0.0;
            legs.push
            ( dsta::OrderLeg
              { buy_what: val.asset.clone()
              , quantity: val.owned 
              , remaining: val.owned
              , action:
                { if is_short { dsta::BuyOrSell::BuyToClose }
                  else { dsta::BuyOrSell::SellToClose }
                }
              , price: 0.0
              }
            );
            flegs.push
            ( dsta::OrderLeg
              { buy_what: val.asset.clone()
              , quantity: val.owned  
              , remaining: val.owned 
              , action:
                { if is_short { dsta::BuyOrSell::SellToOpen }
                  else { dsta::BuyOrSell::BuyToOpen }
                }
              , price: 0.0
              }
            );
          }
          target_index = map_bots_index.get(&trel.who).copied();

          log::trace!("Map Bots index TMEP {:#?}", map_bots_index);

          fake_o2_flip =
          Some
          ( dsta::Order
            { order_legs: flegs
            , order_type: dsta::MarketOrLimit::Limit
            }
          );
          Some
          ( dsta::Order
            { order_legs: legs
            , order_type: dsta::MarketOrLimit::Limit
            }
          )
        }
        else
        { None
        }
      };

      let mut is_for_self = false;
      if let Some(si) = source_index {
        if bot_index == si  {
          is_for_self = true;
        }
      }
      if let Some(ti) = target_index
      { if bot_index == ti {
          is_for_self = true;
        }
      }
      if !is_for_self
      { log::warn!("{}Bot index {} is NOT the TARGET OR SOURCE bot ({:#?} vs {:#?})", dbgspt, bot_index, target_index, source_index);
      }

      match (fake_o1.clone(), fake_o1_flip.clone(), source_index, fake_o2.clone(), fake_o2_flip.clone(), target_index)
      { ( Some(o1), Some(f1), Some(si), Some(o2), Some(f2), Some(ti) ) =>
        { log::trace!("{}Processing full swap between bots {} and {}", dbgspt, si, ti);

          let mut flip = false;
          if si == ti || si >= all_bot_api.len() || ti >= all_bot_api.len()
          { log::error!
            ( "EXIT! {} Illegal bot indexes:\n1.) {}\n2.) {}\n 3.) {}"
              , dbgspt
              , format!("si == ti ? , si = {:#?}, ti = {:#?}", si, ti)
              , format!("si >= all_bot_api.len() ? , si = {:#?}, all_bot_api.len() = {:#?}", si, all_bot_api.len())
              , format!("ti >= all_bot_api.len() ? , ti = {:#?}, all_bot_api.len() = {:#?}", ti, all_bot_api.len())
            );
            do_return!(false);
          }
          let (idxa,idxb) = if si < ti {(si,ti)} else {flip=true;(ti,si)};
          let (left,right) = all_bot_api.split_at_mut(idxb);
          let ( sibi, tibi ) = if flip
          { ( &mut right[0], &mut left[idxa] )
          }
          else
          { ( &mut left[idxa], &mut right[0] )
          };
          let sibi = match sibi
          {   Some(bot) => bot
            , None =>
              { log::warn!("{}Inactive source bot index {} for swap_info", dbgspt, si);
                log::debug!("EXIT! {} - Inactive source bot index {}", dbgspt, si);
                do_return!(false);
              }
          };
          let tibi = match tibi
          {   Some(bot) => bot
            , None =>
              { log::warn!("{}Inactive target bot index {} for swap_info", dbgspt, ti);
                log::debug!("EXIT! {} - Inactive target bot index {}", dbgspt, ti);
                do_return!(false);
              }
          };


          let mut act1 = sibi.robot_brain_state.bot_safe.clone();
          let mut act2 = tibi.robot_brain_state.bot_safe.clone();

          // First, transfer the cash
          let oc1 = core::fn_calculate_order_cost(&o1).unwrap_or(0.0);
          let oc2 = core::fn_calculate_order_cost(&o2).unwrap_or(0.0);
          act1.m_cash += -oc2;
          act2.m_cash += -oc1;

          if act1.m_cash < 0.0 || act2.m_cash < 0.0
          { log::error!("{}Failed cash validation - negative cash would result", dbgspt);
                do_return!(false);
          }

          // Then get the cash and collateral changes for the accounts
            let shen1 = match crate::dr_robo::fn_get_account_shennanagans
            ( &o1
            , &sibi.robot_brain_state.bot_safe
            , &stuff
            )
            { Ok(ok) => { ok }
            , Err(e) =>
              { log::error!("{}Failed shenanigans for source account: {:?}", dbgspt, e);
                do_return!(false);
              }
            };

            let shen1r = match crate::dr_robo::fn_get_account_shennanagans
            ( &f1
            , &tibi.robot_brain_state.bot_safe
            , &stuff
            )
            { Ok(ok) => { ok }
            , Err(e) =>
              { log::error!("{}Failed reverse shenanigans for source account: {:?}", dbgspt, e);
                do_return!(false);
              }
            };

            let shen2 = match crate::dr_robo::fn_get_account_shennanagans
            ( &o2
            , &tibi.robot_brain_state.bot_safe
            , &stuff
            )
            { Ok(ok) => { ok }
            , Err(e) =>
              { log::error!("{}Failed shenanigans for target account: {:?}", dbgspt, e);
                do_return!(false);
              }
            };

            let shen2r = match crate::dr_robo::fn_get_account_shennanagans
            ( &f2
            , &sibi.robot_brain_state.bot_safe
            , &stuff
            )
            { Ok(ok) => { ok }
            , Err(e) =>
              { log::error!("{}Failed reverse shenanigans for target account: {:?}", dbgspt, e);
                do_return!(false);
              }
            };

          // Then Apply the Release orders to the accounts
          {
            match crate::dr_robo::apply_order_to_account
            ( &mut act1
            , &o1
            , &stuff
            )
            { Ok(_) => { }
              Err(e) => 
              { log::error!("{}Can't apply release order 1: {:?}", dbgspt, e); 
                do_return!(false);
              }
            }
            crate::dr_robo::apply_account_cash_and_collateral_changes
            ( &mut act1
            , &shen1
            );

            match crate::dr_robo::apply_order_to_account
            ( &mut act2
            , &o2
            , &stuff
            )
            { Ok(_) => { }
              Err(e) => { log::error!("{}Can't apply release order 2: {:?}", dbgspt, e);
                               do_return!(false); 
              }
            }
            crate::dr_robo::apply_account_cash_and_collateral_changes
            ( &mut act2
            , &shen2
            );
          }

          // Then Apply the Gain orders to the accounts
          { match crate::dr_robo::apply_order_to_account
            ( &mut act1
            , &f2
            , &stuff
            )
            { Ok(_) => { }
              Err(e) => { log::error!("{}Can't apply gain to act 1: {:?}", dbgspt, e); 
                do_return!(false);
              }
            }
            crate::dr_robo::apply_account_cash_and_collateral_changes
            ( &mut act1
            , &shen2r
            );

            match crate::dr_robo::apply_order_to_account
            ( &mut act2
            , &f1
            , &stuff
            )
            { Ok(_) => { }
              Err(e) => { log::error!("{}Can't apply gain to act 2: {:?}", dbgspt, e); 
                do_return!(false);
              }
            }
            crate::dr_robo::apply_account_cash_and_collateral_changes
            ( &mut act2
            , &shen1r
            );
          }

          sibi.robot_brain_state.bot_safe = act1;
          tibi.robot_brain_state.bot_safe = act2;

          log::info!("{}Bot Swap SUCCESS!", dbgspt);

          // Send mind updates for all involved bots
          let mut mind_updates: Vec<(usize, BotMindState)> = Vec::new();
          for &idx in &involved_indices {
              if let Some(Some(bot)) = all_bot_api.get(idx) {
                  mind_updates.push((idx, bot.robot_brain_state.bot_mind.clone()));
              }
          }
          for (idx, mind) in mind_updates {
              let _ = dr_r.tell_dr_new_brain.send((idx, mind)).await;
          }
        }
        ,
        ( Some(o1), Some(_f1), Some(si), None, None, None ) =>
        { log::trace!("{}Processing source-only release for bot {}, f1 not used - ", dbgspt, si);

          if si >= all_bot_api.len() {
            log::error!("{}Invalid source bot index {} >= {}", dbgspt, si, all_bot_api.len());
                do_return!(false);
          }

          let sibi = match all_bot_api.get_mut(si) {
            Some(Some(bot)) => bot,
            Some(None) => {
              log::warn!("{}Inactive source bot index {} for source-only release", dbgspt, si);
                do_return!(false);
            },
            None => {
              log::error!("{}Invalid source bot index {} for source-only release", dbgspt, si);
                do_return!(false);
            }
          };

          let mut act1 = sibi.robot_brain_state.bot_safe.clone();
          let oc1 = core::fn_calculate_order_cost(&o1).unwrap_or(0.0);
          // Apply cash change from order cost
          act1.m_cash += -oc1; 

          if act1.m_cash < 0.0 {
            log::error!("{}Failed cash validation - negative cash would result for source-only", dbgspt);
                do_return!(false);
          }

          // Get account shenanigans for the release order
          let shen1 = match crate::dr_robo::fn_get_account_shennanagans(&o1, &sibi.robot_brain_state.bot_safe, &stuff) {
            Ok(ok) => ok,
            Err(e) => {
              log::error!("{}Failed shenanigans for source-only account: {:?}", dbgspt, e);
                do_return!(false);
            }
          };

          // Apply the release order
          match crate::dr_robo::apply_order_to_account(&mut act1, &o1, &stuff) {
            Ok(_) => {},
            Err(e) => {
              log::error!("{}Can't apply source-only release order: {:?}", dbgspt, e);
                do_return!(false);
            }
          }

          crate::dr_robo::apply_account_cash_and_collateral_changes(&mut act1, &shen1);

          sibi.robot_brain_state.bot_safe = act1;

          log::info!("{}Source-only release SUCCESS!", dbgspt);

          // Send mind updates for all involved bots
          let mut mind_updates: Vec<(usize, BotMindState)> = Vec::new();
          for &idx in &involved_indices {
              if let Some(Some(bot)) = all_bot_api.get(idx) {
                  mind_updates.push((idx, bot.robot_brain_state.bot_mind.clone()));
              }
          }
          for (idx, mind) in mind_updates {
              let _ = dr_r.tell_dr_new_brain.send((idx, mind)).await;
          }
        }
        ,
        ( None, None, None, Some(o2), Some(_f2), Some(ti) ) =>
        { log::trace!("{}Processing target-only release for bot {}", dbgspt, ti);

          if ti >= all_bot_api.len() {
            log::error!("{}Invalid target bot index {} >= {}", dbgspt, ti, all_bot_api.len());
                do_return!(false);
          }

          let tibi = match all_bot_api.get_mut(ti) {
            Some(Some(bot)) => bot,
            Some(None) => {
              log::warn!("{}Inactive target bot index {} for target-only release", dbgspt, ti);
                do_return!(false);
            },
            None => {
              log::error!("{}Invalid target bot index {} for target-only release", dbgspt, ti);
                do_return!(false);
            }
          };

          // TODO: ensure this leg makes sense in unit tests
          let mut act2 = tibi.robot_brain_state.bot_safe.clone();
          let oc2 = core::fn_calculate_order_cost(&o2).unwrap_or(0.0);

          // Apply cash change from order cost
          act2.m_cash += -oc2;

          if act2.m_cash < 0.0 {
            log::error!("{}Failed cash validation - negative cash would result for target-only", dbgspt);
                do_return!(false);
          }

          // Get account shenanigans for the release order
          let shen2 = match crate::dr_robo::fn_get_account_shennanagans(&o2, &tibi.robot_brain_state.bot_safe, &stuff) {
            Ok(ok) => ok,
            Err(e) => {
              log::error!("{}Failed shenanigans for target-only account: {:?}", dbgspt, e);
                do_return!(false);
            }
          };

          // Apply the release order
          match crate::dr_robo::apply_order_to_account(&mut act2, &o2, &stuff) {
            Ok(_) => {},
            Err(e) => {
              log::error!("{}Can't apply target-only release order: {:?}", dbgspt, e);
                do_return!(false);
            }
          }

          crate::dr_robo::apply_account_cash_and_collateral_changes(&mut act2, &shen2);

          tibi.robot_brain_state.bot_safe = act2;

          log::info!("{}Target-only release SUCCESS!", dbgspt);

          // Send mind updates for all involved bots
          let mut mind_updates: Vec<(usize, BotMindState)> = Vec::new();
          for &idx in &involved_indices {
              if let Some(Some(bot)) = all_bot_api.get(idx) {
                  mind_updates.push((idx, bot.robot_brain_state.bot_mind.clone()));
              }
          }
          for (idx, mind) in mind_updates {
              let _ = dr_r.tell_dr_new_brain.send((idx, mind)).await;
          }
        }
        _ =>
        { 
          let errmsg = format!
          ( "{} DEGENERATE swap states! {} {}"
          , dbgspt
          , "fake_o1, fake_o1_flip, source_index, fake_o2, fake_o2_flip, target_index"
          , format!
            ("\n fake_o1 = {:#?}\n, fake_o1_flip = {:#?}\n, source_index = {:#?}\n, fake_o2 = {:#?}\n, fake_o2_flip = {:#?}\n, target_index = {:#?}\n"
              ,fake_o1, fake_o1_flip, source_index, fake_o2, fake_o2_flip, target_index
            )
          );
          panic!("{}",errmsg);
        }
      }
    }
  }
} // end of module dr_robo::from_bota

// Bots communicate to us thru snively
// At first we'll use SendLogNotes, 
// and once the design settles we can turn into Snively::From[What][Thing](info)
async fn internal_log_note_processing
(
    who: usize,
    sniv: dsta::Snively,
    clone_tell_bot0_it_sucks_api: &tokio::sync::mpsc::Sender<String>,
    dr_seek_api: &crate::DrSeekApi,
    // These are only needed when who == 1 (seek bot)
    all_bot_api: &mut Vec<Option<BotApi>>,
    tell_dr_new_robot: &tokio::sync::mpsc::Sender<CtorBot>,
    map_bots_index: &mut std::collections::HashMap<String, usize>,
    dr_robo_api: &DrRoboApi,
    all_bot_checks: &mut Vec<Vec<usize>>

) 
{ let dbgspt = format!("[DR_ROBO LOGPRO]");
  if who == crate::glossary::BOT0_INDEX
  { log::info!("TODO: Internal told a snively processing on {:#?}", sniv);
    if let dsta::Snively::SendLogNote(s) = sniv
    { if s == crate::glossary::TTAI_FAIL_PYTHON_START
      { log::info!("NOTE: dr R told_dr_panic_pls,  Here we can try again, by telling bot 0 to try again");
        match clone_tell_bot0_it_sucks_api.send
        ( format!
          ( "{}", crate::glossary::TTAI_FAIL_FIX_RESTART0
          )
        ).await
        { Ok(_) => 
          { log::info!("DR Robo told bot0 it sucks with ttai fail fix restart0");
          }
          ,
          Err(e) =>
          { panic!("bot0 bot is dead, dr robo canot recover from this at the moment, error = {:#?}", e);
          }
        }
      }
      else
      { log::warn!("Unhandled internal snively log note: {}", s);
      }
    }
    else
    { log::warn!("Unhandled internal snively request type");
      // - may use for loading bots in the future? then dr robo only needs to remember just bota allocations 
    }
  }
  else
  if who == crate::glossary::BOT1_INDEX // From an External USER 
  { return from_seek::process_a_snively
    (   sniv // // Snively : s // new_bot: CtorBot
      , all_bot_api // all_bot_api : &all_bot_api  // all_bot_api: &mut Vec<Option<BotApi>>        
      , dr_seek_api 
      , tell_dr_new_robot
      , map_bots_index // map_bots_index : &map_bots_index  // map_bots_index: &mut std::collections::HashMap<String, usize>
      , dr_robo_api
      , all_bot_checks
    ).await;
  }
  else // Send to the external user
  { match dr_seek_api.tell_dr_snively.try_send(sniv)
    { Ok(()) => 
      { log::info!
        ( "{} Forwarded to frontend snively"
          , dbgspt
        );
      }
      Err(tokio::sync::mpsc::error::TrySendError::Full(e)) => 
      { log::warn!
        ( "{} {} dr_seek_api.tell_dr_snively {}"
        , dbgspt
        , format!("{:#?}",e)
        , crate::glossary::ERROR_CHANNEL_FULL
        );
      }
      Err(tokio::sync::mpsc::error::TrySendError::Closed(e)) => 
      { log::error!
        ( "{}dr_seek_api.tell_dr_snively channel closed, critical error: {}"
        , dbgspt
        , format!("{:#?}",e)
        );
      }
    }
  }
}

pub(crate) async fn f_run_main_dr_robo()->()
{ let dbgspt = format!("[{}]: ", crate::glossary::LOG_PREFIX_DR_ROBO_RUNNING);
  log::info!("{}ENTRY!: dr_robo::f_run_main_dr_robo (DRFRUN)!", dbgspt); 

  // Create the self API 
  let (tell_dr_a_snively, mut told_dr_a_snively)
     = tokio::sync::mpsc::channel::<(usize, dsta::Snively)>   
    ( crate::glossary::CHANNEL_BUFFER_SIZE_SNIVELY
    );
  let (tell_dr_new_robot, mut told_dr_new_robot) =
     tokio::sync::mpsc::channel::<CtorBot>                  
    ( crate::glossary::CHANNEL_BUFFER_SIZE_NEW_ROBOT
    );
  let (tell_dr_pls_trade, mut told_dr_pls_trade) =
     tokio::sync::mpsc::channel::<(usize, u64, dsta::Order)>
    ( crate::glossary::CHANNEL_BUFFER_SIZE_PLS_TRADE
    );
  let (tell_dr_new_brain, mut told_dr_new_brain) =
     tokio::sync::mpsc::channel::<(usize, BotMindState)>    
    ( crate::glossary::CHANNEL_BUFFER_SIZE_NEW_BRAIN
    );
  let (tell_dr_pls_ticks, mut told_dr_pls_ticks) =
     tokio::sync::mpsc::channel::<
     ( usize
     , String
     , tokio::sync::mpsc::Sender<Option<dsta::PushTickerResult>>
     )>
    ( crate::glossary::CHANNEL_BUFFER_SIZE_PLS_TICKS
    );
  let (tell_dr_pls_check, mut told_dr_pls_check) =
     tokio::sync::mpsc::channel::<(usize, String)>
    ( crate::glossary::CHANNEL_BUFFER_SIZE_PLS_CHECK
    );
  let (tell_dr_stop_tick, mut told_dr_stop_tick) =
     tokio::sync::mpsc::channel::<(usize, String)>
    ( crate::glossary::CHANNEL_BUFFER_SIZE_STOP_TICK
    );
  let (tell_dr_stop_info, mut told_dr_stop_info) =
     tokio::sync::mpsc::channel::<(usize, String)>
    ( crate::glossary::CHANNEL_BUFFER_SIZE_STOP_INFO
    );
  let (tell_dr_swap_info, mut told_dr_swap_info) =
     tokio::sync::mpsc::channel::<(usize, BotSwaper)>
    ( crate::glossary::CHANNEL_BUFFER_SIZE_SWAP_INFO
    );
  let dr_robo_api = DrRoboApi 
  {   tell_dr_a_snively
    , tell_dr_pls_trade
    , tell_dr_new_brain
    , tell_dr_pls_ticks
    , tell_dr_pls_check
    , tell_dr_stop_tick
    , tell_dr_stop_info
    , tell_dr_swap_info
  };
  // create the data structures:
  log::info!("{} Making all_bot_api to hold bot info and api handles", dbgspt);
  let mut all_bot_api: Vec<Option<BotApi>> = Vec::new();
  let mut all_bot_checks: Vec<Vec<usize>> = Vec::new();
  let mut map_bots_index: std::collections::HashMap<String, usize> = std::collections::HashMap::new();
  
  // This is for saving & loading, since the actual tick looks is tracked in the dr_r_ttai_loop
  let mut map_tick_looks: std::collections::HashMap<String, Vec<usize>> = std::collections::HashMap::new(); // Ticker -> [Bot indices]
  let mut asset_infos : std::collections::HashMap
  < String
  , dsta::FinAss
  >
  = std::collections::HashMap::new();

  log::info!
  ( "{}",
    r#"
    // Index 0: bot for ttai bot trades".
    // Index 1: bot for user tades" 
    // Index 2: bot for taxes, commissions, etc.
    // Index K: the other bots, made by user
    "#
  );

  let mut new_brain_vec = Vec::new();
  let mut pls_check_vec = Vec::new();
  let mut pls_ticks_vec = Vec::new();
  let mut stop_tick_vec = Vec::new();
  let mut stop_info_vec = Vec::new();
  let mut pls_trade_vec = Vec::new();
  let mut swap_info_vec = Vec::new();

  let mut snively_vec = Vec::new();
  let mut new_robot_vec = Vec::new();


  // Create and spawn TTAI, SEEK, BASE BOTS 0,1,...

  log::trace!("{} Creating TTAI channels...", dbgspt);

  // cross cummunication:
  // BOT0 => TTAI ( gets, trade )
  // TTAI => BOT0 ( )
  // ROBO => TTAI (tick sub, tick pop) (not bota because we want to kill etc)
  // ROBO => BOT0 { trade } + bota
  // thus This needs to be made before making the drttaiapi 
  log::info!("{} Creating dr_robo usage of TTAI channels...", dbgspt);
  let ( tell_dr_ttai_add_ttai_tik, told_dr_ttai_add_ttai_tik ) 
    = tokio::sync::mpsc::channel::
    < ( String
      , u64
      , tokio::sync::mpsc::Sender<dsta::Ticker>
      , tokio::sync::mpsc::Sender<Option<dsta::PushTickerResult>>
      )
    > (crate::glossary::CHANNEL_BUFFER_SIZE_TTAI_TICK);
  let ( tell_dr_ttai_pop_ttai_tik, told_dr_ttai_pop_ttai_tik ) = tokio::sync::mpsc::channel::<(String, u64)>(100);
  // Lets make the bot0 do the rest of the ttai stuff
  log::info!("{} Creating dr_robo usage of Bot 0 channels...", dbgspt);
  let 
  ( tell_dr_bot0_order_inputs // bot zero doesn't request stuff from dr robo   
  , told_dr_bot0_order_inputs 
  ) =  tokio::sync::mpsc::channel::<dr_bot0::NewOrderRequest>
  ( crate::glossary::CHANNEL_BUFFER_SIZE_BOT0_ORDER
  );
  // Actual Bot API Handles:
  let 
  ( _bot0_bridge_to_dr_hand // Bot zero doesn't request stuff from dr robo
  , bot0_bridge_to_dr_foot
  ) = crate::fn_make_bot_api_bridge_dr_handles();
  log::info!("{} Creating SEEK channels...", dbgspt);
  let (seek_hand, seek_foot) = dr_seek::fn_make_hand_foot();
   // note - the seek_hand is what we use to broadcast data to the seek connections
  let dr_seek_api = crate::DrSeekApi  // Bot One - for running the quic server, handling connections to the user interfaces. 
  {   tell_dr_snively     : seek_hand.tx_talk_snively //tokio::sync::mpsc::Sender<dsta::Snively>
    , tell_dr_made_bot    : seek_hand.tx_talk_made_bot //tokio::sync::mpsc::Sender<dsta::MadeBot>
    , tell_dr_got_ttai    : seek_hand.tx_talk_got_ttai //tokio::sync::mpsc::Sender<dsta::GotTtai>
    , tell_dr_got_tick00  : seek_hand.tx_talk_got_tick00 //tokio::sync::mpsc::Sender<dsta::GotTick00> // regular tickers
    , tell_dr_got_tick01  : seek_hand.tx_talk_got_tick01 //tokio::sync::mpsc::Sender<dsta::GotTick01> // options tickers 
  };  
  
  log::info!("{} Spawning BOT 0 tasks...", dbgspt);
  // Core robo/Bota Api Tasks
  let ( tell_bot0_to_trade_api, told_bot0_to_trade_api) = tokio::sync::mpsc::channel::<dsta::Ticker>(100);
  let ( tell_bot0_to_check_api, told_bot0_to_check_api) = tokio::sync::mpsc::channel::<Allocation>(100);
  let ( tell_bot0_it_sucks_api, told_bot0_it_sucks_api) = tokio::sync::mpsc::channel::<String>(100);
  // specific bot0 tasks:
  let ( tell_dr_bot0_cancel,  told_dr_bot0_cancel)  = tokio::sync::mpsc::channel::<dr_bot0::CancelOrderRequest>(64);
  let ( tell_dr_bot0_dryrun,  told_dr_bot0_dryrun)  = tokio::sync::mpsc::channel::<dr_bot0::DryRunRequest>(64);

  let clone_tell_bot0_it_sucks_api = tell_bot0_it_sucks_api.clone();
  let bot0_creation = 
  CtorBot
  { robot_api_brain_state : Allocation
    {   bot_name: format!("0") // pub bot_name: String
      , bot_safe: crate::BotaAccount::default() // dsta::PositionsList
      , bot_mind: crate::BotMindState::Running // pub bot_mind: BotMindState
      , num_swap: 0 // pub bot_lein: f64 // collateral for short positions etc
    } //   pub robot_api_brain_state: Allocation
    , tell_bot_to_trade_api : tell_bot0_to_trade_api //, pub tell_bot_to_trade_api: tokio::sync::mpsc::Sender<dsta::Ticker>
    , tell_bot_to_check_api : tell_bot0_to_check_api //, pub tell_bot_to_check_api: tokio::sync::mpsc::Sender<Allocation>
    , tell_bot_it_sucks_api : tell_bot0_it_sucks_api.clone() //, pub tell_bot_it_sucks_api: tokio::sync::mpsc::Sender<String>
    , told_the_bot_requests : bot0_bridge_to_dr_foot //, pub told_the_bot_requests: BridgeDrFoot
  };
  from_dr_r::process_new_robot
  (   bot0_creation // // new_bot : s // new_bot: CtorBot
    , &mut all_bot_api // all_bot_api : &all_bot_api  // all_bot_api: &mut Vec<Option<BotApi>>        
    , &mut map_bots_index // map_bots_index : &map_bots_index  // map_bots_index: &mut std::collections::HashMap<String, usize>
    , &mut all_bot_checks // all_bot_checks : &all_bot_checks  // all_bot_checks: &mut Vec<Vec<usize>>
    , &dr_seek_api // dr_seek_api : &dr_seek_api  // dr_seek_api: &DrSeekApi
    , &dr_robo_api
  ).await;

  // Bots are told to start in process_new_robot!
  let clone_bot0_api = all_bot_api.get(0).unwrap().as_ref().unwrap().clone();
  let clone_dr_r_api = dr_robo_api.clone(); 
  let clone_tell_dr_ttai_add_ttai_tik = tell_dr_ttai_add_ttai_tik.clone();
  let clone_tell_dr_ttai_pop_ttai_tik = tell_dr_ttai_pop_ttai_tik.clone();
  let clone_tell_dr_snively = clone_dr_r_api.tell_dr_a_snively.clone();
  let clone_tell_dr_snively2 = clone_dr_r_api.tell_dr_a_snively.clone();
  
  // In f_run_main_dr_robo(), replace the dr_bot0 spawn with:

  // ADD THIS: Choose mock or real mode
  let ttai_mode = if std::env::var("PAPER_TRADING").is_ok() 
  { log::warn!("We are using PAPER TRADING!");
    dr_bot0_ttai_bridge::TtaiMode::Mock(dr_bot0_ttai_bridge::default_paper_trading_config())
  } else 
  { log::warn!("We are using REAL TRADING!");
    dr_bot0_ttai_bridge::TtaiMode::Real
  };

  // THEN: Use the mode-aware version
  tokio::spawn(async move {
      crate::dr_bot0::f_run_main_dr_bot0_with_mode(  // ← Changed function name
          ttai_mode,  // ← NEW first parameter
          told_dr_bot0_order_inputs,
          told_dr_bot0_cancel,
          told_dr_bot0_dryrun,
          clone_dr_r_api,
          clone_bot0_api,
          (clone_tell_dr_ttai_add_ttai_tik, told_dr_ttai_add_ttai_tik),
          (clone_tell_dr_ttai_pop_ttai_tik, told_dr_ttai_pop_ttai_tik),
          told_bot0_to_trade_api,
          told_bot0_to_check_api,
          told_bot0_it_sucks_api,
          clone_tell_dr_snively,
      ).await;
  });


  log::info!("{} Spawning SEEK task...", dbgspt);
  let tdnr = tell_dr_new_robot.clone();
  tokio::spawn 
  ( async move 
    { crate::dr_seek::f_run_main_dr_seek
      ( seek_foot
      , tdnr // seek tells robo about new bots
      , clone_tell_dr_snively2
      ).await;
    }
  );

  log::info!("DR ROBO will loop tokio select to wait for the proper startups of bots..."); 
  let mut b0su = false; // bot 0 startup 
  let mut b1su = false; // bot 1 startup
  let mut b2su = false; // bot 2 startup
  let mut dssu = false; // dr seek startup (part of bot1 startup)
  let mut dtsu = false; // dr ttai startup
  loop
  { tokio::select!
    { _ = tokio::signal::ctrl_c() => 
      { log::info!("{} - TOKIO SELECT => Received Ctrl+C, shutting down meanly. TODO: gracefull w/ saves", dbgspt);
        break;
      }
      res = told_dr_a_snively.recv() =>
      { match res
        { Some(s) =>
          { 
            let (who, sniv) = s;

            log::trace!("Startup DR ROBO got a sniv: {:#?}", sniv );
            if who == 0 // DONE
            { if let dsta::Snively::SendLogNote(msg) = sniv
              { if msg == crate::glossary::MAGIC_BOT0_STARTUP_OK
                { log::info!("dr_robo STARTUP OK! part 0: bot_0 startup");
                  b0su = true;
                }
                else if msg == crate::glossary::MAGIC_DTTAI_STARTUP_OK
                { log::info!("dr_robo STARTUP OK! part 0: bot_0's dr_ttai startup");
                  dtsu = true;
                }
              } 
            } else if who == 1
            { if let dsta::Snively::SendLogNote(msg) = sniv
              { if msg == crate::glossary::MAGIC_DSSEEK_STARTUP_OK // DONE
                { log::info!("dr_robo STARTUP OK! part 1: bot_1's dr_seek startup");
                  dssu = true;
                  b1su = true;
                  b2su = true;
                } 
              } 
            } 
            else
            { log::error!("Got a weird snively at startup:, who={:#?}, sniv={:#?}",who,sniv );
              break;
            }
          }
          None =>
          { log::error!
            ( "{} {} During startup's initial wait for being told a snively"
            , dbgspt
            , crate::glossary::ERROR_CHANNEL_CLOSED
            );
            break;
          }
        }
      }
      _ = tokio::time::sleep
      ( std::time::Duration::from_secs
        ( crate::glossary::DEFAULT_STARTUP_TIMEOUT_SECS
        )
      ) => 
      { log::info!("DR_, wait for snivelys: timeout");
        break;
      }
    }
    if b0su && b1su && b2su && dssu && dtsu
    { log::info!("DR ROBO COMPLETED STARTUP OF CORE UNDERLYING BOTS");
      break;
    }
  }

  if !(b0su && b1su && b2su && dssu && dtsu)
  { log::error!
    ( "DR ROBO BAD STARTUP OF CORE UNDERLYING BOTS\n  b0su/MAGIC_BOT0_STARTUP_OK {}\n  b1su {}\n  b2su {}\n  dssu {}\n  dtsu {} "
    , b0su , b1su , b2su , dssu , dtsu
    );
    return;
  }

  log::info!("DR ROBO Entering main state loop");
  loop 
  { tokio::select! 
    { 
      _ = tokio::signal::ctrl_c() => 
      { log::info!("{} - TOKIO SELECT => Received Ctrl+C, shutting down meanly. TODO: gracefull w/ saves", dbgspt);
        break;
      }

      res = told_dr_a_snively.recv_many(&mut snively_vec, 50) =>
      { if res == 0 
        { log::error!("told_dr_a_snively channel closed!");
          break;
        }
        log::debug!("{} // Processing {} snively messages", dbgspt, res);
        for s in snively_vec.drain(..) 
        { let (who, sniv) = s;
          internal_log_note_processing
          (
              who,
              sniv,
              &clone_tell_bot0_it_sucks_api,
              &dr_seek_api,
              &mut all_bot_api,
              &tell_dr_new_robot,
              &mut map_bots_index,
              &dr_robo_api,
              &mut all_bot_checks
          )
          .await;
        }
      }

      res = told_dr_new_robot.recv_many(&mut new_robot_vec, 20) =>
      { if res == 0 
        { log::error!("told_dr_new_robot channel closed!");
          break;
        }
        log::debug!("{} // Processing {} new robot messages", dbgspt, res);
        for s in new_robot_vec.drain(..) 
        { log::trace!("dr_robo got a told_dr_new_robot...");
          from_dr_r::process_new_robot
          (   s
            , &mut all_bot_api
            , &mut map_bots_index
            , &mut all_bot_checks
            , &dr_seek_api
            , &dr_robo_api
          ).await;
        }
      }

      res = told_dr_new_brain.recv_many
      ( &mut new_brain_vec
      , crate::glossary::DEFAULT_RECV_MANY_BATCH_SIZE
      ) => 
      { if res == 0 
        { log::debug!("{} // Brain update channel closed in dr_robo::f_run", dbgspt);
          break;
        }
        log::debug!("{} // Brain update from one of our bot api's", dbgspt);
        for (bot_index, brains) in new_brain_vec.drain(..) 
        { from_bota::process_new_brain
          (   bot_index
            , brains
            , &mut all_bot_api
            , &mut all_bot_checks
            , &mut map_bots_index
            , &dr_robo_api
          ).await;
        }
      }

      res = told_dr_pls_check.recv_many(&mut pls_check_vec, 100) => 
      { log::debug!("{} // told_dr_pls_check requests for data flow (bot account) to the bot api's", dbgspt);
        if res == 0
        { log::debug!("{} // told_dr_pls_check channel closed!...", dbgspt);
          break;
        }
        for (bot_index, the_sub) in pls_check_vec.drain(..) 
        { from_bota::process_a_pls_check
          (   bot_index
            , the_sub
            , &mut all_bot_api
            , &mut all_bot_checks
            , &mut map_bots_index
          ).await;
        }
      }

      res = told_dr_pls_ticks.recv_many(&mut pls_ticks_vec, 100) => 
      { log::debug!("{} // told_dr_pls_ticks Adds the_bot as a subscriber to the subscribed ticker...", dbgspt);
        if res == 0
        { log::debug!("{} // told_dr_pls_ticks channel closed!...", dbgspt);
          break;
        }

        for (bot_index, the_sub, the_rep) in pls_ticks_vec.drain(..) 
        { from_bota::process_pls_ticks
          (   bot_index
            , the_sub
            , &all_bot_api
            , &mut map_tick_looks
            , &tell_dr_ttai_add_ttai_tik
            , the_rep
            , &mut asset_infos
            , &dr_seek_api
          ).await;
        }
      }

      res = told_dr_stop_tick.recv_many(&mut stop_tick_vec, 100) => 
      { log::debug!("{} // told_dr_stop_tick Drops the_bot as a subscriber to the ticker...", dbgspt);
        if res == 0
        { log::debug!("{} // told_dr_stop_tick channel closed!...", dbgspt);
          break;
        }
        for (bot_index, the_tik) in stop_tick_vec.drain(..) 
        { from_bota::process_a_stop_tick
          (  bot_index
            , the_tik
            , &all_bot_api
            , &mut map_tick_looks
            , &tell_dr_ttai_pop_ttai_tik,
          ).await;
        }
      }

      res = told_dr_stop_info.recv_many(&mut stop_info_vec, 100) => 
      { log::debug!("{} // Drops the_bot as a subscriber to the ticker info", dbgspt);
        if res == 0
        { log::debug!("{} // told_dr_stop_info channel closed!...", dbgspt);
          break;
        }
        for (bot_index, the_sub) in stop_info_vec.drain(..) 
        { from_bota::process_a_stop_info
          (   bot_index
            , the_sub
            , &all_bot_api
            , &mut all_bot_checks
            , &map_bots_index
          ).await;
        }
      }

      res = told_dr_swap_info.recv_many(&mut swap_info_vec, 100) => 
      {
        log::debug!("{} // told_dr_swap_info - DR ROBO level Bot api cash/stock allocation management", dbgspt);
        if res == 0
        { log::debug!("{} // told_dr_swap_info channel closed!...", dbgspt);
          break;
        }
        for (bot_index, botswap) in swap_info_vec.drain(..) 
        { from_bota::process_swap_info
          (   bot_index
            , botswap
            , &mut all_bot_api
            , &mut map_bots_index
            , &mut all_bot_checks
            , &dr_robo_api
            , &tell_dr_new_robot 
            , &asset_infos 
            , &dr_robo_api
            , &dr_seek_api
          ).await;
        }
      }

      res = told_dr_pls_trade.recv_many(&mut pls_trade_vec, 100) => 
      { log::debug!("{} told_dr_pls_trade - // From Base bot api, pass the trade order request to the order & account processing bot dr_bot0", dbgspt);
         if res == 0
        { log::debug!("{} // told_dr_pls_trade channel closed!...", dbgspt);
          break;
        }
        for (bot_index, order_number, order) in pls_trade_vec.drain(..) 
        { from_bota::process_pls_trade(
              (bot_index, order_number),
              order,
              &mut all_bot_api,
              &tell_dr_bot0_order_inputs,
              &tell_dr_bot0_dryrun,     // new
              &tell_dr_bot0_cancel,     // new
              &dr_robo_api,
              &asset_infos,
              &map_bots_index,          // new
          ).await;
        }
      }
    }
  }
  log::info!("{}f_loop_dr_r_robotizizer ENDING!", dbgspt);
}