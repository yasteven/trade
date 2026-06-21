
// trade/vsta/src/core/v_haggle_limits/view.rs

use iced::widget::{Column, Row, Text, Toggler};
use crate::base::v_valid_any::{ValidatedInput, Message as ValidatedInputMessage};
use crate::base::v_valid_any::view as validated_input_view;
use crate::base::v_time_span::view as time_span_view;
use super::state::{HaggleLimitsControl, Message};

pub fn view<'a, MessageType>(
    control: &'a HaggleLimitsControl,
    map_message: impl Fn(Message) -> MessageType + 'a + Clone + 'static,
) -> Column<'a, MessageType>
where
    MessageType: 'a + Clone + 'static,
{
    let retry_period_view = time_span_view(
        &control.retry_period_control,
        {
            let map_message = map_message.clone();
            move |msg| map_message(Message::RetryPeriodMessage(msg))
        },
    );

    let delta_choice_input = validated_input_view(
        &control.delta_choice_input,
        "Delta choice (% or $)",
        {
            let map_message = map_message.clone();
            move |msg| match msg {
                ValidatedInputMessage::InputChanged(input) => {
                    map_message(Message::DeltaChoiceChanged(input))
                }
                ValidatedInputMessage::FlashError(elapsed) => map_message(Message::FlashError(elapsed)),
                _ => map_message(Message::DeltaChoiceChanged(String::new())),
            }
        },
    );

    let delta_is_pct_toggle = Toggler::new(control.delta_is_pct)
        .label("Delta as percentage?")
        .on_toggle({
            let map_message = map_message.clone();
            move |is_pct| map_message(Message::DeltaIsPctToggled(is_pct))
        });

    let slippage_max_input = validated_input_view(
        &control.slippage_max_input,
        "Max slippage",
        {
            let map_message = map_message.clone();
            move |msg| match msg {
                ValidatedInputMessage::InputChanged(input) => {
                    map_message(Message::SlippageMaxChanged(input))
                }
                ValidatedInputMessage::FlashError(elapsed) => map_message(Message::FlashError(elapsed)),
                _ => map_message(Message::SlippageMaxChanged(String::new())),
            }
        },
    );

    Column::new()
        .push(Text::new("Haggle Limits").size(18))
        .push(
            Row::new()
                .push(Text::new("Retry Period:"))
                .push(retry_period_view)
                .spacing(10)
        )
        .push(
            Row::new()
                .push(Text::new("Delta Choice:"))
                .push(delta_choice_input)
                .spacing(10)
        )
        .push(delta_is_pct_toggle)
        .push(
            Row::new()
                .push(Text::new("Max Slippage:"))
                .push(slippage_max_input)
                .spacing(10)
        )
        .spacing(10)
        .padding(10)
}
