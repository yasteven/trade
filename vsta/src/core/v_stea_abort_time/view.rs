
// trade/vsta/src/core/v_stealth_bot_done_nice_style/view.rs

use iced::widget::{Column, Row, Text, Button, PickList};
use crate::base::v_valid_any::{ValidatedInput, Message as ValidatedInputMessage};
use crate::base::v_valid_any::view as validated_input_view;
use crate::core::v_bot_action_time::{BotActionTimeControl, view as buzz_time_to_order_view};
use super::state::{StealthBotDoneNiceStyleControl, Message};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ExitStrategyType {
    OtmShort,
    FinalPct,
    PriorDay,
    GoExpire,
}

impl std::fmt::Display for ExitStrategyType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ExitStrategyType::OtmShort => write!(f, "OTM Short"),
            ExitStrategyType::FinalPct => write!(f, "Final Percent"),
            ExitStrategyType::PriorDay => write!(f, "Prior Day"),
            ExitStrategyType::GoExpire => write!(f, "Let Expire"),
        }
    }
}

pub fn view<'a, MessageType>(
    control: &'a StealthBotDoneNiceStyleControl,
    map_message: impl Fn(Message) -> MessageType + 'a + Clone + 'static,
) -> Column<'a, MessageType>
where
    MessageType: 'a + Clone + 'static,
{
    log::debug!("stealth_bot_done_nice_style view BEGIN!");

    let strategy_type = match control.variant {
        dsta::StealthBotDoneNiceStyle::OtmShort => ExitStrategyType::OtmShort,
        dsta::StealthBotDoneNiceStyle::FinalPct(_) => ExitStrategyType::FinalPct,
        dsta::StealthBotDoneNiceStyle::PriorDay(_) => ExitStrategyType::PriorDay,
        dsta::StealthBotDoneNiceStyle::GoExpire => ExitStrategyType::GoExpire,
    };

    let strategy_options = vec![
        ExitStrategyType::OtmShort,
        ExitStrategyType::FinalPct,
        ExitStrategyType::PriorDay,
        ExitStrategyType::GoExpire,
    ];

    // Strategy selector
    let strategy_picker = PickList::new(
        strategy_options,
        Some(strategy_type),
        {
            let map_message = map_message.clone();
            move |selected| {
                let new_variant = match selected {
                    ExitStrategyType::OtmShort => dsta::StealthBotDoneNiceStyle::OtmShort,
                    ExitStrategyType::FinalPct => dsta::StealthBotDoneNiceStyle::FinalPct(50.0),
                    ExitStrategyType::PriorDay => dsta::StealthBotDoneNiceStyle::PriorDay(
                        BotActionTimeControl::new().buzz_time_to_order
                    ),
                    ExitStrategyType::GoExpire => dsta::StealthBotDoneNiceStyle::GoExpire,
                };
                map_message(Message::VariantSelected(new_variant))
            }
        },
    )
    .placeholder("Select exit strategy");

    // Content based on variant
    let content = match &control.variant {
        dsta::StealthBotDoneNiceStyle::OtmShort => {
            log::debug!("stealth_bot_done_nice_style view rendering OtmShort");
            Column::new()
                .push(Text::new("Short leg goes Out-of-The-Money").size(14))
                .spacing(8)
        }
        dsta::StealthBotDoneNiceStyle::FinalPct(current_value) => {
            log::debug!("stealth_bot_done_nice_style view rendering FinalPct");
            let final_pct_input = validated_input_view(
                &control.final_pct_input,
                "Exit at percent gain (0-200%)",
                {
                    let map_message = map_message.clone();
                    move |msg| match msg {
                        ValidatedInputMessage::InputChanged(input) => {
                            map_message(Message::FinalPctChanged(input))
                        }
                        ValidatedInputMessage::FlashError(elapsed) => map_message(Message::FlashError(elapsed)),
                        _ => map_message(Message::FinalPctChanged(String::new())),
                    }
                },
            );

            Column::new()
                .push(Text::new("Exit at Final Percent of Credit").size(14))
                .push(Text::new(format!("Current: {}%", current_value)).size(12))
                .push(final_pct_input)
                .spacing(8)
        }
        dsta::StealthBotDoneNiceStyle::PriorDay(_) => {
            log::debug!("stealth_bot_done_nice_style view rendering PriorDay");
            let prior_day_control = buzz_time_to_order_view(
                &control.prior_day_control,
                {
                    let map_message = map_message.clone();
                    move |msg| {
                        map_message(Message::PriorDayMessage(msg))
                    }
                },
            );

            Column::new()
                .push(Text::new("Exit Prior To Expiration").size(14))
                .push(prior_day_control)
                .spacing(8)
        }
        dsta::StealthBotDoneNiceStyle::GoExpire => {
            log::debug!("stealth_bot_done_nice_style view rendering GoExpire");
            Column::new()
                .push(Text::new("Let contract expire (max loss)").size(14))
                .spacing(8)
        }
    };

    let toggle_button = Button::new(
        Text::new(if control.is_visible { "Hide Details" } else { "Show Details" })
    )
    .on_press(map_message(Message::ToggleVisibility));

    log::trace!("stealth_bot_done_nice_style view END!");

    Column::new()
        .push(
            Row::new()
                .push(Text::new("Exit Strategy:").size(16))
                .push(strategy_picker)
                .spacing(10)
                .align_y(iced::Alignment::Center)
        )
        .push(toggle_button)
        .push(
            if control.is_visible {
                content
            } else {
                Column::new()
            }
        )
        .spacing(10)
        .padding(10)
}