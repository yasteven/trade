// trade/wsta/src/protocol.rs
//
// Browser-facing protocol.
// The browser does not own the canonical bot protocol.
// It sends compact UI packets; wsta translates them to dsta::Snively.

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

    CreateBuzzBot {
        friendly_name: String,
        tracking_tick: String,
        cash_alloc: f64,
        market_direction: String,
        option_expire: u16,
        target_spread: f64,
        bombs_forever: bool,
    },

    CreateStealthBot {
        friendly_name: String,
        tracking_tick: String,
        cash_alloc: f64,
        market_direction: String,
        option_expire: u16,
        option_bucket: u8,
        spread_bucket: u8,
        exit_gain_pct: f64,
        exit_loss_pct: f64,
        use_theo_cost: bool,
    },

    CreateSallyBot {
        friendly_name: String,
        tracking_tick: String,
    },

    CreateSwatBot {
        friendly_name: String,
        tracking_tick: String,
    },
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

fn parse_market_direction(s: &str) -> dsta::MarketDirection {
    match s.trim().to_ascii_lowercase().as_str() {
        "sideways" => dsta::MarketDirection::Sideways,
        "corrects" => dsta::MarketDirection::Corrects,
        _ => dsta::MarketDirection::GetStonk,
    }
}

fn common_bot_info(friendly_name: &str, tracking_tick: &str) -> dsta::CommonBotInfo {
    let mut out = dsta::CommonBotInfo::default();
    out.friendly_name = friendly_name.trim().to_string();
    out.tracking_tick = tracking_tick.trim().to_string();
    out
}

fn default_bot_action_time() -> dsta::BotActionTime {
    dsta::BotActionTime {
        relative_days: 0,
        my_entry_time: dsta::UsaMarketTimes::PowerEnds,
        my_entry_wait: chrono::Duration::seconds(0),
    }
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

            BrowserToWsta::CreateBuzzBot {
                friendly_name,
                tracking_tick,
                cash_alloc,
                market_direction,
                option_expire,
                target_spread,
                bombs_forever,
            } => {
                Some(dsta::Snively::MakeBotBuzz(dsta::MakeBuzzBomber {
                    my_accounting: common_bot_info(friendly_name, tracking_tick),
                    my_cash_alloc: *cash_alloc,
                    stonk_feeling: parse_market_direction(market_direction),
                    option_expire: *option_expire,
                    target_spread: *target_spread,
                    time_to_order: default_bot_action_time(),
                    follow_a_exit: vec![dsta::BuzzBotDoneNiceStyle::default()],
                    algo_cooldown: chrono::Duration::seconds(0),
                    bombs_forever: *bombs_forever,
                }))
            }

            BrowserToWsta::CreateStealthBot {
                friendly_name,
                tracking_tick,
                cash_alloc,
                market_direction,
                option_expire,
                option_bucket,
                spread_bucket,
                exit_gain_pct,
                exit_loss_pct,
                use_theo_cost,
            } => {
                Some(dsta::Snively::MakeStealth(dsta::MakeStealthBot {
                    my_accounting: common_bot_info(friendly_name, tracking_tick),
                    my_cash_alloc: *cash_alloc,
                    stonk_feeling: parse_market_direction(market_direction),
                    option_expire: *option_expire,
                    option_bucket: *option_bucket,
                    spread_bucket: *spread_bucket,
                    exit_gain_pct: *exit_gain_pct,
                    exit_loss_pct: *exit_loss_pct,
                    nice_exit_way: vec![dsta::StealthBotDoneNiceStyle::default()],
                    use_theo_cost: *use_theo_cost,
                }))
            }

            BrowserToWsta::CreateSallyBot {
                friendly_name,
                tracking_tick,
            } => {
                let mut bot = dsta::MakeSallyFakes::default();
                bot.my_accounting = common_bot_info(friendly_name, tracking_tick);
                Some(dsta::Snively::MakeSallyBs(bot))
            }

            BrowserToWsta::CreateSwatBot {
                friendly_name,
                tracking_tick,
            } => {
                Some(dsta::Snively::MakeSwatBot(dsta::MakeSwatBotsGo {
                    my_accounting: common_bot_info(friendly_name, tracking_tick),
                }))
            }
        }
    }
}
