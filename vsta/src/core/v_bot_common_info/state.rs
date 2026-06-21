
// trade/vsta/src/core/v_bot_common_info/state.rs 

use chrono::{TimeDelta, Duration};
use crate::base::v_valid_any::{ValidatedInput, Message as ValidatedInputMessage};
use crate::core::v_haggle_action::{HaggleActionControl, Message as HaggleActionMessage};
use dsta::{CommonBotInfo, HaggleMethod, HaggleAction};

#[derive(Debug, Clone)]
pub enum Message {
    FriendlyNameChanged(String),
    TrackingTickChanged(String),
    HaggleActionMessage(HaggleActionMessage),
    MaxCashRiskBoolChanged(bool),
    MaxCashRiskPercentChanged(String),
    MaxCashRiskDollarChanged(String),
    FlashError(iced::time::Instant),
}

#[derive(Debug, Clone)]
pub struct CommonBotInfoControl {
    pub common_bot_info: CommonBotInfo,
    pub friendly_name_input: String,
    pub tracking_tick_input: String,
    pub haggle_action_control: HaggleActionControl,
    pub max_cash_risk_bool: bool,
    pub max_cash_risk_percent_input: ValidatedInput<f64>,
    pub max_cash_risk_dollar_input: ValidatedInput<f64>,
}

impl CommonBotInfoControl {
    pub fn new() -> Self {
        CommonBotInfoControl {
            common_bot_info: CommonBotInfo::default(),
            friendly_name_input: "".to_string(),
            tracking_tick_input: "".to_string(),
            haggle_action_control: HaggleActionControl::new(),
            max_cash_risk_bool: false,
            max_cash_risk_percent_input: ValidatedInput::new(Box::new(|s| s.parse::<f64>().ok())),
            max_cash_risk_dollar_input: ValidatedInput::new(Box::new(|s| s.parse::<f64>().ok())),
        }
    }
    pub fn old(cbi : dsta::CommonBotInfo) -> Self 
    { 
      let a0 = format!("{}", cbi.friendly_name);
      let a1 = format!("{}", cbi.tracking_tick);
      let a2 = HaggleActionControl::from_dsta(&cbi.haggle_action);
      let a3 = cbi.max_cash_risk.0;
      let a4 = cbi.max_cash_risk.1;
      let a5 = cbi.max_cash_risk.2;
      CommonBotInfoControl 
      {
        common_bot_info: cbi,
        friendly_name_input: a0,
        tracking_tick_input: a1,
        haggle_action_control: a2,
        max_cash_risk_bool: a3,
        max_cash_risk_percent_input: ValidatedInput::old
        ( a4, Box::new(|s| s.parse::<f64>().ok())),
        max_cash_risk_dollar_input: ValidatedInput::old
        ( a5, Box::new(|s| s.parse::<f64>().ok())),
      }
    }


    pub fn sync_to_dsta(&mut self) {
        self.common_bot_info.friendly_name = self.friendly_name_input.clone();
        self.common_bot_info.tracking_tick = self.tracking_tick_input.clone();
        self.common_bot_info.haggle_action = self.haggle_action_control.to_dsta();
        self.common_bot_info.max_cash_risk.0 = self.max_cash_risk_bool;
        if let Some(v) = self.max_cash_risk_percent_input.parsed_value {
            self.common_bot_info.max_cash_risk.1 = v;
        }
        if let Some(v) = self.max_cash_risk_dollar_input.parsed_value {
            self.common_bot_info.max_cash_risk.2 = v;
        }
    }

    /// Sync UI inputs FROM common_bot_info (reverse of sync_to_dsta)
    /// Call this after programmatically updating common_bot_info
    pub fn sync_from_common_bot_info(&mut self) {
        // Update friendly_name
        self.friendly_name_input = self.common_bot_info.friendly_name.clone();
        
        // Update tracking_tick
        self.tracking_tick_input = self.common_bot_info.tracking_tick.clone();
        
        // Update max_cash_risk toggle and values
        let (enabled, percent_val, dollar_val) = self.common_bot_info.max_cash_risk;
        self.max_cash_risk_bool = enabled;
        
        // Update percent input
        self.max_cash_risk_percent_input.value = format!("{}", percent_val);
        self.max_cash_risk_percent_input.parsed_value = Some(percent_val);
        
        // Update dollar input
        self.max_cash_risk_dollar_input.value = format!("{}", dollar_val);
        self.max_cash_risk_dollar_input.parsed_value = Some(dollar_val);
        
        // Update haggle action control (assuming it has a sync method)
        self.haggle_action_control.sync_from_dsta(&self.common_bot_info.haggle_action);
    }


    pub fn update(&mut self, message: Message) {
        match message {
            Message::FriendlyNameChanged(name) => {
                self.friendly_name_input = name;
            }
            Message::TrackingTickChanged(tick) => {
                self.tracking_tick_input = tick;
            }
            Message::HaggleActionMessage(msg) => {
                self.haggle_action_control.update(msg);
            }
            Message::MaxCashRiskBoolChanged(use_max) => {
                self.max_cash_risk_bool = use_max;
            }
            Message::MaxCashRiskPercentChanged(input) => {
                self.max_cash_risk_percent_input.update(ValidatedInputMessage::InputChanged(input));
            }
            Message::MaxCashRiskDollarChanged(input) => {
                self.max_cash_risk_dollar_input.update(ValidatedInputMessage::InputChanged(input));
            }
            Message::FlashError(elapsed) => {
                self.max_cash_risk_percent_input.update(ValidatedInputMessage::FlashError(elapsed));
                self.max_cash_risk_dollar_input.update(ValidatedInputMessage::FlashError(elapsed));
            }
        }
        self.sync_to_dsta();
    }
}