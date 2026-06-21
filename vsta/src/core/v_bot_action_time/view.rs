
// trade/vsta/src/core/v_bot_action_time/view.rs (UPDATED)

use iced::widget::{Column, Row, Text};
use crate::base::v_valid_any::{ValidatedInput, Message as ValidatedInputMessage};
use crate::base::v_valid_any::view as validated_input_view;
use crate::base::v_usa_times::view as usa_market_times_view;
use crate::base::v_time_span::view as time_span_view;
use super::state::{BotActionTimeControl, Message};

pub fn view<'a, MessageType>(
    control: &'a BotActionTimeControl,
    map_message: impl Fn(Message) -> MessageType + 'a + Clone + 'static,
) -> Column<'a, MessageType>
where
    MessageType: 'a + Clone + 'static,
{
    let relative_days_input = validated_input_view(
        &control.relative_days_input,
        "Entry day number (0-6)",
        {
            let map_message = map_message.clone();
            move |msg| match msg {
                ValidatedInputMessage::InputChanged(input) => {
                    map_message(Message::EntryDayNumChanged(input))
                }
                ValidatedInputMessage::FlashError(elapsed) => map_message(Message::FlashError(elapsed)),
                _ => map_message(Message::EntryDayNumChanged("".to_string())),
            }
        },
    );

    let my_entry_time_combobox = usa_market_times_view(
        &control.my_entry_time_combobox,
        {
            let map_message = map_message.clone();
            move |time| map_message(Message::MyEntryTimeChanged(time))
        },
    );

    let my_entry_wait_view = time_span_view(
        &control.my_entry_wait_control,
        {
            let map_message = map_message.clone();
            move |msg| map_message(Message::MyEntryWaitMessage(msg))
        },
    );

    Column::new()
        .push(Text::new("Bot Action Time").size(20))
        .push(
            Row::new()
                .push(Text::new("Entry Day Number:"))
                .push(relative_days_input)
                .spacing(10)
        )
        .push(
            Row::new()
                .push(Text::new("Entry Time:"))
                .push(my_entry_time_combobox)
                .spacing(10)
        )
        .push(
            Row::new()
                .push(Text::new("Entry Wait:"))
                .push(my_entry_wait_view)
                .spacing(10)
        )
        .spacing(10)
}
