// trade/wsta/src/protocol.rs
//
// Browser-facing protocol.
// The browser sends compact UI packets.
// wsta translates them into canonical dsta::Snively messages.

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(tag = "kind", content = "body")]
pub enum BrowserToWsta {
    SelectView { view: WstaView },
    SendLogNote { text: String },

    LiveBotName { name: String },
    StopBotName { name: String },
    KillBotName { name: String },

    SubscribeToTicker { ticker: String },
    UnsubscribeTicker { ticker: String },
    ReportOfAllStatus,

    CreateBuzzBot(CreateBuzzBotInput),
    CreateStealthBot(CreateStealthBotInput),
    CreateSallyBot(CreateSallyBotInput),
    CreateSwatBot(CreateSwatBotInput),
}

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

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(tag = "kind", content = "body")]
pub enum WstaToBrowser {
    Connected {
        message: String,
    },
    SelectedView {
        view: WstaView,
    },
    DebugLine {
        line: String,
    },
    BackendMadeBot {
        value: dsta::MadeBot,
    },
    BackendGotTtai {
        value: dsta::GotTtai,
    },
    BackendGotTick00 {
        value: dsta::GotTick00,
    },
    BackendGotTick01 {
        value: dsta::GotTick01,
    },
    Error {
        message: String,
    },
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct CommonBotInfoInput {
    pub friendly_name: String,
    pub tracking_tick: String,

    #[serde(default)]
    pub max_cash_risk_use_max: bool,

    #[serde(default)]
    pub max_cash_risk_percent: f64,

    #[serde(default)]
    pub max_cash_risk_dollar: f64,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct BotActionTimeInput {
    pub relative_days: u16,
    pub market_time: String,

    #[serde(default)]
    pub wait_seconds: i64,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct BuzzExitInput {
    pub exit_kind: String,

    #[serde(default)]
    pub value: f64,

    #[serde(default)]
    pub time: Option<BotActionTimeInput>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct StealthExitInput {
    pub exit_kind: String,

    #[serde(default)]
    pub value: f64,

    #[serde(default)]
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

    #[serde(default)]
    pub follow_a_exit: Vec<BuzzExitInput>,

    #[serde(default)]
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

    #[serde(default)]
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

    #[serde(default)]
    pub price: f64,

    #[serde(default)]
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

fn parse_market_direction(s: &str) -> dsta::MarketDirection {
    match s.trim().to_ascii_lowercase().as_str() {
        "sideways" | "lull" | "lull market" => dsta::MarketDirection::Sideways,
        "corrects" | "bear" | "bear market" => dsta::MarketDirection::Corrects,
        _ => dsta::MarketDirection::GetStonk,
    }
}

fn parse_market_time(s: &str) -> dsta::UsaMarketTimes {
    match s.trim().to_ascii_lowercase().as_str() {
        "resetttai" | "reset_ttai" => dsta::UsaMarketTimes::ResetTtai,
        "itsclosed" | "its_closed" => dsta::UsaMarketTimes::ItsClosed,
        "hurry5sec" | "hurry_5_sec" => dsta::UsaMarketTimes::Hurry5sec,
        "tminusten" | "tminus_ten" => dsta::UsaMarketTimes::TminusTen,
        "tminus30s" | "tminus_30s" => dsta::UsaMarketTimes::Tminus30s,
        "tminus60s" | "tminus_60s" => dsta::UsaMarketTimes::Tminus60s,
        "deadshort" | "dead_short" => dsta::UsaMarketTimes::DeadShort,
        "powerfive" | "power_five" => dsta::UsaMarketTimes::PowerFive,
        "powereasy" | "power_easy" => dsta::UsaMarketTimes::PowerEasy,
        "powerhour" | "power_hour" => dsta::UsaMarketTimes::PowerHour,
        "fedspeach" | "fed_speach" => dsta::UsaMarketTimes::FedSpeach,
        "fedminute" | "fed_minute" => dsta::UsaMarketTimes::FedMinute,
        "tradetime" | "trade_time" => dsta::UsaMarketTimes::TradeTime,
        "lunchtime" | "lunch_time" => dsta::UsaMarketTimes::LunchTime,
        "euroclose" | "euro_close" => dsta::UsaMarketTimes::EuroClose,
        "donetrend" | "done_trend" => dsta::UsaMarketTimes::DoneTrend,
        "openchaos" | "open_chaos" => dsta::UsaMarketTimes::OpenChaos,
        _ => dsta::UsaMarketTimes::PowerEnds,
    }
}

fn bot_action_time(input: &BotActionTimeInput) -> dsta::BotActionTime {
    dsta::BotActionTime {
        relative_days: input.relative_days,
        my_entry_time: parse_market_time(&input.market_time),
        my_entry_wait: chrono::Duration::seconds(input.wait_seconds),
    }
}

fn common_bot_info(input: &CommonBotInfoInput) -> dsta::CommonBotInfo {
    let mut out = dsta::CommonBotInfo::default();

    out.friendly_name = input.friendly_name.trim().to_string();
    out.tracking_tick = input.tracking_tick.trim().to_string();
    out.max_cash_risk = (
        input.max_cash_risk_use_max,
        input.max_cash_risk_percent,
        input.max_cash_risk_dollar,
    );

    out
}

fn buzz_exit(input: &BuzzExitInput) -> dsta::BuzzBotDoneNiceStyle {
    match input.exit_kind.trim().to_ascii_lowercase().as_str() {
        "spreadvalueloss" | "spread_value_loss" | "loss" => {
            dsta::BuzzBotDoneNiceStyle::SpreadValueLoss(input.value)
        }
        "spreadvaluetime" | "spread_value_time" | "time" => {
            dsta::BuzzBotDoneNiceStyle::SpreadValueTime(
                input
                    .time
                    .as_ref()
                    .map(bot_action_time)
                    .unwrap_or_else(|| dsta::BotActionTime {
                        relative_days: 0,
                        my_entry_time: dsta::UsaMarketTimes::PowerEnds,
                        my_entry_wait: chrono::Duration::seconds(0),
                    }),
            )
        }
        _ => dsta::BuzzBotDoneNiceStyle::SpreadValueGain(input.value),
    }
}

fn stealth_exit(input: &StealthExitInput) -> dsta::StealthBotDoneNiceStyle {
    match input.exit_kind.trim().to_ascii_lowercase().as_str() {
        "finalpct" | "final_pct" | "pct" => {
            dsta::StealthBotDoneNiceStyle::FinalPct(input.value)
        }
        "priorday" | "prior_day" => {
            dsta::StealthBotDoneNiceStyle::PriorDay(
                input
                    .time
                    .as_ref()
                    .map(bot_action_time)
                    .unwrap_or_else(|| dsta::BotActionTime {
                        relative_days: 0,
                        my_entry_time: dsta::UsaMarketTimes::PowerEnds,
                        my_entry_wait: chrono::Duration::seconds(0),
                    }),
            )
        }
        "goexpire" | "go_expire" => dsta::StealthBotDoneNiceStyle::GoExpire,
        _ => dsta::StealthBotDoneNiceStyle::OtmShort,
    }
}

fn parse_buy_or_sell(s: &str) -> dsta::BuyOrSell {
    match s.trim().to_ascii_lowercase().as_str() {
        "selltoopen" | "sell_to_open" | "sell to open" => dsta::BuyOrSell::SellToOpen,
        "buytoclose" | "buy_to_close" | "buy to close" => dsta::BuyOrSell::BuyToClose,
        "selltoclose" | "sell_to_close" | "sell to close" => dsta::BuyOrSell::SellToClose,
        "buyfutures" | "buy_futures" | "buy futures" | "buy" => dsta::BuyOrSell::BuyFutures,
        "sellfutures" | "sell_futures" | "sell futures" | "sell" => dsta::BuyOrSell::SellFutures,
        _ => dsta::BuyOrSell::BuyToOpen,
    }
}

fn parse_order_type(s: &str) -> dsta::MarketOrLimit {
    match s.trim().to_ascii_lowercase().as_str() {
        "market" => dsta::MarketOrLimit::Market,
        _ => dsta::MarketOrLimit::Limit,
    }
}

fn sally_reveal(input: &SallyRevealInput) -> dsta::SallyAction {
    match input.reveal_kind.trim().to_ascii_lowercase().as_str() {
        "bidaskmid" | "bid_ask_mid" | "mid" => {
            dsta::SallyAction::SubmitWhenBidAskMidAttainsInputPrice(input.price)
        }
        "marketside" | "market_side" | "side" => {
            dsta::SallyAction::SubmitWhenMarketSideReachesThisPrice(input.price)
        }
        "timeunlessprice" | "time_unless_price" | "time" => {
            dsta::SallyAction::SubmitAtThisTimeUnlessItReachesPrice(
                input
                    .time
                    .as_ref()
                    .map(bot_action_time)
                    .unwrap_or_else(|| dsta::BotActionTime {
                        relative_days: 0,
                        my_entry_time: dsta::UsaMarketTimes::PowerEnds,
                        my_entry_wait: chrono::Duration::seconds(0),
                    }),
                input.price,
            )
        }
        _ => dsta::SallyAction::SubmitRightAwayButLikeOnlyUseForTest,
    }
}

fn stock_finass(ticker: &str) -> dsta::FinAss {
    dsta::FinAss::StkDeets(dsta::Stk {
        ticker_name: ticker.trim().to_string(),
        ticker_info: String::new(),
        last_ticker: None,
        tick_rulers: None,
    })
}

impl BrowserToWsta {
    pub fn maybe_snively(&self) -> Option<dsta::Snively> {
        match self {
            BrowserToWsta::SelectView { .. } => None,

            BrowserToWsta::SendLogNote { text } => {
                Some(dsta::Snively::SendLogNote(text.clone()))
            }

            BrowserToWsta::LiveBotName { name } => {
                Some(dsta::Snively::LiveBotName(name.clone()))
            }

            BrowserToWsta::StopBotName { name } => {
                Some(dsta::Snively::StopBotName(name.clone()))
            }

            BrowserToWsta::KillBotName { name } => {
                Some(dsta::Snively::KillBotName(name.clone()))
            }

            BrowserToWsta::SubscribeToTicker { ticker } => {
                Some(dsta::Snively::UserDemands(
                    dsta::AttemptDemands::SubscribeToTicker(ticker.clone()),
                ))
            }

            BrowserToWsta::UnsubscribeTicker { ticker } => {
                Some(dsta::Snively::UserDemands(
                    dsta::AttemptDemands::UnsubscribeTicker(ticker.clone()),
                ))
            }

            BrowserToWsta::ReportOfAllStatus => {
                Some(dsta::Snively::UserDemands(
                    dsta::AttemptDemands::ReportOfAllStatus,
                ))
            }

            BrowserToWsta::CreateBuzzBot(input) => {
                let exits = if input.follow_a_exit.is_empty() {
                    vec![dsta::BuzzBotDoneNiceStyle::default()]
                } else {
                    input.follow_a_exit.iter().map(buzz_exit).collect()
                };

                Some(dsta::Snively::MakeBotBuzz(dsta::MakeBuzzBomber {
                    my_accounting: common_bot_info(&input.common),
                    my_cash_alloc: input.cash_alloc,
                    stonk_feeling: parse_market_direction(&input.market_direction),
                    option_expire: input.option_expire,
                    target_spread: input.target_spread,
                    time_to_order: bot_action_time(&input.time_to_order),
                    follow_a_exit: exits,
                    algo_cooldown: chrono::Duration::seconds(input.algo_cooldown_seconds),
                    bombs_forever: input.bombs_forever,
                }))
            }

            BrowserToWsta::CreateStealthBot(input) => {
                let exits = if input.nice_exit_way.is_empty() {
                    vec![dsta::StealthBotDoneNiceStyle::default()]
                } else {
                    input.nice_exit_way.iter().map(stealth_exit).collect()
                };

                Some(dsta::Snively::MakeStealth(dsta::MakeStealthBot {
                    my_accounting: common_bot_info(&input.common),
                    my_cash_alloc: input.cash_alloc,
                    stonk_feeling: parse_market_direction(&input.market_direction),
                    option_expire: input.option_expire,
                    option_bucket: input.option_bucket,
                    spread_bucket: input.spread_bucket,
                    exit_gain_pct: input.exit_gain_pct,
                    exit_loss_pct: input.exit_loss_pct,
                    nice_exit_way: exits,
                    use_theo_cost: input.use_theo_cost,
                }))
            }

            BrowserToWsta::CreateSallyBot(input) => {
                Some(dsta::Snively::MakeSallyBs(dsta::MakeSallyFakes {
                    my_accounting: common_bot_info(&input.common),
                    my_hide_order: dsta::Order {
                        order_legs: vec![dsta::OrderLeg {
                            buy_what: stock_finass(&input.order.ticker),
                            quantity: input.order.quantity,
                            remaining: input.order.quantity,
                            action: parse_buy_or_sell(&input.order.action),
                            price: input.order.price,
                        }],
                        order_type: parse_order_type(&input.order.order_type),
                    },
                    my_reveal_way: sally_reveal(&input.reveal),
                }))
            }

            BrowserToWsta::CreateSwatBot(input) => {
                Some(dsta::Snively::MakeSwatBot(dsta::MakeSwatBotsGo {
                    my_accounting: common_bot_info(&input.common),
                }))
            }
        }
    }
}
