
// trade/dsta/src/lib.rs - Simplified broker data

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use chrono;

// ==================================================================
// PART 1: DSTA finance types wrapper for tasty trade broker
// ==================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Ticker 
{ pub name: String,
  pub bid: f64,
  pub bid_size: f64,
  pub ask: f64,
  pub ask_size: f64,
  pub time: chrono::DateTime<chrono::Utc>,
  pub theo_price: Option<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TickTier {
    /// Premium threshold where this tick size starts applying.
    /// `None` means "for all higher premiums" (highest tier).
    pub threshold: Option<f64>,
    /// Tick size (minimum price increment) for this tier.
    pub value: f64,
}

/// Tick rulers rules for different asset contexts
#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct TickRules {
    /// Tiers for the underlying (stock/future/ETF shares price)
    pub underlying_tiers: Vec<TickTier>,

    /// Tiers for single option leg premiums
    pub option_tiers: Vec<TickTier>,

    /// Tiers for spread/combo/vertical/calendar prices
    pub spread_tiers: Vec<TickTier>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct TickSnapResult {
    /// The tick size that was applied (based on price and asset type)
    pub tick_size: f64,

    /// The final snapped price (closest valid tick)
    pub tick_snap: f64,

    /// "Pass" price:
    /// - If we snapped UP   → floor(original price) @ tick_size
    /// - If we snapped DOWN → ceil(original price) @ tick_size
    /// - If no adjustment   → None (or Some(original price) if you prefer)
    pub tick_pass: Option<f64>,
}


#[derive(Debug, Default, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum OptRite 
{ Put,
  #[default]
  Call,
}

#[derive(Debug, Default, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum OptStyle 
{ Usa,
  Eur,
  #[default]
  Etc,
}

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct Opt 
{ pub ticker_name: String,
  pub option_name: String,
  pub option_rite: OptRite,
  pub strike_spot: f64,
  pub expire_date: chrono::DateTime<chrono::Utc>,
  pub option_type: OptStyle,
  pub tick_rulers: Option<TickRules>, // none=>0.01, 0.05, 0.25, 0.50, etc, depending on asset. 
  pub last_ticker: Option<Ticker>,
}

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct Bnd 
{ pub ticker_name: String,
  pub ticker_info: String,
  pub coupon_rate: f64,
  pub expire_date: chrono::DateTime<chrono::Utc>,
  pub tick_rulers: Option<TickRules>, // none=>0.01, 0.05, 0.25, 0.50, etc, depending on asset. 
  pub last_ticker: Option<Ticker>,
}

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct Stk 
{ pub ticker_name: String,
  pub ticker_info: String,
  pub tick_rulers: Option<TickRules>, // none=>0.01, 0.05, 0.25, 0.50, etc, depending on asset. 
  pub last_ticker: Option<Ticker>, 
}

#[derive(Debug, Clone, Serialize, Deserialize)]
//#[serde(tag = "financial_asset", content = "details")]
pub enum FinAss 
{ StkDeets(Stk),
  BndDeets(Bnd),
  OptDeets(Opt),
}
/*
public FinAss impl has
{ pub fn ticker_name(&self) -> String
  pub fn order_name(&self)-> String
  pub fn set_ticker(&mut self, tick : Ticker)
  pub fn get_multiplier(&self) -> f64 
  pub fn get_normalized_cost(&self, price: f64) -> f64 
  pub fn tick_rulers(&self) -> TickRules 
  get_tick_and_snap
*/

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct AccountUpdate // from ttrs
{ pub ttai_uid: String,
  pub long_usd: f64,
  pub can_sell: f64,
  pub long_stk: f64,
  pub sell_stk: f64,
  pub long_opt: f64,
  pub sell_opt: f64,
  pub long_fut: f64,
  pub sell_fut: f64,
  pub long_fop: f64,
  pub sell_fop: f64,
  pub long_xxx: f64,
  pub sell_xxx: f64,
  pub long_bnd: f64,
  pub next_usd: f64,
  pub summings: f64,
  pub cash_out: f64,
}

#[derive(Debug, Default, Copy, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum BuyOrSell 
{ #[default]
  BuyToOpen,
  SellToOpen,
  BuyToClose,
  SellToClose,
  BuyFutures,
  SellFutures,
}

#[derive(Debug, Default, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum MarketOrLimit 
{ Market, // illegal in usta
  #[default]
  Limit,
}

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct OrderLeg 
{ pub buy_what: FinAss,
  pub quantity: f64,
  pub remaining: f64,
  pub action: BuyOrSell,
  pub price: f64,
}

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct Order 
{ pub order_legs: Vec<OrderLeg>,
  pub order_type: MarketOrLimit,
}

#[derive(Debug, Default, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum OrderFill 
{ #[default]
  Live,
  Done,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PlacedOrder 
{ pub order_hash: u64,
  pub legs_fills: Vec<OrderLeg>,
  pub order_fill: OrderFill,
}

#[derive(Clone, Debug, Default, serde::Serialize, serde::Deserialize)]
pub struct OwnedPosition 
{ pub asset: FinAss,
  pub owned: f64,
  pub price: f64,
}

#[derive(Default, Clone, Debug, Serialize, Deserialize)]
pub struct PositionsList 
{ pub positions: HashMap<String, OwnedPosition>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
//#[serde(tag = "push_order_result", content = "details")]
pub enum PushOrderResult 
{ ValidID { new_id: u64 },
  FailedID { new_id: u64 },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Derivatives 
{ pub expire_time: chrono::DateTime<chrono::Utc>,
  pub option_puts: Vec<Opt>,
  pub option_calls: Vec<Opt>,
}

#[derive(Default, Clone, Serialize, Deserialize)]
pub struct PushTickerResult 
{ pub ass_deets: FinAss,
  pub opt_chain: Vec<Derivatives>,
}

//Useless stuff for now:

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct GoodbyeMessage 
{ pub goodbye: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct OrderID 
{ pub order_hash: u64,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct NamedTicker 
{ pub ticker_name: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Dummy 
{ pub empty: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
//#[serde(tag = "pipe_command", content = "command_input")]
pub enum PipeCommand 
{ TickPush(NamedTicker),
  TickDrop(NamedTicker),
  OrderPush(Order),
  OrderDrop(OrderID),
  GetPositionsList(Dummy),
  GetBalances(Dummy),
  GetPlacedOrders(Dummy),
  ShutDown(GoodbyeMessage),
}

// ==================================================================
// PART 2: DSTA ROBOT common TYPES
// ==================================================================

#[derive(Default, Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct CommonBotStat
{ pub haggle_number: u64, // Number of times we had to deal with a waiting order with resubmission
  pub my_wins_count: u64, // total winning trades
  pub my_loss_count: u64, // number of losing trades
  pub my_wins_total: f64, // total dollars won
  pub my_loss_total: f64, // total dollars lost  (mark to market = won + lost)
  pub most_won_cash: (f64,f64), // (% won based on max cash risk, cash won) in a single trade => high score, max of below vector
  pub pl_performace: Vec<(chrono::NaiveTime, chrono::NaiveTime,f64,f64)> // (1st order complete time, exit positions order complete time, % won ^ , cash won), for each trade
}

#[derive(Copy, Clone, Debug, Default, serde::Serialize, serde::Deserialize, PartialEq)]
pub enum UsaMarketTimes 
{ ResetTtai,
  #[default]
  PowerEnds,
  ItsClosed,
  Hurry5sec,
  TminusTen,
  Tminus30s,
  Tminus60s,
  DeadShort,
  PowerFive,
  PowerEasy,
  PowerHour,
  FedSpeach,
  FedMinute,
  TradeTime,
  LunchTime,
  EuroClose,
  DoneTrend,
  OpenChaos,
}

#[derive(Copy, Clone, Debug, Default, serde::Serialize, serde::Deserialize, PartialEq, Eq)]
pub enum MarketDirection 
{ #[default]
  GetStonk,
  Sideways,
  Corrects,
}

#[derive(Copy, PartialEq, Clone, Debug, serde::Serialize, serde::Deserialize)]
pub enum HaggleAction 
{ // our choices for how we price the entry order and deal with a order wait time-out
  JustCancel(HaggleMethod), // at retry just cancel-no haggle.
  AnOyVeyJew(HaggleMethod), // at retry, haggle for the price. 
}

#[derive(Copy, Clone, PartialEq, Debug, serde::Serialize, serde::Deserialize)]
pub struct HaggleLimits
{ pub retry_period : chrono::Duration, // or cancel time
  pub delta_choice : f64,
  pub delta_is_pct : bool,
  pub slippage_max : f64, 
}
#[derive(Copy, Clone, PartialEq, Debug, serde::Serialize, serde::Deserialize)]
pub enum HaggleMethod // how we enter an order and how we retry it
{ MinimumOfMidpointOrTheoreticalThenConcede(HaggleLimits)
, VirtualMarketOrderWithLimitOrdThenConcede(HaggleLimits)
, CheapLimitOrderThenIncrementDeltaUntilMax(HaggleLimits)
}
#[derive(Default, Copy, PartialEq, Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct HaggleContext 
{ pub combo_mid:      f64, // quoted prices
  pub combo_bid:      f64, // quoted prices
  pub combo_ask:      f64, // quoted prices
  pub price_we_have:  f64, // realistic market side (bid for credit, ask for debit)
  pub price_we_want:  f64, // ideal aggressive price
  pub is_net_credit:  bool, // have < want
  pub initial_spread: f64, // combo_ask - combo_bid at trigger time
}
// NOTE: for haggle stuff, we have:
//
// dsta::create_initial_haggle_order(hide_order: &Order,haggle_method: &HaggleMethod,) -> (Order, HaggleContext)
// 
/// Returns absolute slippage introduced this step (combo-level concession).
// impl HaggleLimits::apply_haggle_changes( &self, order: &mut Order, ctx: &HaggleContext,) -> f64 
// 
// HaggleAction::get_retry_period

// ==================================================================
// PART 2: DSTA ROBOT CONSTRUCTOR TYPES
// ==================================================================

#[derive(Default, Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct CommonBotInfo 
{ pub friendly_name: String,
  pub tracking_tick: String, // because all bots only follow 1 
  pub haggle_action: HaggleAction, // what to do when we wait to long 
  pub max_cash_risk: (bool, f64, f64 ), // ( ? , %, $ ) bool == false => select min of next 2: % of total cash or set cash value; bool = true choose max. This specifies max $ to use in a bot's algo
}

#[derive(Clone, Debug, Serialize, Deserialize, )]
pub struct MakeBuzzBomber 
{ /// Shared accounting & performance tracking
  pub my_accounting: CommonBotInfo,
  /// How much buying power / cash to request at startup (only once)
  pub my_cash_alloc: f64,
  /// Market bias → determines put credit vs call credit spreads
  pub stonk_feeling: MarketDirection,
  /// Minimum days to expiration we are willing to trade
  pub option_expire: u16,
  /// Target credit we want to receive per contract (e.g. 0.25 → $25)
  pub target_spread: f64,
  /// When and how to time the periodic entry order
  pub time_to_order: BotActionTime,
  /// Multiple possible exit conditions (any one triggers close)
  pub follow_a_exit: Vec<BuzzBotDoneNiceStyle>,
  /// Cooldown period after successful entry (seconds)
  pub algo_cooldown: chrono::Duration,
  /// Whether to repeat forever or stop after state resets
  pub bombs_forever: bool,
}
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct BotActionTime 
{ /// For Entry, the day of the week (0 = sunday for futures) which 
  /// we run the entry order logic (thus bots repeat weekly)
  /// For Exit, the number of days before option_expire (0 ok) which
  /// we run the exit order logic.
  pub relative_days: u16,
  /// Target market time-of-day on the entry day
  pub my_entry_time: UsaMarketTimes,
  /// Additional delay after my_entry_time (can be negative for early)
  pub my_entry_wait: chrono::Duration,
}
// public impl has:
  // pub fn is_entry_window(&self, now: chrono::DateTime<chrono::Utc>, window: chrono::Duration) -> bool
  // pub fn is_entry_time(&self, now: chrono::DateTime<chrono::Utc>) -> bool
  // pub fn get_entry_time(&self, reference: chrono::DateTime<chrono::Utc>) -> Option<chrono::DateTime<chrono::Utc>>
  // pub fn has_entry_time_passed(&self, now: chrono::DateTime<chrono::Utc>) -> bool
  // pub fn get_exit_time(&self) -> Option<chrono::DateTime<chrono::Utc>> // assme same day
  // pub fn has_exit_time_passed(&self, now: chrono::DateTime<chrono::Utc>) -> bool
  // We'll do same with exits too later, they pass in the expiry time

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum BuzzBotDoneNiceStyle {
    /// Close when current spread value ≤ this credit (profit taking)
    SpreadValueGain(f64),
    /// Close when current spread value ≥ this credit (loss cutting)
    SpreadValueLoss(f64),
    /// Close at a specific calendar time relative to expiry
    SpreadValueTime(BotActionTime),
}
impl Default for BuzzBotDoneNiceStyle {
    fn default() -> Self {
        BuzzBotDoneNiceStyle::SpreadValueGain(0.05) // Default to SpreadValueGain with 0.05 - buy back at $5
    }
}

#[derive(Default, Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct MakeStealthBot 
{ pub my_accounting: CommonBotInfo, // For Account managment
  pub my_cash_alloc: f64, // startup we request this. track updates from bot activity for future startups.
  pub stonk_feeling: MarketDirection, // for entry, we buying put/call?
  pub option_expire: u16, // minimum days for option expiry
  pub option_bucket: u8, // for entry, 0 => atm, +1 to go otm by 1, etc
  pub spread_bucket: u8, // for exit, number of buckets from entry itm direction
  pub exit_gain_pct: f64, // // 0 to +100% (or higher), as a function of target_spread
  pub exit_loss_pct: f64, // 0 to -100% but can go below -100% due to being a spread in a way...
  pub nice_exit_way: Vec<StealthBotDoneNiceStyle>, // Position Exit Strategy
  pub use_theo_cost: bool 
}

#[derive(Default, Clone, Debug, serde::Serialize, serde::Deserialize, PartialEq)]
pub enum SallyAction
{ #[default]
  SubmitRightAwayButLikeOnlyUseForTest,
  SubmitWhenBidAskMidAttainsInputPrice(f64),
  SubmitWhenMarketSideReachesThisPrice(f64), // buy: ask <= price, sell: bid >= price
  SubmitAtThisTimeUnlessItReachesPrice(BotActionTime, f64), // at startup look at this_price vs current, get price difference (% or $), at this time, get current price, adjust to same difference, sumbit order.
}

#[derive(Default, Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct MakeSallyFakes
{ pub my_accounting: CommonBotInfo, // For Account managment
  pub my_hide_order: Order, // what we pass to the broker
  pub my_reveal_way: SallyAction, // how we pass to the broker.
}
#[derive(Default, Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct MakeSwatBotsGo
{ pub my_accounting: CommonBotInfo, // For Account managment
}

#[derive(Default, Clone, Debug, serde::Serialize, serde::Deserialize)]
pub enum StealthBotDoneNiceStyle 
{ #[default] 
  OtmShort, // short leg goes OTM 
  FinalPct(f64), // short leg gain %
  PriorDay(BotActionTime), // will exit relative_days before expiry
  GoExpire, // let the contract expire
}


#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub enum AttemptDemands 
{ SubscribeToTicker(String),
  UnsubscribeTicker(String),
  ReportOfAllStatus
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, Copy, PartialEq, Eq)]
pub enum BotList
{ DrRobotnik //"🖻""
, DrBotZero0 //"💻""
, DrBotOnes1 //"📱""
, DrBotTwos2 //"🖧""
, BuzzBomber 
, StealthBot
, SallyFakes
, SwatBotFun
/* FUTURE BOTS:
  DragonBots
  GoCoconuts
  MuttskiBot
  GoScorpion
  UncleChuck
  BadBatBots
  Centipedes
  SniperBots
  SonicCrush
  GoCrabmeat
  HoverCraft
  SpyEyesBot
  Sattellite
  CloudBurst
  MoleBotDig
  CaveKraken
  GoDrillBot
  Pollutions
  GoDinosaur
*/
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub enum Snively {
  // Dr Robo Related Stuff
    MakeBotBuzz(MakeBuzzBomber),
    MakeStealth(MakeStealthBot),
    MakeSallyBs(MakeSallyFakes),
    MakeSwatBot(MakeSwatBotsGo),

    LiveBotName(String),
    KillBotName(String),
    StopBotName(String),

    EmeraldInfo(PushTickerResult),
    
    SendLogNote(String), // For two-way debug/adnmin comms 
    
    UserDemands(AttemptDemands),
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub enum MadeBot {
    Made(String),
    Fail(String),
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub enum GotTtai {
    NewAccountUpdate(AccountUpdate),
    NewPositionsList(PositionsList),
    NewPushOrderTask(Order, PushOrderResult),
    NewOrderToUpdate(PlacedOrder),
    NewPushTickerRes(PushTickerResult),
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)] // + Default
pub struct GotTick00
{ pub ticker : Ticker
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)] // + Default
pub struct GotTick01
{ pub ticker : Ticker
}

// ==================================================================
// PART 3: CONVERSION MODULE (ttrs -> dsta)
// ==================================================================

pub mod convert;

// ==================================================================
// PART 4: SEEK/SEEM CHANNELS
// ==================================================================


pub mod ksta 
{
  use super::
  {   Snively
    , MadeBot
    , GotTtai
    , GotTick00 // Tickers tracked by the bots 
    , GotTick01 // Tickers tracked by the user 
    // Expand as needed, but dont' need to because snively
  };

  seek::seek! 
  {   Snively
    , MadeBot
    , GotTtai
    , GotTick00 // tickers from robots
    , GotTick01 // tickers from 
    // Expand as needed, but dont' need to because snively
  }
}

mod fun;
pub use crate::fun::*;
// e.g. has UsaMarketTimes fn to_utc(&self) -> chrono::NaiveTime 

#[cfg(test)]
pub mod tests;
