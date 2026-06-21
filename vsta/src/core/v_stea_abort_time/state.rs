// trade/vsta/src/core/v_stealth_bot_done_nice_style/state.rs

use dsta::StealthBotDoneNiceStyle;
use crate::base::v_valid_any::{ValidatedInput, Message as ValidatedInputMessage};
use crate::core::v_bot_action_time::{BotActionTimeControl, Message as BotActionTimeMessage};

#[derive(Debug, Clone)]
pub enum Message {
    VariantSelected(StealthBotDoneNiceStyle),
    FinalPctChanged(String),
    PriorDayMessage(BotActionTimeMessage),
    FlashError(iced::time::Instant),
    ToggleVisibility,
}

#[derive(Debug, Clone)]
pub struct StealthBotDoneNiceStyleControl {
    pub variant: StealthBotDoneNiceStyle,
    pub is_visible: bool,
    pub final_pct_input: ValidatedInput<f64>,
    pub prior_day_control: BotActionTimeControl,
}

impl StealthBotDoneNiceStyleControl {
    pub fn new() -> Self {
        StealthBotDoneNiceStyleControl {
            variant: StealthBotDoneNiceStyle::OtmShort,
            is_visible: true,
            final_pct_input: ValidatedInput::new(Box::new(|s| s.parse::<f64>().ok())),
            prior_day_control: BotActionTimeControl::new(),
        }
    }

    pub fn update(&mut self, message: Message) {
        match message {
            Message::VariantSelected(variant) => {
                self.variant = variant;
                self.is_visible = true;

                // Sync input fields with new variant values
                match &self.variant {
                    StealthBotDoneNiceStyle::FinalPct(val) => {
                        self.final_pct_input.value = format!("{}", val);
                        self.final_pct_input.parsed_value = Some(*val);
                    }
                    _ => {}
                }
            }
            Message::FinalPctChanged(input) => {
                self.final_pct_input.update(ValidatedInputMessage::InputChanged(input));
                if let Some(value) = self.final_pct_input.parsed_value {
                    self.variant = StealthBotDoneNiceStyle::FinalPct(value);
                }
            }
            Message::PriorDayMessage(msg) => {
                self.prior_day_control.update(msg);
                self.variant = StealthBotDoneNiceStyle::PriorDay(
                    self.prior_day_control.buzz_time_to_order.clone()
                );
            }
            Message::FlashError(elapsed) => {
                match &self.variant {
                    StealthBotDoneNiceStyle::FinalPct(_) => {
                        self.final_pct_input.update(ValidatedInputMessage::FlashError(elapsed));
                    }
                    _ => {}
                }
            }
            Message::ToggleVisibility => {
                self.is_visible = !self.is_visible;
            }
        }
    }
}

// Add type alias for exit strategy control
pub type ExitStrategyControl = StealthBotDoneNiceStyleControl;

