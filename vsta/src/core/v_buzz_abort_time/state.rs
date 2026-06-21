// trade/vsta/src/core/v_buzz_abort_time/state.rs

use dsta::BuzzBotDoneNiceStyle;
use crate::base::v_valid_any::{ValidatedInput, Message as ValidatedInputMessage};
use crate::core::v_bot_action_time::{BotActionTimeControl, Message as BotActionTimeMessage};

#[derive(Debug, Clone)]
pub enum Message {
    VariantSelected(BuzzBotDoneNiceStyle),    
    SpreadValueGainChanged(String),  
    SpreadValueLossChanged(String),  
    SpreadValueTimeMessage(BotActionTimeMessage),
    FlashError(iced::time::Instant),
    ToggleVisibility,
}

#[derive(Debug, Clone)]
pub struct BuzzBotDoneNiceStyleControl {
    pub variant: BuzzBotDoneNiceStyle,
    pub is_visible: bool,
    pub spread_value_gain_input: ValidatedInput<f64>,
    pub spread_value_loss_input: ValidatedInput<f64>,
    pub spread_value_time_control: BotActionTimeControl,
}

impl BuzzBotDoneNiceStyleControl {
    pub fn new() -> Self {
        BuzzBotDoneNiceStyleControl {
            variant: BuzzBotDoneNiceStyle::SpreadValueGain(0.05),
            is_visible: true,
            spread_value_gain_input: ValidatedInput::new(Box::new(|s| s.parse::<f64>().ok())),
            spread_value_loss_input: ValidatedInput::new(Box::new(|s| s.parse::<f64>().ok())),
            spread_value_time_control: BotActionTimeControl::new(),
        }
    }

    pub fn update(&mut self, message: Message) {
        match message {
            Message::VariantSelected(variant) => {
                self.variant = variant;
                self.is_visible = true;
                
                // Sync input fields with new variant values
                match &self.variant {
                    BuzzBotDoneNiceStyle::SpreadValueGain(val) => {
                        self.spread_value_gain_input.value = format!("{}", val);
                        self.spread_value_gain_input.parsed_value = Some(*val);
                    }
                    BuzzBotDoneNiceStyle::SpreadValueLoss(val) => {
                        self.spread_value_loss_input.value = format!("{}", val);
                        self.spread_value_loss_input.parsed_value = Some(*val);
                    }
                    BuzzBotDoneNiceStyle::SpreadValueTime(_) => {
                        // Time control already synced
                    }
                }
            }
             Message::SpreadValueGainChanged(input) => {
                // ← CHANGE: Let ValidatedInput handle parsing
                self.spread_value_gain_input.update(ValidatedInputMessage::InputChanged(input));
                // Only update variant if parsing succeeded
                if let Some(value) = self.spread_value_gain_input.parsed_value {
                    self.variant = BuzzBotDoneNiceStyle::SpreadValueGain(value);
                }
                // Don't overwrite on every keystroke!
            }
            Message::SpreadValueLossChanged(input) => {
                // ← CHANGE: Let ValidatedInput handle parsing
                self.spread_value_loss_input.update(ValidatedInputMessage::InputChanged(input));
                // Only update variant if parsing succeeded
                if let Some(value) = self.spread_value_loss_input.parsed_value {
                    self.variant = BuzzBotDoneNiceStyle::SpreadValueLoss(value);
                }
            }
            Message::SpreadValueTimeMessage(msg) => {
                self.spread_value_time_control.update(msg);
                self.variant = BuzzBotDoneNiceStyle::SpreadValueTime(
                    self.spread_value_time_control.buzz_time_to_order.clone()
                );
            }
            Message::FlashError(elapsed) => {
                match &self.variant {
                    BuzzBotDoneNiceStyle::SpreadValueGain(_) => {
                        self.spread_value_gain_input.update(ValidatedInputMessage::FlashError(elapsed));
                    }
                    BuzzBotDoneNiceStyle::SpreadValueLoss(_) => {
                        self.spread_value_loss_input.update(ValidatedInputMessage::FlashError(elapsed));
                    }
                    BuzzBotDoneNiceStyle::SpreadValueTime(_) => {
                        self.spread_value_time_control.update(BotActionTimeMessage::FlashError(elapsed));
                    }
                }
            }
            Message::ToggleVisibility => {
                self.is_visible = !self.is_visible;
            }
        }
    }
}

// Add type alias for ExitConditionControl
pub type ExitConditionControl = BuzzBotDoneNiceStyleControl;