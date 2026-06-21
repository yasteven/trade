
// trade/vsta/src/base/v_time_span/view.rs

use iced::widget::{PickList, Row};
use crate::base::v_valid_any::{ValidatedInput, Message as ValidatedInputMessage};
use crate::base::v_valid_any::view as validated_input_view;

use super::state::{TimeSpanControl, Message, TimeSpanUnit};

pub fn view<'a, MessageType>(
    time_span_control: &'a  TimeSpanControl,
    map_message: impl Fn(Message) -> MessageType + 'a + Clone,
) -> Row<'a, MessageType>
where
    MessageType: 'a + Clone,
{
    let units = vec![
        TimeSpanUnit::Milliseconds,
        TimeSpanUnit::Seconds,
        TimeSpanUnit::Minutes,
        TimeSpanUnit::Hours,
        TimeSpanUnit::Days,
    ];

    let value_input = validated_input_view(
        &time_span_control.value_input,
        "Enter time span value",
        {
            let map_message = map_message.clone();
            move |msg| match msg {
                ValidatedInputMessage::InputChanged(input) => map_message(Message::ValueChanged(input)),
                ValidatedInputMessage::FlashError(elapsed) => map_message(Message::FlashError(elapsed)),
                _ => map_message(Message::ValueChanged("".to_string())),
            }
        },
    );

    let unit_picker = PickList::new(
        units,
        Some(time_span_control.unit_selector.clone()),
        move |unit| map_message(Message::UnitChanged(unit)),
    )
    .placeholder("Unit");

    Row::new()
        .push(value_input)
        .push(unit_picker)
        .spacing(10)
}