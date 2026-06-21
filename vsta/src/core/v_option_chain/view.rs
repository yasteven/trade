// trade/vsta/src/core/v_option_chain/view.rs
//
// CHANGED vs old version:
//   - view() now takes `data: Option<&dsta::PushTickerResult>` as second arg
//     (replaces reading control.expiry_levels / control.underlying_ticker)
//   - ExpiryLevels built on the fly at the top of view(), not stored in control
//   - render_expiry_selector / render_chain_display receive &[ExpiryLevel] slice
//   - visible_expiries() logic inlined (was a method on OptionChainControl)

use iced::widget::{button, column, container, row, scrollable, text, Text, Column};
use iced::{Alignment, Element, Length, Theme};

use crate::core::v_option_chain::{ExpiryLevel, Message, OptionChainControl};

// ─────────────────────────────────────────────────────────────────────────────
// Public entry point
// ─────────────────────────────────────────────────────────────────────────────

pub fn view(
    control: &OptionChainControl,
    data: Option<&dsta::PushTickerResult>,
    on_message: impl Fn(Message) -> crate::Message + 'static + Copy,
) -> Element<'static, crate::Message> {
    log::debug!("v_option_chain::view rendering");

    // Build sorted expiry levels on the fly — NOT stored in control.
    let mut expiry_levels: Vec<ExpiryLevel> = data
        .map(|d| d.opt_chain.iter().map(ExpiryLevel::from_derivatives).collect())
        .unwrap_or_default();
    expiry_levels.sort_by(|a, b| a.expire_date.cmp(&b.expire_date));

    let expiry_buttons = render_expiry_selector(control, &expiry_levels, on_message);
    let chain_display = render_chain_display(control, &expiry_levels, on_message);

    row![
        container(expiry_buttons)
            .width(Length::Fixed(140.0))
            .height(Length::Fill)
            .padding(2)
            .style(|_theme| iced::widget::container::Style {
                background: Some(iced::Background::Color(
                    iced::Color::from_rgba(0.65, 0.69, 0.05, 0.3),
                )),
                ..Default::default()
            }),
        container(chain_display)
            .width(Length::Fill)
            .height(Length::Fill)
            .padding(2)
            .style(|_theme| iced::widget::container::Style {
                background: Some(iced::Background::Color(
                    iced::Color::from_rgba(0.35, 0.09, 0.35, 0.3),
                )),
                ..Default::default()
            }),
    ]
    .spacing(2)
    .align_y(Alignment::Start)
    .into()
}

// ─────────────────────────────────────────────────────────────────────────────
// Expiry selector column
// ─────────────────────────────────────────────────────────────────────────────

fn render_expiry_selector(
    control: &OptionChainControl,
    expiry_levels: &[ExpiryLevel],
    on_message: impl Fn(Message) -> crate::Message + Copy,
) -> Element<'static, crate::Message> {
    log::debug!(
        "render_expiry_selector: {} expiries available",
        expiry_levels.len()
    );

    if expiry_levels.is_empty() {
        return container(text("No chains available"))
            .width(Length::Fill)
            .height(Length::Fill)
            .center_x(Length::Fill)
            .center_y(Length::Fill)
            .into();
    }

    let mut expiry_col = column![].width(Length::Fill).spacing(4);

    expiry_col = expiry_col.push(
        button(text("↑").size(12))
            .width(Length::Fill)
            .on_press(on_message(Message::ScrollExpiryUp)),
    );

    // Replaces control.visible_expiries() — inlined here.
    let start = control.expiry_scroll_offset;
    let end = (start + control.expiry_visible_count).min(expiry_levels.len());

    for abs_index in start..end {
        if let Some(expiry) = expiry_levels.get(abs_index) {
            let is_selected = control.selected_expiry_index == abs_index;
            let days_to_expiry = (expiry.expire_date - chrono::Utc::now()).num_days();
            let label = format!(
                "{}d\n{}",
                days_to_expiry,
                expiry.expire_date.format("%m/%d")
            );

            expiry_col = expiry_col.push(
                button(Text::new(label).size(10))
                    .width(Length::Fill)
                    .padding(6)
                    .on_press(on_message(Message::SelectExpiry(abs_index))),
            );
        }
    }

    expiry_col = expiry_col.push(
        button(text("↓").size(12))
            .width(Length::Fill)
            .on_press(on_message(Message::ScrollExpiryDown)),
    );

    scrollable(expiry_col).height(Length::Fill).width(Length::Fill).into()
}

// ─────────────────────────────────────────────────────────────────────────────
// Option chain grid
// ─────────────────────────────────────────────────────────────────────────────

fn render_chain_display(
    control: &OptionChainControl,
    expiry_levels: &[ExpiryLevel],
    on_message: impl Fn(Message) -> crate::Message + Copy + 'static,
) -> Element<'static, crate::Message> {
    log::debug!("render_chain_display");

    // Replaces control.current_expiry() — index into the locally-built slice.
    if let Some(expiry) = expiry_levels.get(control.selected_expiry_index) {
        let strikes = expiry.strikes();

        if strikes.is_empty() {
            return container(text("No options at this expiry"))
                .width(Length::Fill)
                .height(Length::Fill)
                .center_x(Length::Fill)
                .center_y(Length::Fill)
                .into();
        }

        let header = render_grid_header();
        let body = render_grid_body(expiry, &strikes, control.selected_option.as_ref(), on_message);

        column![header, scrollable(body).height(Length::Fill).width(Length::Fill)]
            .width(Length::Fill)
            .height(Length::Fill)
            .spacing(0)
            .into()
    } else {
        container(text("Select an expiry"))
            .width(Length::Fill)
            .height(Length::Fill)
            .center_x(Length::Fill)
            .center_y(Length::Fill)
            .into()
    }
}

fn render_grid_header() -> Element<'static, crate::Message> {
    let header_style = |_theme: &Theme| iced::widget::container::Style {
        background: Some(iced::Background::Color(iced::Color::from_rgb(0.2, 0.2, 0.2))),
        ..Default::default()
    };

    let header_row = row![
        container(text("CALL\nBID").size(10)).width(Length::FillPortion(1)).center_x(Length::Fill),
        container(text("ASK").size(10)).width(Length::FillPortion(1)).center_x(Length::Fill),
        container(text("LAST").size(10)).width(Length::FillPortion(1)).center_x(Length::Fill),
        container(text("THEO").size(10)).width(Length::FillPortion(1)).center_x(Length::Fill),
        container(text("STRIKE").size(10)).width(Length::FillPortion(1)).center_x(Length::Fill),
        container(text("THEO").size(10)).width(Length::FillPortion(1)).center_x(Length::Fill),
        container(text("LAST").size(10)).width(Length::FillPortion(1)).center_x(Length::Fill),
        container(text("ASK").size(10)).width(Length::FillPortion(1)).center_x(Length::Fill),
        container(text("PUT\nBID").size(10)).width(Length::FillPortion(1)).center_x(Length::Fill),
    ]
    .spacing(2)
    .width(Length::Fill);

    container(header_row)
        .width(Length::Fill)
        .padding(4)
        .style(header_style)
        .into()
}

fn render_grid_body(
    expiry: &ExpiryLevel,
    strikes: &[f64],
    selected_option: Option<&crate::core::v_option_chain::SelectedOption>,
    on_message: impl Fn(Message) -> crate::Message + 'static + Copy,
) -> Column<'static, crate::Message> {
    let mut col = column![];

    for &strike in strikes {
        let call_opt = expiry.call_at_strike(strike);
        let put_opt  = expiry.put_at_strike(strike);

        let (call_bid, call_ask, call_last, call_theo) =
            derive_side_values(call_opt.and_then(|o| o.last_ticker.clone()));
        let (put_bid, put_ask, put_last, put_theo) =
            derive_side_values(put_opt.and_then(|o| o.last_ticker.clone()));

        let row = row![
            grid_cell_button("CALL_BID",  call_bid,  call_opt, on_message),
            grid_cell_button("CALL_ASK",  call_ask,  call_opt, on_message),
            grid_cell_button("CALL_LAST", call_last, call_opt, on_message),
            grid_cell_button("CALL_THEO", call_theo, call_opt, on_message),
            container(text(format!("{:.2}", strike)).size(10))
                .width(Length::FillPortion(1))
                .center_x(Length::Fill)
                .padding(2),
            grid_cell_button("PUT_THEO", put_theo, put_opt, on_message),
            grid_cell_button("PUT_LAST", put_last, put_opt, on_message),
            grid_cell_button("PUT_ASK",  put_ask,  put_opt, on_message),
            grid_cell_button("PUT_BID",  put_bid,  put_opt, on_message),
        ]
        .spacing(2)
        .width(Length::Fill);

        col = col.push(row);
    }

    col.spacing(1)
}

// ─────────────────────────────────────────────────────────────────────────────
// Helpers (unchanged logic vs old version)
// ─────────────────────────────────────────────────────────────────────────────

fn derive_side_values(ticker_opt: Option<dsta::Ticker>) -> (String, String, String, String) {
    if let Some(ticker) = ticker_opt {
        let bid  = if ticker.bid > 0.0 { format!("{:.2}", ticker.bid) } else { String::new() };
        let ask  = if ticker.ask > 0.0 { format!("{:.2}", ticker.ask) } else { String::new() };
        let last = if (ticker.bid - ticker.ask).abs() < f64::EPSILON && ticker.bid > 0.0 {
            format!("{:.2}", ticker.bid)
        } else {
            String::new()
        };
        let theo = ticker.theo_price.map(|v| format!("{:.2}", v)).unwrap_or_default();
        (bid, ask, last, theo)
    } else {
        (String::new(), String::new(), String::new(), String::new())
    }
}

fn grid_cell_button(
    column_name: &'static str,
    value: String,
    opt: Option<&dsta::Opt>,
    on_message: impl Fn(Message) -> crate::Message + Copy,
) -> Element<'static, crate::Message> {
    let content_text = if value.is_empty() {
        text(" ").size(10)
    } else {
        text(value).size(10)
    };

    if let Some(opt_ref) = opt {
        let finass = dsta::FinAss::OptDeets(opt_ref.clone());
        button(content_text)
            .width(Length::FillPortion(1))
            .padding(2)
            .on_press(on_message(Message::GridCellClicked {
                column_name: column_name.to_string(),
                finass,
            }))
            .into()
    } else {
        container(content_text)
            .width(Length::FillPortion(1))
            .padding(2)
            .center_x(Length::Fill)
            .into()
    }
}