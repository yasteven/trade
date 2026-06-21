// trade/vsta/src/form/v_swat_make/view.rs - SIMPLIFIED

use crate::form::v_swat_make::{MakeSwatV, Message, ControlTab};
use crate::core::v_bot_common_info::view as common_bot_info_view;
use iced::widget::{Button, Column, Row, Text, Scrollable, Space};
use iced::{Element, Length, Alignment};

/// View function with proper lifetimes
pub fn view<'a>(
    dr_r: &'a MakeSwatV,
    emerald_control: &'a crate::core::v_emerald_collection::EmeraldCollectionControl,
) -> Row<'a, crate::Message> {
    log::debug!("v_swat_make::view BEGIN");

    // ============================================================
    // COLUMN 1: SETTINGS (left side)
    // ============================================================
    let common_bot_info = common_bot_info_view(
        &dr_r.common_bot_info_control,
        |msg| crate::Message::DrR(
            crate::lens::v_dr_r::Message::FromMakeSwat(
                Message::CommonBotInfoMessage(msg)
            )
        ),
    );

    let v_create_button = Button::new(Text::new("✓ Create Swat Bots"))
        .on_press(crate::Message::DrR(crate::lens::v_dr_r::Message::FromMakeSwat(
            Message::PressedCreateBot
        )))
        .width(Length::Fill)
        .padding(10);

    let settings_column = Column::new()
        .push(Space::new().height(Length::Fixed(1.0)))
        .push(Text::new("Create Swat Bots").size(24))
        .push(Space::new().height(Length::Fixed(10.0)))
        .push(common_bot_info)
        .push(Space::new().height(Length::Fixed(10.0)))
        .push(v_create_button)
        .spacing(5)
        .padding(2)
        .width(Length::Fixed(330.0));

    let settings_scroll = Scrollable::new(settings_column)
        .height(Length::Fill);

    // ============================================================
    // COLUMN 2: TAB SELECTOR (middle)
    // ============================================================
    let assets_tab_button = Button::new(
        iced::widget::container(Text::new("📊\nAssets"))
            .center_y(Length::Fill)
            .center_x(Length::Fill)
            .width(Length::Shrink)
    )
    .height(Length::Fixed(100.0))
    .width(Length::Fixed(100.0))
    .padding(2)
    .on_press(crate::Message::DrR(
        crate::lens::v_dr_r::Message::FromMakeSwat(
            Message::SwitchTab(ControlTab::Assets)
        )
    ));

    let settings_tab_button = Button::new(Text::new("⚙️\nSettings"))
        .on_press(crate::Message::DrR(crate::lens::v_dr_r::Message::FromMakeSwat(
            Message::SwitchTab(ControlTab::SwatSettings)
        )))
        .width(Length::Fill)
        .padding(12);

    let tab_buttons = Column::new()
        .push(Text::new("Options").size(14))
        .push(Space::new().height(Length::Fixed(8.0)))
        .push(assets_tab_button)
        .push(settings_tab_button)
        .spacing(5)
        .padding(10)
        .width(Length::Fixed(120.0));

    // ============================================================
    // COLUMN 3: CONTROL VIEW (right side)
    // ============================================================
    let control_view: Element<'a, crate::Message> = match dr_r.current_control_tab {
        ControlTab::Assets => {
            // Show emerald collection
            crate::core::v_emerald_collection::view(&emerald_control)
        }
        ControlTab::SwatSettings => {
            // Show presets
            crate::core::v_swat_preset::view(
                &dr_r.swat_presets,
                |preset| crate::Message::DrR(
                    crate::lens::v_dr_r::Message::FromMakeSwat(
                        Message::SwatPresetSelected(preset)
                    )
                ),
            )
        }
    };

    // With this:
    let control_area: Element<'a, crate::Message> = match dr_r.current_control_tab {
        ControlTab::Assets => {
            // Emerald has its own internal scrolling — no outer Scrollable
            iced::widget::container(control_view)
                .width(Length::Fill)
                .height(Length::Fill)
                .into()
        }
        _ => {
            Scrollable::new(control_view)
                .height(Length::Fill)
                .width(Length::Fill)
                .into()
        }
    };

    // ============================================================
    // FINAL LAYOUT: settings | tabs | controls
    // ============================================================
    iced::widget::row![
        iced::widget::container(settings_scroll)
            .width(Length::Fixed(360.0))
            .height(Length::Fill),
        iced::widget::container(tab_buttons)
            .width(Length::Fixed(121.0))
            .height(Length::Fill),
        iced::widget::container(control_area)
            .width(Length::Fill)
            .height(Length::Fill)
            .padding(20)
            .style(|_theme| iced::widget::container::Style {
                background: Some(iced::Background::Color(
                    iced::Color::from_rgba(0.05, 0.19, 0.49, 0.2)
                )),
                ..Default::default()
            })
    ]
    .width(Length::Fill)
    .height(Length::Fill)
    .spacing(20)
    .padding(20)
}