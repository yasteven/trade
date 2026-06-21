
// trade/vsta/src/core/v_haggle_method/view.rs

use iced::widget::{Column, Row, Text, PickList};
use super::state::{HaggleMethodControl, HaggleMethodType, Message};
use crate::core::v_haggle_limits::view as haggle_limits_view;

pub fn view<'a, MessageType>(
    control: &'a HaggleMethodControl,
    map_message: impl Fn(Message) -> MessageType + 'a + Clone + 'static,
) -> Column<'a, MessageType>
where
    MessageType: 'a + Clone + 'static,
{
    let method_options = vec![
        HaggleMethodType::MinimumOfMidpointOrTheoreticalThenConcede,
        HaggleMethodType::VirtualMarketOrderWithLimitOrdThenConcede,
        HaggleMethodType::CheapLimitOrderThenIncrementDeltaUntilMax,
    ];

    let method_picker = PickList::new(
        method_options,
        Some(control.method_type),
        {
            let map_message = map_message.clone();
            move |selected| map_message(Message::MethodVariantSelected(selected))
        },
    )
    .placeholder("Select haggle method");

    let limits_view = haggle_limits_view(
        &control.limits_control,
        {
            let map_message = map_message.clone();
            move |msg| map_message(Message::LimitsMessage(msg))
        },
    );

    Column::new()
        .push(Text::new("Haggle Method").size(18))
        .push(
            Row::new()
                .push(Text::new("Method:").size(16))
                .push(method_picker)
                .spacing(10)
                .align_y(iced::Alignment::Center)
        )
        .push(limits_view)
        .spacing(10)
        .padding(10)
}
