// usta/src/tests/util.rs

use crate::*;
use crate::dr_robo::*;
use dsta::*;
use std::collections::HashMap;
use log::{LevelFilter, info, error};
use log4rs::{
    config::{Appender, Config, Root},
    encode::pattern::PatternEncoder,
    Handle,
};

pub(crate) fn setup_logenv() {
    let _ = env_logger::builder()
        .is_test(true)
        .format_timestamp_millis()  // This gives millisecond precision in logs
        .format_module_path(false)
        .format_target(false)
        .try_init();
}

pub(crate) fn setup_log4rs(test_name: &str) 
-> Result<log4rs::Handle, Box<dyn std::error::Error>> 
{ // Create log directory if it doesn't exist
  std::fs::create_dir_all("test_logs")?;
  // Define log file path
  let log_file = format!("test_logs/{}.log", test_name);
  // Configure file appender to flush immediately
  let file_appender = log4rs::append::file::FileAppender::builder()
  .append(false)
  .encoder(Box::new(log4rs::encode::pattern::PatternEncoder::new("{d(%Y-%m-%d %H:%M:%S%.3f)} {l} {m}\n")))
  .build(log_file)?;

  // Configure the logger
  let config = log4rs::config::Config::builder()
    .appender( log4rs::config::Appender::builder().build("file", Box::new(file_appender)))
    .build
    ( Root::builder()
        .appender("file")
        .build(log::LevelFilter::Trace),
    )?
  ;

  // Initialize the logger and return the handle
  let handle = log4rs::init_config(config)?;
  Ok(handle)
}

pub(crate) fn load_creds() -> serde_json::Value 
{ let creds_str = include_str!("../../../ttrs/tests/test-creds.json");
  let res: serde_json::Value = serde_json::from_str(creds_str).expect("Invalid test_creds.json");
  log::info!("Loaded credentials!");
  if res["client_id"]
    .as_str()
    .map(|s| s.is_empty())
    .unwrap_or(true)
  { panic!("client_id is missing or empty in test-creds.json");
  } 
  if res["client_secret"]
    .as_str()
    .map(|s| s.is_empty())
    .unwrap_or(true)
  { panic!("client_secret is missing or empty in test-creds.json");
  }
  if res["refresh_token"]
    .as_str()
    .map(|s| s.is_empty())
    .unwrap_or(true)
  { panic!("refresh_token is missing or empty in test-creds.json");
  }

  log::info!("All required credentials present");
  res
}




    impl BotaAccount 
    {
        pub(crate) fn new() -> Self {
            Self::default()
        }

        pub(crate) fn with_cash(mut self, cash: f64) -> Self {
            self.m_cash = cash;
            self
        }

        pub(crate) fn with_short_put(mut self, strike: f64, qty: f64, days: f64, price: f64) -> Self {
            let ticker = format!("STK {}_P_{}", strike, days);
            self.m_short_puts = Some(create_position(fn_get_deets(&ticker), qty, price));
            self
        }

        pub(crate) fn with_short_call(mut self, strike: f64, qty: f64, days: f64, price: f64) -> Self {
            let ticker = format!("STK {}_C_{}", strike, days);
            self.m_short_calls = Some(create_position(fn_get_deets(&ticker), qty, price));
            self
        }

        pub(crate) fn with_long_call(mut self, strike: f64, qty: f64, days: f64, price: f64) -> Self {
            let ticker = format!("STK {}_C_{}", strike, days);
            self.m_long_calls = Some(create_position(fn_get_deets(&ticker), qty, price));
            self
        }

        pub(crate) fn with_long_put(mut self, strike: f64, qty: f64, days: f64, price: f64) -> Self {
            let ticker = format!("STK {}_P_{}", strike, days);
            self.m_long_puts = Some(create_position(fn_get_deets(&ticker), qty, price));
            self
        }
         // NEW: Add stock position
    pub(crate) fn with_stock(mut self, price: f64, qty: f64, cost_basis: f64) -> Self {
        let ticker = "STK";
        self.m_stock = Some(OwnedPosition {
            asset: fn_get_deets(ticker),
            owned: qty,
            price: cost_basis,
        });
        self
    }

    }

pub(crate) fn create_position
( asset: dsta::FinAss, qty: f64, price: f64
) -> OwnedPosition 
{ OwnedPosition 
  { asset
    , owned: qty
    , price
  }
}

pub(crate) fn fn_get_deets(ticker_name: &str) -> dsta::FinAss 
{
    let cleaned = ticker_name.replace('_', " ");
    let parts: Vec<&str> = cleaned.split_whitespace().collect();

    if parts.len() == 1 
    { // Just the underlying, e.g. "FAKE" or "STK"
      return dsta::FinAss::StkDeets
      ( dsta::Stk {
            ticker_name: parts[0].to_string(),
            ticker_info: "Fake stock for testing".to_string(),
            last_ticker: None,
            tick_rulers: None,
        });
    }

    if parts.len() != 4 || (parts[2] != "C" && parts[2] != "P") {
        panic!("Invalid fake ticker format: '{}'", ticker_name);
    }

    let underlying = parts[0];
    let strike: f64 = parts[1].parse().unwrap();
    let expiry_days: i64 = parts[3].parse().unwrap();
    let option_rite = if parts[2] == "C" {
        dsta::OptRite::Call
    } else {
        dsta::OptRite::Put
    };

    let expire_date = chrono::Utc::now() + chrono::Duration::days(expiry_days);

    let ticker_name_underscore = format!("{} {}_{}_{}", underlying, strike, parts[2], expiry_days);

    dsta::FinAss::OptDeets(dsta::Opt {
        ticker_name: ticker_name_underscore,
        option_name: format!("{} {} {} {}", underlying, strike, parts[2], expiry_days),
        option_rite,
        strike_spot: strike,
        expire_date,
        option_type: dsta::OptStyle::Usa,
        last_ticker: None,
        tick_rulers:None,
    })
}


pub(crate) fn make_stuff_with_tickers(tickers: &[&str]) -> HashMap<String, FinAss> {
    let mut stuff: HashMap<String, FinAss> = HashMap::new();

    for &ticker_str in tickers {
        let asset = fn_get_deets(ticker_str);  // already creates proper StkDeets or OptDeets
        let key = asset.order_name();          // ← this is the critical change!
        
        log::trace!("Inserting asset with key: {} (from {})", key, ticker_str);
        stuff.insert(key, asset);
    }

    // Always add a default stock if not present (for tests that expect "STK")
    if !stuff.contains_key("STK") {
        let stk = fn_get_deets("STK");
        stuff.insert(stk.order_name(), stk);
    }

    stuff
}