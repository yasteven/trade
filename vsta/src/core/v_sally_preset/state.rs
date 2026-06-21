
// trade/vsta/src/core/v_sally_preset/state.rs

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SallyPresetOption {
    TinyBTC,        // 
    SmallBTC,       // 
    MidBTC,         // 
    LargeBTC,       // 
    Custom,         // 
}

impl SallyPresetOption 
{
  /// Returns the target US-dollar spend for BTC presets.
  /// None for non-BTC presets.
  pub fn btc_dollar_amount(&self) -> Option<f64> {
      match self {
          SallyPresetOption::TinyBTC   => Some(0.69),
          SallyPresetOption::SmallBTC  => Some(1.0),
          SallyPresetOption::MidBTC => Some(10.0),
          _ => None,
      }
  }

  pub fn to_haggle_method(&self) -> dsta::HaggleMethod 
  { match self 
    { _ => dsta::HaggleMethod::VirtualMarketOrderWithLimitOrdThenConcede
      ( dsta::HaggleLimits 
        { retry_period: chrono::Duration::milliseconds(3000),
          delta_choice: 10.0,
          delta_is_pct: true,
          slippage_max: 250.0,
        }
      )
    }
  }
          
    pub fn display_name(&self) -> &'static str {
        match self {
            Self::TinyBTC => "Tiny",
            Self::SmallBTC => "Small",
            Self::MidBTC => "Mid",
            Self::LargeBTC => "Large",
            Self::Custom => "Custom",
        }
    }

    pub fn btc_quantity_display(&self) -> &'static str {
        match self {
            Self::TinyBTC => "0.0001 BTC",
            Self::SmallBTC => "0.001 BTC",
            Self::MidBTC => "0.01 BTC",
            Self::LargeBTC => "0.1 BTC",
            Self::Custom => "User Input",
        }
    }

    pub fn btc_quantity(&self) -> Option<f64> {
        match self {
            Self::TinyBTC => Some(0.0001),
            Self::SmallBTC => Some(0.001),
            Self::MidBTC => Some(0.01),
            Self::LargeBTC => Some(0.1),
            Self::Custom => None,
        }
    }

    pub fn all() -> &'static [Self] {
        &[Self::TinyBTC, Self::SmallBTC, Self::MidBTC, Self::LargeBTC, Self::Custom]
    }


}

#[derive(Debug, Clone)]
pub enum Message {
    SelectPreset(SallyPresetOption),
}

#[derive(Debug, Clone)]
pub struct SallyPresets {
    pub selected: Option<SallyPresetOption>,
}

impl SallyPresets {
    pub fn new() -> Self {
        SallyPresets { selected: None }
    }

    pub fn update(&mut self, message: Message) {
        match message {
            Message::SelectPreset(preset) => {
                self.selected = Some(preset);
            }
        }
    }
}

impl Default for SallyPresets {
    fn default() -> Self {
        Self::new()
    }
}