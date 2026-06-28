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
        }
    }
}
