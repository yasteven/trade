// trade/vsta/src/main.rs 

use iced::{
  Element, Subscription, Task,
  widget::{Button, Column, Row, Text},
  Length, Alignment
};

mod base;
mod core;
mod form;
mod lens;
mod dat;


pub mod glossary
{
// Core signals from ROBO to BOT over snively:
pub(crate) const MAIN_SLOG : u64 = 100;
pub(crate) const TICK_SLOG : u64 = 100;

}



fn kill_pid(pid: i32)
{
  log::debug!("kill_pid called for PID: {}", pid);
  let result = std::process::Command::new("kill")
    .arg("-SIGINT")
    .arg(pid.to_string())
    .output();
  match result
  {
    Ok(output) =>
    {
      if output.status.success()
      {
        log::debug!("Successfully sent SIGINT to PID: {}", pid);
      }
      else
      {
        log::error!(
          "Failed to send SIGINT to PID: {}, stderr: {:?}",
          pid,
          String::from_utf8_lossy(&output.stderr)
        );
      }
    }
    , Err(e) =>
    {
      log::error!("Error executing kill command for PID: {}: {}", pid, e);
    }
  }
}

// MESSAGE HIERARCHY:
// Application top-level message enum that routes to all subsystems
#[derive(Debug, Clone)]
enum Message
{ DrR(crate::lens::v_dr_r::Message)
  , DrRBatch(Vec<crate::lens::v_dr_r::Message>)
  , Buzz(crate::lens::v_buzz::Message)
  , Stev(crate::lens::v_stev::Message)

  , SwitchToDrR(())
  , SwitchToBuzz(())
  , SwitchToStev(())

  , ToggleConsole
  , WindowSizeUpdated(iced::Size)
  , WindowClosed
  , WindowOpened(iced::window::Id)
  , None
}

enum View
{ DrR
  , Buzz
  , Stev
}

struct SonicApp
{ current_view: View
  , view_dr_r: crate::lens::v_dr_r::DrRobotnikV
  , broadcast_rx_arc: std::sync::Arc<tokio::sync::Mutex<tokio::sync::broadcast::Receiver<crate::lens::v_dr_r::Message>>>
  , mpsc_rx_arc: std::sync::Arc<tokio::sync::Mutex<tokio::sync::mpsc::Receiver<crate::lens::v_dr_r::Message>>>
  , view_buzz: crate::lens::v_buzz::BuzzBomberV
  , view_stev: crate::lens::v_stev::StevV
  , window_size: iced::Size
  , connection_manager: Option<ConnectionManager>
  , window_id: Option<iced::window::Id>
  , dr_r_icon_handle: iced::advanced::image::Handle
  , trop_icon_handle: iced::advanced::image::Handle
  , stev_icon_handle: iced::advanced::image::Handle
  , beac_icon_handle: iced::advanced::image::Handle
  , console_expanded: bool
}


impl SonicApp {
  fn new(
      broadcast_rx_arc: std::sync::Arc<tokio::sync::Mutex<tokio::sync::broadcast::Receiver<crate::lens::v_dr_r::Message>>>,
      mpsc_rx_arc: std::sync::Arc<tokio::sync::Mutex<tokio::sync::mpsc::Receiver<crate::lens::v_dr_r::Message>>>,
      tell_remote: tokio::sync::mpsc::Sender<crate::lens::v_dr_r::Message>,
      connection_manager: ConnectionManager,
      aimd_backend: std::sync::Arc<aimd::AllmBackend>,
  ) -> (Self, Task<Message>) {
      log::trace!("SonicApp::new BEGIN! Creating new SonicApp instance...");
      
      let dr_r = crate::lens::v_dr_r::DrRobotnikV::new(tell_remote);
      log::debug!("SonicApp::new created DrRobotnikV");

      let buzz = crate::lens::v_buzz::BuzzBomberV::new();
      log::debug!("SonicApp::new created BuzzBomberV");
      
      let stev = crate::lens::v_stev::StevV::new().with_aimd_backend(aimd_backend);
      log::debug!("SonicApp::new created StevV");

      let dr_r_icon_handle = iced::advanced::image::Handle::from_bytes(
          include_bytes!("../jpg/dr_r_icon_a.jpg").as_slice()
      );
      let trop_icon_handle = iced::advanced::image::Handle::from_bytes(
          include_bytes!("../jpg/trop_icon.jpg").as_slice()
      );
      let stev_icon_handle = iced::advanced::image::Handle::from_bytes(
          include_bytes!("../jpg/sonic_1.jpg").as_slice()
      );
      let beac_icon_handle = iced::advanced::image::Handle::from_bytes(
          include_bytes!("../jpg/beac_1.jpg").as_slice()
      );
      
      let app = SonicApp {
          current_view: View::DrR,
          view_dr_r: dr_r,
          broadcast_rx_arc,
          mpsc_rx_arc,
          view_buzz: buzz,
          view_stev: stev,
          window_size: iced::Size::new(1200.0, 800.0),
          connection_manager: Some(connection_manager),
          window_id: None,
          dr_r_icon_handle,
          trop_icon_handle,
          stev_icon_handle,
          beac_icon_handle,
          console_expanded: false,
      };
      
      log::trace!("SonicApp::new END! Successfully created SonicApp instance");
      (app, Task::none())
  }
}

fn subscription(state: &SonicApp) -> Subscription<Message>
{ log::trace!("vsta::main subscription BEGIN! Setting up subscriptions...");
  
  let receiver_arc = state.broadcast_rx_arc.clone();
  
  Subscription::batch
  ( vec!
    [ state.view_dr_r.subscription().map(Message::DrR)
      , iced::advanced::subscription::from_recipe(PersistentBroadcastRecipe {
          receiver_arc
        }).map(Message::DrR)
    
      , match &state.current_view
        { View::DrR =>
          { log::debug!("vsta::main subscription DrR view, no additional subscription");
            Subscription::none()
          }
          , View::Buzz =>
          { log::debug!("vsta::main subscription Buzz view, no additional subscription");
            Subscription::none()
          }
          , View::Stev =>
          { log::debug!("vsta::main subscription Stev view, no additional subscription");
            Subscription::none()
          }
        }
      , iced::event::listen_with
        ( |event, _status, id|
          { match event
            { iced::Event::Window(iced::window::Event::Opened { .. }) =>
              { log::debug!("vsta::main subscription detected window opened for id: {:?}", id);
                Some(Message::WindowOpened(id))
              }
              , iced::Event::Window(iced::window::Event::Resized(size)) =>
              { Some(Message::WindowSizeUpdated(size))
              }
              , iced::Event::Window(iced::window::Event::CloseRequested) =>
              { log::debug!("vsta::main subscription detected window close request");
                Some(Message::WindowClosed)
              }
              , iced::Event::Window(iced::window::Event::Closed) =>
              { Some(Message::WindowClosed)
              }
              , _ =>
              { None
              }
            }
          }
        )
      , iced::advanced::subscription::from_recipe(SignalRecipe {})
        .map(|_| Message::WindowClosed)
    ]
  )
}

fn update(state: &mut SonicApp, message: Message) -> Task<Message>
{ let result = match message
  { Message::DrR(dr_r_msg) =>
    { slog::trace!(crate::glossary::TICK_SLOG,"vsta::main update received DrR message: {:?}", dr_r_msg);
      let res = state.view_dr_r.update(dr_r_msg).map(Message::DrR);
      if let View::DrR = state.current_view
      { res
      }
      else
      { log::warn!("vsta::main update got a DrR message when not in DrR view, but forwarding anyway");
        res
      }
    }
    , Message::DrRBatch(messages) =>
    { log::trace!("vsta::main update received DrRBatch with {} messages", messages.len());
      let tasks = messages
        .into_iter()
        .map(|msg|
        {
          state.view_dr_r.update(msg).map(Message::DrR)
        })
        .collect::<Vec<_>>();
      tasks.into_iter().fold
      ( iced::Task::none()
      , |acc, task| acc.chain(task)
      )
    }
    , Message::Buzz(buzz_msg) =>
    { log::debug!("vsta::main update received Buzz message: {:?}", buzz_msg);
      if let View::Buzz = state.current_view
      { log::trace!("vsta::main update Buzz view active, updating BuzzBomberV...");
        state.view_buzz.update(buzz_msg).map(Message::Buzz)
      }
      else
      { log::trace!("vsta::main update Buzz message ignored, not in Buzz view");
        Task::none()
      }
    }
    , Message::Stev(stev_msg) =>
    { log::debug!("vsta::main update received Stev message: {:?}", stev_msg);
      if let View::Stev = state.current_view
      { log::trace!("vsta::main update Stev view active, updating StevV...");
        state.view_stev.update(stev_msg).map(Message::Stev)
      }
      else
      { log::trace!("vsta::main update Stev message ignored, not in Stev view");
        Task::none()
      }
    }
    , Message::SwitchToDrR(_) =>
    { log::debug!("vsta::main update switching to DrR view");
      state.current_view = View::DrR;
      Task::none()
    }
    , Message::SwitchToBuzz(_) =>
    { log::debug!("vsta::main update switching to Buzz view");
      state.current_view = View::Buzz;
      Task::none()
    }
    , Message::SwitchToStev(_) =>
    { log::debug!("vsta::main update switching to Stev view");
      state.current_view = View::Stev;
      Task::none()
    }
    , Message::ToggleConsole =>
    { log::debug!("vsta::main update toggling console");
      state.console_expanded = !state.console_expanded;
      Task::none()
    }
    , Message::WindowSizeUpdated(size) =>
    { log::debug!("vsta::main update window size updated to: {}x{}", size.width, size.height);
      state.window_size = size;
      Task::none()
    }
    , Message::WindowOpened(id) =>
    { log::debug!("vsta::main update window opened with id: {:?}", id);
      state.window_id = Some(id);
      Task::none()
    }
    , Message::WindowClosed =>
    { log::debug!("vsta::main update received WindowClosed message");
      if let Some(manager) = state.connection_manager.take()
      { log::trace!("vsta::main update initiating TASK WindowClosed ConnectionManager shutdown");
        if let Some(window_id) = state.window_id
        { return Task::perform
          ( async move
            { log::trace!("vsta::main update TASK WindowClosed: awaiting shutdown...");
              manager.shutdown().await;
              log::trace!("vsta::main update WindowClosed: shutdown completed");
            }
            , |_| Message::None
          )
          .chain(iced::window::close(window_id));
        }
        log::warn!("vsta::main update WindowClosed: no window_id available, shutdown without closing");
        return Task::perform
        ( async move
          { log::trace!("vsta::main update TASK WindowClosed: awaiting shutdown...");
            manager.shutdown().await;
            log::trace!("vsta::main update WindowClosed: shutdown completed");
          }
          , |_| Message::None
        );
      }
      log::trace!("vsta::main update WindowClosed: no ConnectionManager to shut down");
      if let Some(window_id) = state.window_id
      {
        iced::window::close(window_id)
      }
      else
      {
        log::warn!("vsta::main update WindowClosed: no window_id available, cannot close window");
        Task::none()
      }
    }
    , Message::None =>
    {
      log::trace!("vsta::main update received None message, no action taken");
      Task::none()
    }
  };
  slog::debug!(crate::glossary::TICK_SLOG,"vsta::main update END! Message processed, returning Task");
  result
}


// Replace the entire view function:
fn view(state: &SonicApp) -> Element<Message>
{ let dbgspt = format!("[TVSMV]");
  slog::debug!
  ( glossary::MAIN_SLOG
  , "{} <=> vsta::main view BEGIN! Rendering UI..."
  , dbgspt
  );

  use iced::advanced::image;

  let sidebar_width = Length::Fixed(101.0);

  // Top sidebar - main controls
  let sidebar = Column::new()
    .width(sidebar_width)
    .height(Length::Fill)
    .align_x(Alignment::Center)
    .spacing(10)
    .push
    ( Button::new
      ( iced::widget::image::Image::new
        ( state.dr_r_icon_handle.clone()
        )
        .width(Length::Fixed(100.0))
        .height(Length::Fixed(100.0))
      )
      .on_press(Message::SwitchToDrR(()))
    )
    .push
    ( Button::new
      ( iced::widget::image::Image::new
        ( state.trop_icon_handle.clone()
        )
        .width(Length::Fixed(100.0))
        .height(Length::Fixed(100.0))
      )
      .on_press(Message::SwitchToBuzz(()))
    )
    .push
    ( Button::new
      ( iced::widget::image::Image::new
        ( state.stev_icon_handle.clone()
        )
        .width(Length::Fixed(100.0))
        .height(Length::Fixed(100.0))
      )
      .on_press(Message::SwitchToStev(()))
    )
    .push
    ( iced::widget::Space::new()
      .width(Length::Fill).height(Length::Fill)
    )
    .push
    ( if log::max_level() >= log::LevelFilter::Debug
      { slog::trace!
        ( glossary::MAIN_SLOG
        , "{} vsta::main view rendering debug window size text"
        , dbgspt
        );
        Text::new(format!("{}x{}", state.window_size.width.round() as u32, state.window_size.height.round() as u32))
          .size(10)
      }
      else
      { slog::trace!
        ( glossary::MAIN_SLOG
        , "{} vsta::main view rendering default text"
        , dbgspt
        );
        Text::new("SeeYa")
          .size(12)
      }
    );

  // Console button at the bottom
  let console_button = Button::new
    ( iced::widget::image::Image::new
      ( state.beac_icon_handle.clone()
      )
      .width(Length::Fixed(100.0))
      .height(Length::Fixed(100.0))
    )
    .on_press(Message::ToggleConsole);

  // Main content area
  let viewpage = match &state.current_view
  { View::DrR =>
    { slog::trace!
      ( glossary::MAIN_SLOG
      , "{} vsta::main view rendering DrR view"
      , dbgspt
      );
      crate::lens::v_dr_r::view(&state.view_dr_r)
    }
    , View::Buzz =>
    { slog::trace!
      ( glossary::MAIN_SLOG
      , "{} vsta::main view rendering DrR view"
      , dbgspt
      );
      log::debug!("vsta::main view rendering Buzz view");
      crate::lens::v_buzz::view(&state.view_buzz).map(Message::Buzz)
    }
    , View::Stev =>
    { slog::trace!
      ( glossary::MAIN_SLOG
      , "{} vsta::main view rendering Stev view"
      , dbgspt
      );
      crate::lens::v_stev::view(&state.view_stev) 
    }
  };

  // Build the layout - console area at bottom
  let main_content_height = if state.console_expanded
  { // When expanded, give main content less space
    Length::Fixed(151.0)
  } else // When collapsed, main content fills available space
  { Length::Fill
  };
  let console_area_height = if state.console_expanded
  { // When expanded, give main content less space
    Length::Fill
  } else // When collapsed, give some space
  { Length::Fixed(151.0)
  };
  let main_row = iced::widget::row![sidebar, viewpage]
    .height(main_content_height);

  // Get log notes from v_dr_r instead of top-level state
  let console_view = crate::lens::v_log_notes::view(&state.view_dr_r.my_log_notes);
  let console_area = iced::widget::Container::new
  ( iced::widget::row!
    [ iced::widget::column![console_button]
        .height(Length::Fixed(101.0))
        .width(sidebar_width)
    , console_view
    ]
    .width(Length::Fill)
    .height(console_area_height)
    .spacing(0),
  ).style
  ( |_theme| iced::widget::container::Style 
    { background: Some(iced::Background::Color(iced::Color::from_rgba(0.0, 0.0, 0.0, 0.8))),
    ..Default::default()
    }
  ).width(Length::Fill).height(console_area_height);

  // Main column: content on top (varies), console area on bottom (varies)
  let main_column = Column::new()
    .width(Length::Fill)
    .height(Length::Fill)
    .push(main_row)
    .push(console_area)
  ;

  let result = Row::new()
    .push(main_column)
    .into();
  slog::debug!
  ( glossary::MAIN_SLOG
  , "{} EXIT! vsta::main view END! UI rendered"
  , dbgspt
  );
  result
}


// For the Snively Messages over KSTA
struct PersistentBroadcastRecipe
{ receiver_arc: std::sync::Arc<tokio::sync::Mutex<tokio::sync::broadcast::Receiver<crate::lens::v_dr_r::Message>>>
}

impl iced::advanced::subscription::Recipe for PersistentBroadcastRecipe
{ type Output = crate::lens::v_dr_r::Message;
  fn hash(&self, state: &mut iced::advanced::subscription::Hasher)
  { use std::hash::Hash;
    std::any::TypeId::of::<Self>().hash(state);
  }
  fn stream(
    self: Box<Self>,
    _input: iced::advanced::subscription::EventStream,
  ) -> iced::futures::stream::BoxStream<'static, Self::Output> {
      use futures::StreamExt;

      let receiver_arc = self.receiver_arc.clone();

      iced::futures::stream::unfold(
          (receiver_arc, Vec::new()),  // (Arc<Mutex<...>>, pending batch)
          |(arc_rx, mut pending)| async move {
              // 1. Drain from pending batch first
              if let Some(msg) = pending.pop() {
                  return Some((msg, (arc_rx, pending)));
              }

              // 2. Try to collect a new batch without blocking
              let mut batch = Vec::new();
              {
                  // Scope the guard — drop it before we move arc_rx
                  let mut guard = arc_rx.lock().await;

                let mut count = 0;
                while count < 300 {
                    match guard.try_recv() {
                        Ok(msg) => { batch.push(msg); count += 1; }
                        Err(tokio::sync::broadcast::error::TryRecvError::Empty) => break,
                        Err(tokio::sync::broadcast::error::TryRecvError::Lagged(n)) => {
                            log::warn!("Lagged {} messages!", n);
                            continue;
                        }
                        Err(tokio::sync::broadcast::error::TryRecvError::Closed) => {
                            log::error!("Broadcast channel closed!");
                            return None;
                        }
                    }
                }
                if count == 300 {
                    log::warn!("Hit 300 msg batch limit — channel pressure high?");
                }
                                  
                                  // guard drops here → borrow ends
              }

              if !batch.is_empty() {
                  log::info!("Broadcast batch collected {} messages", batch.len());
                  batch.reverse(); // so pop() gives oldest first
                  let next = batch.pop().unwrap();
                  return Some((next, (arc_rx, batch))); // safe: guard already dropped
              }

              // 3. Nothing batched → do blocking recv
              let msg = {
                  // Another scoped guard
                  let mut guard = arc_rx.lock().await;
                  match guard.recv().await {
                      Ok(m) => m,
                      Err(e) => {
                          log::error!("Broadcast recv failed: {:?}", e);
                          return None;
                      }
                  }
                  // guard drops here
              };

              slog::trace!( crate::glossary::TICK_SLOG, "Single broadcast message received");
              Some((msg, (arc_rx, Vec::new())))
          },
      )
      .boxed()
  }
}





// for other stuff
struct SignalRecipe {}
impl iced::advanced::subscription::Recipe for SignalRecipe
{ type Output = ();
  fn hash(&self, state: &mut iced::advanced::subscription::Hasher)
  { use std::hash::Hash;
    std::any::TypeId::of::<Self>().hash(state);
  }

  fn stream
  ( self: Box<Self>
    , _input: iced::advanced::subscription::EventStream
  ) -> iced::futures::stream::BoxStream<'static, Self::Output>
  { use futures::stream::StreamExt;
    let fut = async move
    { use tokio::signal::unix::*;
      let mut sigint = signal(SignalKind::interrupt()).unwrap();
      let mut sigterm = signal(SignalKind::terminate()).unwrap();
      let mut sighup = signal(SignalKind::hangup()).unwrap();
      let mut sigquit = signal(SignalKind::quit()).unwrap();
      let mut sigalrm = signal(SignalKind::alarm()).unwrap();
      let mut sigpipe = signal(SignalKind::pipe()).unwrap();
      let mut sigusr1 = signal(SignalKind::user_defined1()).unwrap();
      let mut sigusr2 = signal(SignalKind::user_defined2()).unwrap();
      let mut sigchld = signal(SignalKind::child()).unwrap();
      tokio::select!
      {  _ = tokio::signal::ctrl_c() => ()
        , _ = sigint.recv() => ()
        , _ = sigterm.recv() => ()
        , _ = sighup.recv() => ()
        , _ = sigquit.recv() => ()
        , _ = sigalrm.recv() => ()
        , _ = sigpipe.recv() => ()
        , _ = sigusr1.recv() => ()
        , _ = sigusr2.recv() => ()
        , _ = sigchld.recv() => ()
      }
      log::info!("Received OS signal, initiating graceful shutdown via WindowClosed");
      ()
    };
    iced::futures::stream::once(fut).boxed()
  }
}

struct ConnectionManager
{ endpoint: quinn::Endpoint
  , conn_tx: tokio::sync::mpsc::Sender<(std::net::SocketAddr, String)>
  , client_kernel_task: tokio::task::JoinHandle<()>
  , tasks: tokio::task::JoinSet<()>
  , tell_v_dr_r: tokio::sync::broadcast::Sender<crate::lens::v_dr_r::Message>
}

impl ConnectionManager
{ fn new 
  ( server_addr: String
    , tell_v_dr_r: tokio::sync::broadcast::Sender<crate::lens::v_dr_r::Message>
    , told_remote: tokio::sync::mpsc::Receiver<crate::lens::v_dr_r::Message>
  ) -> Self
  { log::trace!("ConnectionManager::new({}) BEGIN!", server_addr);
    log::debug!("ConnectionManager::new() let trusted_cert = match v_dr_r::cert::example_client_load_trusted_cert()...");
    let trusted_cert = match crate::lens::v_dr_r::cert::example_client_load_trusted_cert()
    { Ok(tc) =>
      { log::debug!("ConnectionManager::new() successfully loaded trusted certificate");
        tc
      }
      , Err(e) =>
      { panic!("ConnectionManager::new()::v_dr_r::cert::example_client_load_trusted_cert() ERROR: {}", e);
      }
    };
    log::debug!("ConnectionManager::new() installing default CryptoProvider...");
    match rustls::crypto::CryptoProvider::install_default(rustls::crypto::ring::default_provider())
    {  _ =>
      {
        log::debug!("CryptoProvider installed successfully");
      }
    };
    log::debug!("ConnectionManager::new() let crypto_config = rustls::ClientConfig::builder() ... ");
    let crypto_config = rustls::ClientConfig::builder()
      .dangerous()
      .with_custom_certificate_verifier(crate::lens::v_dr_r::cert::SelfSignedVerifier::new(trusted_cert))
      .with_no_client_auth();
    let crypto_config = std::sync::Arc::new(crypto_config);
    log::trace!("ConnectionManager::new() crypto_config created");
    log::debug!("ConnectionManager::new() let mut client_config = quinn::ClientConfig::new(std::sync::Arc::new( ... ");
    let mut client_config = quinn::ClientConfig::new
    (
      std::sync::Arc::new
      (
        quinn::crypto::rustls::QuicClientConfig::try_from(crypto_config).unwrap()
      )
    );
    log::debug!("ConnectionManager::new() let mut transport_config = quinn::TransportConfig::default(); ... ");
    let mut transport_config = quinn::TransportConfig::default();
    transport_config.max_idle_timeout(None);
    transport_config.keep_alive_interval(Some(tokio::time::Duration::from_secs(365 * 24 * 60 * 60)));
    client_config.transport_config(std::sync::Arc::new(transport_config));
    log::trace!("ConnectionManager::new() transport_config set");
    let mut endpoint = quinn::Endpoint::client("0.0.0.0:0".parse().unwrap()).unwrap();
    endpoint.set_default_client_config(client_config);
    log::debug!("ConnectionManager::new() endpoint created and configured");
    let (conn_tx, conn_rx) = tokio::sync::mpsc::channel(100);
    let (new_conn_tx, new_conn_rx) = tokio::sync::mpsc::channel(100);
    log::trace!("ConnectionManager::new() channels created");
    let mut client_kernel = dsta::ksta::ClientKernel::new(endpoint.clone(), conn_rx, new_conn_tx);
    let client_kernel_task = tokio::spawn
    (
      async move
      {
        log::trace!("ConnectionManager::new() client_kernel_task BEGIN!");
        if let Err(e) = client_kernel.run().await
        {
          log::error!("ClientKernel failed: {:?}", e);
        }
        log::trace!("ConnectionManager::new() client_kernel_task END!");
      }
    );
    let mut manager = ConnectionManager
    {
      endpoint
      , conn_tx
      , client_kernel_task
      , tasks: tokio::task::JoinSet::new()
      , tell_v_dr_r
    };
    log::debug!("ConnectionManager::new() manager struct initialized");
    manager.connect(server_addr, new_conn_rx, told_remote);
    log::trace!("ConnectionManager::new() END! ConnectionManager created and connect called");
    manager
  }

  fn connect
  (
    &mut self
    , server_addr: String
    , mut new_conn_rx: tokio::sync::mpsc::Receiver<dsta::ksta::ConnectionHandle>
    , mut told_remote: tokio::sync::mpsc::Receiver<crate::lens::v_dr_r::Message>
  )
  {
    let lsa = format!("{}", server_addr);
    log::trace!("ConnectionManager::connect({}) BEGIN!", lsa);
    let conn_tx = self.conn_tx.clone();
    let tell_v_dr_r_a = self.tell_v_dr_r.clone();
    self.tasks.spawn
    (
      async move
      {
        let tell_v_dr_r_b = tell_v_dr_r_a.clone();
        log::trace!("ConnectionManager::connect({}) self.tasks.spawn BEGIN! = [CMcsts]", server_addr);
        let server_addr = match server_addr.parse()
        {
          Ok(addr) =>
          {
            log::debug!("ConnectionManager::connect({}) parsed server address successfully", server_addr);
            addr
          }
          , Err(e) =>
          {
            log::debug!("ConnectionManager::connect({}) failed to parse server address: {}", server_addr, e);
            let _ = tell_v_dr_r_b.send(crate::lens::v_dr_r::Message::Disconnected(format!("ConnectionManager::connect Parse error: {}", e)));
            return;
          }
        };
        log::trace!("[Cmcsts] :: if let Err(e) = conn_tx.send({}).await // connect to server...", server_addr);
        if let Err(e) = conn_tx.send((server_addr, "localhost".to_string())).await
        {
          log::debug!("ConnectionManager::connect({}) failed to send connection request: {}", server_addr, e);
          tell_v_dr_r_b.send
          (
            crate::lens::v_dr_r::Message::Disconnected
            (
              format!("Send error: {}", e)
            )
          );
          return;
        }
        log::trace!("[Cmcsts{}] :: if let Some(mut handle) = new_conn_rx.recv().await // wait for connection...", server_addr);
        if let Some(mut handle) = new_conn_rx.recv().await
        {
          log::debug!("ConnectionManager::connect({}) self.tasks.spawn = [CMcsts] // Got Connection Handle, spawning subtasks...", server_addr);
          log::debug!("Connection established, sending Connected to frontend immediately");
          let _ = tell_v_dr_r_a.send(crate::lens::v_dr_r::Message::Connected);
          let greet_handle_tds = handle.to_dude_snively.clone();
          let tell_v_dr_r_greet = tell_v_dr_r_a.clone();
          tokio::spawn
          (
            async move
            {
              tokio::time::sleep(tokio::time::Duration::from_millis(150)).await;
              log::debug!("Sending connection confirmation log note to server");
              if let Err(e) = greet_handle_tds.send
              (
                dsta::Snively::SendLogNote
                (
                  "Frontend connected successfully! GUI ready for buzz bombing.".to_string()
                )
              ).await
              {
                log::error!("Failed to send connection log note: {:?}", e);
                let _ = tell_v_dr_r_greet.send(crate::lens::v_dr_r::Message::Disconnected
                (
                  "Failed to send connection confirmation".to_string()
                ));
              }
            }
          );
          let mut cancel_rx = handle.cancel_conn_tasks_receiver.clone();
          let tell_v_dr_r = tell_v_dr_r_a.clone();
          tokio::spawn
          (
            async move
            {
              log::debug!("[CMcsts{}] SPAWN TASK 1 = [1CMcsts] // rx snively type...", server_addr);
              loop
              {
                tokio::select!
                {
                  changed = cancel_rx.changed() =>
                  {
                    log::trace!("[1Cmcsts{}] cancel_rx.changed() triggered", server_addr);
                    if changed.is_ok()
                    {
                      let reason = cancel_rx.borrow().reason.clone();
                      if cancel_rx.borrow().cancel
                      {
                        log::debug!("[1Cmcsts{}] cancel requested, reason: {}", server_addr, reason);
                        if let Err(e) = tell_v_dr_r.send(crate::lens::v_dr_r::Message::Disconnected(reason.clone()))
                        { log::error!("[1Cmcsts{}] cancel failed, reason: {:#?}", server_addr, e);
                        }
                        log::debug!("[1Cmcsts{}] tell_v_dr_r.send(v_dr_r::Message::Disconnected(reason)).await; DONE ", server_addr);
                        break;
                      }
                    }
                    else
                    {
                      log::debug!("[1Cmcsts{}] cancel_rx changed failed, breaking loop", server_addr);
                      break;
                    }
                  }
                  , Some(snively) = handle.from_dude_snively.recv() =>
                  {
                    log::warn!(
                        "[SNIVELY_INCOMING] connection={} received Snively variant={:?} len={}",
                        server_addr,
                        std::mem::discriminant(&snively),
                        match &snively {
                            dsta::Snively::SendLogNote(n) => n.len(),
                            _ => 0,
                        }
                    );

                    log::info!("[1Cmcsts{}] // Got snively message: {:#?}...", server_addr, snively);
                    let msg = match snively
                    {  dsta::Snively::SendLogNote(note) => crate::lens::v_dr_r::Message::RecvLogNote(note)
                      , dsta::Snively::EmeraldInfo(info) => crate::lens::v_dr_r::Message::RecvEmerald(info)
                      , _ => 
                      { let msgmsg = format!("We got unhandled Snively, not a log note or emeral info!?");
                        log::debug!("{}", msgmsg);
                        crate::lens::v_dr_r::Message::RecvLogNote(msgmsg)
                      }
                      // Looks like this would  do a loopback LOL ?
                      //, other => crate::lens::v_dr_r::Message::SendSnively(other)
                    };
                    log::debug!("[1Cmcsts{}] let _ = tell_v_dr_r.send(msg); // sending snively message to GUI...", server_addr);

                    if let Err(e) = tell_v_dr_r.send(msg) 
                    { log::error!("[BCAST_SEND_FAIL] failed to send RecvLogNote to broadcast: {:?}", e);
                      log::error!("[1Cmcsts{}] let _ = tell_v_dr_r.send(msg) FAILED; = {:#?}",server_addr, e);
                    } else 
                    { slog::trace!
                      (  crate::glossary::TICK_SLOG
                      , "[BCAST_SEND_OK] forwarded log note to broadcast"
                      );
                    }
                  }
                }
              }
              log::debug!("[CMcsts{}] DONE TASK 1 = [1CMcsts] // FINISHED", server_addr);
            }
          );
          let mut cancel_rx = handle.cancel_conn_tasks_receiver.clone();
          let tell_v_dr_r = tell_v_dr_r_a.clone();
          tokio::spawn
          (
            async move
            {
              log::debug!("[CMcsts{}] SPAWN TASK 2 = [2CMcsts] // rx madebot type...", server_addr);
              loop
              {
                tokio::select!
                {
                  changed = cancel_rx.changed() =>
                  {
                    log::trace!("[2Cmcsts{}] cancel_rx.changed() triggered", server_addr);
                    if changed.is_ok() && cancel_rx.borrow().cancel
                    {
                      log::debug!("[2Cmcsts{}] cancel requested, breaking loop", server_addr);
                      break;
                    }
                  }
                  , Some(madebot) = handle.from_dude_madebot.recv() =>
                  {
                    log::trace!("[2Cmcsts{}] received madebot: {:?}", server_addr, madebot);
                    let _ = tell_v_dr_r.send(crate::lens::v_dr_r::Message::RecvMadeBot(madebot));
                  }

                  , Some(dat) = handle.from_dude_gottick00.recv() =>
                  {
                    slog::trace!
                    ( crate::glossary::TICK_SLOG
                    , "[2Cmcsts{}] received tick00: {:?}", server_addr, dat
                    );
                    let msg = crate::lens::v_dr_r::Message::RecvTicks00(dat.ticker);
                    if let Err(e) = tell_v_dr_r.send(msg) 
                    { log::error!("[BCAST_SEND_FAIL] failed to send RecvLogNote to broadcast: {:?}", e);
                      log::error!("[1Cmcsts{}] let _ = tell_v_dr_r.send(msg) FAILED; = {:#?}",server_addr, e);
                    } else 
                    { slog::trace!( crate::glossary::TICK_SLOG
                      , "[BCAST_SEND_OK] forwarded log note to broadcast"
                      );
                    }
                  }
                }
              }
              log::debug!("[CMcsts{}] DONE TASK 2 = [2CMcsts] // FINISHED", server_addr);
            }
          );
         
         
          let mut cancel_rx = handle.cancel_conn_tasks_receiver.clone();
          let tell_v_dr_r = tell_v_dr_r_a.clone();
          tokio::spawn
          (
            async move
            {
              log::debug!("[CMcsts{}] SPAWN TASK 4 = [4CMcsts] // tx messages, based on the told_remote from v_dr_r...", server_addr);
              loop
              {
                tokio::select!
                {
                  changed = cancel_rx.changed() =>
                  {
                    log::trace!("[4Cmcsts{}] cancel_rx.changed() triggered", server_addr);
                    if changed.is_ok() && cancel_rx.borrow().cancel
                    {
                      let erm = format!("[4Cmcsts{}], the task for quic remote tx, got a cancel_rx on its connection handle.", server_addr);
                      log::debug!("{}", erm);
                      let _ = tell_v_dr_r.send(crate::lens::v_dr_r::Message::Disconnected(format!("Connection handle raised cancel_rx (caught in tx task)")));
                      log::debug!("[4Cmcsts{}] cancel_rx.changed() triggered => break from loop after tell_v_dr_r", server_addr);
                      break;
                    }
                  }
                  , Some(msg) = told_remote.recv() =>
                  {
                    log::trace!("[4Cmcsts{}] received message to process: {:?}", server_addr, msg);
                    match msg
                    {
                      crate::lens::v_dr_r::Message::SendSnively(bot_talk) =>
                      {
                        log::debug!("[4Cmcsts{}] sending snively: {:?}", server_addr, bot_talk);
                        let _ = handle.to_dude_snively.send(bot_talk).await;
                      }
                      , crate::lens::v_dr_r::Message::CancelConnection =>
                      {
                        log::error!("CANCEL CONNECTION IS GOTTEN HERE!");
                        log::debug!("[4Cmcsts{}] sending cancel connection", server_addr);
                        let _ = handle.cancel_conn_tasks_sender.send
                        (
                          dsta::ksta::CancelInfo
                          {
                            cancel: true
                            , reason: format!("v_dr_r::Message::CancelConnection")
                            , connection_number: 69
                          }
                        );
                        let erm = format!("[4Cmcsts{}], the task for quic remote tx, has v_dr_r saying cancel connection!.", server_addr);
                        log::debug!("{}", erm);
                        let _ = tell_v_dr_r.send(crate::lens::v_dr_r::Message::Disconnected(format!("Visual command to disconnect (caught in tx task)")));
                        log::debug!("[4Cmcsts{}] v_dr_r saying cancel connection => break from loop after tell_v_dr_r",server_addr);
                        break;
                      }
                      , _ =>
                      {
                        log::debug!("vsta::main() Ignoring unsendable message in send task: {:?}", msg);
                      }
                    }
                  }
                }
              }
              log::debug!("[CMcsts{}] DONE TASK 4 = [4CMcsts] // FINISHED", server_addr);
            }
          );
        }
        else
        {
          log::debug!("ConnectionManager::connect({}) no connection handle received", server_addr);
          let tell_v_dr_r = tell_v_dr_r_a.clone();
          let _ = tell_v_dr_r.send(crate::lens::v_dr_r::Message::Disconnected("No connection handle received".to_string()));
        }
        log::trace!("ConnectionManager::connect({}) self.tasks.spawn END! = [CMcsts]", server_addr);
      }
    );
    log::trace!("ConnectionManager::connect({}) END!", lsa);
  }

  async fn run(mut self)
  {
    log::trace!("ConnectionManager::run BEGIN!");
    while self.tasks.join_next().await.is_some()
    {
      log::trace!("ConnectionManager::run single task completed...");
    }
    log::debug!("ConnectionManager::run completed all tasks");
    log::trace!("ConnectionManager::run aborting client_kernel_task...");
    self.client_kernel_task.abort();
    let _ = self.client_kernel_task.await;
    log::debug!("ConnectionManager::run aborted client_kernel_task");
    log::trace!("ConnectionManager::run END!");
  }

  async fn shutdown(self)
  {
    log::trace!("ConnectionManager::shutdown BEGIN!");
    log::debug!("ConnectionManager::shutdown started");
    log::trace!("ConnectionManager::shutdown sending CancelConnection...");
    if let Err(e) = self.tell_v_dr_r.send(crate::lens::v_dr_r::Message::CancelConnection)
    {
      log::error!("Failed to send CancelConnection: {:?}", e);
    }
    log::trace!("ConnectionManager::shutdown calling run() to wait for tasks...");
    self.run().await;
    log::debug!("ConnectionManager::shutdown completed");
    log::trace!("ConnectionManager::shutdown END!");
  }
}

fn boot_sonic_app(
    server_addr: String,
    tell_v_dr_r_tx: tokio::sync::broadcast::Sender<crate::lens::v_dr_r::Message>,
    broadcast_rx_arc: std::sync::Arc<tokio::sync::Mutex<tokio::sync::broadcast::Receiver<crate::lens::v_dr_r::Message>>>,
    mpsc_rx_arc: std::sync::Arc<tokio::sync::Mutex<tokio::sync::mpsc::Receiver<crate::lens::v_dr_r::Message>>>,
    tell_remote_tx: tokio::sync::mpsc::Sender<crate::lens::v_dr_r::Message>,
) -> (SonicApp, iced::Task<Message>) {
    log::trace!("boot_sonic_app BEGIN! (iced runtime)");

    let tell_remote_rx = {
        let mut guard = mpsc_rx_arc.blocking_lock();
        let (tx, rx) = tokio::sync::mpsc::channel(256);
        let old_rx = std::mem::replace(&mut *guard, rx);
        old_rx
    };

    let manager = ConnectionManager::new(
        server_addr,
        tell_v_dr_r_tx.clone(),
        tell_remote_rx,
    );

    log::debug!("boot_sonic_app created ConnectionManager");

    // Initialize AIMD backend synchronously and extract only the backend
    let (aimd_backend, mut alert_rx) = init_aimd_backend_sync();
    let aimd_backend = std::sync::Arc::new(aimd_backend);
    log::debug!("boot_sonic_app initialized AIMD backend");

    // Spawn alert listener in background to forward to main LogNotes via Message::LogNotes
    tokio::spawn(async move {
        while let Some(alert) = alert_rx.recv().await {
            let alert_str = format!("{:#?}", alert);
            log::info!("AIMD Alert: {}", alert_str);
        }
    });

    let (app, task) = SonicApp::new(
        broadcast_rx_arc,
        mpsc_rx_arc,
        tell_remote_tx,
        manager,
        aimd_backend,  // ← PASS ARC<AllmBackend> HERE
    );

    log::trace!("boot_sonic_app END!");
    (app, task)
}

/// Synchronous wrapper for backend initialization
/// Returns (AllmBackend, alert receiver)
fn init_aimd_backend_sync() -> (
    aimd::AllmBackend,
    tokio::sync::mpsc::UnboundedReceiver<aimd::AllmAlert>,
) {
    let path = "tests/private-list.json";
    let keys = aimd::load_init_list_from_private_list(path);
    let args = aimd::AllmBackArgs {
        init_list: keys,
        max_attempts: 2,
    };
    
    let (alert_tx, alert_rx) = tokio::sync::mpsc::unbounded_channel::<aimd::AllmAlert>();
    let backend = aimd::AllmBackend::new_config(args, alert_tx);
    
    log::info!("AIMD backend initialized");
    (backend, alert_rx)
}

fn main() -> Result<(), std::io::Error> 
{

  log::trace!("main BEGIN!");

  // Initialize the logger
  {
      use std::io::Write;

      // RGB to ANSI helpers
      pub fn rgb_to_ansi_foreground(r: u8, g: u8, b: u8) -> String {
          format!("\x1b[38;2;{};{};{}m", r, g, b)
      }

      pub fn rgb_to_ansi_background(r: u8, g: u8, b: u8) -> String {
          format!("\x1b[48;2;{};{};{}m", r, g, b)
      }

      pub const ANSI_RESET: &str = "\x1b[0m";

      // Rainbow letter colors (classic ROYGBIV order, adjusted for RAINBOW)
      fn rainbow_letter(c: char) -> &'static str {
          match c.to_ascii_uppercase() {
              'R' => "\x1b[38;2;255;0;0m",     // Red
              'A' => "\x1b[38;2;255;165;0m",   // Orange
              'I' => "\x1b[38;2;255;255;0m",   // Yellow
              'N' => "\x1b[38;2;0;255;0m",     // Green
              'B' => "\x1b[38;2;0;0;255m",     // Blue
              'O' => "\x1b[38;2;75;0;130m",    // Indigo
              'W' => "\x1b[38;2;148;0;211m",   // Violet
              _   => "\x1b[37m",              // Fallback: white
          }
      }

      // Build rainbow version of "RAINBOW"
      fn rainbow_text() -> String {
          let word = "RAINBOW";
          let mut colored = String::new();
          for c in word.chars() {
              colored.push_str(rainbow_letter(c));
              colored.push(c);
          }
          colored.push_str(ANSI_RESET);
          colored
      }

      env_logger::Builder::new()
          .parse_env("RUST_LOG")
          .format(|buf, record| {
              let timestamp = chrono::Local::now();
              let level_color = match record.level() {
                  log::Level::Error => "\x1b[31m",
                  log::Level::Warn  => "\x1b[33m",
                  log::Level::Info  => "\x1b[32m",
                  log::Level::Debug => "\x1b[34m",
                  log::Level::Trace => "\x1b[35m",
              };
              let reset_color = "\x1b[0m";
              let dim = "\x1b[90m";

              let mut message = record.args().to_string();

              // [RED]...[/RED] style tags
              let color_map = [
                  ("RED",     "\x1b[31m"),
                  ("GREEN",   "\x1b[32m"),
                  ("YELLOW",  "\x1b[33m"),
                  ("BLUE",    "\x1b[34m"),
                  ("MAGENTA", "\x1b[35m"),
                  ("CYAN",    "\x1b[36m"),
                  ("WHITE",   "\x1b[37m"),
                  ("BOLD",    "\x1b[1m"),
                  ("ORANGE",  "\x1b[38;5;208m"),
                  ("PURPLE",  "\x1b[38;5;135m"),
              ];

              for (tag, ansi) in color_map.iter() {
                  let open = format!("[{}]", tag);
                  let close = format!("[/{}]", tag);
                  message = message
                      .replace(&open, ansi)
                      .replace(&close, reset_color);
              }

              // Special keyword highlights
              message = message
                  .replace("PYTHON", "\x1b[36mPYTHON\x1b[0m")
                  .replace("STDERR", "\x1b[31mSTDERR\x1b[0m")
                  .replace(
                      "Steven E. Elliott",
                      &format!
                      ( "{}{}{}{}"
                        ,  rgb_to_ansi_foreground(123, 4, 69)
                        , rgb_to_ansi_background(0, 44, 6)
                        , "Steven E. Elliott"
                        , ANSI_RESET
                      )
                  )
                  // RAINBOW → colorful letters!
                  .replace("RAINBOW", &rainbow_text())
                  .replace("MAGIC", &rainbow_text());

              writeln!(
                  buf,
                  "{} [{}] {}{}{} - {}",
                  timestamp.format("%H:%M:%S.%3f"),
                  format!("{}{}{}", level_color, record.level(), reset_color),
                  dim,
                  record.target(),
                  reset_color,
                  message
              )
          })
          .init();
  }

  log::debug!("main initialized BASIC env_logger");
  
  let args: Vec<String> = std::env::args().collect();
  let server_addr = if args.len() > 1 {
      log::debug!("main using provided server address: {}", args[1]);
      args[1].clone()
  } else {
      log::warn!("No server address provided, defaulting to 127.0.0.1:4433");
      String::from("127.0.0.1:4433")
  };
  
  log::info!("Starting SonicApp with server address: {}", server_addr);
  let own_pid = get_own_pid();
  log::debug!("main obtained own PID: {}", own_pid);
  
  let (tell_v_dr_r_tx, tell_v_dr_r_rx) = tokio::sync::broadcast::channel(1024);
  let (tell_remote_tx, tell_remote_rx) = tokio::sync::mpsc::channel(256);
  log::debug!("main created channels");
  
  let broadcast_rx_arc = std::sync::Arc::new(
      tokio::sync::Mutex::new(tell_v_dr_r_rx)
  );
  let mpsc_rx_arc = std::sync::Arc::new(
      tokio::sync::Mutex::new(tell_remote_rx)
  );
  
  let server_addr_clone = server_addr.clone();
  let tell_v_dr_r_tx_clone = tell_v_dr_r_tx.clone();
  let broadcast_rx_arc_clone = broadcast_rx_arc.clone();
  let mpsc_rx_arc_clone = mpsc_rx_arc.clone();
  
  const CUSTOM_FONT: iced::Font = iced::Font::with_name("DejaVu Sans Mono");
  log::trace!("main starting iced application...");
  
  let iced_result = iced::application(
      move || {
          boot_sonic_app(
              server_addr_clone.clone(),
              tell_v_dr_r_tx_clone.clone(),
              broadcast_rx_arc_clone.clone(),
              mpsc_rx_arc_clone.clone(),
              tell_remote_tx.clone(),
          )
      },
      update,
      view,
  )
  .title("Sonic 5")
  .settings(iced::Settings {
      default_font: CUSTOM_FONT,
      ..iced::Settings::default()
  })
  .window(iced::window::Settings {
      size: iced::Size::new(1200.0, 600.0),
      position: iced::window::Position::Centered,
      min_size: Some(iced::Size::new(800.0, 600.0)),
      max_size: None,
      resizable: true,
      decorations: true,
      transparent: false,
      level: iced::window::Level::Normal,
      icon: None,
      visible: true,
      exit_on_close_request: true,
      platform_specific: iced::window::settings::PlatformSpecific::default(),
      blur: false,
      closeable: true,
      fullscreen: false,
      maximized: false,
      minimizable: true,
  })
  .subscription(subscription)
  .theme(dark_theme)
  .run();
  
  log::trace!("main END! Application finished");
  match iced_result {
      Ok(()) => {
          log::debug!("Application completed successfully");
      }
      Err(e) => {
          log::error!("Application failed: {:?}", e);
          return Err(std::io::Error::new(std::io::ErrorKind::Other, e.to_string()));
      }
  }
  log::info!("Vsta nice shutdown!");
  Ok(())
}

fn dark_theme(_state: &SonicApp) -> iced::theme::Theme
{ iced::theme::Theme::Dark
}

fn get_own_pid() -> i32
{ std::process::id() as i32
}