// trade/vsta/src/form/v_sally_make/state.rs

use crate::base::v_valid_any::ValidatedInput;
use crate::core::v_bot_common_info::CommonBotInfoControl;
use crate::core::v_sally_enter_way::SallyActionControl;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ControlTab
{ Common,
  Orders,
  Preset,
}

#[derive(Debug, Clone)]
pub enum Message
{ // Messages for our control:
  SwitchTab(ControlTab),
  PressedCreateBot,
  ProcessCreateBot(dsta::MakeSallyFakes),

  // messages for sub-controls
  CommonBotInfoMessage
  ( crate::core::v_bot_common_info::Message
  ),
  SallyPresetSelected(crate::core::v_sally_preset::SallyPresetOption),
  SallyActionMessage
  ( crate::core::v_sally_enter_way::Message
  ),
  FlashError(iced::time::Instant),
  EmeraldCollectionMessage
  ( crate::core::v_emerald_collection::Message
  ),

  // Order leg editing
  UpdateOrderLegTicker(String),
  UpdateOrderLegQuantity(String),
  UpdateOrderLegQuantityBadInput(iced::time::Instant),
  UpdateOrderLegPrice(String),
  UpdateOrderLegPriceBadInput(iced::time::Instant),
  UpdateOrderLegAction(dsta::BuyOrSell),
  UpdateOrderType(dsta::MarketOrLimit),
}

#[derive(Debug, Clone)]
pub struct MakeSallyV
{ // Main underlying dsta:: Structure:
  pub mv_make_sally_bot: dsta::MakeSallyFakes,
  // Own Controls:
  pub current_control_tab: ControlTab,
  // Common Tab
  pub common_bot_info_control: CommonBotInfoControl,
  // Orders tab
  pub sally_action_control: SallyActionControl,
  pub order_leg_quantity_input: ValidatedInput<f64>,
  pub order_leg_price_input: ValidatedInput<f64>,
  // Preset Tab
  pub sally_presets: crate::core::v_sally_preset::SallyPresets,
  // Emerald Info
  pub selected_assets: crate::dat::SelectedAssets,
}

impl MakeSallyV
{
  pub fn new() -> Self
  { let sally_bot = dsta::MakeSallyFakes
    { my_accounting:
      { let mut ma = dsta::CommonBotInfo::default();
        ma.friendly_name = format!("Sally Fakes 1");
        ma.tracking_tick = format!("BTC/USD");
        ma
      }
      , my_hide_order: dsta::Order
      { order_legs: vec!
        [ dsta::OrderLeg
          { buy_what: dsta::FinAss::StkDeets
            ( dsta::Stk { ticker_name: format!("BTC/USD"), ..Default::default() }
            ),
            quantity: 0.00004,
            remaining: 0.00004,
            action: dsta::BuyOrSell::BuyToOpen,
            price: 91000.0,
          }
        ],
        order_type: dsta::MarketOrLimit::Limit,
      }
      , my_reveal_way: dsta::SallyAction::SubmitRightAwayButLikeOnlyUseForTest,
    };

    MakeSallyV
    { common_bot_info_control: CommonBotInfoControl::old(sally_bot.my_accounting.clone()),
      sally_action_control: SallyActionControl::new(),
      order_leg_quantity_input: ValidatedInput::new(Box::new(|s| s.trim().parse::<f64>().ok())),
      order_leg_price_input: ValidatedInput::new(Box::new(|s| s.trim().parse::<f64>().ok())),
      mv_make_sally_bot: sally_bot,
      current_control_tab: ControlTab::Common,
      selected_assets: crate::dat::SelectedAssets::default(),
      sally_presets: crate::core::v_sally_preset::SallyPresets::new(),
    }
  }

  pub fn update(&mut self, message: Message) -> iced::Task<Message>
  { match message
    { Message::PressedCreateBot =>
      { log::info!("Create Bot Pressed on Sally!");
        iced::Task::done(Message::ProcessCreateBot(self.mv_make_sally_bot.clone()))
      }
      Message::ProcessCreateBot(_) =>
      { log::error!("Unreachable - ProcessCreateBot");
        iced::Task::none()
      }
      Message::CommonBotInfoMessage(msg) =>
      { self.common_bot_info_control.update(msg);
        self.mv_make_sally_bot.my_accounting = self.common_bot_info_control.common_bot_info.clone();
        iced::Task::none()
      }

      Message::SallyPresetSelected(preset) =>
      { log::info!("Applying sally preset: {:?}", preset);

        // ── 1. Apply haggle method ───────────────────────────────────
        let haggle_method = preset.to_haggle_method();
        let haggle_action = dsta::HaggleAction::AnOyVeyJew(haggle_method);
        self.common_bot_info_control.common_bot_info.haggle_action = haggle_action;
        self.mv_make_sally_bot.my_accounting = self.common_bot_info_control.common_bot_info.clone();

        // ── 2. Get current ask price from the selected FinAss ────────
        let current_ask: Option<f64> = self.selected_assets
          .selected_finass
          .as_ref()
          .and_then(|fa| fa.last_ticker())
          .map(|tk| tk.ask)
          .filter(|&ask| ask > 1e-9); // reject zero / unset prices

        // ── 3. Compute (quantity, price) from preset + context ───────
        use crate::core::v_sally_preset::SallyPresetOption::*;
        let (new_qty, new_price): (Option<f64>, Option<f64>) = match preset
        {
          // BTC nominal: qty = dollars / ask
          TinyBTC =>
          { let price = current_ask;
            let qty   = current_ask.map(|a| 0.69_f64 / a);
            (qty, price)
          }
          SmallBTC =>
          { let price = current_ask;
            let qty   = current_ask.map(|a| 1.69_f64 / a);
            (qty, price)
          }
          MediumBTC =>
          { let price = current_ask;
            let qty   = current_ask.map(|a| 10.69_f64 / a);
            (qty, price)
          }
          LargeBTC =>
          { let price = current_ask;
            let qty   = current_ask.map(|a| 100.69_f64 / a);
            (qty, price)
          }

          // Stock shares: fixed qty, price = ask
          OneShare      => (Some(1.0),   current_ask),
          TenShares     => (Some(10.0),  current_ask),
          HundredShares => (Some(100.0), current_ask),

          // Option contracts: fixed qty, price = mid if available
          OneContract =>
          { let mid_price = self.selected_assets
              .selected_finass
              .as_ref()
              .and_then(|fa| fa.last_ticker())
              .and_then(|tk|
              { if tk.bid > 1e-9 && tk.ask > 1e-9
                { Some((tk.bid + tk.ask) / 2.0)
                } else { None }
              });
            (Some(1.0), mid_price.or(current_ask))
          }
          TenContracts =>
          { let mid_price = self.selected_assets
              .selected_finass
              .as_ref()
              .and_then(|fa| fa.last_ticker())
              .and_then(|tk|
              { if tk.bid > 1e-9 && tk.ask > 1e-9
                { Some((tk.bid + tk.ask) / 2.0)
                } else { None }
              });
            (Some(10.0), mid_price.or(current_ask))
          }
        };

        // ── 4. Push quantity into input + order leg ──────────────────
        if let Some(qty) = new_qty
        { let qty_str = format!("{:.8}", qty).trim_end_matches('0').trim_end_matches('.').to_string();
          self.order_leg_quantity_input.update(
            crate::base::v_valid_any::Message::InputChanged(qty_str)
          );
          if let Some(leg) = self.mv_make_sally_bot.my_hide_order.order_legs.get_mut(0)
          { leg.quantity  = qty;
            leg.remaining = qty;
          }
        }

        // ── 5. Push price into input + order leg ─────────────────────
        if let Some(price) = new_price
        { let price_str = format!("{:.2}", price);
          self.order_leg_price_input.update(
            crate::base::v_valid_any::Message::InputChanged(price_str)
          );
          if let Some(leg) = self.mv_make_sally_bot.my_hide_order.order_legs.get_mut(0)
          { leg.price = price;
          }
        }

        // ── 6. Record the selected preset ────────────────────────────
        self.sally_presets.update(
          crate::core::v_sally_preset::Message::SelectPreset(preset)
        );

        // ── 7. Switch to Orders tab so user sees the result ──────────
        self.current_control_tab = ControlTab::Orders;

        iced::Task::none()
      }

      Message::SallyActionMessage(msg) =>
      { self.sally_action_control.update(msg);
        self.mv_make_sally_bot.my_reveal_way = self.sally_action_control.variant.clone();
        iced::Task::none()
      }
      Message::SwitchTab(tab) =>
      { log::debug!("MakeSallyV: switching to tab: {:?}", tab);
        self.current_control_tab = tab;
        iced::Task::none()
      }
      Message::EmeraldCollectionMessage(_msg) =>
      { log::debug!("MakeSallyV: forwarding emerald message to parent");
        iced::Task::none()
      }
      Message::UpdateOrderLegTicker(new_ticker) =>
      { if let Some(leg) = self.mv_make_sally_bot.my_hide_order.order_legs.get_mut(0)
        { if let dsta::FinAss::StkDeets(x) = &mut leg.buy_what
          { x.ticker_name = new_ticker;
          }
        }
        iced::Task::none()
      }
      Message::UpdateOrderLegQuantity(input) =>
      { self.order_leg_quantity_input.update(crate::base::v_valid_any::Message::InputChanged(input));
        if let Some(qty) = self.order_leg_quantity_input.parsed_value
        { if let Some(leg) = self.mv_make_sally_bot.my_hide_order.order_legs.get_mut(0)
          { leg.quantity  = qty;
            leg.remaining = qty;
          }
        }
        iced::Task::none()
      }
      Message::UpdateOrderLegPrice(input) =>
      { self.order_leg_price_input.update(crate::base::v_valid_any::Message::InputChanged(input));
        if let Some(price) = self.order_leg_price_input.parsed_value
        { if let Some(leg) = self.mv_make_sally_bot.my_hide_order.order_legs.get_mut(0)
          { leg.price = price;
          }
        }
        iced::Task::none()
      }
      Message::UpdateOrderLegQuantityBadInput(_elapsed) =>
      { if let Some(progress) = self.order_leg_quantity_input.error_flash.as_mut()
        { *progress += 0.016;
          if *progress >= 1.0
          { self.order_leg_quantity_input.error_flash = None;
          }
        }
        iced::Task::none()
      }
      Message::UpdateOrderLegPriceBadInput(_elapsed) =>
      { if let Some(progress) = self.order_leg_price_input.error_flash.as_mut()
        { *progress += 0.016;
          if *progress >= 1.0
          { self.order_leg_price_input.error_flash = None;
          }
        }
        iced::Task::none()
      }
      Message::UpdateOrderLegAction(new_action) =>
      { if let Some(leg) = self.mv_make_sally_bot.my_hide_order.order_legs.get_mut(0)
        { leg.action = new_action;
        }
        iced::Task::none()
      }
      Message::UpdateOrderType(new_type) =>
      { self.mv_make_sally_bot.my_hide_order.order_type = new_type;
        iced::Task::none()
      }
      Message::FlashError(instant) =>
      { self.order_leg_quantity_input.update(crate::base::v_valid_any::Message::FlashError(instant));
        self.order_leg_price_input.update(crate::base::v_valid_any::Message::FlashError(instant));
        iced::Task::none()
      }
    }
  }

  pub fn subscription(&self) -> iced::Subscription<Message>
  { let subs: Vec<iced::Subscription<Message>> = vec!
    [ if self.order_leg_quantity_input.error_flash.is_some()
      { iced::time::every(std::time::Duration::from_millis(16))
          .map(Message::UpdateOrderLegQuantityBadInput)
      } else
      { iced::Subscription::none()
      }
      ,
      if self.order_leg_price_input.error_flash.is_some()
      { iced::time::every(std::time::Duration::from_millis(16))
          .map(Message::UpdateOrderLegPriceBadInput)
      } else
      { iced::Subscription::none()
      }
    ];
    iced::Subscription::batch(subs)
  }

  /// Called by parent (v_dr_r) when an emerald type is selected.
  pub fn set_selected_emerald(&mut self, et: crate::dat::EmeraldTypes)
  { log::debug!("MakeSallyV: emerald selected: {}", et.label());
    self.selected_assets.selected_emerald_type = Some(et.clone());
    self.common_bot_info_control.common_bot_info.friendly_name = format!("SA.{}", et.label());
    self.common_bot_info_control.common_bot_info.tracking_tick = et.base_ticker();
    self.common_bot_info_control.sync_from_common_bot_info();
  }

  /// Called by parent (v_dr_r) when a FinAss (option chain row) is selected.
  pub fn set_selected_finass(&mut self, finass: dsta::FinAss)
  { log::debug!("MakeSallyV: finass selected: {}", finass.order_name());
    let order_name = finass.order_name();
    let mid_price: Option<f64> =
    { finass.last_ticker().and_then
      ( |tk|
        { if tk.bid > 1e-9 && tk.ask > 1e-9
          { Some((tk.bid + tk.ask) / 2.0)
          } else
          { None
          }
        }
      )
    };

    self.selected_assets.selected_finass = Some(finass.clone());
    self.common_bot_info_control.common_bot_info.friendly_name = format!("SA.F.{}", order_name);
    self.common_bot_info_control.common_bot_info.tracking_tick = order_name.clone();
    self.common_bot_info_control.sync_from_common_bot_info();

    if let Some(leg) = self.mv_make_sally_bot.my_hide_order.order_legs.get_mut(0)
    { leg.buy_what = finass;
      if let Some(mid) = mid_price
      { leg.price = mid;
        self.order_leg_price_input.value        = format!("{:.2}", mid);
        self.order_leg_price_input.parsed_value = Some(mid);
      }
    }
  }
}