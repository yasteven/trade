
// trade/vsta/src/core/v_haggle_action/state.rs

use dsta::{HaggleAction, HaggleMethod};
use crate::core::v_haggle_method::{HaggleMethodControl, Message as HaggleMethodMessage};

#[derive(Debug, Clone)]
pub enum Message {
    ActionVariantSelected(HaggleActionType),
    MethodMessage(HaggleMethodMessage),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HaggleActionType {
    JustCancel,
    AnOyVeyJew,
}

impl std::fmt::Display for HaggleActionType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            HaggleActionType::JustCancel => write!(f, "Just Cancel"),
            HaggleActionType::AnOyVeyJew => write!(f, "Haggle"),
        }
    }
}

#[derive(Debug, Clone)]
pub struct HaggleActionControl {
    pub action_variant: HaggleActionType,
    pub method_control: HaggleMethodControl,
}

impl HaggleActionControl {
    pub fn new() -> Self {
        HaggleActionControl {
            action_variant: HaggleActionType::JustCancel,
            method_control: HaggleMethodControl::new(),
        }
    }

    pub fn from_dsta(action: &HaggleAction) -> Self {
        let action_variant = match action {
            HaggleAction::JustCancel(_) => HaggleActionType::JustCancel,
            HaggleAction::AnOyVeyJew(_) => HaggleActionType::AnOyVeyJew,
        };

        let method = match action {
            HaggleAction::JustCancel(m) => m,
            HaggleAction::AnOyVeyJew(m) => m,
        };

        HaggleActionControl {
            action_variant,
            method_control: HaggleMethodControl::from_dsta(method),
        }
    }

    pub fn sync_from_dsta(&mut self, action: &HaggleAction) {
        // Update action variant
        self.action_variant = match action {
            HaggleAction::JustCancel(_) => HaggleActionType::JustCancel,
            HaggleAction::AnOyVeyJew(_) => HaggleActionType::AnOyVeyJew,
        };
        
        // Extract method and sync method control
        let method = match action {
            HaggleAction::JustCancel(m) => m,
            HaggleAction::AnOyVeyJew(m) => m,
        };
        self.method_control.sync_from_dsta(method);
    }
    
    pub fn to_dsta(&self) -> HaggleAction {
        let method = self.method_control.to_dsta();
        match self.action_variant {
            HaggleActionType::JustCancel => HaggleAction::JustCancel(method),
            HaggleActionType::AnOyVeyJew => HaggleAction::AnOyVeyJew(method),
        }
    }

    pub fn update(&mut self, message: Message) {
        match message {
            Message::ActionVariantSelected(variant) => {
                self.action_variant = variant;
            }
            Message::MethodMessage(msg) => {
                self.method_control.update(msg);
            }
        }
    }
}
