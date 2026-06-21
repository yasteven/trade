
// trade/vsta/src/lens/v_dr_r/state.rs

use iced::Subscription;

#[derive(Debug, Clone)]
pub enum ViewBotMake
{
  MakeNone
  , MakeBuzz
  , MakeStealth
  , MakeSally
  , MakeSwat
}

#[derive(Debug, Clone)]
pub enum ViewBotTalk
{
  Startup
  , Connecting(String)
  , Connected
  , Failed(String)
}

#[derive(Debug, Clone)]
pub enum Message 
{
  // Auto things
    ConnectToBackend(String),
    CancelConnection,
    Connected,
    Disconnected(String),

  // Used from the other frontends 
    SendSnively(dsta::Snively), // This is actallu what we get from remote so bad name
    
  // Used to decode the RX snively
    RecvLogNote(String),           
    RecvMadeBot(dsta::MadeBot),
    RecvGotTtai(dsta::GotTtai),
    RecvEmerald(dsta::PushTickerResult), // used to set our ticker hashmap, presets, etc.
    RecvTicks00(dsta::Ticker),


    // Iced things
    FromMakeBuzz(crate::form::v_buzz_make::Message),
    FromMakeStealth(crate::form::v_stealth_make::Message),
    FromMakeSally(crate::form::v_sally_make::Message),
    FromMakeSwat(crate::form::v_swat_make::Message),
    FromMakeNico(crate::form::v_nico::Message),
    ClearLogNotes,
    ViewSwitcher(ViewBotMake),
}

pub struct DrRobotnikV 
{
    tell_remote: tokio::sync::mpsc::Sender<crate::lens::v_dr_r::Message>,
  // mv => This Stuff for the view: // TO See the ttai actions:
    pub mv_connection_state: ViewBotTalk,
    pub mv_our_tool_display: ViewBotMake,
    pub mv_account_0_update: dsta::AccountUpdate,
    pub mv_all_my_positions: dsta::PositionsList,
    pub mv_every_sent_order: Vec<(dsta::Order, dsta::PushOrderResult)>,
    pub mv_order_update_vec: Vec<dsta::PlacedOrder>,

    pub mv_all_push_tickers: std::collections::HashMap
    < String // resuld's finass::ticker_name() TICKER NAME -> ticker
    , dsta::PushTickerResult
    >,

    pub mv_live_bot_by_name: String,
    pub mv_stop_bot_by_name: String,
    pub mv_kill_bot_by_name: String,
    pub mv_to_send_log_note: String,

  // My => my sub views
    pub my_emerald_collection_control: crate::core::EmeraldCollectionControl,

    pub my_make_buzz_bomber: crate::form::v_buzz_make::MakeBuzzV,
    pub my_make_stealth_bot: crate::form::v_stealth_make::MakeStealthV,
    pub my_make_sally_bot: crate::form::v_sally_make::MakeSallyV,
    pub my_make_swat_bot: crate::form::v_swat_make::MakeSwatV,
    pub my_log_notes: crate::lens::v_log_notes::LogNotesV,    // ← ADD BACK
    pub my_nico: crate::form::v_nico::MakeNicoV,
  // Cached image handles for the DrR view sidebar (created once at startup)
    pub icon_dr_r: iced::advanced::image::Handle,
    pub icon_buzz: iced::advanced::image::Handle,
    pub icon_stealth: iced::advanced::image::Handle,
    pub icon_sally: iced::advanced::image::Handle,
    pub icon_swat: iced::advanced::image::Handle,
    pub icon_logs: iced::advanced::image::Handle,
}

impl DrRobotnikV
{
  pub fn new
  ( tell_remote: tokio::sync::mpsc::Sender<crate::lens::v_dr_r::Message>
  ) -> Self
  {
    let options1 =
    vec!
    [
      dsta::MarketDirection::GetStonk
      , dsta::MarketDirection::Sideways
      , dsta::MarketDirection::Corrects
    ];

    let options2 =
    vec!
    [
      dsta::UsaMarketTimes::PowerEnds // 3:50 pm
      , dsta::UsaMarketTimes::ItsClosed // 3:59:59 pm
      , dsta::UsaMarketTimes::Hurry5sec // 3:59:55 pm
      , dsta::UsaMarketTimes::TminusTen // 3:59:50 pm
      , dsta::UsaMarketTimes::Tminus30s // 3:59:30 pm
      , dsta::UsaMarketTimes::Tminus60s // 3:59:00 pm
      , dsta::UsaMarketTimes::DeadShort // 3:45 pm
      , dsta::UsaMarketTimes::PowerFive // 3:55 pm
      , dsta::UsaMarketTimes::PowerEasy // 3:30 pm
      , dsta::UsaMarketTimes::PowerHour // 3:00 pm
      , dsta::UsaMarketTimes::FedSpeach // 2:30 pm
      , dsta::UsaMarketTimes::FedMinute // 2:00 pm
      , dsta::UsaMarketTimes::TradeTime // 1:00 pm
      , dsta::UsaMarketTimes::LunchTime // 12:00 pm
      , dsta::UsaMarketTimes::EuroClose // 10:30 am
      , dsta::UsaMarketTimes::DoneTrend // 10:00 am
      , dsta::UsaMarketTimes::OpenChaos // 9:30 am
    ];

    let icon_dr_r = iced::advanced::image::Handle::from_bytes(include_bytes!("../../../jpg/dr_r_icon_b.jpg").as_slice());
    let icon_buzz = iced::advanced::image::Handle::from_bytes(include_bytes!("../../../jpg/buzz_icon.jpg").as_slice());
    let icon_stealth = iced::advanced::image::Handle::from_bytes(include_bytes!("../../../jpg/stealth_icon.jpg").as_slice());
    let icon_sally = iced::advanced::image::Handle::from_bytes(include_bytes!("../../../jpg/sally_icon.jpg").as_slice());
    let icon_swat = iced::advanced::image::Handle::from_bytes(include_bytes!("../../../jpg/swat_icon.jpg").as_slice());
    let icon_logs = iced::advanced::image::Handle::from_bytes(include_bytes!("../../../jpg/logs_icon_b.jpg").as_slice());

    DrRobotnikV
    {
      tell_remote : tell_remote
      , mv_connection_state: ViewBotTalk::Startup // !("STARTUP")
      , mv_our_tool_display: ViewBotMake::MakeNone
      , mv_account_0_update: dsta::AccountUpdate::default()
      , mv_all_my_positions: dsta::PositionsList::default()
      , mv_every_sent_order: Vec::new()
      , mv_order_update_vec: Vec::new()
      , mv_all_push_tickers: std::collections::HashMap::new()
      , mv_live_bot_by_name: format!("")
      , mv_stop_bot_by_name: format!("")
      , mv_kill_bot_by_name: format!("")
      , mv_to_send_log_note: format!("")

  
      , my_emerald_collection_control: crate::core::EmeraldCollectionControl::new()
      , my_make_buzz_bomber: crate::form::v_buzz_make::MakeBuzzV::new()
      , my_make_stealth_bot: crate::form::v_stealth_make::MakeStealthV::new()
      , my_make_sally_bot: crate::form::v_sally_make::MakeSallyV::new()
      , my_make_swat_bot: crate::form::v_swat_make::MakeSwatV::new()

      , my_nico: crate::form::v_nico::MakeNicoV::new()  
      , my_log_notes: crate::lens::v_log_notes::LogNotesV::new()
      
      , icon_dr_r
      , icon_buzz
      , icon_stealth
      , icon_sally
      , icon_swat
      , icon_logs
    }
  }

  pub fn update(&mut self, message: crate::lens::v_dr_r::Message) -> iced::Task<crate::lens::v_dr_r::Message>
  {
    match message
    {
      // BACKEND DATA & CONNECTION
        Message::ConnectToBackend(server_addr) =>
        { log::trace!("v_dr_r::state::update::Message::ConnectToBackend({})", server_addr);
          self.mv_connection_state = ViewBotTalk::Connecting(format!("SERVER {}", server_addr));
          let tx = self.tell_remote.clone();
          iced::Task::perform
          (
            async move
            {
              tx.send( crate::lens::v_dr_r::Message::ConnectToBackend(server_addr)).await
                .map_err
                (
                  |e| Box::new(e) as Box<dyn std::error::Error + Send + Sync>
                )
            }
            , |result| match result
            {
              Ok(()) =>
              {
                log::info!("RAISING crate::lens::v_dr_r::Message::Connected due to OK result from tx.send crate::lens::v_dr_r::Message::ConnectToBackend(server_addr)");
                crate::lens::v_dr_r::Message::Connected
              }
              , Err(e) =>
              {
                let erm =format!("tx.send( crate::lens::v_dr_r::Message::ConnectToBackend(server_addr)).await Failed to send ConnectToBackend: {:?}", e);
                log::error!("RAISING crate::lens::v_dr_r::Message::Disconnected crate::lens::v_dr_r::Message::Connected due to {} ", erm);
                crate::lens::v_dr_r::Message::Disconnected(erm)
              }
            }
          )
        }
        , Message::Connected =>
        {
          log::info!("Connected to backend");
          self.mv_connection_state = ViewBotTalk::Connected; // format!("CONNECTTED!");
          iced::Task::none()
        }
        , Message::Disconnected(reason) =>
        {
          log::info!("Disconnected: {}", reason);
          self.mv_connection_state = ViewBotTalk::Failed(format!("DISCONNECTTED!: {}", reason));
          iced::Task::none()
        }
        , crate::lens::v_dr_r::Message::SendSnively(bot_talk) =>
        {
          let tx = self.tell_remote.clone();
          iced::Task::perform
          (
            async move
            {
              log::info!("crate::lens::v_dr_r::Message::SendSnively(bot_talk) => requesting iced task perform mpsc tx.send(Message::SendSnively(bot_talk)).await");
              tx.send(Message::SendSnively(bot_talk)).await
                .map_err(|e| Box::new(e) as Box<dyn std::error::Error + Send + Sync>)
            }
            ,
            |result| match result
            {
              Ok(()) =>
              {
                log::info!("RAISING crate::lens::v_dr_r::Message::Connected due to OK result from tx.send(Message::SendSnively(bot_talk)).await");
                Message::Connected
              }
              , Err(e) =>
              {
                log::error!("Failed to send Snively: {:?}", e);
                Message::Disconnected("SendSnively error".to_string())
              }
            }
          )
        }
        , Message::RecvMadeBot(bot_talk) =>
        {
          log::debug!("Received dsta::MadeBot: {:?}", bot_talk);
          iced::Task::none()
        }
        , Message::CancelConnection =>
        {
          log::debug!("v_dr_r::update(dr_r) Received v_dr_r::Message::CancelConnection");
          self.mv_connection_state = ViewBotTalk::Failed(format!("CancelConnection called?"));
          let tx = self.tell_remote.clone();

          iced::Task::perform
          (
            async move
            {
              // uh, make sure the rx of the cancelconnection on the main side doesn't
              // do something dumb like pass to an update call dr_r cancel this would be stak overflow
              tx.send(crate::lens::v_dr_r::Message::CancelConnection).await
                .map_err(|e| Box::new(e) as Box<dyn std::error::Error + Send + Sync>)
            }
            ,
            |result| match result
            {
              Ok(()) =>
              {
                log::info!("crate::lens::v_dr_r::Message::CancelConnection sent OK Manual cancelation to the main");
                crate::lens::v_dr_r::Message::Disconnected(format!("crate::lens::v_dr_r::Message::CancelConnection"))
              }
              , Err(e) =>
              {
                let erm = format!("crate::lens::v_dr_r::Message::CancelConnection Failed to mpsc send CancelConnection to the main: {:?}", e);
                log::error!("{}", erm);
                crate::lens::v_dr_r::Message::Disconnected(erm)
              }
            }
          )
        }
        , Message::RecvGotTtai(got_ttai) =>
        {
          log::debug!("Received dsta::GotTtai: {:?}", got_ttai);
          iced::Task::none()
        }

        , Message::RecvEmerald(got_info) => 
        {
          log::debug!("v_dr_r::update RecvEmerald: {:#?}", got_info);
        
          // Store in THE canonical location (in DrRobotnikV.emerald_collection_control)
          self.my_emerald_collection_control.update(
              crate::core::v_emerald_collection::Message::UpdateEmerald(got_info.clone())
          );
            
          iced::Task::none()
        }

        , Message::RecvTicks00(got_info) => 
        { slog::trace!(crate::glossary::TICK_SLOG,"v_dr_r::update RecvTicks00: {:#?}", got_info);
          self.my_emerald_collection_control.update
          ( crate::core::v_emerald_collection::Message::UpdateTickers(got_info)
          );
          iced::Task::none()
        }

      // VIEW INTERACTION UPDATES

      , Message::ViewSwitcher(next_view) => 
      { log::trace!("v_Dr_r - new  viewswitcher, next = {:#?}", next_view);
          self.mv_our_tool_display = next_view;
          
          // Update emerald collection display context
          match self.mv_our_tool_display 
          {
              ViewBotMake::MakeBuzz => {
                  self.my_emerald_collection_control
                      .set_display_context(crate::core::BotTypeContext::BuzzBot);
              }
              ViewBotMake::MakeStealth => {
                  self.my_emerald_collection_control
                      .set_display_context(crate::core::BotTypeContext::StealthBot);
              }
              ViewBotMake::MakeSally => {
                  self.my_emerald_collection_control
                      .set_display_context(crate::core::BotTypeContext::SallyBot);
              }
              ViewBotMake::MakeSwat => {
                  self.my_emerald_collection_control
                      .set_display_context(crate::core::BotTypeContext::SwatBot);
              }
              ViewBotMake::MakeNone => {
                  // Neutral
              }
          }
          iced::Task::none()
      }

      , Message::FromMakeBuzz(input) =>
      { match input
        {
          crate::form::v_buzz_make::Message::ProcessCreateBot(bot) =>
          {
            log::info!("THIS IS WHERE WE SEND CREATE A BOT: \n{:#?}",bot);

            let tx = self.tell_remote.clone();
            iced::Task::perform
            (
              async move
              {
                log::debug!("THIS IS WHERE iced::Task::perform mpsc SEND CREATE A BOT: \n{:#?}",bot);
                tx.send
                (
                  Message::SendSnively
                  (
                    dsta::Snively::MakeBotBuzz(bot)
                  )
                ).await
                .map_err
                (
                  |e| Box::new(e)
                    as Box<dyn std::error::Error + Send + Sync>
                )
              }
              ,
              |result| match result
              {
                Ok(()) =>
                {
                  let erm = format!("iced::Task::perform mpsc SEND CREATE A BOT OK => set Connected state");
                  Message::Connected
                }
                , Err(e) =>
                {
                  let erm = format!("iced::Task::perform mpsc SEND CREATE A BOT ERROR: {:?}", e);
                  log::error!("{}",erm);
                  Message::Disconnected(erm)
                }
              }
            )
          }
          , _ =>
          {
            self.my_make_buzz_bomber.update(input) // Task<v_buzz_make::Message>
                .map(crate::lens::v_dr_r::Message::FromMakeBuzz) // Task<v_dr_r::Message>
          }
        }
      }
      , Message::FromMakeStealth(input) =>
      { match input
        { crate::form::v_stealth_make::Message::ProcessCreateBot(bot) =>
          { log::info!("THIS IS WHERE WE SEND CREATE A STEALTH BOT: \n{:#?}",bot);
            let tx = self.tell_remote.clone();
            iced::Task::perform
            ( async move
              { log::debug!("THIS IS WHERE iced::Task::perform mpsc SEND CREATE A STEALTH BOT: \n{:#?}",bot);
                tx.send
                ( Message::SendSnively
                  ( dsta::Snively::MakeStealth(bot)
                  )
                ).await
                .map_err
                ( |e| Box::new(e) as Box<dyn std::error::Error + Send + Sync>
                )
              }
              ,
              |result| match result
              { Ok(()) =>
                { let erm = format!("iced::Task::perform mpsc SEND CREATE A STEALTH BOT OK => set Connected state");
                  Message::Connected
                }
                , Err(e) =>
                { let erm = format!("iced::Task::perform mpsc SEND CREATE A STEALTH BOT ERROR: {:?}", e);
                  log::error!("{}",erm);
                  Message::Disconnected(erm)
                }
              }
            )
          }
          , _ =>
          { self.my_make_stealth_bot.update(input) // Task<v_stealth_make::Message>
                .map(crate::lens::v_dr_r::Message::FromMakeStealth) // Task<v_dr_r::Message>
          }
        }
      }
      , Message::FromMakeSally(input) =>
      { log::debug!("v_dr_r processing a FromMakeSally");
        match input
        {
          crate::form::v_sally_make::Message::ProcessCreateBot(how) =>
          { log::info!("Sending create SALLY BOT");
            let tx = self.tell_remote.clone();
            iced::Task::perform
            ( async move
              { tx.send(Message::SendSnively(dsta::Snively::MakeSallyBs(how))).await
                .map_err(|e| Box::new(e) as Box<dyn std::error::Error + Send + Sync>)
              }
              , |result| match result
              { Ok(()) => Message::Connected
                , Err(e) =>
                { log::error!("Send error: {:?}", e);
                  Message::Disconnected(format!("SendSnively error: {:?}", e))
                }
              }
            )
          }

          // Emerald collection messages → update emerald control
          , crate::form::v_sally_make::Message::EmeraldCollectionMessage(msg) =>
          { log::trace!("v_dr_r: EmeraldCollectionMessage for Sally");
            
            let _emerald_task = self.my_emerald_collection_control.update(msg.clone());
            
            // Sync selected emerald type to Sally
            if let Some(et) = self.my_emerald_collection_control.selected_type.clone() 
            { self.my_make_sally_bot.set_selected_emerald(et);
            }
            // Sync selected FinAss (option chain click) to Sally
            if let Some(finass) = self.my_emerald_collection_control.get_selected_finass() 
            { self.my_make_sally_bot.set_selected_finass(finass);
            }

            iced::Task::none()
          }

          // Everything else → delegate to Sally view
          , _ =>
          { self.my_make_sally_bot.update(input).map(Message::FromMakeSally)
          }
        }
      }
      , Message::FromMakeSwat(input) =>
      { log::debug!("v_dr_r processing a FromMakeSwat");
        match input
        {
          // Create bot
          crate::form::v_swat_make::Message::ProcessCreateBot(how) =>
          { log::info!("Sending create SWAT BOT");
            let tx = self.tell_remote.clone();
            iced::Task::perform
            ( async move
              { tx.send(Message::SendSnively(dsta::Snively::MakeSwatBot(how))).await
                .map_err(|e| Box::new(e) as Box<dyn std::error::Error + Send + Sync>)
              }
              , |result| match result
              { Ok(()) => Message::Connected
                , Err(e) =>
                { log::error!("Send error: {:?}", e);
                  Message::Disconnected(format!("SendSnively error: {:?}", e))
                }
              }
            )
          }


          // Emerald collection messages → update emerald control AND sync to view state
          , crate::form::v_swat_make::Message::EmeraldCollectionMessage(msg) =>
          { log::trace!("v_dr_r: EmeraldCollectionMessage for Swat");
            log::trace!("v_dr_r // update the canonical emerald collection");
            let _emerald_task = self.my_emerald_collection_control.update(msg.clone());
            if let Some(et) = self.my_emerald_collection_control.selected_type.clone() 
            { log::trace!("v_dr_r // Sync selected emerald type to Swat");
              self.my_make_swat_bot.set_selected_emerald(et);
            } else
            { log::trace!("v_dr_r // NO SYNC selected emerald type to Swat");
            }

            
            if let Some(finass) = self.my_emerald_collection_control.get_selected_finass() 
            { log::trace!("v_dr_r // Sync selected finass type to Swat");
              self.my_make_swat_bot.set_selected_finass(finass);
            }
            else
            { log::trace!("v_dr_r // NO SYNC selected finass type to Swat");
            }
            iced::Task::none()
          }
          
          // Preset selection
          , crate::form::v_swat_make::Message::SwatPresetSelected(preset) =>
          { log::info!("Swat preset selected: {:?}", preset);
            self.my_make_swat_bot.apply_swat_preset(preset);
            iced::Task::none()
          }

          // Tab switching
          , crate::form::v_swat_make::Message::SwitchTab(tab) =>
          { log::debug!("Swat: switching tab");
            self.my_make_swat_bot.update(input).map(Message::FromMakeSwat)
          }

          // Everything else → delegate to Swat view
          , _ =>
          { self.my_make_swat_bot.update(input).map(Message::FromMakeSwat)
          }
        }
      }

      , Message::FromMakeNico(input) =>
      {
        match input
        {
          crate::form::v_nico::Message::PressedSendMessage =>
          {
            log::info!("Nico send message");
            self.my_nico.update(input).map(crate::lens::v_dr_r::Message::FromMakeNico)
          }
          , _ =>
          {
            self.my_nico.update(input).map(crate::lens::v_dr_r::Message::FromMakeNico)
          }
        }
      }

      // LOG NOTES control
      , Message::ClearLogNotes =>
      { let _ = self.my_log_notes.update(crate::lens::v_log_notes::Message::ClearAllNotes);
        iced::Task::none()
      }
      , Message::RecvLogNote(note) =>
      { log::trace!("v_dr_r Message::RecvLogNote(note) note = {:#?}", note);
        let cloned_note = note.clone();
        let _ = self.my_log_notes.update(crate::lens::v_log_notes::Message::AddRecvNote(cloned_note));
        //self.mv_to_send_log_note = note; // why was this here?
        iced::Task::none()
      }
    
      // End of the Message processing
    }
  }

  pub fn subscription(&self) -> Subscription<Message>
  { let subs : Vec<iced::Subscription<Message>> = vec!
    [ Subscription::none()
    ];
    Subscription::batch(subs)
  }
}