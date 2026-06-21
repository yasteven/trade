
// trade/vsta/src/base/v_usa_times/view.rs

use super::state::UsaMarketTimesComboBox;
use super::super::v_combo_box::view as combobox_view;
use dsta::UsaMarketTimes;

pub fn view<'a, MessageType: Clone + 'a + 'static>
//pub fn view<'a, MessageType>
(
    combobox: &'a UsaMarketTimesComboBox,
    map_message: impl Fn(UsaMarketTimes) -> MessageType + 'a + 'static,
) -> iced::widget::ComboBox<'a, UsaMarketTimes, MessageType>
where
    MessageType: 'a,
{
    combobox_view(
        &combobox.combobox,
        "Select a market time",
        map_message,
    )
}

