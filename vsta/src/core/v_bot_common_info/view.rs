
// trade/vsta/src/core/v_bot_common_info/view.rs


use iced::widget::{Column, Row, Text, TextInput, PickList, Toggler, Button, Container};
use crate::base::v_valid_any::{ValidatedInput, Message as ValidatedInputMessage};
use crate::base::v_valid_any::view as validated_input_view;
use crate::core::v_haggle_action::view as haggle_action_view;
use super::state::{CommonBotInfoControl, Message};
use dsta::HaggleAction;

pub fn view<'a, MessageType>(
    control: &'a CommonBotInfoControl,
    map_message: impl Fn(Message) -> MessageType + 'a + Clone + 'static,
) -> Column<'a, MessageType>
where
    MessageType: 'a + Clone + 'static,
{
    // Friendly Name input
    let friendly_name_input = TextInput::new(
        "Friendly Name",
        &control.friendly_name_input,
    )
    .on_input({
        let map_message = map_message.clone();
        move |name| map_message(Message::FriendlyNameChanged(name))
    });

    // Tracking Tick input
    let tracking_tick_input = TextInput::new(
        "Tracking Tick",
        &control.tracking_tick_input,
    )
    .on_input({
        let map_message = map_message.clone();
        move |tick| map_message(Message::TrackingTickChanged(tick))
    });

    // Haggle Action control
    let haggle_action_view_elem = haggle_action_view(
        &control.haggle_action_control,
        {
            let map_message = map_message.clone();
            move |msg| map_message(Message::HaggleActionMessage(msg))
        },
    );

    // Max Cash Risk toggle and inputs
    let max_cash_risk_toggle = Toggler::new(control.max_cash_risk_bool)
        .label("Use Max Cash Risk")
        .on_toggle({
            let map_message = map_message.clone();
            move |use_max| map_message(Message::MaxCashRiskBoolChanged(use_max))
        });

    let max_cash_risk_percent_input = validated_input_view(
        &control.max_cash_risk_percent_input,
        "Max Cash Risk %",
        {
            let map_message = map_message.clone();
            move |msg| match msg {
                ValidatedInputMessage::InputChanged(input) => map_message(Message::MaxCashRiskPercentChanged(input)),
                ValidatedInputMessage::FlashError(elapsed) => map_message(Message::FlashError(elapsed)),
                _ => map_message(Message::MaxCashRiskPercentChanged("".to_string())),
            }
        },
    );

    let max_cash_risk_dollar_input = validated_input_view(
        &control.max_cash_risk_dollar_input,
        "Max Cash Risk $",
        {
            let map_message = map_message.clone();
            move |msg| match msg {
                ValidatedInputMessage::InputChanged(input) => map_message(Message::MaxCashRiskDollarChanged(input)),
                ValidatedInputMessage::FlashError(elapsed) => map_message(Message::FlashError(elapsed)),
                _ => map_message(Message::MaxCashRiskDollarChanged("".to_string())),
            }
        },
    );

    Column::new()
        .push(Text::new("Common Bot Info").size(20))
        .push(iced::widget::row![Text::new("Friendly Name:"), friendly_name_input])
        .push(iced::widget::row![Text::new("Tracking Tick:"), tracking_tick_input])
        .push(haggle_action_view_elem)
        .push(max_cash_risk_toggle)
        .push(max_cash_risk_percent_input)
        .push(max_cash_risk_dollar_input)
        .spacing(10)
}