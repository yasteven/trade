
// trade/vsta/src/core/v_haggle_limits/state.rs

use chrono::Duration;
use crate::base::v_valid_any::{ValidatedInput, Message as ValidatedInputMessage};
use crate::base::v_time_span::{TimeSpanControl, Message as TimeSpanMessage};

#[derive(Debug, Clone)]
pub enum Message {
    RetryPeriodMessage(TimeSpanMessage),
    DeltaChoiceChanged(String),
    DeltaIsPctToggled(bool),
    SlippageMaxChanged(String),
    FlashError(iced::time::Instant),
}

#[derive(Debug, Clone)]
pub struct HaggleLimitsControl {
    pub retry_period_control: TimeSpanControl,
    pub delta_choice_input: ValidatedInput<f64>,
    pub delta_is_pct: bool,
    pub slippage_max_input: ValidatedInput<f64>,
}

impl HaggleLimitsControl {
    pub fn new() -> Self {
        HaggleLimitsControl {
            retry_period_control: TimeSpanControl::new(),
            delta_choice_input: ValidatedInput::new(Box::new(|s| s.parse::<f64>().ok())),
            delta_is_pct: true,
            slippage_max_input: ValidatedInput::new(Box::new(|s| s.parse::<f64>().ok())),
        }
    }

    pub fn from_dsta(limits: &dsta::HaggleLimits) -> Self {
        let mut control = Self::new();
        control.retry_period_control = TimeSpanControl {
            time_span: crate::base::v_time_span::TimeSpan::from_duration(limits.retry_period),
            value_input: ValidatedInput::new(Box::new(|s| s.parse::<f64>().ok())),
            unit_selector: crate::base::v_time_span::TimeSpanUnit::Seconds,
        };
        control.delta_choice_input.value = format!("{}", limits.delta_choice);
        control.delta_choice_input.parsed_value = Some(limits.delta_choice);
        control.delta_is_pct = limits.delta_is_pct;
        control.slippage_max_input.value = format!("{}", limits.slippage_max);
        control.slippage_max_input.parsed_value = Some(limits.slippage_max);
        control
    }

    pub fn to_dsta(&self) -> dsta::HaggleLimits {
        dsta::HaggleLimits {
            retry_period: self.retry_period_control.time_span.to_duration(),
            delta_choice: self.delta_choice_input.parsed_value.unwrap_or(25.0),
            delta_is_pct: self.delta_is_pct,
            slippage_max: self.slippage_max_input.parsed_value.unwrap_or(0.1),
        }
    }

    pub fn sync_from_dsta(&mut self, limits: &dsta::HaggleLimits) {
        // Update retry_period_control
        self.retry_period_control.time_span = 
            crate::base::v_time_span::TimeSpan::from_duration(limits.retry_period);
        self.retry_period_control.sync_from_time_span();
        
        // Update delta_choice input
        self.delta_choice_input.value = format!("{}", limits.delta_choice);
        self.delta_choice_input.parsed_value = Some(limits.delta_choice);
        
        // Update delta_is_pct toggle
        self.delta_is_pct = limits.delta_is_pct;
        
        // Update slippage_max input
        self.slippage_max_input.value = format!("{}", limits.slippage_max);
        self.slippage_max_input.parsed_value = Some(limits.slippage_max);
    }


    pub fn update(&mut self, message: Message) {
        match message {
            Message::RetryPeriodMessage(msg) => {
                self.retry_period_control.update(msg);
            }
            Message::DeltaChoiceChanged(input) => {
                self.delta_choice_input.update(ValidatedInputMessage::InputChanged(input));
            }
            Message::DeltaIsPctToggled(is_pct) => {
                self.delta_is_pct = is_pct;
            }
            Message::SlippageMaxChanged(input) => {
                self.slippage_max_input.update(ValidatedInputMessage::InputChanged(input));
            }
            Message::FlashError(elapsed) => {
                self.delta_choice_input.update(ValidatedInputMessage::FlashError(elapsed));
                self.slippage_max_input.update(ValidatedInputMessage::FlashError(elapsed));
            }
        }
    }
}
