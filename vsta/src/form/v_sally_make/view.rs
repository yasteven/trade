// trade/vsta/src/form/v_sally_make/view.rs
// Only the changed section is the Preset tab arm — everything else is identical.

use crate::form::v_sally_make::{MakeSallyV, Message, ControlTab};
use crate::base::v_valid_any::view as validated_input_view;
use crate::core::v_bot_common_info::view as common_bot_info_view;
use crate::core::v_sally_enter_way::view as sally_action_view;
use iced::widget::{Button, Column, Row, Text, TextInput, PickList, Container, Scrollable, Space};
use iced::{Element, Length, Alignment};

pub fn view<'a>
( dr_r: &'a MakeSallyV
, emerald_control: &'a crate::core::v_emerald_collection::EmeraldCollectionControl,
) -> Row<'a, crate::Message>
{ log::debug!("v_sally_make::view BEGIN");

  // ============================================================
  // COLUMN 1: TAB SELECTOR
  // ============================================================
  let common_tab_button = Button::new
  ( iced::widget::container(Text::new("💻\nCommon"))
    .center_y(Length::Fill)
    .center_x(Length::Fill)
    .width(Length::Shrink)
  ).height(Length::Fixed(100.0)).width(Length::Fixed(100.0))
  .padding(2)
  .on_press
  ( crate::Message::DrR
    ( crate::lens::v_dr_r::Message::FromMakeSally
      ( Message::SwitchTab(ControlTab::Common) )
    )
  );

  let orders_tab_button = Button::new
  ( iced::widget::container(Text::new("🐿🐿️\nOrders"))
    .center_y(Length::Fill)
    .center_x(Length::Fill)
    .width(Length::Shrink)
  ).height(Length::Fixed(100.0)).width(Length::Fixed(100.0))
  .padding(2)
  .on_press
  ( crate::Message::DrR
    ( crate::lens::v_dr_r::Message::FromMakeSally
      ( Message::SwitchTab(ControlTab::Orders) )
    )
  );

  let preset_tab_button = Button::new
  ( iced::widget::container(Text::new("💾\nPreset"))
    .center_y(Length::Fill)
    .center_x(Length::Fill)
    .width(Length::Shrink)
  ).height(Length::Fixed(100.0)).width(Length::Fixed(100.0))
  .padding(2)
  .on_press
  ( crate::Message::DrR
    ( crate::lens::v_dr_r::Message::FromMakeSally
      ( Message::SwitchTab(ControlTab::Preset) )
    )
  );

  let v_create_button = Button::new
  ( Text::new("✓ Create Sally Bot")
  ).on_press
  ( crate::Message::DrR
    ( crate::lens::v_dr_r::Message::FromMakeSally
      ( Message::PressedCreateBot )
    )
  )
  .width(Length::Fill)
  .padding(2);

  // Highlight the active tab button with a subtle indicator
  let active_label = match dr_r.current_control_tab
  { ControlTab::Common => "▶ Common",
    ControlTab::Orders => "▶ Orders",
    ControlTab::Preset => "▶ Preset",
  };

  let tab_buttons = Column::new()
    .push(Text::new("Options").size(14))
    .push(Space::new().height(Length::Fixed(8.0)))
    .push(common_tab_button)
    .push(orders_tab_button)
    .push(preset_tab_button)
    .push(Space::new().height(Length::Fixed(4.0)))
    .push(Text::new(active_label).size(10))
    .push(Space::new().height(Length::Fill))
    .push(v_create_button)
    .spacing(5)
    .padding(10)
    .width(Length::Fixed(120.0));

  // ============================================================
  // COLUMN 2: CONTENT (tab-dependent)
  // ============================================================
  let control_view: Element<'a, crate::Message> = match dr_r.current_control_tab
  { ControlTab::Common =>
    { common_bot_info_view
      ( &dr_r.common_bot_info_control,
        |msg| crate::Message::DrR
        ( crate::lens::v_dr_r::Message::FromMakeSally
          ( Message::CommonBotInfoMessage(msg) )
        )
      ).into()
    }
    ControlTab::Orders =>
    { render_order_config(dr_r)
    }
    ControlTab::Preset =>
    { // Pass selected_assets so the preset view shows the right options
      crate::core::v_sally_preset::view
      ( &dr_r.sally_presets,
        &dr_r.selected_assets,              // ← context for BTC / stock / option
        |preset| crate::Message::DrR
        ( crate::lens::v_dr_r::Message::FromMakeSally
          ( Message::SallyPresetSelected(preset) )
        )
      )
    }
  };

  let control_scroll = Scrollable::new(control_view)
    .height(Length::Fill)
    .width(Length::Fill);

  // ============================================================
  // COLUMN 3: EMERALD VIEW
  // ============================================================
  let emerald_view: Element<'a, crate::Message>
    = crate::core::v_emerald_collection::view(&emerald_control);

  // ============================================================
  // FINAL LAYOUT
  // ============================================================
  iced::widget::row!
  [ iced::widget::container(tab_buttons)
      .width(Length::Fixed(121.0))
      .height(Length::Fill)
  , iced::widget::container(control_scroll)
      .width(Length::FillPortion(1))
      .height(Length::Fill)
      .padding(20)
      .style
      ( |_theme| iced::widget::container::Style
        { background: Some(iced::Background::Color(iced::Color::from_rgba(0.05, 0.19, 0.49, 0.2))),
          ..Default::default()
        }
      )
  , iced::widget::container(emerald_view)
      .width(Length::FillPortion(2))
      .height(Length::Fill)
  ]
  .width(Length::Fill)
  .spacing(10)
  .padding(10)
}

fn render_order_config<'a>(dr_r: &'a MakeSallyV) -> Element<'a, crate::Message>
{ let current_order = &dr_r.mv_make_sally_bot.my_hide_order;
  let leg = if current_order.order_legs.is_empty()
  { dsta::OrderLeg::default()
  } else
  { current_order.order_legs[0].clone()
  };

  let leg_ticker_name = leg.buy_what.order_name();

  let leg_ticker = TextInput::new("Leg Ticker", &leg_ticker_name)
    .on_input(move |s|
    { crate::Message::DrR
      ( crate::lens::v_dr_r::Message::FromMakeSally
        ( Message::UpdateOrderLegTicker(s) )
      )
    });

  let quantity_background = match dr_r.order_leg_quantity_input.error_flash
  { Some(progress) =>
    { let t = progress.min(1.0);
      iced::Color { r: 1.0, g: (0.9 * t) as f32, b: (0.9 * t) as f32, a: 1.0 }
    }
    None => iced::Color::from_rgb(0.9, 0.9, 0.9),
  };

  let price_background = match dr_r.order_leg_price_input.error_flash
  { Some(progress) =>
    { let t = progress.min(1.0);
      iced::Color { r: 1.0, g: (0.9 * t) as f32, b: (0.9 * t) as f32, a: 1.0 }
    }
    None => iced::Color::from_rgb(0.9, 0.9, 0.9),
  };

  let leg_quantity = TextInput::new("Quantity", &dr_r.order_leg_quantity_input.value)
    .padding(10)
    .style(move |_theme, _status| iced::widget::text_input::Style
    { background:   iced::Background::Color(quantity_background),
      border:       iced::Border { color: iced::Color::BLACK, width: 1.0, radius: 4.0.into() },
      icon:         iced::Color::BLACK,
      placeholder:  iced::Color::from_rgb(0.3, 0.3, 0.3),
      value:        iced::Color::BLACK,
      selection:    iced::Color::from_rgb(0.7, 0.7, 1.0),
    })
    .on_input(move |s|
    { crate::Message::DrR
      ( crate::lens::v_dr_r::Message::FromMakeSally
        ( Message::UpdateOrderLegQuantity(s) )
      )
    });

  let leg_price = TextInput::new("Limit Price", &dr_r.order_leg_price_input.value)
    .padding(10)
    .style(move |_theme, _status| iced::widget::text_input::Style
    { background:   iced::Background::Color(price_background),
      border:       iced::Border { color: iced::Color::BLACK, width: 1.0, radius: 4.0.into() },
      icon:         iced::Color::BLACK,
      placeholder:  iced::Color::from_rgb(0.3, 0.3, 0.3),
      value:        iced::Color::BLACK,
      selection:    iced::Color::from_rgb(0.7, 0.7, 1.0),
    })
    .on_input(move |s|
    { crate::Message::DrR
      ( crate::lens::v_dr_r::Message::FromMakeSally
        ( Message::UpdateOrderLegPrice(s) )
      )
    });

  static ACTION_OPTIONS: &[dsta::BuyOrSell] = &
  [ dsta::BuyOrSell::BuyToOpen,
    dsta::BuyOrSell::SellToOpen,
    dsta::BuyOrSell::BuyToClose,
    dsta::BuyOrSell::SellToClose,
  ];

  static ORDER_TYPE_OPTIONS: &[dsta::MarketOrLimit] = &
  [ dsta::MarketOrLimit::Market,
    dsta::MarketOrLimit::Limit,
  ];

  let leg_action_pick = PickList::new
  ( ACTION_OPTIONS,
    Some(leg.action),
    move |chosen|
    { crate::Message::DrR
      ( crate::lens::v_dr_r::Message::FromMakeSally
        ( Message::UpdateOrderLegAction(chosen) )
      )
    },
  );

  let order_type_pick = PickList::new
  ( ORDER_TYPE_OPTIONS,
    Some(current_order.order_type.clone()),
    move |chosen|
    { crate::Message::DrR
      ( crate::lens::v_dr_r::Message::FromMakeSally
        ( Message::UpdateOrderType(chosen) )
      )
    },
  );

  let v_sally_enter_way = sally_action_view
  ( &dr_r.sally_action_control,
    |msg| crate::Message::DrR
    ( crate::lens::v_dr_r::Message::FromMakeSally
      ( Message::SallyActionMessage(msg) )
    ),
  );

  // Show a small hint if a preset was applied
  let preset_hint: Option<Element<'a, crate::Message>> =
    dr_r.sally_presets.selected.map(|p|
      Text::new(format!("Preset applied: {}", p.label().replace('\n', " ")))
        .size(10)
        .into()
    );

  let mut col = Column::new()
    .push(Text::new("Hidden Order Configuration").size(18))
    .push(Space::new().height(Length::Fixed(10.0)));

  if let Some(hint) = preset_hint
  { col = col.push(hint).push(Space::new().height(Length::Fixed(6.0)));
  }

  col
    .push(Row::new().push(Text::new("Ticker:")).push(leg_ticker).align_y(Alignment::Center).spacing(10))
    .push(Row::new().push(Text::new("Quantity:")).push(leg_quantity).align_y(Alignment::Center).spacing(10))
    .push(Row::new().push(Text::new("Price:")).push(leg_price).align_y(Alignment::Center).spacing(10))
    .push(Row::new().push(Text::new("Action:")).push(leg_action_pick).align_y(Alignment::Center).spacing(10))
    .push(Row::new().push(Text::new("Order Type:")).push(order_type_pick).align_y(Alignment::Center).spacing(10))
    .push(Space::new().height(Length::Fixed(15.0)))
    .push(Text::new("Entry Strategy").size(18))
    .push(Space::new().height(Length::Fixed(10.0)))
    .push(v_sally_enter_way)
    .spacing(8)
    .padding(10)
    .width(Length::Fill)
    .into()
}