
// trade/vsta/src/form/v_stealth_make/state.rs

use crate::base::v_valid_any::ValidatedInput;
use crate::base::v_combo_box::ComboBoxState;
use crate::core::v_bot_common_info::CommonBotInfoControl;
use crate::core::v_stea_abort_time::ExitStrategyControl;

#[derive(Debug, Clone)]
pub enum Message {
    PressedCreateBot,
    ProcessCreateBot(dsta::MakeStealthBot),

    // New messages for sub-controls
    CommonBotInfoMessage(crate::core::v_bot_common_info::Message),
    MyCashAllocChanged(String),
    MarketDirectionChanged(dsta::MarketDirection),
    OptionExpireChanged(String),
    OptionBucketChanged(String),
    SpreadBucketChanged(String),
    ExitGainPctChanged(String),
    ExitLossPctChanged(String),
    ExitStrategyMessage(usize, crate::core::v_stea_abort_time::Message),
    FlashError(iced::time::Instant),

    AddExitStrategy,
    RemoveExitStrategy(usize),
    UseTheoChanged(bool),
}

#[derive(Debug, Clone)]
pub struct MakeStealthV {
    pub mv_make_stealth_bot: dsta::MakeStealthBot,

    // NEW: Sub-controls for modern view
    pub common_bot_info_control: CommonBotInfoControl,
    pub my_cash_alloc_input: ValidatedInput<f64>,
    pub market_direction_combo: ComboBoxState<dsta::MarketDirection>,
    pub option_expire_input: ValidatedInput<u16>,
    pub option_bucket_input: ValidatedInput<u8>,
    pub spread_bucket_input: ValidatedInput<u8>,
    pub exit_gain_pct_input: ValidatedInput<f64>,
    pub exit_loss_pct_input: ValidatedInput<f64>,
    pub exit_strategy_controls: Vec<ExitStrategyControl>,
}

impl MakeStealthV {
    pub fn sync_exit_strategies(&mut self) {
        self.mv_make_stealth_bot.nice_exit_way = self
            .exit_strategy_controls
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

        let stealth_bot = dsta::MakeStealthBot {
            my_accounting: {
                let mut ma = dsta::CommonBotInfo::default();
                ma.friendly_name = format!("Stealth Bot 1");
                ma.tracking_tick = format!("SPY");
                ma
            },
            my_cash_alloc: 150.0,
            stonk_feeling: dsta::MarketDirection::GetStonk,
            option_expire: 5,
            option_bucket: 0,
            spread_bucket: 1,
            exit_gain_pct: 50.0,
            exit_loss_pct: -50.0,
            nice_exit_way: vec![dsta::StealthBotDoneNiceStyle::OtmShort],
            use_theo_cost: false,
        };

        MakeStealthV {
            // New controls
            common_bot_info_control: CommonBotInfoControl::new(),
            my_cash_alloc_input: ValidatedInput::new(Box::new(|s| s.trim().parse::<f64>().ok())),
            market_direction_combo: ComboBoxState::new(options1.clone()),
            option_expire_input: ValidatedInput::new(Box::new(|s| s.trim().parse::<u16>().ok())),
            option_bucket_input: ValidatedInput::new(Box::new(|s| s.trim().parse::<u8>().ok())),
            spread_bucket_input: ValidatedInput::new(Box::new(|s| s.trim().parse::<u8>().ok())),
            exit_gain_pct_input: ValidatedInput::new(Box::new(|s| s.trim().parse::<f64>().ok())),
            exit_loss_pct_input: ValidatedInput::new(Box::new(|s| s.trim().parse::<f64>().ok())),
            exit_strategy_controls: vec![ExitStrategyControl::new()],

            // Main struct
            mv_make_stealth_bot: stealth_bot,
        }
    }

    pub fn update(&mut self, message: Message) -> iced::Task<Message> {
        self.sync_exit_strategies();

        match message {
            Message::PressedCreateBot => {
                iced::Task::done(Message::ProcessCreateBot(self.mv_make_stealth_bot.clone()))
            }
            Message::ProcessCreateBot(_) => {
                log::error!("Unreachable - ProcessCreateBot");
                iced::Task::none()
            }
            Message::CommonBotInfoMessage(msg) => {
                self.common_bot_info_control.update(msg);
                self.mv_make_stealth_bot.my_accounting = self.common_bot_info_control.common_bot_info.clone();
                iced::Task::none()
            }
            Message::MyCashAllocChanged(input) => {
                self.my_cash_alloc_input.update(crate::base::v_valid_any::Message::InputChanged(input));
                if let Some(val) = self.my_cash_alloc_input.parsed_value {
                    self.mv_make_stealth_bot.my_cash_alloc = val;
                }
                iced::Task::none()
            }
            Message::MarketDirectionChanged(direction) => {
                self.mv_make_stealth_bot.stonk_feeling = direction;
                iced::Task::none()
            }
            Message::OptionExpireChanged(input) => {
                self.option_expire_input.update(crate::base::v_valid_any::Message::InputChanged(input));
                if let Some(val) = self.option_expire_input.parsed_value {
                    self.mv_make_stealth_bot.option_expire = val;
                }
                iced::Task::none()
            }
            Message::OptionBucketChanged(input) => {
                self.option_bucket_input.update(crate::base::v_valid_any::Message::InputChanged(input));
                if let Some(val) = self.option_bucket_input.parsed_value {
                    self.mv_make_stealth_bot.option_bucket = val;
                }
                iced::Task::none()
            }
            Message::SpreadBucketChanged(input) => {
                self.spread_bucket_input.update(crate::base::v_valid_any::Message::InputChanged(input));
                if let Some(val) = self.spread_bucket_input.parsed_value {
                    self.mv_make_stealth_bot.spread_bucket = val;
                }
                iced::Task::none()
            }
            Message::ExitGainPctChanged(input) => {
                self.exit_gain_pct_input.update(crate::base::v_valid_any::Message::InputChanged(input));
                if let Some(val) = self.exit_gain_pct_input.parsed_value {
                    self.mv_make_stealth_bot.exit_gain_pct = val;
                }
                iced::Task::none()
            }
            Message::ExitLossPctChanged(input) => {
                self.exit_loss_pct_input.update(crate::base::v_valid_any::Message::InputChanged(input));
                if let Some(val) = self.exit_loss_pct_input.parsed_value {
                    self.mv_make_stealth_bot.exit_loss_pct = val;
                }
                iced::Task::none()
            }
            Message::ExitStrategyMessage(i, msg) => {
                if let Some(control) = self.exit_strategy_controls.get_mut(i) {
                    control.update(msg);
                    if i < self.mv_make_stealth_bot.nice_exit_way.len() {
                        self.mv_make_stealth_bot.nice_exit_way[i] = control.variant.clone();
                    }
                }
                iced::Task::none()
            }
            Message::FlashError(instant) => {
                self.my_cash_alloc_input.update(crate::base::v_valid_any::Message::FlashError(instant));
                self.option_expire_input.update(crate::base::v_valid_any::Message::FlashError(instant));
                self.option_bucket_input.update(crate::base::v_valid_any::Message::FlashError(instant));
                self.spread_bucket_input.update(crate::base::v_valid_any::Message::FlashError(instant));
                self.exit_gain_pct_input.update(crate::base::v_valid_any::Message::FlashError(instant));
                self.exit_loss_pct_input.update(crate::base::v_valid_any::Message::FlashError(instant));
                iced::Task::none()
            }
            Message::AddExitStrategy => {
                self.exit_strategy_controls.push(ExitStrategyControl::new());
                iced::Task::none()
            }
            Message::RemoveExitStrategy(index) => {
                self.exit_strategy_controls.remove(index);
                iced::Task::none()
            }
            Message::UseTheoChanged(use_theo) => {
                self.mv_make_stealth_bot.use_theo_cost = use_theo;
                iced::Task::none()
            }
        }
    }

    pub fn subscription(&self) -> iced::Subscription<Message> {
        iced::Subscription::none()
    }
}

