
// trade/vsta/src/base/v_time_span/state.rs

use iced::widget::text_input;
use chrono::Duration;
use std::str::FromStr;
use crate::base::v_valid_any::{ValidatedInput, Message as ValidatedInputMessage};

#[derive(Debug, Clone, PartialEq, Eq, Copy)]
pub enum TimeSpanUnit {
    Milliseconds,
    Seconds,
    Minutes,
    Hours,
    Days,
}

impl std::fmt::Display for TimeSpanUnit {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TimeSpanUnit::Milliseconds => write!(f, "ms"),
            TimeSpanUnit::Seconds => write!(f, "sec"),
            TimeSpanUnit::Minutes => write!(f, "min"),
            TimeSpanUnit::Hours => write!(f, "hr"),
            TimeSpanUnit::Days => write!(f, "day"),
        }
    }
}

#[derive(Debug, Clone)]
pub enum Message {
    ValueChanged(String),
    UnitChanged(TimeSpanUnit),
    FlashError(iced::time::Instant),
}

#[derive(Debug, Clone)]
pub struct TimeSpan {
    pub value: f64,
    pub unit: TimeSpanUnit,
}

impl TimeSpan {
    pub fn to_duration(&self) -> Duration {
        let seconds = match self.unit {
            TimeSpanUnit::Milliseconds => self.value / 1000.0,
            TimeSpanUnit::Seconds => self.value,
            TimeSpanUnit::Minutes => self.value * 60.0,
            TimeSpanUnit::Hours => self.value * 3600.0,
            TimeSpanUnit::Days => self.value * 86400.0,
        };
        Duration::seconds(seconds as i64)
    }

    pub fn from_duration(duration: Duration) -> Self {
        let total_seconds = duration.num_seconds() as f64;
        TimeSpan {
            value: total_seconds,
            unit: TimeSpanUnit::Seconds,
        }
    }

    /// Convert current value to a new unit, returning the converted value as a string
    pub fn convert_to_unit(&self, new_unit: TimeSpanUnit) -> String {
        // First convert current value to total seconds
        let total_seconds = match self.unit {
            TimeSpanUnit::Milliseconds => self.value / 1000.0,
            TimeSpanUnit::Seconds => self.value,
            TimeSpanUnit::Minutes => self.value * 60.0,
            TimeSpanUnit::Hours => self.value * 3600.0,
            TimeSpanUnit::Days => self.value * 86400.0,
        };

        // Then convert from seconds to the new unit
        let converted = match new_unit {
            TimeSpanUnit::Milliseconds => total_seconds * 1000.0,
            TimeSpanUnit::Seconds => total_seconds,
            TimeSpanUnit::Minutes => total_seconds / 60.0,
            TimeSpanUnit::Hours => total_seconds / 3600.0,
            TimeSpanUnit::Days => total_seconds / 86400.0,
        };

        // Format nicely: remove trailing zeros and unnecessary decimals
        if converted.fract() == 0.0 {
            format!("{:.0}", converted)
        } else {
            format!("{}", converted).trim_end_matches('0').trim_end_matches('.').to_string()
        }
    }
}

#[derive(Debug, Clone)]
pub struct TimeSpanControl {
    pub time_span: TimeSpan,
    pub value_input: ValidatedInput<f64>,
    pub unit_selector: TimeSpanUnit,
}

impl TimeSpanControl {
    pub fn new() -> Self {
        TimeSpanControl {
            time_span: TimeSpan {
                value: 0.0,
                unit: TimeSpanUnit::Seconds,
            },
            value_input: ValidatedInput::new(Box::new(|s| s.parse::<f64>().ok())),
            unit_selector: TimeSpanUnit::Seconds,
        }
    }

    pub fn update(&mut self, message: Message) {
        match message {
            Message::ValueChanged(input) => {
                self.value_input.update(ValidatedInputMessage::InputChanged(input));
                if let Some(value) = self.value_input.parsed_value {
                    self.time_span.value = value;
                }
            }
            Message::UnitChanged(new_unit) => {
                log::debug!(
                    "TimeSpanControl::update UnitChanged from {} to {}",
                    self.unit_selector,
                    new_unit
                );

                // Convert the current value to the new unit
                let converted_value_str = self.time_span.convert_to_unit(new_unit);
                log::debug!(
                    "Converted value from {} {} to {}",
                    self.time_span.value,
                    self.unit_selector,
                    converted_value_str
                );

                // Update the display string
                self.value_input.value = converted_value_str.clone();

                // Parse and update the actual value
                if let Ok(converted_value) = converted_value_str.parse::<f64>() {
                    self.value_input.parsed_value = Some(converted_value);
                    self.time_span.value = converted_value;
                }

                // Update the unit selector
                self.unit_selector = new_unit;
                self.time_span.unit = new_unit;

                log::debug!(
                    "TimeSpanControl::update UnitChanged complete: {} {}",
                    self.time_span.value,
                    self.unit_selector
                );
            }
            Message::FlashError(elapsed) => {
                self.value_input.update(ValidatedInputMessage::FlashError(elapsed));
            }
        }
    }

    // Call this after programmatically updating time_span
    pub fn sync_from_time_span(&mut self) {
        // Update value_input to match time_span
        self.value_input.value = format!("{}", self.time_span.value);
        self.value_input.parsed_value = Some(self.time_span.value);
        
        // Update unit_selector to match time_span
        self.unit_selector = self.time_span.unit;
        
        log::debug!(
            "TimeSpanControl::sync_from_time_span: {} {}",
            self.time_span.value,
            self.time_span.unit
        );
    }
}