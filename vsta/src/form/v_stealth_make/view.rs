
// trade/vsta/src/form/v_stealth_make/view.rs

use crate::form::v_stealth_make::{MakeStealthV, Message};

use crate::base::v_valid_any::view as validated_input_view;
use crate::base::v_combo_box::view as combo_box_view;

use crate::core::v_bot_common_info::view as common_bot_info_view;
use crate::core::v_stea_abort_time::view as stealth_exit_strategy_view;

pub fn view(dr_r: &MakeStealthV)
-> iced::widget::Row<'_, crate::Message>
{
    log::debug!("view(dr_r: &MakeStealthV)");

    // Common Bot Info Section
    let v_bot_common_info = common_bot_info_view
    (
        &dr_r.common_bot_info_control,
        |msg| crate::Message::DrR(crate::lens::v_dr_r::Message::FromMakeStealth(Message::CommonBotInfoMessage(msg))),
    );

    // My Cash Alloc Input
    let v_my_cash_alloc = validated_input_view(
        &dr_r.my_cash_alloc_input,
        "Enter bot's cash allocation",
        |msg| match msg {
            crate::base::v_valid_any::Message::InputChanged(input) => crate::Message::DrR(
                crate::lens::v_dr_r::Message::FromMakeStealth(Message::MyCashAllocChanged(input)),
            ),
            crate::base::v_valid_any::Message::FlashError(elapsed) => crate::Message::DrR(
                crate::lens::v_dr_r::Message::FromMakeStealth(Message::FlashError(elapsed)),
            ),
            _ => crate::Message::DrR(
                crate::lens::v_dr_r::Message::FromMakeStealth(Message::MyCashAllocChanged(String::new())),
            ),
        },
    );

    // Market Direction ComboBox
    let v_market_direction = combo_box_view(
        &dr_r.market_direction_combo,
        "Select market direction",
        |direction| crate::Message::DrR(crate::lens::v_dr_r::Message::FromMakeStealth(Message::MarketDirectionChanged(direction))),
    );

    // Option Expire Input
    let v_option_expire = validated_input_view(
        &dr_r.option_expire_input,
        "Option expire (days)",
        |msg| match msg {
            crate::base::v_valid_any::Message::InputChanged(input) => crate::Message::DrR(
                crate::lens::v_dr_r::Message::FromMakeStealth(Message::OptionExpireChanged(input)),
            ),
            crate::base::v_valid_any::Message::FlashError(elapsed) => crate::Message::DrR(
                crate::lens::v_dr_r::Message::FromMakeStealth(Message::FlashError(elapsed)),
            ),
            _ => crate::Message::DrR(
                crate::lens::v_dr_r::Message::FromMakeStealth(Message::OptionExpireChanged(String::new())),
            ),
        },
    );

    // Option Bucket Input
    let v_option_bucket = validated_input_view(
        &dr_r.option_bucket_input,
        "Option bucket (0=ATM)",
        |msg| match msg {
            crate::base::v_valid_any::Message::InputChanged(input) => crate::Message::DrR(
                crate::lens::v_dr_r::Message::FromMakeStealth(Message::OptionBucketChanged(input)),
            ),
            crate::base::v_valid_any::Message::FlashError(elapsed) => crate::Message::DrR(
                crate::lens::v_dr_r::Message::FromMakeStealth(Message::FlashError(elapsed)),
            ),
            _ => crate::Message::DrR(
                crate::lens::v_dr_r::Message::FromMakeStealth(Message::OptionBucketChanged(String::new())),
            ),
        },
    );

    // Spread Bucket Input
    let v_spread_bucket = validated_input_view(
        &dr_r.spread_bucket_input,
        "Spread bucket",
        |msg| match msg {
            crate::base::v_valid_any::Message::InputChanged(input) => crate::Message::DrR(
                crate::lens::v_dr_r::Message::FromMakeStealth(Message::SpreadBucketChanged(input)),
            ),
            crate::base::v_valid_any::Message::FlashError(elapsed) => crate::Message::DrR(
                crate::lens::v_dr_r::Message::FromMakeStealth(Message::FlashError(elapsed)),
            ),
            _ => crate::Message::DrR(
                crate::lens::v_dr_r::Message::FromMakeStealth(Message::SpreadBucketChanged(String::new())),
            ),
        },
    );

    // Exit Gain Pct Input
    let v_exit_gain_pct = validated_input_view(
        &dr_r.exit_gain_pct_input,
        "Exit gain %",
        |msg| match msg {
            crate::base::v_valid_any::Message::InputChanged(input) => crate::Message::DrR(
                crate::lens::v_dr_r::Message::FromMakeStealth(Message::ExitGainPctChanged(input)),
            ),
            crate::base::v_valid_any::Message::FlashError(elapsed) => crate::Message::DrR(
                crate::lens::v_dr_r::Message::FromMakeStealth(Message::FlashError(elapsed)),
            ),
            _ => crate::Message::DrR(
                crate::lens::v_dr_r::Message::FromMakeStealth(Message::ExitGainPctChanged(String::new())),
            ),
        },
    );

    // Exit Loss Pct Input
    let v_exit_loss_pct = validated_input_view(
        &dr_r.exit_loss_pct_input,
        "Exit loss %",
        |msg| match msg {
            crate::base::v_valid_any::Message::InputChanged(input) => crate::Message::DrR(
                crate::lens::v_dr_r::Message::FromMakeStealth(Message::ExitLossPctChanged(input)),
            ),
            crate::base::v_valid_any::Message::FlashError(elapsed) => crate::Message::DrR(
                crate::lens::v_dr_r::Message::FromMakeStealth(Message::FlashError(elapsed)),
            ),
            _ => crate::Message::DrR(
                crate::lens::v_dr_r::Message::FromMakeStealth(Message::ExitLossPctChanged(String::new())),
            ),
        },
    );

    // Use Theo Cost Toggle
    let v_use_theo = iced::widget::Toggler::new(dr_r.mv_make_stealth_bot.use_theo_cost)
        .label("Use Theoretical Cost")
        .on_toggle(|value| crate::Message::DrR(crate::lens::v_dr_r::Message::FromMakeStealth(Message::UseTheoChanged(value))));

    // Exit Strategies Section
    let v_exit_strategies = render_exit_strategies(dr_r);

    // Create Bot Button
    let v_create_button = iced::widget::Button::new(iced::widget::Text::new("Create Stealth Bot"))
        .on_press(crate::Message::DrR(crate::lens::v_dr_r::Message::FromMakeStealth(Message::PressedCreateBot)));

    let main_column = iced::widget::column!
    [ iced::widget::Space::new().height(iced::Length::Fixed(5.0)),
      iced::widget::Text::new("Create Stealth Bot").size(24),
      iced::widget::Space::new().height(iced::Length::Fixed(5.0)),
      // Common Bot Info
      v_bot_common_info,
      iced::widget::Space::new().height(iced::Length::Fixed(5.0)),
      // Bot-Specific Settings
      iced::widget::Text::new("Bot Configuration").size(20),
      iced::widget::Space::new().height(iced::Length::Fixed(5.0)),
      iced::widget::row!["Cash Allocation:", v_my_cash_alloc].align_y(iced::Alignment::Center).spacing(10),
      iced::widget::row!["Market Direction:", v_market_direction].align_y(iced::Alignment::Center).spacing(10),
      iced::widget::row!["Option Expire:", v_option_expire].align_y(iced::Alignment::Center).spacing(10),
      iced::widget::row!["Option Bucket:", v_option_bucket].align_y(iced::Alignment::Center).spacing(10),
      iced::widget::row!["Spread Bucket:", v_spread_bucket].align_y(iced::Alignment::Center).spacing(10),
      iced::widget::row!["Exit Gain %:", v_exit_gain_pct].align_y(iced::Alignment::Center).spacing(10),
      iced::widget::row!["Exit Loss %:", v_exit_loss_pct].align_y(iced::Alignment::Center).spacing(10),
      iced::widget::row![v_use_theo].spacing(10),
      // Exit Strategies
      iced::widget::Space::new().height(iced::Length::Fixed(5.0)),
      v_exit_strategies,
      // Create Button
      iced::widget::Space::new().height(iced::Length::Fixed(5.0)),
      v_create_button
    ]
    .spacing(5)
    .padding(20);

    let main_column_scroll = iced::widget::Scrollable::new(main_column)
      .height(iced::Length::Fill);

    // Main layout
    iced::widget::row!
    [ main_column_scroll
    ]
}

fn render_exit_strategies(dr_r: &MakeStealthV) -> iced::Element<'_, crate::Message> {
    log::debug!("render_exit_strategies");

    let mut column = iced::widget::Column::new().spacing(10);

    column = column.push(iced::widget::Text::new("Exit Strategies").size(20));

    // Render each exit strategy control
    for (i, control) in dr_r.exit_strategy_controls.iter().enumerate() {
        let strategy_view = stealth_exit_strategy_view(
            control,
            move |msg| crate::Message::DrR(crate::lens::v_dr_r::Message::FromMakeStealth(Message::ExitStrategyMessage(i, msg))),
        );

        let button_row = iced::widget::row![
            iced::widget::Button::new("Remove").on_press(crate::Message::DrR(
                crate::lens::v_dr_r::Message::FromMakeStealth(Message::RemoveExitStrategy(i))
            )),
            iced::widget::Space::new().width(iced::Length::Fixed(10.0)),
        ]
        .spacing(10);

        let strategy_container = iced::widget::column![
            iced::widget::Text::new(format!("Exit Strategy {}", i + 1)).size(16),
            button_row,
            strategy_view
        ]
        .spacing(5)
        .padding(10);

        column = column.push(iced::widget::container(strategy_container).style(|_theme: &iced::Theme| {
            iced::widget::container::Style {
                border: iced::Border {
                    color: iced::Color::from_rgb(0.7, 0.7, 0.7),
                    width: 1.0,
                    radius: 4.0.into(),
                },
                ..Default::default()
            }
        }));
    }

    // Add new exit strategy button
    let add_button = iced::widget::Button::new("+ Add Exit Strategy")
        .on_press(crate::Message::DrR(crate::lens::v_dr_r::Message::FromMakeStealth(Message::AddExitStrategy)));

    column = column.push(add_button);

    column.into()
}