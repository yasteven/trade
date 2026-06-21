// trade/usta/src/sbot/mod.rs
pub mod swat_bot;
pub mod buzz_bomber;
pub mod stealth_bot;
pub mod sally_bot;
pub mod errors;

// sbot/mod.rs — fully qualified, no use statements required
// sbot/mod.rs — fully qualified paths, no use statements

pub trait HasFriendlyName {
    fn friendly_name(&self) -> &str;
}

pub async fn f_report_error<T, B>(
    stone: &T,
    brain: &B,
    stuff: &crate::Allocation,
    to_dr: &crate::BridgeDrHand,
    print: &str, // the main error message
) -> crate::sbot::errors::BotResult
where
    T: std::fmt::Debug + HasFriendlyName,
    B: std::fmt::Debug,
{
    let type_tag = std::any::type_name_of_val(stone)
        .split("::")
        .last()
        .unwrap_or("UNKNOWN")
        .to_uppercase();

    let dbgspt = format!("[{}_ERROR {}]", type_tag, stuff.bot_name);
    let timestamp = chrono::Utc::now().format("%Y-%m-%d %H:%M:%S%.f").to_string();

    let dbgmsg = format!(
        "{} \n stone: {:#?} \n brain: {:#?} \n stuff: {:#?} \n ",
        print, stone, brain, stuff
    );
    log::error!("{}", dbgmsg);

    let errmsg = format!(
        "{} ( from friendly-name {} )",
        print,
        stone.friendly_name()
    );

    to_dr.tell_dr_a_snively
        .send(dsta::Snively::SendLogNote(errmsg))
        .await
        .map_err(|e| {
            let fail_msg = format!(
                "{} [{}] CRITICAL: Failed to send error note to Snively → core/logging likely dead",
                dbgspt, timestamp
            );
            log::error!("{}: {}", fail_msg, e);
            crate::sbot::errors::BotError::Other(fail_msg)
        })?;

    log::trace!("{} [{}] Error successfully reported to Snively", dbgspt, timestamp);
    Ok(())
}

pub async fn f_report_info<T, B>(
    stone: &T,
    _brain: &B,
    stuff: &crate::Allocation,
    to_dr: &crate::BridgeDrHand,
    print: &str, // the main info message
) -> crate::sbot::errors::BotResult
where
    T: std::fmt::Debug + HasFriendlyName,
    B: std::fmt::Debug,
{
    let type_tag = std::any::type_name_of_val(stone)
        .split("::")
        .last()
        .unwrap_or("UNKNOWN")
        .to_uppercase();

    let dbgspt = format!("[{}_INFO {}]", type_tag, stuff.bot_name);
    let timestamp = chrono::Utc::now().format("%Y-%m-%d %H:%M:%S%.f").to_string();

    let msgmsg = format!("{}", print);
    log::info!("{} Going to tell snively: {}", dbgspt, msgmsg);

    to_dr.tell_dr_a_snively
        .send(dsta::Snively::SendLogNote(msgmsg))
        .await
        .map_err(|e| {
            let fail_msg = format!(
                "{} [{}] CRITICAL: Failed to send info note to Snively → core/logging likely dead",
                dbgspt, timestamp
            );
            log::error!("{}: {}", fail_msg, e);
            crate::sbot::errors::BotError::Other(fail_msg)
        })?;

    log::trace!("{} [{}] Info successfully reported to Snively", dbgspt, timestamp);
    Ok(())
}

pub async fn fn_load_bot_state<T, B>(
    friendly_name: &str,
) -> crate::sbot::errors::BotResult<(T, B, crate::Allocation)>
where
    T: serde::de::DeserializeOwned + HasFriendlyName,
    B: serde::de::DeserializeOwned,
{
  let dbgspt = format!("[{}_LOAD {}]", std::any::type_name::<T>().split("::").last().unwrap_or("UNK").to_uppercase(), friendly_name);
  let timestamp = chrono::Utc::now().format("%Y-%m-%d %H:%M:%S%.f").to_string();
  let path = format!("./dat/{}.json", friendly_name);
  log::debug!("{} ENTRY! [{}] Attempting to load from {}", dbgspt, timestamp, path);
  let json_content = match tokio::fs::read_to_string(&path).await
  { Ok(content) => content,
    Err(e) =>
    { let errmsg = format!("{} [{}] File read failed: {}", dbgspt, timestamp, e);
      return Err(crate::sbot::errors::BotError::IoError(errmsg));
    }
  };
  match serde_json::from_str::<(T, B, crate::Allocation)>(&json_content)
  { Ok((stone, brain, stuff)) =>
    { log::debug!("{} [{}] Successfully loaded state from {}", dbgspt, timestamp, path);
      Ok((stone, brain, stuff))
    }
    Err(e) =>
    { let errmsg = format!("[LOAD STATE {}] [{}] JSON parse failed: {}", friendly_name, timestamp, e);
      let pcont = format!("Problematic content (first 300 chars): {}", &json_content[..300.min(json_content.len())]);
      Err(crate::sbot::errors::BotError::JsonError(format!("{}\n{}", errmsg, pcont)))
    }
  }
}

pub async fn fn_save_bot_state<T, B>(
    stone: &T,
    brain: &B,
    stuff: &crate::Allocation,
    to_dr: &crate::BridgeDrHand,
) -> crate::sbot::errors::BotResult
where
    T: serde::Serialize + HasFriendlyName + std::fmt::Debug,
    B: serde::Serialize + std::fmt::Debug,
{
  let friendly_name = stone.friendly_name();
  let dbgspt = format!("[{}_SAVE {}]", std::any::type_name::<T>().split("::").last().unwrap_or("UNK").to_uppercase(), friendly_name);
  let timestamp = chrono::Utc::now().format("%Y-%m-%d %H:%M:%S%.f").to_string();
  log::debug!("{} ENTRY! [{}] : Preparing to save state", dbgspt, timestamp);
  let dat_dir = std::path::Path::new("./dat");
  if !dat_dir.exists()
  { tokio::fs::create_dir_all(&dat_dir).await.expect("Failed to create ./dat directory for bot state");
    log::warn!("Created ./dat directory for bot state files");
  }
  let path = format!("./dat/{}.json", friendly_name);
  let state_snapshot = (stone, brain, stuff);
  let json = serde_json::to_string_pretty(&state_snapshot)
    .map_err(|e| {
      let errmsg = format!("{} Failed to serialize state: {}", dbgspt, e);
      crate::sbot::errors::BotError::JsonError(errmsg)
    })?;
  tokio::fs::write(&path, json).await
    .map_err(|e| {
      let msg = format!("Failed to write state file {}: {}", path, e);
      let errmsg = format!("{} [{}] {}", dbgspt, timestamp, msg);
      crate::sbot::errors::BotError::IoError(errmsg)
    })?;
  let msgmsg = format!("[SAVE STATE {}] [{}] Successfully saved state to {}", friendly_name, timestamp, path);
  let _ = crate::sbot::f_report_info(stone, brain, stuff, to_dr, &msgmsg).await;
  Ok(())
}

pub fn update_position_ticker
( account: &mut crate::BotaAccount,
  ticker: &dsta::Ticker,
) 
{ let ticker_name = &ticker.name;
  let try_update = |pos_opt: &mut Option<dsta::OwnedPosition>| 
  { if let Some(pos) = pos_opt 
    { match &mut pos.asset 
      { dsta::FinAss::StkDeets(stk) if stk.ticker_name == *ticker_name => 
        { stk.last_ticker = Some(ticker.clone());
          slog::trace!
          ( crate::glossary::TICK_SLOG
          , "[TICKER_UPDATE] Updated stock {}"
          , ticker_name
          );
          return true;
        }
        dsta::FinAss::OptDeets(opt) if opt.ticker_name == *ticker_name => 
        { opt.last_ticker = Some(ticker.clone());
          slog::trace!
          ( crate::glossary::TICK_SLOG
          , "[TICKER_UPDATE] Updated option {}"
          , ticker_name
          );
          return true;
        }
        _ => {}
      }
    }
    false
  };
  if !try_update(&mut account.m_stock)
  { if !try_update(&mut account.m_long_calls)
    { if !try_update(&mut account.m_long_puts)
      { if !try_update(&mut account.m_short_calls) 
        { if !try_update(&mut account.m_short_puts)
          { // log::warn!("Swat Bot got un-tracked ticker !");
          }
        }
      }
    }
  }
}

//===================================================================
// SHARED HELPERS: Position lookup by order_name
//===================================================================
pub fn position_for_order_name<'a>(
    account: &'a crate::BotaAccount,
    ticker: &str,
) -> Option<&'a dsta::OwnedPosition> {
    if let Some(pos) = &account.m_stock {
        if pos.asset.order_name() == ticker {
            return Some(pos);
        }
    }
    if let Some(pos) = &account.m_long_calls {
        if pos.asset.order_name() == ticker {
            return Some(pos);
        }
    }
    if let Some(pos) = &account.m_long_puts {
        if pos.asset.order_name() == ticker {
            return Some(pos);
        }
    }
    if let Some(pos) = &account.m_short_calls {
        if pos.asset.order_name() == ticker {
            return Some(pos);
        }
    }
    if let Some(pos) = &account.m_short_puts {
        if pos.asset.order_name() == ticker {
            return Some(pos);
        }
    }
    None
}

pub fn merge_fresher_tickers
( target: &mut crate::BotaAccount,
  source: &crate::BotaAccount,
) 
{ fn merge_pos
  ( target_pos_opt: &mut Option<dsta::OwnedPosition>
  , source_pos_opt: &Option<dsta::OwnedPosition>
  ) 
  { if let (Some(target_pos), Some(source_pos)) 
    = (target_pos_opt, source_pos_opt) 
    { let target_ticker = match &target_pos.asset 
      { dsta::FinAss::StkDeets(s) => s.last_ticker.as_ref(),
        dsta::FinAss::OptDeets(o) => o.last_ticker.as_ref(),
        _ => 
        { log::warn!("Steven doesnt do bnds yet"); None },
      };

      let source_ticker = match &source_pos.asset 
      { dsta::FinAss::StkDeets(s) => s.last_ticker.as_ref(),
        dsta::FinAss::OptDeets(o) => o.last_ticker.as_ref(),
            _ => { log::warn!("Steven doesnt do bnds yet"); None },
      };

      let prefer_source_ticker = match (source_ticker, target_ticker) 
      { (Some(s), Some(t)) => s.time >= t.time,
        (Some(_), None)     => true,
        (None, Some(_))     => false,
        (None, None)        => false,
      };

      if prefer_source_ticker 
      { if let Some(ticker) = source_ticker.cloned() 
        { match &mut target_pos.asset 
          { dsta::FinAss::StkDeets(s) => s.last_ticker = Some(ticker.clone()),
            dsta::FinAss::OptDeets(o) => o.last_ticker = Some(ticker.clone()),
            _ => {}
          }
        }
      }
    }
  }
  merge_pos(&mut target.m_stock, &source.m_stock);
  merge_pos(&mut target.m_long_calls, &source.m_long_calls);
  merge_pos(&mut target.m_long_puts, &source.m_long_puts);
  merge_pos(&mut target.m_short_calls, &source.m_short_puts);
  merge_pos(&mut target.m_short_puts, &source.m_short_puts);
}