// trade/vsta/src/core/v_emerald_collection/view.rs
//
// CHANGED vs old version:
//   - view_option_chain_panel: uses control.current_push_result() and passes it
//     into v_option_chain::view() as `data` param instead of calling
//     option_chain_view(&control.option_chain, on_oc_msg) (old 2-arg signature).
//   - view_option_chain_panel: no longer needs control.current_ticker() to guard
//     the panel — current_push_result() being Some/None is the single gate.
//   - Everything else (type selector, color selector, button factories) is unchanged.

use crate::dat::{EmeraldColor, EmeraldTypes};
use super::Message;
use crate::core::v_option_chain;
use iced::{
    widget::{button, column, container, image, row, scrollable, text, Column, Space},
    Alignment, Element, Length,
};

// ============================================================
// Public entry point  (unchanged)
// ============================================================

pub fn view(
    control: &crate::core::v_emerald_collection::EmeraldCollectionControl,
) -> Element<'static, crate::Message> {
    log::debug!("v_emerald_collection::view called");
    let type_col  = view_type_selector(control);
    let chain_col = view_option_chain_panel(control);

    let is_master = matches!(control.selected_type, Some(EmeraldTypes::MasterEmerald));
    if is_master {
        row![type_col, chain_col]
            .width(Length::Fill)
            .height(Length::Fill)
            .spacing(2)
            .into()
    } else {
        let color_col = view_color_selector(control);
        row![type_col, color_col, chain_col]
            .width(Length::Fill)
            .height(Length::Fill)
            .spacing(2)
            .into()
    }
}

// ============================================================
// Column 1: Type selector  (unchanged)
// ============================================================

fn view_type_selector(
    control: &crate::core::v_emerald_collection::EmeraldCollectionControl,
) -> Element<'static, crate::Message> {
    let sel       = control.selected_type.as_ref();
    let is_master = matches!(sel, Some(EmeraldTypes::MasterEmerald));
    let is_super  = matches!(sel, Some(EmeraldTypes::SuperEmerald(_)));
    let is_chaos  = matches!(sel, Some(EmeraldTypes::ChaosEmerald(_)));
    let is_jewel  = matches!(sel, Some(EmeraldTypes::RandomJewels(_)));

    let master_btn: Option<Element<'static, crate::Message>> =
        if control.should_show_emerald_type(&EmeraldTypes::MasterEmerald) {
            let price = control.price_line_for(&EmeraldTypes::MasterEmerald);
            Some(color_button(
                EmeraldTypes::MasterEmerald,
                control.master_icon_handle.clone(),
                "Master (/ES)".to_string(),
                price,
                is_master,
                control.display_context,
            ))
        } else {
            None
        };

    let super_btn = type_button(
        "Super\nEmerald",
        control.supers_icon_handle.clone(),
        is_super,
        make_select_msg(EmeraldTypes::SuperEmerald(EmeraldColor::Green), control.display_context),
    );
    let chaos_btn = type_button(
        "Chaos\nEmerald",
        control.chaoss_icon_handle.clone(),
        is_chaos,
        make_select_msg(EmeraldTypes::ChaosEmerald(EmeraldColor::Green), control.display_context),
    );
    let jewel_btn = type_button(
        "Random\nJewels",
        control.jewels_icon_handle.clone(),
        is_jewel,
        crate::Message::DrR(crate::lens::v_dr_r::Message::RecvLogNote(
            "Random Jewels: select from the color panel →".to_string(),
        )),
    );

    let mut col = column![]
        .spacing(2)
        .padding(4)
        .height(Length::Fill)
        .width(Length::Fixed(116.0));

    col = col.push(text("Type").size(12));
    col = col.push(Space::new().height(Length::Fixed(4.0)));

    if let Some(btn) = master_btn {
        col = col.push(btn);
        col = col.push(Space::new().height(Length::Fixed(4.0)));
    }

    col = col
        .push(super_btn)
        .push(Space::new().height(Length::Fixed(4.0)))
        .push(chaos_btn)
        .push(jewel_btn);

    container(scrollable(col).height(Length::Fill))
        .width(Length::Fixed(101.0))
        .height(Length::Fill)
        .padding(2)
        .style(|_theme| iced::widget::container::Style {
            background: Some(iced::Background::Color(
                iced::Color::from_rgba(0.89, 0.09, 0.05, 0.55),
            )),
            ..Default::default()
        })
        .into()
}

fn type_button(
    label: &'static str,
    handle: iced::widget::image::Handle,
    _is_selected: bool,
    on_press: crate::Message,
) -> Element<'static, crate::Message> {
    let icon = image(handle)
        .width(Length::Fixed(100.0))
        .height(Length::Fixed(100.0));
    let inner = column![text(label).size(10), icon]
        .spacing(2)
        .align_x(Alignment::Center)
        .height(Length::Fixed(100.0))
        .width(Length::Fixed(100.0));
    button(inner)
        .padding(2)
        .width(Length::Fixed(100.0))
        .height(Length::Fixed(100.0))
        .on_press(on_press)
        .into()
}

// ============================================================
// Column 2: Color selector  (unchanged)
// ============================================================

fn view_color_selector(
    control: &crate::core::v_emerald_collection::EmeraldCollectionControl,
) -> Element<'static, crate::Message> {
    let mut col = Column::new().spacing(4).padding(4);

    match &control.selected_type {
        Some(EmeraldTypes::MasterEmerald) | None => {
            col = col.push(text("Master Emerald").size(13));
            let price = control.price_line_for(&EmeraldTypes::MasterEmerald);
            col = col.push(color_button(
                EmeraldTypes::MasterEmerald,
                control.master_icon_handle.clone(),
                "Master (/ES)".to_string(),
                price,
                true,
                control.display_context,
            ));
        }

        Some(EmeraldTypes::SuperEmerald(selected_color)) => {
            col = col.push(text("Super Emeralds").size(13));
            for color in EmeraldColor::all_colors() {
                let et = EmeraldTypes::SuperEmerald(*color);
                if !control.should_show_emerald_type(&et) { continue; }
                let handle = control.super_icon_handles
                    .get(color)
                    .cloned()
                    .unwrap_or_else(|| control.default_icon_handle.clone());
                let label = format!("{} ({})", color.name(), color.super_ticker());
                let price = control.price_line_for(&et);
                let is_sel = selected_color == color;
                col = col.push(color_button(et, handle, label, price, is_sel, control.display_context));
            }
        }

        Some(EmeraldTypes::ChaosEmerald(selected_color)) => {
            col = col.push(text("Chaos Emeralds").size(13));
            for color in EmeraldColor::all_colors() {
                let et = EmeraldTypes::ChaosEmerald(*color);
                if !control.should_show_emerald_type(&et) { continue; }
                let handle = control.chaos_icon_handles
                    .get(color)
                    .cloned()
                    .unwrap_or_else(|| control.default_icon_handle.clone());
                let label = format!("{} ({})", color.name(), color.chaos_ticker());
                let price = control.price_line_for(&et);
                let is_sel = selected_color == color;
                col = col.push(color_button(et, handle, label, price, is_sel, control.display_context));
            }
        }

        Some(EmeraldTypes::RandomJewels(_)) => {
            col = col.push(text("Random Jewels").size(13));
            let mut jewels: Vec<(EmeraldTypes, String)> = control
                .emeralds
                .iter()
                .filter_map(|(_, (et, _))| {
                    if let EmeraldTypes::RandomJewels(_) = et {
                        if control.should_show_emerald_type(et) {
                            Some((et.clone(), control.price_line_for(et)))
                        } else { None }
                    } else { None }
                })
                .collect();
            jewels.sort_by(|(a, _), (b, _)| a.label().cmp(&b.label()));

            if jewels.is_empty() {
                col = col.push(text("(none yet)").size(10));
            } else {
                for (et, price) in jewels {
                    let is_sel = control.selected_type.as_ref() == Some(&et);
                    col = col.push(color_button(
                        et,
                        control.default_icon_handle.clone(),
                        String::new(),
                        price,
                        is_sel,
                        control.display_context,
                    ));
                }
            }
        }
    }

    container(scrollable(col).height(Length::Fill).width(Length::Fixed(130.0)))
        .width(Length::Fixed(140.0))
        .height(Length::Fill)
        .padding(2)
        .style(|_theme| iced::widget::container::Style {
            background: Some(iced::Background::Color(
                iced::Color::from_rgba(0.05, 0.89, 0.05, 0.12),
            )),
            ..Default::default()
        })
        .into()
}

fn color_button(
    et: EmeraldTypes,
    handle: iced::widget::image::Handle,
    label: String,
    price: String,
    _is_selected: bool,
    display_context: Option<crate::core::BotTypeContext>,
) -> Element<'static, crate::Message> {
    let display_label = if label.is_empty() { et.label() } else { label };
    let icon = image(handle)
        .width(Length::Fixed(100.0))
        .height(Length::Fixed(100.0));
    let inner = column![text(display_label).size(10), icon, text(price).size(8)]
        .spacing(2)
        .align_x(Alignment::Center)
        .width(Length::Fixed(110.0));
    button(inner)
        .padding(2)
        .width(Length::Fixed(114.0))
        .on_press(make_select_msg(et, display_context))
        .into()
}

fn make_select_msg(
    et: EmeraldTypes,
    display_context: Option<crate::core::BotTypeContext>,
) -> crate::Message {
    let ec_msg = Message::SelectEmeraldType(et);
    match display_context {
        Some(crate::core::BotTypeContext::SwatBot) => crate::Message::DrR(
            crate::lens::v_dr_r::Message::FromMakeSwat(
                crate::form::v_swat_make::Message::EmeraldCollectionMessage(ec_msg),
            ),
        ),
        Some(crate::core::BotTypeContext::SallyBot) => crate::Message::DrR(
            crate::lens::v_dr_r::Message::FromMakeSally(
                crate::form::v_sally_make::Message::EmeraldCollectionMessage(ec_msg),
            ),
        ),
        _ => crate::Message::DrR(crate::lens::v_dr_r::Message::RecvLogNote(
            "WARNING: emerald clicked with no bot display_context set".to_string(),
        )),
    }
}

// ============================================================
// Column 3: Option chain panel
// CHANGED: uses current_push_result() and passes data into v_option_chain::view()
// ============================================================

fn view_option_chain_panel(
    control: &crate::core::v_emerald_collection::EmeraldCollectionControl,
) -> Element<'static, crate::Message> {
    let dc = control.display_context;

    let on_oc_msg = move |msg: crate::core::v_option_chain::Message| {
        let ec_msg = Message::OptionChainMessage(msg);
        match dc {
            Some(crate::core::BotTypeContext::SwatBot) => crate::Message::DrR(
                crate::lens::v_dr_r::Message::FromMakeSwat(
                    crate::form::v_swat_make::Message::EmeraldCollectionMessage(ec_msg),
                ),
            ),
            Some(crate::core::BotTypeContext::SallyBot) => crate::Message::DrR(
                crate::lens::v_dr_r::Message::FromMakeSally(
                    crate::form::v_sally_make::Message::EmeraldCollectionMessage(ec_msg),
                ),
            ),
            _ => crate::Message::DrR(crate::lens::v_dr_r::Message::RecvLogNote(
                "OC click: no display_context".to_string(),
            )),
        }
    };

    // CHANGED: single read path — data comes from emeralds map, not option_chain.last_emerald.
    // None = no emerald selected or no data yet → show placeholder.
    // Some = pass live reference directly into the view (no clone needed).
    match control.current_push_result() {
        None => {
            container(text("Select an emerald to view the option chain").size(12))
                .width(Length::Fill)
                .height(Length::Fill)
                .center_x(Length::Fill)
                .center_y(Length::Fill)
                .into()
        }
        Some(pr) => {
            let ticker     = pr.ass_deets.ticker_name();
            let title      = text(format!("Option Chain: {}", ticker)).size(14);
            // CHANGED: passes Some(pr) as second arg — new view() signature.
            let chain_view = v_option_chain::view(&control.option_chain, Some(pr), on_oc_msg);
            column![title, chain_view]
                .width(Length::Fill)
                .height(Length::Fill)
                .padding(2)
                .into()
        }
    }
}