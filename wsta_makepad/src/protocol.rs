// trade/wsta_makepad/src/protocol.rs
// WASM-safe JSON protocol mirror; no dsta dependency here.

#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum WstaView {
    DrR,
    Buzz,
    Stealth,
    Sally,
    Swat,
    Ttai,
    Nico,
    Logs,
}

impl Default for WstaView {
    fn default() -> Self {
        Self::DrR
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DrTool {
    Overview,
    MakeBuzz,
    MakeStealth,
    MakeSally,
    MakeSwat,
    TtaiOverview,
}

impl Default for DrTool {
    fn default() -> Self {
        Self::Overview
    }
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(tag = "kind", content = "body")]
pub enum BrowserToWsta {
    SelectView { view: WstaView },
    SendLogNote { text: String },
    ReportOfAllStatus,
    CreateBuzzBot(CreateBuzzBotInput),
    CreateStealthBot(CreateStealthBotInput),
    CreateSallyBot(CreateSallyBotInput),
    CreateSwatBot(CreateSwatBotInput),
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct CommonBotInfoInput {
    pub friendly_name: String,
    pub tracking_tick: String,
    pub max_cash_risk_use_max: bool,
    pub max_cash_risk_percent: f64,
    pub max_cash_risk_dollar: f64,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct BotActionTimeInput {
    pub relative_days: u16,
    pub market_time: String,
    pub wait_seconds: i64,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct BuzzExitInput {
    pub exit_kind: String,
    pub value: f64,
    pub time: Option<BotActionTimeInput>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct StealthExitInput {
    pub exit_kind: String,
    pub value: f64,
    pub time: Option<BotActionTimeInput>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct CreateBuzzBotInput {
    pub common: CommonBotInfoInput,
    pub cash_alloc: f64,
    pub market_direction: String,
    pub option_expire: u16,
    pub target_spread: f64,
    pub time_to_order: BotActionTimeInput,
    pub follow_a_exit: Vec<BuzzExitInput>,
    pub algo_cooldown_seconds: i64,
    pub bombs_forever: bool,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct CreateStealthBotInput {
    pub common: CommonBotInfoInput,
    pub cash_alloc: f64,
    pub market_direction: String,
    pub option_expire: u16,
    pub option_bucket: u8,
    pub spread_bucket: u8,
    pub exit_gain_pct: f64,
    pub exit_loss_pct: f64,
    pub nice_exit_way: Vec<StealthExitInput>,
    pub use_theo_cost: bool,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct SallyOrderInput {
    pub ticker: String,
    pub quantity: f64,
    pub price: f64,
    pub action: String,
    pub order_type: String,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct SallyRevealInput {
    pub reveal_kind: String,
    pub price: f64,
    pub time: Option<BotActionTimeInput>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct CreateSallyBotInput {
    pub common: CommonBotInfoInput,
    pub order: SallyOrderInput,
    pub reveal: SallyRevealInput,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct CreateSwatBotInput {
    pub common: CommonBotInfoInput,
}

pub fn common(name: &str, ticker: &str) -> CommonBotInfoInput {
    CommonBotInfoInput {
        friendly_name: name.to_string(),
        tracking_tick: ticker.to_string(),
        max_cash_risk_use_max: false,
        max_cash_risk_percent: 0.0,
        max_cash_risk_dollar: 0.0,
    }
}

pub fn action_time() -> BotActionTimeInput {
    BotActionTimeInput {
        relative_days: 0,
        market_time: "PowerEnds".to_string(),
        wait_seconds: 0,
    }
}
