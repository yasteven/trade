
// trade/vsta/src/core/v_sally_enter_way/state.rs

use dsta::SallyAction;
use crate::base::v_valid_any::{ValidatedInput, Message as ValidatedInputMessage};

#[derive(Debug, Clone)]
pub enum Message {
  VariantSelected(SallyAction),
  PriceInputChanged(String),
  PriceInputBadInput(iced::time::Instant),
  DeltaInputChanged(String),
  DeltaInputBadInput(iced::time::Instant),
  FlashError(iced::time::Instant),
  ToggleVisibility,
}

#[derive(Debug, Clone)]
pub struct SallyActionControl {
  pub variant: SallyAction,
  pub is_visible: bool,
  pub price_input: ValidatedInput<f64>,
  pub delta_input: ValidatedInput<f64>,
}

impl SallyActionControl {
  pub fn new() -> Self {
    SallyActionControl {
      variant: SallyAction::SubmitRightAwayButLikeOnlyUseForTest,
      is_visible: true,
      price_input: ValidatedInput::new(Box::new(|s| s.parse::<f64>().ok())),
      delta_input: ValidatedInput::new(Box::new(|s| s.parse::<f64>().ok())),
    }
  }

  pub fn update(&mut self, message: Message) {
    match message {
      Message::VariantSelected(variant) => {
        self.variant = variant;
        self.is_visible = true;

        match &self.variant {
          SallyAction::SubmitWhenBidAskMidAttainsInputPrice(val) => {
            self.price_input.value = format!("{}", val);
            self.price_input.parsed_value = Some(*val);
          }
          SallyAction::SubmitWhenMarketSideReachesThisPrice(val) => {
            self.price_input.value = format!("{}", val);
            self.price_input.parsed_value = Some(*val);
          }
          SallyAction::SubmitAtThisTimeUnlessItReachesPrice(price, delta) => // TODO: this is now: (BotActionTime, price)
          { //self.price_input.value = format!("{}", price);
            //self.price_input.parsed_value = Some(*price);
            self.delta_input.value = format!("{}", delta);
            self.delta_input.parsed_value = Some(*delta);
          }
          _ => {}
        }
      }
      Message::PriceInputChanged(input) => {
        self.price_input.update(ValidatedInputMessage::InputChanged(input));
        if let Some(value) = self.price_input.parsed_value 
        { self.variant = match &self.variant 
          { SallyAction::SubmitWhenBidAskMidAttainsInputPrice(_) => 
            {
              SallyAction::SubmitWhenBidAskMidAttainsInputPrice(value)
            }
            SallyAction::SubmitWhenMarketSideReachesThisPrice(_) => {
              SallyAction::SubmitWhenMarketSideReachesThisPrice(value)
            }
            SallyAction::SubmitAtThisTimeUnlessItReachesPrice(_, delta) => 
            { // TODO: BotAction version
              // SallyAction::SubmitAtThisTimeUnlessItReachesPrice(value, *delta)
              SallyAction::SubmitWhenMarketSideReachesThisPrice(*delta) // placeholder
            }
            other => SallyAction::SubmitRightAwayButLikeOnlyUseForTest
          };
        }
      }
      Message::DeltaInputChanged(input) => {
        self.delta_input.update(ValidatedInputMessage::InputChanged(input));
        if let Some(delta_val) = self.delta_input.parsed_value {
          if let SallyAction::SubmitAtThisTimeUnlessItReachesPrice(price, _) = &self.variant {
            self.variant = SallyAction::SubmitAtThisTimeUnlessItReachesPrice(price.clone(), delta_val);
          }
        }
      }
      Message::PriceInputBadInput(_elapsed) => {
        if let Some(progress) = self.price_input.error_flash.as_mut() {
          *progress += 0.016;
          if *progress >= 1.0 {
            self.price_input.error_flash = None;
          }
        }
      }
      Message::DeltaInputBadInput(_elapsed) => {
        if let Some(progress) = self.delta_input.error_flash.as_mut() {
          *progress += 0.016;
          if *progress >= 1.0 {
            self.delta_input.error_flash = None;
          }
        }
      }
      Message::FlashError(elapsed) => {
        self.price_input.update(ValidatedInputMessage::FlashError(elapsed));
        self.delta_input.update(ValidatedInputMessage::FlashError(elapsed));
      }
      Message::ToggleVisibility => {
        self.is_visible = !self.is_visible;
      }
    }
  }
}
