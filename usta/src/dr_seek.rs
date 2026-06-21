//dr_seek.rs
//use crate::*;


fn example_server_load_certs_and_key()
-> Result
< ( Vec<rustls::pki_types::CertificateDer<'static>>
  , rustls::pki_types::PrivateKeyDer<'static>
  )
, Box<dyn std::error::Error>
> 
{
  let cert_file = std::fs::File::open("cert.pem")?;
  let mut cert_reader = std::io::BufReader::new(cert_file);
  let certs: Vec<rustls::pki_types::CertificateDer<'static>>
    = rustls_pemfile::certs(&mut cert_reader).collect::<Result<Vec<_>, _>>()?;
  if certs.is_empty() {
    return Err("No certificates found in cert.pem".into());
  }
  let key_file = std::fs::File::open("key.pem")?;
  let mut key_reader = std::io::BufReader::new(key_file);
  let mut keys = rustls_pemfile::pkcs8_private_keys(&mut key_reader)
    .collect::<Result<Vec<_>, _>>()?;
  if keys.is_empty() {
    return Err("No private keys found in key.pem".into());
  }
  let key = rustls::pki_types::PrivateKeyDer::from(keys.remove(0));
  Ok((certs, key))
}


fn run_dr_r_generate_serverkernel() ->
Result
< ( dsta::ksta::ServerKernel
  , tokio::sync::mpsc::Receiver<dsta::ksta::ConnectionHandle>
  )
, Box<dyn std::error::Error>
> // This function generates the seek server kernel 
{ let dbgspt = format!("[RDRGSK] :");

  let (new_connection_sender, new_connection_receiver) = tokio::sync::mpsc::channel(256);
  log::debug!("\n  [ENTRY] {} run_dr_r_generate_serverkernel Creating server kernel...",dbgspt);
  match rustls::crypto::CryptoProvider::install_default(rustls::crypto::ring::default_provider()) 
  { Err(_) => 
    { log::debug!("{} rustls::crypto::CryptoProvider::install_default Result NOT infalliable i guess",dbgspt);
    }
    ,
    _ => 
    { log::debug!("{} rustls::crypto::CryptoProvider::install_default Result infalliable i guess",dbgspt);
    }
  };
  log::debug!("{} calling Load certificates and key...",dbgspt);
  let (certs, key) = match example_server_load_certs_and_key()
  { Ok(ok) => ok
  , Err(e) =>
    { log::error!("{} Could not load certs and keys: {:#?}", dbgspt, e);
      return Err(e);
    }
  };
  
  log::debug!("{} Creating server configuration...",dbgspt);
  let mut server_config = quinn::ServerConfig::with_single_cert(certs, key)?;
  let mut transport_config = quinn::TransportConfig::default();
  transport_config.max_idle_timeout(None); // No idle timeout
  let one_year = tokio::time::Duration::from_secs(365 * 24 * 60 * 60); // 1 year in seconds
  transport_config.keep_alive_interval(Some(one_year)); // Keep-alive every year
  let transport_config = std::sync::Arc::new(transport_config);
  server_config.transport_config(transport_config);
  log::debug!("{} Creating the QUIC endpoint...",dbgspt);
  let endpoint = quinn::Endpoint::server(server_config, "0.0.0.0:4433".parse()?)?;
  log::debug!("{} Createing the ServerKernel instance...",dbgspt);
  Ok
  ( ( dsta::ksta::ServerKernel::new
      ( endpoint
      , new_connection_sender
      )
    , new_connection_receiver
    )
  )
}

pub(crate) struct Hand
{   pub tx_talk_snively   : tokio::sync::mpsc::Sender<dsta::Snively>
  , pub tx_talk_made_bot   : tokio::sync::mpsc::Sender<dsta::MadeBot>
  , pub tx_talk_got_ttai   : tokio::sync::mpsc::Sender<dsta::GotTtai>
  , pub tx_talk_got_tick00 : tokio::sync::mpsc::Sender<dsta::GotTick00>
  , pub tx_talk_got_tick01 : tokio::sync::mpsc::Sender<dsta::GotTick01>
}

pub(crate) struct Foot
{   pub rx_talk_snively : tokio::sync::mpsc::Receiver<dsta::Snively>
  , pub rx_talk_made_bot : tokio::sync::mpsc::Receiver<dsta::MadeBot>
  , pub rx_talk_got_ttai : tokio::sync::mpsc::Receiver<dsta::GotTtai>
  , pub rx_talk_got_tick00 : tokio::sync::mpsc::Receiver<dsta::GotTick00>
  , pub rx_talk_got_tick01 : tokio::sync::mpsc::Receiver<dsta::GotTick01>
}

pub(crate) fn fn_make_hand_foot() -> (Hand,Foot)
{ let (a0,b0) = tokio::sync::mpsc::channel::<dsta::Snively>(1024);
  let (a1,b1) = tokio::sync::mpsc::channel::<dsta::MadeBot>(128);
  let (a3,b3) = tokio::sync::mpsc::channel::<dsta::GotTtai>(128);
  let (a4,b4) = tokio::sync::mpsc::channel::<dsta::GotTick00>(8*1024);
  let (a5,b5) = tokio::sync::mpsc::channel::<dsta::GotTick01> (8*1024);
  ( Hand
    {   tx_talk_snively   : a0
      , tx_talk_made_bot   : a1
      , tx_talk_got_ttai   : a3
      , tx_talk_got_tick00 : a4 
      , tx_talk_got_tick01 : a5 
    }
  , Foot
    {   rx_talk_snively   : b0
      , rx_talk_made_bot   : b1
      , rx_talk_got_ttai   : b3
      , rx_talk_got_tick00 : b4 
      , rx_talk_got_tick01 : b5 
    }
  )
}

fn pre_prevent_reserved_names(who : &str) -> bool
{ who.starts_with("USER_") 
  || who.starts_with("DEAD_")
  || who.starts_with("0")
  || who.starts_with("`")
  || who.starts_with("KILL_")
  || who.len() < 6
}

async fn fn_run_sniv
( sniv : dsta::Snively 
, txsn : tokio::sync::mpsc::Sender<dsta::Snively>
, tell_dr_new_robot : &tokio::sync::mpsc::Sender<crate::CtorBot>
, tell_dr_a_snively : &tokio::sync::mpsc::Sender<(usize, dsta::Snively)>
) -> Result<(), Box<dyn std::error::Error>>
{ let dbgspt = format!("DRS_FRS");
  log::debug!("ENTRY! - dr_seek::fn_run_sniv = {}", dbgspt);
  match sniv
  { 
    //--------------------------
    // bot construction managment
    //--------------------------
    dsta::Snively::MakeBotBuzz(s) => // (s:MakeBuzzBomber)                                   
    { log::info!("{} dr_seek got a snively::MakeBotBuzz = {:#?} ", dbgspt,s);
      if pre_prevent_reserved_names(&s.my_accounting.friendly_name)
      { log::error!("TODO: alert the frontend they can't use names that start with USER_...,DEAD_...,0...");
      }
      else
      { let _lazy = tell_dr_a_snively.send((1,dsta::Snively::MakeBotBuzz(s))).await;
      }
    }
    , 
    dsta::Snively::MakeStealth(s) => // (s:MakeStealthbot)                                   
    { log::info!("{} dr_seek got a snively::MakeBotBuzz = {:#?} ", dbgspt,s);
      if pre_prevent_reserved_names(&s.my_accounting.friendly_name)
      { log::error!("TODO: alert the frontend they can't use names that start with USER_...,DEAD_...,0...");
      }
      else
      { let _lazy = tell_dr_a_snively.send((1,dsta::Snively::MakeStealth(s))).await;
      }
    }
    dsta::Snively::MakeSallyBs(s)=>
    { if pre_prevent_reserved_names(&s.my_accounting.friendly_name)
      { log::error!("TODO: alert the frontend they can't use names that start with USER_...,DEAD_...,0...");
      }
      else
      { let _lazy = tell_dr_a_snively.send((1,dsta::Snively::MakeSallyBs(s))).await;
      }
    }

    dsta::Snively::MakeSwatBot(s)=>
    { if pre_prevent_reserved_names(&s.my_accounting.friendly_name)
      { log::error!("TODO: alert the frontend they can't use names that start with USER_...,DEAD_...,0..., less than 6");
      }
      else
      { let _lazy = tell_dr_a_snively.send((1,dsta::Snively::MakeSwatBot(s))).await;
      }
    }
    //--------------------------
    // NOTE: bot expansions go here
    //--------------------------
    , 
    //--------------------------
    // Regular bot manamenet
    //--------------------------
    dsta::Snively::LiveBotName(s) => // (String) // Turns on a bot for processing        
    { log::info!("{} dr_seek got a snively::LiveBotName = {:#?} ", dbgspt,s);
      let _lazy = tell_dr_a_snively.send((1,dsta::Snively::LiveBotName(s))).await;
    }
    ,
    dsta::Snively::KillBotName(s) => // (String) // Tells bot to do position exit and die
    { log::info!("{} dr_seek got a snively::KillBotName = {:#?} ", dbgspt,s);
      let _lazy = tell_dr_a_snively.send((1,dsta::Snively::KillBotName(s))).await;
    }
    ,
    dsta::Snively::StopBotName(s) => // (String) // Pauses an live processing bot          
    { log::info!("{} dr_seek got a snively::StopBotName = {:#?} ", dbgspt,s);
      let _lazy = tell_dr_a_snively.send((1,dsta::Snively::StopBotName(s))).await;
    }
    ,
    dsta::Snively::SendLogNote(s) => // (String) // Just for tesing right now              
    { log::info!("{} dr_seek got a snively::SendLogNote = {:#?} ", dbgspt,s);
      let _lazy = tell_dr_a_snively.send((1,dsta::Snively::SendLogNote(s))).await;
    }  
    , 
    //--------------------------
    // Frontend Updates
    //--------------------------
    dsta::Snively::EmeraldInfo(s) => // dsta::Finass
    { log::info!("{} dr_seek got a snively::EmeraldInfo = {:#?} ", dbgspt,s);
      let _lazy = tell_dr_a_snively.send((1,dsta::Snively::EmeraldInfo(s))).await;
      let _lazy = tell_dr_a_snively.send((1,dsta::Snively::SendLogNote(format!("Emerald Get!")))).await;
    }
    , 
    //--------------------------
    // Advanced requests
    //--------------------------

  // Dr Robo Related Stuff
    // MakeBotBuzz(MakeBuzzBomber),
    // MakeStealth(MakeStealthBot),
    // MakeSallyBs(MakeSallyFakes),

    // LiveBotName(String),
    // KillBotName(String),
    // StopBotName(String),
    // SendLogNote(String), // For 2 - way comms
    
    dsta::Snively::UserDemands(s) => // (AttemptDemands)) // Just for tesing right now              
    { log::info!("{} dr_seek got a snively::UserDemands = {:#?} ", dbgspt,s);
      let _lazy = tell_dr_a_snively.send((1,dsta::Snively::UserDemands(s))).await;
    }


  }
  Ok(())
}

pub(crate) async fn f_run_main_dr_seek // run_dr_robotnik_talk_seek // boot 
( mut output_talk : Foot
, tell_dr_new_robot : tokio::sync::mpsc::Sender<crate::CtorBot>
, tell_dr_a_snively : tokio::sync::mpsc::Sender<(usize, dsta::Snively)>
) -> () // Result<(), Box<dyn std::error::Error>>
{ let (mut server_kernel, mut new_connection_receiver) 
  = match run_dr_r_generate_serverkernel()
  { Ok(res) => res
    , _ =>
    { log::error!("Fuckin couldn't start the seek serverkernel");
      return ();
    }
  };
    
  log::info!
  ( "ENTRY: run_dr_robotnik_talk_seek() [RDRTS]; WHAT THIS DOES = {}"
    , r#"
        The point of run_dr_robotnik_talk_seek is to create X tasks:
        1) server kernel run => gets connections from the frontends.
        2) outgoing message sender (gets messages here, sends to 3+)
        3+?) task for each connected connection for rx/tx - 
        0) Last tasks are spawned in a while loop means this function runs indefinitly
        -1) if 0 fails, All of the above tasks are stopped.
      
        It takes in the seek! kernel and seek! receiver for comms, 
        and the receivers (Foot) for data that needs to be sent 
        out over the comms
      "#
  );

  let (broadcast_tx, _) = tokio::sync::broadcast::channel::<()>(1);
  let mut broadcast_rx = broadcast_tx.subscribe();

  log::debug!("[RDRTS] -> spawn first task T1 (seek server kernel run)...");
  let broadcast_tx_t1 = broadcast_tx.clone();
  tokio::spawn
  ( async move 
    { log::debug!("[RDRTS][T1] Spawning the server kernel's run method in a separate task...");
      let (cancel_run_tx, cancel_run_rx) 
        = tokio::sync::mpsc::channel::<()>(2); 
      struct CancelGuard(tokio::sync::mpsc::Sender<()>);
      impl Drop for CancelGuard 
      { fn drop(&mut self) 
        { let _ = futures::executor::block_on(self.0.send(()));
        }
      }

      let mut bt1 = broadcast_tx_t1.subscribe(); 
      let _guard = CancelGuard(cancel_run_tx);
      tokio::select!
      { res = server_kernel.run(cancel_run_rx) => 
        { log::debug!("[RDRTS][T1] res = server_kernel.run(cancel_run_rx) => FINISHED");
          match res
          { Ok(_) => 
            { log::error!("[RDRTS][T1] ServerKernel run ended unexpectedly l_l...");
            }
            Err(e) =>
            { log::error!("[RDRTS][T1] ServerKernel run failed: {:#?}", e);
            }
          }
        }
        // _ = tokio::signal::ctrl_c() => 
        // { log::info!("[RDRTS][T1] selected ctrl-c over server_kernel.run()!");
        // }
        res = bt1.recv() => 
        { match res
          { Ok(ok) =>
            { log::info!("[RDRTS][T1] received broadcast shutdown signal: {:#?}", ok);
            }
            Err(e) =>
            { log::info!("[RDRTS][T1] FAILED broadcast_tx_t1.subscribe().recv(): {:#?}", e);
            }
          }
        }
      }
      let _ = broadcast_tx_t1.send(());
      log::debug!("[RDRTS][T1] EXIT: task T1");
    }
  );

  // NOTE: T2 processes all outgoing message types (Snively, MadeBot, etc.) in a single tokio::select! loop, 
  // which could become unwieldy if more message types are added. 
  // Consider splitting T2 into separate tasks per message type for better 
  // scalability.
  log::debug!("[RDRTS] -> spawn next task T2 (outgoing seek sender)...");
  let (tx_handout, mut rx_handout) = tokio::sync::mpsc::channel::<Hand>(100);
  let broadcast_tx_t2 = broadcast_tx.clone();
  tokio::spawn
  ( async move 
    { log::debug!("[RDRTS][T2] Spawning the collector");
      let mut viewers: Vec<Option<Hand>> = Vec::new();
      
      async fn process_talk<T: Clone + Send + Sync + std::fmt::Debug + 'static>
      (   viewers: &mut Vec<Option<Hand>>
        , msg: T
        , field_name: &str
        , tx_lambda: fn(&Hand) -> &tokio::sync::mpsc::Sender<T>
      ) 
      { slog::trace!
        ( crate::glossary::TICK_SLOG,
          "[RDRTS][T2] collector processing own talk"
        );
        let mut failed_indices = Vec::new();
        for (idx, viewer) in viewers.iter().enumerate() 
        { if let Some(hand) = viewer 
          { if let Err(e) = ( tx_lambda(hand) ).send( msg.clone() ).await 
            { log::error!("[RDRTS][T2] Failed to send {} to viewer {}: {:#?}", field_name, idx, e);
              failed_indices.push(idx);
            } 
            else 
            { slog::debug!
              ( crate::glossary::TICK_SLOG
              , "[RDRTS][T2] Sent {} to viewer {} = {:#?}", field_name, idx, msg
              );
            }
          }
        }
        for idx in failed_indices.iter().rev() {
          viewers[*idx] = None;
          log::info!("[RDRTS][T2] Marked viewer {} as None due to send failure", idx);
        }
        while !viewers.is_empty() && viewers.last().is_none() // == Some(&None)
        { viewers.pop();
          log::debug!("[RDRTS][T2] Popped None viewer from back");
        }
      }

      let mut bt2 = broadcast_tx_t2.subscribe();

      loop 
      { tokio::select! 
        { res = rx_handout.recv() =>
          { match res
            { Some(hand) =>
              { while !viewers.is_empty() && viewers.last().is_none() // == Some(&None) 
                { viewers.pop();
                  log::debug!("[RDRTS][T2] Popped None viewer from back before adding new Hand");
                }
                if let Some(idx) = viewers.iter().position(|v| v.is_none()) {
                  log::debug!("[RDRTS][T2] Reusing slot {} for new Hand", idx);
                  viewers[idx] = Some(hand);
                } else {
                  log::debug!("[RDRTS][T2] Appending new viewer");
                  viewers.push(Some(hand));
                }
              }
              None =>
              { log::error!("[RDRTS][T2] Handout channel closed");
                break;
              }
            }
          }
          res = output_talk.rx_talk_snively.recv() =>
          { log::debug!("[RDRTS][T2] collector loop Rx Snively!");
            match res
            { Some(talk) =>
              { process_talk
                ( &mut viewers
                , talk
                , "Snively"
                , |hand| &hand.tx_talk_snively
                ).await;
              }
              None =>
              { log::error!("[RDRTS][T2] Snively channel closed");
                break;
              }
            }
          }
          res = output_talk.rx_talk_made_bot.recv() =>
          { match res
            { Some(talk) =>
              { process_talk(&mut viewers, talk, "MadeBot", |hand| &hand.tx_talk_made_bot).await;
              }
              None =>
              { log::error!("[RDRTS][T2] MadeBot channel closed");
                break;
              }
            }
          }
          
          res = output_talk.rx_talk_got_ttai.recv() =>
          { match res
            { Some(talk) =>
              { process_talk(&mut viewers, talk, "GotTtai", |hand| &hand.tx_talk_got_ttai).await;
              }
              None =>
              { log::error!("[RDRTS][T2] GotTtai channel closed");
                break;
              }
            }
          }
          res = output_talk.rx_talk_got_tick00.recv() =>
          { match res
            { Some(talk) =>
              { process_talk(&mut viewers, talk, "GotTick00", |hand| &hand.tx_talk_got_tick00).await;
              }
              None =>
              { log::error!("[RDRTS][T2] GotTick00 channel closed");
                break;
              }
            }
          }
          res = output_talk.rx_talk_got_tick01.recv() =>
          { match res
            { Some(talk) =>
              { process_talk(&mut viewers, talk, "GotTick01", |hand| &hand.tx_talk_got_tick01).await;
              }
              None =>
              { log::error!("[RDRTS][T2] GotTick01 channel closed");
                break;
              }
            }
          }
          _ = bt2.recv() => 
          { log::info!("[RDRTS][T2] received broadcast shutdown signal");
            break;
          }
        }
      } 

      log::error!("[RDRTS][T2] BROADCASTING EXIT: task T2");
      let _ = broadcast_tx_t2.send(());
      log::debug!("[RDRTS][T2] EXIT: task T2");
    }
  );


  log::info!("[RDRTS] Starting loop to handle the MPSC messages of new connections...");
  // we should do this infinitely...
  let mut cancel_txs : Vec<tokio::sync::watch::Sender<dsta::ksta::CancelInfo>> = Vec::new();
  let mut connection_count = 0;
  

  log::info!("[RDRTS] alerting dr_robo of completed startup");
  let _ = tell_dr_a_snively.send
  ( ( 1, dsta::Snively::SendLogNote(format!("dssu")) )
  ).await;

  loop 
  { tokio::select! 
    {
      _ = broadcast_rx.recv() => 
      {
        log::error!("[RDRTS] received broadcast shutdown signal, exiting connection loop");
        for cancel_tx in cancel_txs
        { let _ = cancel_tx.send
          ( dsta::ksta::CancelInfo
            { cancel : true
            , reason : format!("[RDRTS] Starting loop to handle the MPSC messages of new connections received broadcast shutdown signal, sending to all connections")
            , connection_number : 0
            } 
          );
        }
        break;
      }
      res = new_connection_receiver.recv() => 
      { let mut handle = match res
        { Some(hand) => 
          { hand
          }
          None =>
          { log::error!("ERROR! loop to handle mpsc messages of new seek connections has failed, no rx on thing!");
            continue;
            // break;
          }
        };

        connection_count += 1;
        log::info!
        ( "[RDRTS] Received new connection [{}]: C_ID {}, Remote: {}\n{}"
          , connection_count 
          , handle.c_id
          , handle.remote_addr
          , " For example, Going to Spawn a task to handle each of this connection's channels"
        );
        let cancel_tx = handle.cancel_conn_tasks_sender.clone();
        cancel_txs.push(cancel_tx);
        let mut cancel_rx = handle.cancel_conn_tasks_receiver.clone();
        
        let c_tx_sniv = handle.to_dude_snively.clone();
        let c_tx_made = handle.to_dude_madebot.clone();
        let c_tx_ttai = handle.to_dude_gotttai.clone();
        let c_tx_tik0 = handle.to_dude_gottick00.clone();
        let c_tx_tik1 = handle.to_dude_gottick01.clone();
        
        let hand = Hand 
        {   tx_talk_snively: c_tx_sniv
          , tx_talk_made_bot: c_tx_made
          , tx_talk_got_ttai: c_tx_ttai
          , tx_talk_got_tick00: c_tx_tik0
          , tx_talk_got_tick01: c_tx_tik1
        };
        log::debug!("[RDRTS] Sending Hand for connection {} to T2", handle.c_id);
        if let Err(e) = tx_handout.send(hand).await {
          log::error!("[RDRTS] Failed to send Hand for connection {}: {:#?}", handle.c_id, e);
        }

        let broadcast_tx_t3 = broadcast_tx.clone();
        let mut bt3 = broadcast_tx_t3.subscribe();
        let c_tx_sniv1 = handle.to_dude_snively.clone();
        let c_tx_newrb = tell_dr_new_robot.clone();
        let c_txdrsniv = tell_dr_a_snively.clone();
        tokio::spawn
        ( async move
          { log::debug!("[RDRTS][T3.{}] Backend spawned task to handle Snively messages", handle.c_id);
            loop
            { tokio::select!
              { res = handle.from_dude_snively.recv() =>
                { match res
                  { Some(snively) =>
                    { log::trace!("[RDRTS][T3.{}] Got SOME frontend snively", handle.c_id);
                      match fn_run_sniv
                      ( snively
                      , c_tx_sniv1.clone()
                      , &c_tx_newrb
                      , &c_txdrsniv
                      ).await
                      { Ok(_e) => { }
                        Err(e) => { log::error!("[RDRTS][T3.{}] ERROR:{:#?}\n FUTURE: Figure out about propagating this error usta main 295 run_dr_r_handle_snivley", handle.c_id,e); }
                      }
                    }
                    None =>
                    { log::error!("[RDRTS][T3.{}] Got a NONE from the snively recv... => channel closed", handle.c_id);
                      break;
                    }
                  }
                }
                _ = cancel_rx.changed() => 
                { let info = cancel_rx.borrow();
                  if info.cancel 
                  { log::debug!("[RDRTS][T3.{}] Comms Snively rx sees cancelation signal: {:#?}", handle.c_id, info);
                    break;
                  }
                }
                _ = bt3.recv() => 
                { log::info!("[RDRTS][T3.{}] received broadcast shutdown signal", handle.c_id);
                  break;
                }
              }
            }
            // log::error!("[RDRTS][T3] DONT BROADCAST EXIT!: task T3 is a per-connection");
            // let _ = broadcast_tx_t3.send(());
            log::debug!("[RDRTS][T3.{}] EXIT: Backend ending task to handle Snively messages", handle.c_id);
          }
        );

        //log::info!("[RDRTS] Sleeping 2 seconds after making a SEEK connection");
        //tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;
      }
    }
  }

  log::info!("[RDRTS] Connection handling loop exited! (ENDING PROGRAM)");
  log::debug!("[RDRTS] EXIT: run_dr_robotnik_talk_seek");
  //Ok(())
  ()
}