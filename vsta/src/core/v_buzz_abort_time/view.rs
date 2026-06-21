// trade/vsta/src/core/v_buzz_abort_time/view.rs

use iced::widget::{Column, Row, Text, Button, PickList};
use crate::base::v_valid_any::{ValidatedInput, Message as ValidatedInputMessage};
use crate::base::v_valid_any::view as validated_input_view;
use crate::core::v_bot_action_time::{BotActionTimeControl, view as buzz_time_to_order_view};
use super::state::{BuzzBotDoneNiceStyleControl, Message};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ExitVariantType {
    Gain,
    Loss,
    Time,
}

impl std::fmt::Display for ExitVariantType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ExitVariantType::Gain => write!(f, "Spread Value Gain"),
            ExitVariantType::Loss => write!(f, "Spread Value Loss"),
            ExitVariantType::Time => write!(f, "Spread Value Time"),
        }
    }
}

pub fn view<'a, MessageType>(
    control: &'a BuzzBotDoneNiceStyleControl,
    map_message: impl Fn(Message) -> MessageType + 'a + Clone + 'static,
) -> Column<'a, MessageType>
where
    MessageType: 'a + Clone + 'static,
{
    let variant_type = match control.variant {
        dsta::BuzzBotDoneNiceStyle::SpreadValueGain(_) => ExitVariantType::Gain,
        dsta::BuzzBotDoneNiceStyle::SpreadValueLoss(_) => ExitVariantType::Loss,
        dsta::BuzzBotDoneNiceStyle::SpreadValueTime(_) => ExitVariantType::Time,
    };

    let variant_options = vec![
        ExitVariantType::Gain,
        ExitVariantType::Loss,
        ExitVariantType::Time,
    ];

    // Variant selector
    let variant_picker = PickList::new(
        variant_options,
        Some(variant_type),
        {
            let map_message = map_message.clone();
            move |selected| {
                let new_variant = match selected {
                    ExitVariantType::Gain => dsta::BuzzBotDoneNiceStyle::SpreadValueGain(0.05),
                    ExitVariantType::Loss => dsta::BuzzBotDoneNiceStyle::SpreadValueLoss(0.45),
                    ExitVariantType::Time => dsta::BuzzBotDoneNiceStyle::SpreadValueTime(
                        BotActionTimeControl::new().buzz_time_to_order
                    ),
                };
                map_message(Message::VariantSelected(new_variant))
            }
        },
    )
    .placeholder("Select exit type");

    // Content based on variant
    let content = match &control.variant {
        dsta::BuzzBotDoneNiceStyle::SpreadValueGain(current_value) => {
            let spread_value_gain_input = validated_input_view(
                &control.spread_value_gain_input,
                "Spread value gain (0.01 - 0.99)",
                {
                    let map_message = map_message.clone();
                    move |msg| match msg {
                        ValidatedInputMessage::InputChanged(input) => {
                            // ← JUST pass the string through, let control.update() handle parsing
                            map_message(Message::SpreadValueGainChanged(input))
                        }
                        ValidatedInputMessage::FlashError(elapsed) => map_message(Message::FlashError(elapsed)),
                        _ => map_message(Message::SpreadValueGainChanged(String::new())),
                    }
                },
            );


            Column::new()
                .push(Text::new("Spread Value Gain (in dollars)").size(14))
                .push(Text::new(format!("Current: ${:.2}", current_value)).size(12))
                .push(spread_value_gain_input)
                .spacing(8)
        }
        dsta::BuzzBotDoneNiceStyle::SpreadValueLoss(current_value) => {
            let spread_value_loss_input = validated_input_view(
                &control.spread_value_loss_input,
                "Spread value loss (0.01 - 2.00)",
                {
                    let map_message = map_message.clone();
                    move |msg| match msg {
                        ValidatedInputMessage::InputChanged(input) => {
                            map_message(Message::SpreadValueLossChanged(input))
                        }
                        ValidatedInputMessage::FlashError(elapsed) => map_message(Message::FlashError(elapsed)),
                        _ => map_message(Message::SpreadValueLossChanged(String::new())),
                    }
                },
            );
            Column::new()
                .push(Text::new("Spread Value Loss (in dollars)").size(14))
                .push(Text::new(format!("Current: ${:.2}", current_value)).size(12))
                .push(spread_value_loss_input)
                .spacing(8)
        }
        dsta::BuzzBotDoneNiceStyle::SpreadValueTime(_) => {
            let spread_value_time_control = buzz_time_to_order_view(
                &control.spread_value_time_control,
                {
                    let map_message = map_message.clone();
                    move |msg| map_message(Message::SpreadValueTimeMessage(msg))
                },
            );

            Column::new()
                .push(Text::new("Spread Value Time").size(14))
                .push(spread_value_time_control)
                .spacing(8)
        }
    };

    let toggle_button = Button::new(
        Text::new(if control.is_visible { "Hide Details" } else { "Show Details" })
    )
    .on_press(map_message(Message::ToggleVisibility));

    Column::new()
        .push(
            Row::new()
                .push(Text::new("Exit Type:").size(16))
                .push(variant_picker)
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