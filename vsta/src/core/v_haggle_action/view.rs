
// trade/vsta/src/core/v_haggle_action/view.rs

use iced::widget::{Column, Row, Text, PickList};
use super::state::{HaggleActionControl, HaggleActionType, Message};
use crate::core::v_haggle_method::view as haggle_method_view;

pub fn view<'a, MessageType>(
    control: &'a HaggleActionControl,
    map_message: impl Fn(Message) -> MessageType + 'a + Clone + 'static,
) -> Column<'a, MessageType>
where
    MessageType: 'a + Clone + 'static,
{
    let action_options = vec![
        HaggleActionType::JustCancel,
        HaggleActionType::AnOyVeyJew,
    ];

    let action_picker = PickList::new(
        action_options,
        Some(control.action_variant),
        {
            let map_message = map_message.clone();
            move |selected| map_message(Message::ActionVariantSelected(selected))
        },
    )
    .placeholder("Select haggle action");

    let method_view = haggle_method_view(
        &control.method_control,
        {
            let map_message = map_message.clone();
            move |msg| map_message(Message::MethodMessage(msg))
        },
    );

    Column::new()
        .push(
            Row::new()
                .push(Text::new("Haggle Action:").size(16))
                .push(action_picker)
                .spacing(10)
                .align_y(iced::Alignment::Center)
        )
        .push(method_view)
        .spacing(10)
        .padding(10)
}
