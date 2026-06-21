// Main
// uttai/src/main.rs
mod sbot;
pub(crate) mod core;
// Helper structures and functions for initial simple traceability testing
pub mod helper
{  
  pub struct LoopMetrics 
  {   times: Vec<f64>
    , last_candle: tokio::time::Instant // or tokio::time::Instant
    , candles_1s: Vec<(f64, f64, f64)>
    , candles_10s: Vec<(f64, f64, f64)>
    , candles_100s: Vec<(f64, f64, f64)>
    ,
  }
  impl LoopMetrics 
  { pub fn new() -> Self 
    { Self 
      {   times: Vec::new()
        , last_candle: tokio::time::Instant::now()
        , candles_1s: Vec::new()
        , candles_10s: Vec::new()
        , candles_100s: Vec::new(),
      }
    }

    pub fn record(&mut self, elapsed_ms: f64) 
    { self.times.push(elapsed_ms);
      if self.last_candle.elapsed().as_secs() >= 1 
      { if !self.times.is_empty() 
        { let min = self.times.iter().copied().fold(f64::INFINITY, f64::min);
          let max = self.times.iter().copied().fold(f64::NEG_INFINITY, f64::max);
          let avg = self.times.iter().sum::<f64>() / self.times.len() as f64;
          log::debug!("1s candle recorded: min={}ms, max={}ms, avg={}ms", min, max, avg);
          self.candles_1s.push((min, max, avg));
          self.times.clear();
          self.last_candle = tokio::time::Instant::now();
          while self.candles_1s.len() > 86_500 { self.candles_1s.remove(0); }
          if self.candles_1s.len() >= 10 
          {
            let chunk = &self.candles_1s[self.candles_1s.len() - 10..];
            let min = chunk.iter().map(|c| c.0).fold(f64::INFINITY, f64::min);
            let max = chunk.iter().map(|c| c.1).fold(f64::NEG_INFINITY, f64::max);
            let avg = chunk.iter().map(|c| c.2).sum::<f64>() / 10.0;
            self.candles_10s.push((min, max, avg));
            log::debug!("10s candle recorded: min={}ms, max={}ms, avg={}ms", min, max, avg);
            while self.candles_10s.len() > 86_400 { self.candles_10s.remove(0); }
          }

          if self.candles_10s.len() >= 10 
          {
            let chunk = &self.candles_10s[self.candles_10s.len() - 10..];
            let min = chunk.iter().map(|c| c.0).fold(f64::INFINITY, f64::min);
            let max = chunk.iter().map(|c| c.1).fold(f64::NEG_INFINITY, f64::max);
            let avg = chunk.iter().map(|c| c.2).sum::<f64>() / 10.0;
            self.candles_100s.push((min, max, avg));
            log::debug!("100s candle recorded: min={}ms, max={}ms, avg={}ms", min, max, avg);
            while self.candles_100s.len() > 86_400 { self.candles_100s.remove(0); }
          }
        }
      }
    }
  }
}

// NOTE: only the BotA and Bot0 are allowed to use this api
#[derive(Clone, Debug)]
pub(crate) struct DrRoboApi // The roboticizer, manages all the robots
{   pub tell_dr_a_snively: tokio::sync::mpsc::Sender<(usize, dsta::Snively)>
  , pub tell_dr_new_brain: tokio::sync::mpsc::Sender<(usize, BotMindState)>
  , pub tell_dr_pls_trade: tokio::sync::mpsc::Sender<(usize, u64, dsta::Order)>
  , pub tell_dr_pls_ticks: tokio::sync::mpsc::Sender<(usize, String, tokio::sync::mpsc::Sender<Option<dsta::PushTickerResult>>)>
  , pub tell_dr_pls_check: tokio::sync::mpsc::Sender<(usize, String)>
  , pub tell_dr_stop_tick: tokio::sync::mpsc::Sender<(usize, String)>
  , pub tell_dr_stop_info: tokio::sync::mpsc::Sender<(usize, String)>
  , pub tell_dr_swap_info: tokio::sync::mpsc::Sender<(usize, BotSwaper)>
}
// Responsible for the allocations, the tasks, the id's etc for all robots.  
// It recognizes the special bots' (0,1,TBD), ability to do magic
// It is controlled by one of it's own bots, Dr Seek
// so there are magic bots and basic bots, basic being user-created.
pub mod dr_robo; // HAS A THREAD

// ============================================================== //
// ============================================================== //
// ============================================================== //
#[derive(Clone, Debug)]
pub(crate) struct BotApi // Structure dr_robo keeps for each bot.
{   pub robot_brain_state: Allocation // basic robot memory
  , pub robot_bota_sender: DrBotaApiHand // robot commands :
      // The trade,check,sucks is what passes to api implementers
      // The start,pause,drain,ghost,death, are used internally 
}
#[derive(Clone, Debug)]
pub(crate) struct DrBotaApiHand // The Info + basic commands dr_robo can give to its bots.
{   pub tell_bot_to_start: tokio::sync::mpsc::Sender<String>
  , pub tell_bot_to_pause: tokio::sync::mpsc::Sender<()>
  , pub tell_bot_to_death: tokio::sync::mpsc::Sender<String>
  , pub tell_bot_to_trade: tokio::sync::mpsc::Sender<dsta::Ticker>
  , pub tell_bot_to_check: tokio::sync::mpsc::Sender<Allocation>
  , pub tell_bot_it_sucks: tokio::sync::mpsc::Sender<String>
}
pub(crate) struct DrBotaApiFoot // The Info + basic commands dr_robo can give to its bots.
{   pub told_bot_to_start: tokio::sync::mpsc::Receiver<String>
  , pub told_bot_to_pause: tokio::sync::mpsc::Receiver<()>
  , pub told_bot_to_death: tokio::sync::mpsc::Receiver<String>
  , pub told_bot_to_trade: tokio::sync::mpsc::Receiver<dsta::Ticker>
  , pub told_bot_to_check: tokio::sync::mpsc::Receiver<Allocation>
  , pub told_bot_it_sucks: tokio::sync::mpsc::Receiver<String>
}

pub(crate) fn fn_make_bot_api_hand_and_foot() -> ( DrBotaApiHand, DrBotaApiFoot )
{ let (tell_bot_to_start, told_bot_to_start) = tokio::sync::mpsc::channel::<String>(100);
  let (tell_bot_to_pause, told_bot_to_pause) = tokio::sync::mpsc::channel::<()>(100);
  let (tell_bot_to_death, told_bot_to_death) = tokio::sync::mpsc::channel::<String>(100);
  let (tell_bot_to_trade, told_bot_to_trade) = tokio::sync::mpsc::channel::<dsta::Ticker>(100);
  let (tell_bot_to_check, told_bot_to_check) = tokio::sync::mpsc::channel::<Allocation>(100);
  let (tell_bot_it_sucks, told_bot_it_sucks) = tokio::sync::mpsc::channel::<String>(100);
  ( DrBotaApiHand // The Info + basic commands dr_robo can give to its bots.
    {   tell_bot_to_start: tell_bot_to_start // tokio::sync::mpsc::Sender<String>
      , tell_bot_to_pause: tell_bot_to_pause // tokio::sync::mpsc::Sender<()>
      , tell_bot_to_death: tell_bot_to_death // tokio::sync::mpsc::Sender<String>
      , tell_bot_to_trade: tell_bot_to_trade // tokio::sync::mpsc::Sender<dsta::Ticker>
      , tell_bot_to_check: tell_bot_to_check // tokio::sync::mpsc::Sender<Allocation>
      , tell_bot_it_sucks: tell_bot_it_sucks // tokio::sync::mpsc::Sender<String>
    }
  , DrBotaApiFoot // The Info + basic commands dr_robo can give to its bots.
    {   told_bot_to_start: told_bot_to_start // tokio::sync::mpsc::Receiver<String>
      , told_bot_to_pause: told_bot_to_pause // tokio::sync::mpsc::Receiver<()>
      , told_bot_to_death: told_bot_to_death // tokio::sync::mpsc::Receiver<String>
      , told_bot_to_trade: told_bot_to_trade // tokio::sync::mpsc::Receiver<dsta::Ticker>
      , told_bot_to_check: told_bot_to_check // tokio::sync::mpsc::Receiver<Allocation>
      , told_bot_it_sucks: told_bot_it_sucks // tokio::sync::mpsc::Receiver<String>
    }
  )
} 
pub mod dr_bota;


#[derive(Clone)]
pub(crate) struct DrTtaiApi // Runs along Bot Zero - for interacting with TTAI (tasty trade application interface)
{ //   tell_dr_add_ttai_tik: tokio::sync::mpsc::Sender<(String, u64, tokio::sync::mpsc::Sender<dsta::Ticker>)> // Sender(tick, who, sender to who)
  // , tell_dr_pop_ttai_tik: tokio::sync::mpsc::Sender<(String, u64)>
    pub tell_dr_order_submit: tokio::sync::mpsc::Sender
    < ( dsta::Order
      , tokio::sync::oneshot::Sender
        < Result<dsta::PushOrderResult, Box<dyn std::error::Error + Send + Sync>> 
        >
      )
    >,
    pub tell_dr_order_cancel: tokio::sync::mpsc::Sender<(
        u64,
        tokio::sync::oneshot::Sender<Result<bool, Box<dyn std::error::Error + Send + Sync>>>,
    )>,

    pub tell_dr_order_dryrun: tokio::sync::mpsc::Sender<(
        dsta::Order,
        tokio::sync::oneshot::Sender<Result<f64, Box<dyn std::error::Error + Send + Sync>>>,
    )>,

  pub tell_dr_order_status: tokio::sync::mpsc::Sender
  < tokio::sync::oneshot::Sender
    < Result<Vec<dsta::PlacedOrder>, Box<dyn std::error::Error + Send + Sync>> 
    >
  >,

}
// Does order processing for all the other bots, 
// At startup, creates the initial allocation (future new allocations go to bot 1)
pub mod dr_ttai_mock;
pub mod dr_ttai; 
pub mod dr_bot0_ttai_bridge;
pub mod dr_bot0;


#[derive(Clone)]
pub(crate) struct DrSeekApi  // Bot One - for running the quic server, handling connections to the user interfaces. 
{   tell_dr_snively: tokio::sync::mpsc::Sender<dsta::Snively>
  , tell_dr_made_bot: tokio::sync::mpsc::Sender<dsta::MadeBot>
  , tell_dr_got_ttai: tokio::sync::mpsc::Sender<dsta::GotTtai>
  , tell_dr_got_tick00: tokio::sync::mpsc::Sender<dsta::GotTick00> // regular tickers
  , tell_dr_got_tick01: tokio::sync::mpsc::Sender<dsta::GotTick01> // options tickers 
}
pub mod dr_seek; 
// passes commands to dr robo
// forwards it's tickers to the frontend 

// Enums and Structs for the Base Bot API
  #[derive(Clone, Copy, Debug, PartialEq, Default, serde::Serialize, serde::Deserialize)]
  pub enum BotMindState // 
  {   #[default] Running // it runs normally
    , Trading
    , Stopped // it is paused (we tell it only its own allocation info)
    , Seppuku // it is killing itself
    , Zombied // it is dead and will be deleted next data refresh
    
  }
  #[derive(Clone, Debug, Default, serde::Serialize, serde::Deserialize)]
  pub struct Allocation // the actual bot information 
  {   pub bot_name: String
    , pub bot_safe: crate::BotaAccount
    , pub bot_mind: BotMindState
    , pub num_swap: u64 // counter for times the bot requested a swap
  }

  #[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
  pub struct BotaAccount // Only 1 underlying type, only 1 of each option type
  { pub m_cash: f64,
    pub m_stock: Option<dsta::OwnedPosition>,
    pub m_long_calls: Option<dsta::OwnedPosition>,
    pub m_long_puts: Option<dsta::OwnedPosition>,
    pub m_short_calls: Option<dsta::OwnedPosition>,
    pub m_short_puts: Option<dsta::OwnedPosition>,
    pub m_call_credit_spread_lein: f64,
    pub m_short_put_lein: f64,
    pub m_put_credit_spread_lein: f64,
    pub m_short_iron_condor_lein: f64,
  }
  impl Default for BotaAccount 
  { fn default() -> Self 
    { BotaAccount 
      {   m_cash: 0.0
        , m_stock: None
        , m_long_calls: None
        , m_long_puts: None
        , m_short_calls: None
        , m_short_puts: None
        , m_call_credit_spread_lein: 0.0
        , m_short_put_lein: 0.0
        , m_put_credit_spread_lein: 0.0
        , m_short_iron_condor_lein: 0.0,
      }
    }
  }

  #[derive(Clone, Debug)]
  pub(crate) struct SwapWat
  { who : String
  , wat : dsta::PositionsList
  , how : f64
  }

  #[derive(Clone, Debug)]
  pub(crate) struct BotSwaper // For exchanging capital between bots
  {   pub source_offering: Option<SwapWat> // What source bot allocation is giving to Target Bot
    , pub target_released: Option<SwapWat> // What target bot allocation is giving to Source Bot
  }

// What the base bot api passes to dr robo so it can be managed within the framework properly
// DR ROBO USES THIS TO TALK TO BASE BOT API

// BASE BOT API REVEALS THIS DR ROBO INTERFACE TO ITS IMPLEMENTERS:
pub(crate) struct BridgeDrFoot
{   pub told_dr_pls_trade: tokio::sync::mpsc::Receiver<dsta::Order>
  , pub told_dr_new_brain: tokio::sync::mpsc::Receiver<Allocation>
  , pub told_dr_pls_ticks: tokio::sync::mpsc::Receiver<(String, tokio::sync::mpsc::Sender<Option<dsta::PushTickerResult>>)>
  , pub told_dr_pls_check: tokio::sync::mpsc::Receiver<String>
  , pub told_dr_stop_tick: tokio::sync::mpsc::Receiver<String>
  , pub told_dr_stop_info: tokio::sync::mpsc::Receiver<String>
  , pub told_dr_swap_info: tokio::sync::mpsc::Receiver<BotSwaper>
  , pub told_dr_a_snively: tokio::sync::mpsc::Receiver<dsta::Snively>
  
}

pub(crate) struct BridgeDrHand
{   pub tell_dr_pls_trade: tokio::sync::mpsc::Sender<dsta::Order>
  , pub tell_dr_new_brain: tokio::sync::mpsc::Sender<Allocation>
  , pub tell_dr_pls_ticks: tokio::sync::mpsc::Sender<(String, tokio::sync::mpsc::Sender<Option<dsta::PushTickerResult>>)>
  , pub tell_dr_pls_check: tokio::sync::mpsc::Sender<String>
  , pub tell_dr_stop_tick: tokio::sync::mpsc::Sender<String>
  , pub tell_dr_stop_info: tokio::sync::mpsc::Sender<String>
  , pub tell_dr_swap_info: tokio::sync::mpsc::Sender<BotSwaper>
  , pub tell_dr_a_snively: tokio::sync::mpsc::Sender<dsta::Snively>
}

pub(crate) fn fn_make_bot_api_bridge_dr_handles() -> (BridgeDrHand, BridgeDrFoot )
{ 
  let ( tell_dr_pls_trade, told_dr_pls_trade ) =  tokio::sync::mpsc::channel::<dsta::Order>(100);
  let ( tell_dr_new_brain, told_dr_new_brain ) =  tokio::sync::mpsc::channel::<Allocation>(100);
  let ( tell_dr_pls_ticks, told_dr_pls_ticks ) =  tokio::sync::mpsc::channel::<(String,tokio::sync::mpsc::Sender<Option<dsta::PushTickerResult>>)>(100);
  let ( tell_dr_pls_check, told_dr_pls_check ) =  tokio::sync::mpsc::channel::<String>(100);
  let ( tell_dr_stop_tick, told_dr_stop_tick ) =  tokio::sync::mpsc::channel::<String>(100);
  let ( tell_dr_stop_info, told_dr_stop_info ) =  tokio::sync::mpsc::channel::<String>(100);
  let ( tell_dr_swap_info, told_dr_swap_info ) =  tokio::sync::mpsc::channel::<BotSwaper>(100);
  let ( tell_dr_a_snively, told_dr_a_snively ) =  tokio::sync::mpsc::channel::<dsta::Snively>(100);
  ( BridgeDrHand
    {   tell_dr_pls_trade : tell_dr_pls_trade // tokio::sync::mpsc::Sender<dsta::Order>
      , tell_dr_new_brain : tell_dr_new_brain // tokio::sync::mpsc::Sender<Allocation>
      , tell_dr_pls_ticks : tell_dr_pls_ticks // tokio::sync::mpsc::Sender<String>
      , tell_dr_pls_check : tell_dr_pls_check // tokio::sync::mpsc::Sender<String>
      , tell_dr_stop_tick : tell_dr_stop_tick // tokio::sync::mpsc::Sender<String>
      , tell_dr_stop_info : tell_dr_stop_info // tokio::sync::mpsc::Sender<String>
      , tell_dr_swap_info : tell_dr_swap_info // tokio::sync::mpsc::Sender<BotSwaper>
      , tell_dr_a_snively : tell_dr_a_snively
    }
  , BridgeDrFoot
    {   told_dr_pls_trade: told_dr_pls_trade // tokio::sync::mpsc::Receiver<dsta::Order>
      , told_dr_new_brain: told_dr_new_brain // tokio::sync::mpsc::Receiver<Allocation>
      , told_dr_pls_ticks: told_dr_pls_ticks // tokio::sync::mpsc::Receiver<String>
      , told_dr_pls_check: told_dr_pls_check // tokio::sync::mpsc::Receiver<String>
      , told_dr_stop_tick: told_dr_stop_tick // tokio::sync::mpsc::Receiver<String>
      , told_dr_stop_info: told_dr_stop_info // tokio::sync::mpsc::Receiver<String>
      , told_dr_swap_info: told_dr_swap_info // tokio::sync::mpsc::Receiver<BotSwaper>
      , told_dr_a_snively:told_dr_a_snively
    }
  )
} 

// The main errors that the base bot API can encounter (
// This defines communications that the bot implementer 
// must deal with in order to be compliant with the framework )
pub enum FixFails 
{   Start
  , Pause
  , Drain
  , Ghost
  , Death
  , Bonus
  , Trade(String)
  , Sucks(String)
}
impl FixFails 
{ pub fn to_string(&self) -> String 
  { match self 
    { FixFails::Start => format!("`Dr R said Start! // passed name of the bot (load internal state, DOING: dr r shall pass the loaded account in the tell_bot_to_check)")
    , FixFails::Pause => format!("`Dr R said Pause! // don't trade (save the internal state - dr r base api did the account)")
    , FixFails::Drain => format!("`Dr R said Drain! // exit all positions (the base bot api loop system shall do it automatically)")
    , FixFails::Ghost => format!("`Dr R said Ghost! // Dr R loop system puts bot in Zombied and kills all the bot's base api tasks, exiting base api loop. shall happen after pause. - YOUR API IMPLEMENTATION SHOULD manage memory appropriately here.")
    , FixFails::Death => format!("`Dr R said Death! // Ghost, but then deletes the base bot api memory (no more ?).")
    , FixFails::Bonus => format!("`Dr R said Bonus! // Dr R loop system is processing allocation - will broadcast raise a mpsc bot to check, including self")
    , FixFails::Trade(why) => format!("`Dr R fail Trade! // {}", why)
    , FixFails::Sucks(why) => format!("`Dr R said Sucks! // {}", why)
    }
  }
}

// CtorBot is: 
// Info from the implementer of the API passed to to the base of the api
pub(crate) struct CtorBot 
{   pub robot_api_brain_state: crate::Allocation // 
  , pub tell_bot_to_trade_api: tokio::sync::mpsc::Sender<dsta::Ticker>
  , pub tell_bot_to_check_api: tokio::sync::mpsc::Sender<Allocation>
  , pub tell_bot_it_sucks_api: tokio::sync::mpsc::Sender<String>
  , pub told_the_bot_requests: BridgeDrFoot
}

#[tokio::main]
async fn main() 
-> Result<(), Box<dyn std::error::Error>> 
{ 
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
  log::info!("Starting RAINBOW Steven E. Elliott (SEE's) example Using Tasty Trade Application Interface [UTTAI]...");  
  dr_robo::f_run_main_dr_robo().await;
  Ok(())
}

#[cfg(test)]
mod tests;

pub mod glossary
{
// Core signals from ROBO to BOT over snively:
pub(crate) const SWAP_SUCC : &str = "SWAP_SUCC";
pub(crate) const SWAP_FAIL : &str = "SWAP_FAIL";
pub(crate) const ORDER_BAD : &str = "ORDER_BAD";
pub(crate) const ORDER_FLY : &str = "ORDER_FLY";

pub(crate) const TICK_SLOG : u64 = 100;

// Bot and Index Limits
pub(crate) const MAX_BOT_NAME_LENGTH: usize = 256;
pub(crate) const MIN_BOT_NAME_LENGTH: usize = 5; 
pub(crate) const MAX_BOT_INDEX: u64 = 0xFFFFF;
pub(crate) const MAX_ORDER_NUMBER: u64 = !0xFFFFF;

// Channel and Buffer Sizes
pub(crate) const CHANNEL_BUFFER_SIZE_SNIVELY: usize = 1000;
pub(crate) const CHANNEL_BUFFER_SIZE_NEW_ROBOT: usize = 10;
pub(crate) const CHANNEL_BUFFER_SIZE_PLS_TRADE: usize = 1000;
pub(crate) const CHANNEL_BUFFER_SIZE_NEW_BRAIN: usize = 1000;
pub(crate) const CHANNEL_BUFFER_SIZE_PLS_TICKS: usize = 1000;
pub(crate) const CHANNEL_BUFFER_SIZE_PLS_CHECK: usize = 1000;
pub(crate) const CHANNEL_BUFFER_SIZE_STOP_TICK: usize = 1000;
pub(crate) const CHANNEL_BUFFER_SIZE_STOP_INFO: usize = 1000;
pub(crate) const CHANNEL_BUFFER_SIZE_SWAP_INFO: usize = 1000;
pub(crate) const CHANNEL_BUFFER_SIZE_TTAI_TICK: usize = 100;
pub(crate) const CHANNEL_BUFFER_SIZE_BOT0_ORDER: usize = 100;
pub(crate) const CHANNEL_BUFFER_SIZE_TICK_RESULT: usize = 2;

// Special Bot Indexes
pub(crate) const BOT0_INDEX: usize = 0;
pub(crate) const BOT1_INDEX: usize = 1;
//pub(crate) const BOT2_INDEX: usize = 2;

// Error and Debug Messages
pub const TTAI_FAIL_PYTHON_START : &'static str = "Running the python code failed to start";
pub const TTAI_FAIL_FIX_RESTART0 : &'static str = "Restart the python code please";

pub(crate) const ERROR_ROBOT_NAME_TOO_BIG: &str = "ROBOT NAME IS TOO BIG!";
pub(crate) const ERROR_ROBOT_NAME_TOO_SMALL: &str = "ROBOT NAME IS TOO SMALL!, 3 letters minimum";
//pub(crate) const ERROR_DUPLICATE_BOT_NAME: &str = "Robot with name {} already exists at index {}";
pub(crate) const ERROR_NO_BOT_HAND: &str = "no bot hand at index!";
pub(crate) const ERROR_NO_TICK_VEC: &str = "No check vector found for bot index {}";
pub(crate) const ERROR_NO_ASSET_INFO: &str = "no asset info for leg!";
pub(crate) const ERROR_NO_STOCK_POSITION_TO_CLOSE: &str = "No stock position for closing...";
pub(crate) const ERROR_NO_OPTION_POSITION_TO_CLOSE: &str = "No option position for closing...";
pub(crate) const ERROR_INSUFFICIENT_LEIN_CASH: &str = "Insufficient cash to cover lein";
pub(crate) const ERROR_INSUFFICIENT_OPTION_POSITION_FOR_CLOSING: &str = "Insufficient position for closing";
pub(crate) const ERROR_CHANNEL_FULL: &str = "channel full, error log not sent";
pub(crate) const ERROR_CHANNEL_CLOSED: &str = "CHANNEL CLOSED! this is a critical error!";


// Log and Debug Prefixes
pub(crate) const LOG_PREFIX_DR_ROBO_NEW_BOT: &str = "DR_ROBO_NEW_BOT";
pub(crate) const LOG_PREFIX_DR_ROBO_DO_SNIV: &str = "DR_ROBO_DO_SNIV";
pub(crate) const LOG_PREFIX_DR_ROBO_ORD_REQ: &str = "DR_ROBO_ORD_REQ";
pub(crate) const LOG_PREFIX_DR_ROBO_N_BRAIN: &str = "DR_ROBO_N_BRAIN";
pub(crate) const LOG_PREFIX_DR_ROBO_REQ_TIK: &str = "DR_ROBO_REQ_TIK";
pub(crate) const LOG_PREFIX_DR_ROBO_REQ_ACT: &str = "DR_ROBO_REQ_ACT";
pub(crate) const LOG_PREFIX_DR_ROBO_BYE_TIK: &str = "DR_ROBO_BYE_TIK";
pub(crate) const LOG_PREFIX_DR_ROBO_BYE_ACT: &str = "DR_ROBO_BYE_ACT";
pub(crate) const LOG_PREFIX_DR_ROBO_SWAPPER: &str = "DR_ROBO_SWAPPER";
pub(crate) const LOG_PREFIX_DR_ROBO_NAME_OK: &str = "DR_ROBO_NAME_OK";
pub(crate) const LOG_PREFIX_DR_ROBO_RUNNING: &str = "DR_ROBO_RUNNING";
pub(crate) const LOG_PREFIX_BOTA_ACCOUNT_OK: &str = "[LEGAL_CHECK]";

// Special Values for Bot0 and Magic Strings
pub(crate) const USER_CASH_BOT_NAME: &str = "USER_CASH";
pub(crate) const MAGIC_BOT0_NAME: &str = "MAGIC.0.$";
pub(crate) const MAGIC_BOT0_CASH_INJECT: &str = "Magic bot0 @ startup cash credit complete.";
pub(crate) const MAGIC_BOT0_STARTUP_OK: &str = "b0su";
pub(crate) const MAGIC_DTTAI_STARTUP_OK: &str = "dtsu";
pub(crate) const MAGIC_DSSEEK_STARTUP_OK: &str = "dssu";
pub(crate) const TOUCH_NO_HOW_WHO_IN_A_SWAP: f64 = 0.0;

// Miscellaneous
pub(crate) const DEFAULT_STARTUP_TIMEOUT_SECS: u64 = 5;
pub(crate) const DEFAULT_RECV_MANY_BATCH_SIZE: usize = 100;
//pub(crate) const DEFAULT_BIOHASH_BOT_INDEX_MASK: u64 = 0xFFFFF;
pub(crate) const DEFAULT_BIOHASH_ORDER_NUM_SHIFT: u64 = 20;

}