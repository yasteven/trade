
// trade/vsta/src/form/v_buzz_make/state.rs

use crate::base::v_valid_any::ValidatedInput;
use crate::base::v_combo_box::ComboBoxState;
use crate::core::v_bot_common_info::CommonBotInfoControl;
use crate::core::v_bot_action_time::BotActionTimeControl;
use crate::core::v_buzz_abort_time::ExitConditionControl;

#[derive(Debug, Clone)]
pub enum Message {
    PressedCreateBot,
    ProcessCreateBot(dsta::MakeBuzzBomber),

    // New messages for sub-controls
    CommonBotInfoMessage(crate::core::v_bot_common_info::Message),
    MyCashAllocChanged(String),
    MarketDirectionChanged(dsta::MarketDirection),
    OptionExpireChanged(String),
    TargetSpreadChanged(String),
    TimeToOrderMessage(crate::core::v_bot_action_time::Message),
    AlgoCooldownChanged(String),
    BombsForeverToggled(bool),
    ExitConditionMessage(usize, crate::core::v_buzz_abort_time::Message),
    FlashError(iced::time::Instant),

    AddExitCondition,
    EditExitCondition(usize),
    RemoveExitCondition(usize),
    SetNewExitVariant(dsta::BuzzBotDoneNiceStyle),
    SetExitValue(usize, f64),
    SetExitTime(usize, dsta::BotActionTime),
    CancelEditExit,
}

#[derive(Debug, Clone)]
pub struct ExitConditionUI {
    pub variant: dsta::BuzzBotDoneNiceStyle,
    pub is_editing: bool,
}

#[derive(Clone, Debug)]
pub struct MakeBuzzV {
    pub mv_make_buzz_bomber: dsta::MakeBuzzBomber,

    // NEW: Sub-controls for modern view
    pub common_bot_info_control: CommonBotInfoControl,
    pub my_cash_alloc_input: ValidatedInput<f64>,
    pub market_direction_combo: ComboBoxState<dsta::MarketDirection>,
    pub option_expire_input: ValidatedInput<u16>,
    pub target_spread_input: ValidatedInput<f64>,
    pub time_to_order_control: BotActionTimeControl,
    pub algo_cooldown_input: ValidatedInput<f64>,
    pub exit_condition_controls: Vec<ExitConditionControl>,
    pub editing_exit_index: Option<usize>,
}

impl MakeBuzzV {
    pub fn sync_exit_conditions(&mut self) {
        self.mv_make_buzz_bomber.follow_a_exit = self
            .exit_condition_controls
            .iter()
            .map(|ec| ec.variant.clone())
            .collect();
    }

    pub fn new() -> Self {
        let options1 = vec![
            dsta::MarketDirection::GetStonk,
            dsta::MarketDirection::Sideways,
            dsta::MarketDirection::Corrects,
        ];

        let bomber = dsta::MakeBuzzBomber {
            my_accounting: {
                let mut ma = dsta::CommonBotInfo::default();
                ma.friendly_name = format!("Buzz Bomber 1");
                ma.tracking_tick = format!("SPY");
                ma
            },
            my_cash_alloc: 150.0,
            stonk_feeling: dsta::MarketDirection::GetStonk,
            option_expire: 5,
            target_spread: 0.20,
            time_to_order: dsta::BotActionTime {
                relative_days: 1,
                my_entry_time: dsta::UsaMarketTimes::Hurry5sec,
                my_entry_wait: chrono::Duration::seconds(0),
            },
            follow_a_exit: vec![
                dsta::BuzzBotDoneNiceStyle::SpreadValueGain(0.05),
                dsta::BuzzBotDoneNiceStyle::SpreadValueLoss(0.45),
            ],
            algo_cooldown: chrono::Duration::seconds(20),
            bombs_forever: false,
        };

        MakeBuzzV {
            // New controls
            common_bot_info_control: CommonBotInfoControl::new(),
            my_cash_alloc_input: ValidatedInput::new(Box::new(|s| s.trim().parse::<f64>().ok())),
            market_direction_combo: ComboBoxState::new(options1.clone()),
            option_expire_input: ValidatedInput::new(Box::new(|s| s.trim().parse::<u16>().ok())),
            target_spread_input: ValidatedInput::new(Box::new(|s| s.trim().parse::<f64>().ok())),
            time_to_order_control: BotActionTimeControl::new(),
            algo_cooldown_input: ValidatedInput::new(Box::new(|s| s.trim().parse::<f64>().ok())),
            exit_condition_controls: Vec::new(),
            editing_exit_index: None,

            // Main struct
            mv_make_buzz_bomber: bomber,
        }
    }

    pub fn update(&mut self, message: Message) -> iced::Task<Message> {
        self.sync_exit_conditions();

        match message {
            Message::PressedCreateBot => {
                iced::Task::done(Message::ProcessCreateBot(self.mv_make_buzz_bomber.clone()))
            }
            Message::ProcessCreateBot(_) => {
                log::error!("Unreachable - ProcessCreateBot");
                iced::Task::none()
            }
            Message::CommonBotInfoMessage(msg) => {
                self.common_bot_info_control.update(msg);
                self.mv_make_buzz_bomber.my_accounting = self.common_bot_info_control.common_bot_info.clone();
                iced::Task::none()
            }
            Message::MyCashAllocChanged(input) => {
                self.my_cash_alloc_input.update(crate::base::v_valid_any::Message::InputChanged(input));
                if let Some(val) = self.my_cash_alloc_input.parsed_value {
                    self.mv_make_buzz_bomber.my_cash_alloc = val;
                }
                iced::Task::none()
            }
            Message::MarketDirectionChanged(direction) => {
                self.mv_make_buzz_bomber.stonk_feeling = direction;
                iced::Task::none()
            }
            Message::OptionExpireChanged(input) => {
                self.option_expire_input.update(crate::base::v_valid_any::Message::InputChanged(input));
                if let Some(val) = self.option_expire_input.parsed_value {
                    self.mv_make_buzz_bomber.option_expire = val;
                }
                iced::Task::none()
            }
            Message::TargetSpreadChanged(input) => {
                self.target_spread_input.update(crate::base::v_valid_any::Message::InputChanged(input));
                if let Some(val) = self.target_spread_input.parsed_value {
                    self.mv_make_buzz_bomber.target_spread = val;
                }
                iced::Task::none()
            }
            Message::TimeToOrderMessage(msg) => {
                self.time_to_order_control.update(msg);
                self.mv_make_buzz_bomber.time_to_order = self.time_to_order_control.buzz_time_to_order.clone();
                iced::Task::none()
            }
            Message::AlgoCooldownChanged(input) => {
                self.algo_cooldown_input.update(crate::base::v_valid_any::Message::InputChanged(input));
                if let Some(val) = self.algo_cooldown_input.parsed_value {
                    self.mv_make_buzz_bomber.algo_cooldown = chrono::Duration::seconds(val as i64);
                }
                iced::Task::none()
            }
            Message::BombsForeverToggled(value) => {
                self.mv_make_buzz_bomber.bombs_forever = value;
                iced::Task::none()
            }
            Message::ExitConditionMessage(i, msg) => {
                if let Some(control) = self.exit_condition_controls.get_mut(i) {
                    control.update(msg);
                    if i < self.mv_make_buzz_bomber.follow_a_exit.len() {
                        self.mv_make_buzz_bomber.follow_a_exit[i] = control.variant.clone();
                    }
                }
                iced::Task::none()
            }
            Message::FlashError(instant) => {
                self.my_cash_alloc_input.update(crate::base::v_valid_any::Message::FlashError(instant));
                self.option_expire_input.update(crate::base::v_valid_any::Message::FlashError(instant));
                self.target_spread_input.update(crate::base::v_valid_any::Message::FlashError(instant));
                self.algo_cooldown_input.update(crate::base::v_valid_any::Message::FlashError(instant));
                iced::Task::none()
            }
            Message::AddExitCondition => {
                self.exit_condition_controls.push(ExitConditionControl::new());
                iced::Task::none()
            }
            Message::EditExitCondition(index) => {
                self.editing_exit_index = Some(index);
                iced::Task::none()
            }
            Message::RemoveExitCondition(index) => {
                self.exit_condition_controls.remove(index);
                iced::Task::none()
            }
            Message::SetNewExitVariant(variant) => {
                if let Some(control) = self.exit_condition_controls.get_mut(self.editing_exit_index.unwrap_or(0)) {
                    control.update(crate::core::v_buzz_abort_time::Message::VariantSelected(variant));
                }
                iced::Task::none()
            }
            Message::SetExitValue(index, value) => {
                if let Some(control) = self.exit_condition_controls.get_mut(index) {
                    match control.variant {
                        dsta::BuzzBotDoneNiceStyle::SpreadValueGain(ref mut val) => *val = value,
                        dsta::BuzzBotDoneNiceStyle::SpreadValueLoss(ref mut val) => *val = value,
                        _ => {}
                    }
                }
                iced::Task::none()
            }
            Message::SetExitTime(index, time) => {
                if let Some(control) = self.exit_condition_controls.get_mut(index) {
                    if let dsta::BuzzBotDoneNiceStyle::SpreadValueTime(ref mut t) = control.variant {
                        *t = time;
                    }
                }
                iced::Task::none()
            }
            Message::CancelEditExit => {
                self.editing_exit_index = None;
                iced::Task::none()
            }
        }
    }

    pub fn subscription(&self) -> iced::Subscription<Message> {
        iced::Subscription::none()
    }
}
