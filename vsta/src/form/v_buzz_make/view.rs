
// trade/vsta/src/form/v_buzz_make/view.rs

use crate::form::v_buzz_make::{MakeBuzzV, Message};

use crate::base::v_valid_any::view as validated_input_view;
use crate::base::v_combo_box::view as combo_box_view;

use crate::core::v_bot_common_info::view as common_bot_info_view;
use crate::core::v_bot_action_time::view as buzz_time_to_order_view;
use crate::core::v_buzz_abort_time::view as buzz_bot_done_nice_style_view;

pub fn view(dr_r: &MakeBuzzV) 
-> iced::Element<'_, crate::Message> 
{
    log::debug!("view(dr_r: &MakeBuzzV)");

    // Common Bot Info Section
    let v_bot_common_info = common_bot_info_view
    (
        &dr_r.common_bot_info_control,
        |msg| crate::Message::DrR(crate::lens::v_dr_r::Message::FromMakeBuzz(Message::CommonBotInfoMessage(msg))),
    );

    // My Cash Alloc Input
    let v_my_cash_alloc = validated_input_view(
        &dr_r.my_cash_alloc_input,
        "Enter bot's cash allocation",
        |msg| match msg {
            crate::base::v_valid_any::Message::InputChanged(input) => crate::Message::DrR(
                crate::lens::v_dr_r::Message::FromMakeBuzz(Message::MyCashAllocChanged(input)),
            ),
            crate::base::v_valid_any::Message::FlashError(elapsed) => crate::Message::DrR(
                crate::lens::v_dr_r::Message::FromMakeBuzz(Message::FlashError(elapsed)),
            ),
            _ => crate::Message::DrR(
                crate::lens::v_dr_r::Message::FromMakeBuzz(Message::MyCashAllocChanged(String::new())),
            ),
        },
    );

    // Market Direction ComboBox
    let v_market_direction = combo_box_view(
        &dr_r.market_direction_combo,
        "Select market direction",
        |direction| crate::Message::DrR(crate::lens::v_dr_r::Message::FromMakeBuzz(Message::MarketDirectionChanged(direction))),
    );

    // Option Expire Input
    let v_option_expire = validated_input_view(
        &dr_r.option_expire_input,
        "Option expire (days)",
        |msg| match msg {
            crate::base::v_valid_any::Message::InputChanged(input) => crate::Message::DrR(
                crate::lens::v_dr_r::Message::FromMakeBuzz(Message::OptionExpireChanged(input)),
            ),
            crate::base::v_valid_any::Message::FlashError(elapsed) => crate::Message::DrR(
                crate::lens::v_dr_r::Message::FromMakeBuzz(Message::FlashError(elapsed)),
            ),
            _ => crate::Message::DrR(
                crate::lens::v_dr_r::Message::FromMakeBuzz(Message::OptionExpireChanged(String::new())),
            ),
        },
    );

    // Target Spread Input
    let v_target_spread = validated_input_view(
        &dr_r.target_spread_input,
        "Target spread (0.01 - 0.99)",
        |msg| match msg {
            crate::base::v_valid_any::Message::InputChanged(input) => crate::Message::DrR(
                crate::lens::v_dr_r::Message::FromMakeBuzz(Message::TargetSpreadChanged(input)),
            ),
            crate::base::v_valid_any::Message::FlashError(elapsed) => crate::Message::DrR(
                crate::lens::v_dr_r::Message::FromMakeBuzz(Message::FlashError(elapsed)),
            ),
            _ => crate::Message::DrR(
                crate::lens::v_dr_r::Message::FromMakeBuzz(Message::TargetSpreadChanged(String::new())),
            ),
        },
    );

    // Time To Order Control
    let v_time_to_order = buzz_time_to_order_view(
        &dr_r.time_to_order_control,
        |msg| crate::Message::DrR(crate::lens::v_dr_r::Message::FromMakeBuzz(Message::TimeToOrderMessage(msg))),
    );

    // Algo Cooldown Input
    let v_algo_cooldown = validated_input_view(
        &dr_r.algo_cooldown_input,
        "Algo cooldown (seconds)",
        |msg| match msg {
            crate::base::v_valid_any::Message::InputChanged(input) => crate::Message::DrR(
                crate::lens::v_dr_r::Message::FromMakeBuzz(Message::AlgoCooldownChanged(input)),
            ),
            crate::base::v_valid_any::Message::FlashError(elapsed) => crate::Message::DrR(
                crate::lens::v_dr_r::Message::FromMakeBuzz(Message::FlashError(elapsed)),
            ),
            _ => crate::Message::DrR(
                crate::lens::v_dr_r::Message::FromMakeBuzz(Message::AlgoCooldownChanged(String::new())),
            ),
        },
    );

    // Toggler for iced 0.13
    let v_bombs_forever = iced::widget::Toggler::new(dr_r.mv_make_buzz_bomber.bombs_forever)
        .label("Bombs Forever")
        .on_toggle(|value| crate::Message::DrR(crate::lens::v_dr_r::Message::FromMakeBuzz(Message::BombsForeverToggled(value))));

    // Exit Conditions Section
    let v_exit_conditions = render_exit_conditions(dr_r);

    // Create Bot Button
    let v_create_button = iced::widget::Button::new(iced::widget::Text::new("Create Buzz Bomber"))
        .on_press(crate::Message::DrR(crate::lens::v_dr_r::Message::FromMakeBuzz(Message::PressedCreateBot)));



  let main_column = iced::widget::column!
  [ iced::widget::Space::new().height(iced::Length::Fixed(5.0)),
    iced::widget::Text::new("Create Buzz Bomber Bot").size(24),
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
    iced::widget::row!["Target Spread:", v_target_spread].align_y(iced::Alignment::Center).spacing(10),
    iced::widget::row!["Algo Cooldown:", v_algo_cooldown].align_y(iced::Alignment::Center).spacing(10),
    iced::widget::row![v_bombs_forever].spacing(10),
    // Time To Order
    iced::widget::Space::new().height(iced::Length::Fixed(5.0)),
    v_time_to_order,
    // Exit Conditions
    iced::widget::Space::new().height(iced::Length::Fixed(5.0)),
    v_exit_conditions,
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
  ].into()
}

fn render_exit_conditions(dr_r: &MakeBuzzV) -> iced::Element<'_, crate::Message> {
    log::debug!("render_exit_conditions");

    let mut column = iced::widget::Column::new().spacing(10);

    column = column.push(iced::widget::Text::new("Exit Conditions").size(20));

    // Render each exit condition control
    for (i, control) in dr_r.exit_condition_controls.iter().enumerate() {
        let is_editing = dr_r.editing_exit_index == Some(i);

        let exit_view = buzz_bot_done_nice_style_view(
            control,
            move |msg| crate::Message::DrR(crate::lens::v_dr_r::Message::FromMakeBuzz(Message::ExitConditionMessage(i, msg))),
        );

        let button_row = iced::widget::row![
            if is_editing {
                iced::widget::Button::new("Collapse").on_press(crate::Message::DrR(
                    crate::lens::v_dr_r::Message::FromMakeBuzz(Message::EditExitCondition(usize::MAX))
                ))
            } else {
                iced::widget::Button::new("Edit").on_press(crate::Message::DrR(
                    crate::lens::v_dr_r::Message::FromMakeBuzz(Message::EditExitCondition(i))
                ))
            },
            iced::widget::Space::new().width(iced::Length::Fixed(10.0)),
            iced::widget::Button::new("Remove").on_press(crate::Message::DrR(
                crate::lens::v_dr_r::Message::FromMakeBuzz(Message::RemoveExitCondition(i))
            ))
        ]
        .spacing(10);

        let condition_container = iced::widget::column!
        [ iced::widget::Text::new(format!("Exit Condition {}", i + 1)).size(16)
        , button_row
        , if is_editing || control.is_visible 
          { exit_view
          } 
          else // color pink so show we aren't draing stuff
          { iced::widget::Column::new().push
            ( iced::widget::Container::new
              ( iced::widget::Space::new
                (
                ).width
                ( iced::Length::Fixed(10.0)
                ).height
                ( iced::Length::Fixed(10.0)
                )
              )
              .style
              ( |_theme: &iced::Theme| iced::widget::container::Style 
                { background: Some
                  ( iced::Background::Color
                    ( iced::Color::from_rgb(1.0, 0.0, 0.8)
                    )
                  ),
                  ..Default::default()
                }
              )
            )

                //iced::widget::Space::new(iced::Length::Fixed(0.0), iced::Length::Fixed(0.0))
            }
        ]
        .spacing(5)
        .padding(10);

        column = column.push(iced::widget::container(condition_container).style(|_theme: &iced::Theme| {
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

    // Add new exit condition button
    let add_button = iced::widget::Button::new("+ Add Exit Condition")
        .on_press(crate::Message::DrR(crate::lens::v_dr_r::Message::FromMakeBuzz(Message::AddExitCondition)));

    column = column.push(add_button);

    column.into()
}
