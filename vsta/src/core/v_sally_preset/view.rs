
// trade/vsta/src/core/v_sally_preset/view.rs

use super::{SallyPresets, SallyPresetOption, Message};
use iced::widget::{Button, Column, Text};
use iced::{Element, Length, Alignment};

pub fn view<'a, F>(
    presets: &'a SallyPresets,
    on_select: F,
) -> Element<'a, crate::Message>
where
    F: Fn(SallyPresetOption) -> crate::Message + 'a,
{
    let preset_buttons: Vec<Element<'a, crate::Message>> = SallyPresetOption::all()
        .iter()
        .map(|&preset| {
            let is_selected = presets.selected == Some(preset);
            let button = Button::new(
                Column::new()
                    .push(Text::new(preset.display_name()).size(12))
                    .push(Text::new(preset.btc_quantity_display()).size(9))
                    .align_x(Alignment::Center)
                    .spacing(2)
            )
            .on_press(on_select(preset))
            .padding(8)
            .width(Length::Fill);


            button.into()
        })
        .collect();

    Column::new()
        .push(Text::new("₿ BTC Size").size(16).width(Length::Fill))
        .push(iced::widget::Space::new().height(Length::Fixed(8.0)))
        .extend(preset_buttons)
        .spacing(6)
        .padding(10)
        .width(Length::Fill)
        .into()
}