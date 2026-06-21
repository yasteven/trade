
// trade/vsta/src/core/v_sally_enter_way/view.rs

use iced::widget::{Column, Row, Text, Button, PickList};
use crate::base::v_valid_any::{ValidatedInput, Message as ValidatedInputMessage};
use crate::base::v_valid_any::view as validated_input_view;
use super::state::{SallyActionControl, Message};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SallyActionType {
  RightAway,
  BidAskMid,
  MarketSide,
  TimeShifted,
}

impl std::fmt::Display for SallyActionType {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    match self {
      SallyActionType::RightAway => write!(f, "Submit Right Away"),
      SallyActionType::BidAskMid => write!(f, "Bid/Ask Mid"),
      SallyActionType::MarketSide => write!(f, "Market Side"),
      SallyActionType::TimeShifted => write!(f, "Time Shifted"),
    }
  }
}

pub fn view<'a, MessageType>(
  control: &'a SallyActionControl,
  map_message: impl Fn(Message) -> MessageType + 'a + Clone + 'static,
) -> Column<'a, MessageType>
where
  MessageType: 'a + Clone + 'static,
{
  let action_type = match control.variant {
    dsta::SallyAction::SubmitRightAwayButLikeOnlyUseForTest => SallyActionType::RightAway,
    dsta::SallyAction::SubmitWhenBidAskMidAttainsInputPrice(_) => SallyActionType::BidAskMid,
    dsta::SallyAction::SubmitWhenMarketSideReachesThisPrice(_) => SallyActionType::MarketSide,
    dsta::SallyAction::SubmitAtThisTimeUnlessItReachesPrice(_, _) => SallyActionType::TimeShifted,
  };

  let action_options = vec![
    SallyActionType::RightAway,
    SallyActionType::BidAskMid,
    SallyActionType::MarketSide,
    SallyActionType::TimeShifted,
  ];

  let action_picker = PickList::new(
    action_options,
    Some(action_type),
    {
      let map_message = map_message.clone();
      move |selected| {
        let new_variant = match selected {
          SallyActionType::RightAway => dsta::SallyAction::SubmitRightAwayButLikeOnlyUseForTest,
          SallyActionType::BidAskMid => dsta::SallyAction::SubmitWhenBidAskMidAttainsInputPrice(0.0),
          SallyActionType::MarketSide => dsta::SallyAction::SubmitWhenMarketSideReachesThisPrice(0.0),
          SallyActionType::TimeShifted => dsta::SallyAction::SubmitWhenMarketSideReachesThisPrice(0.0), // TODO: we need to make this handle the bot action time in the view: dsta::SallyAction::SubmitAtThisTimeUnlessItReachesPrice(BotActionTime, 0.0),
        };
        map_message(Message::VariantSelected(new_variant))
      }
    },
  )
  .placeholder("Select reveal strategy");

  let content = match &control.variant {
    dsta::SallyAction::SubmitRightAwayButLikeOnlyUseForTest => {
      Column::new()
        .push(Text::new("Submit order immediately (test only)").size(14))
        .spacing(8)
    }
    dsta::SallyAction::SubmitWhenBidAskMidAttainsInputPrice(current_value) => {
      let price_input = validated_input_view(
        &control.price_input,
        "Target price",
        {
          let map_message = map_message.clone();
          move |msg| match msg {
            ValidatedInputMessage::InputChanged(input) => map_message(Message::PriceInputChanged(input)),
            ValidatedInputMessage::FlashError(elapsed) => map_message(Message::FlashError(elapsed)),
            _ => map_message(Message::PriceInputChanged(String::new())),
          }
        },
      );

      Column::new()
        .push(Text::new("Submit when Bid/Ask mid attains price").size(14))
        .push(Text::new(format!("Target: ${:.2}", current_value)).size(12))
        .push(price_input)
        .spacing(8)
    }
    dsta::SallyAction::SubmitWhenMarketSideReachesThisPrice(current_value) => {
      let price_input = validated_input_view(
        &control.price_input,
        "Target price (buy: ask <=, sell: bid >=)",
        {
          let map_message = map_message.clone();
          move |msg| match msg {
            ValidatedInputMessage::InputChanged(input) => map_message(Message::PriceInputChanged(input)),
            ValidatedInputMessage::FlashError(elapsed) => map_message(Message::FlashError(elapsed)),
            _ => map_message(Message::PriceInputChanged(String::new())),
          }
        },
      );

      Column::new()
        .push(Text::new("Submit when market side reaches price").size(14))
        .push(Text::new(format!("Target: ${:.2}", current_value)).size(12))
        .push(price_input)
        .spacing(8)
    }
    dsta::SallyAction::SubmitAtThisTimeUnlessItReachesPrice(price, delta) => {
      let price_input = validated_input_view(
        &control.price_input,
        "Base price",
        {
          let map_message = map_message.clone();
          move |msg| match msg {
            ValidatedInputMessage::InputChanged(input) => map_message(Message::PriceInputChanged(input)),
            ValidatedInputMessage::FlashError(elapsed) => map_message(Message::FlashError(elapsed)),
            _ => map_message(Message::PriceInputChanged(String::new())),
          }
        },
      );

      let delta_input = validated_input_view(
        &control.delta_input,
        "Price delta (% or $)",
        {
          let map_message = map_message.clone();
          move |msg| match msg {
            ValidatedInputMessage::InputChanged(input) => map_message(Message::DeltaInputChanged(input)),
            ValidatedInputMessage::FlashError(elapsed) => map_message(Message::FlashError(elapsed)),
            _ => map_message(Message::DeltaInputChanged(String::new())),
          }
        },
      );

      Column::new()
        .push(Text::new("Submit at time with shifted price").size(14))
        .push(Text::new(format!("Base: ${:.2}, Delta: {:.2}", price, delta)).size(12))
        .push(price_input)
        .push(delta_input)
        .spacing(8)
    }
  };

  let toggle_button = Button::new(
    Text::new(if control.is_visible { "Hide Details" } else { "Show Details" })
  )
  .on_press(map_message(Message::ToggleVisibility));

  Column::new()
    .push(
      Row::new()
        .push(Text::new("Reveal Strategy:").size(16))
        .push(action_picker)
        .spacing(10)
        .align_y(iced::Alignment::Center)
    )
    .push(toggle_button)
    .push(
      if control.is_visible {
        content
      } else {
        Column::new()
      }
    )
    .spacing(10)
    .padding(10)
}
