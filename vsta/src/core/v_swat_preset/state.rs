
// trade/vsta/src/core/v_swat_preset/state.rs

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SwatPresetOption {
    StingyHaggle,
    NoHaggle,
    AggressiveHaggle,
}

impl SwatPresetOption {
    pub fn display_name(&self) -> &'static str {
        match self {
            Self::StingyHaggle => "Stingy Haggle",
            Self::NoHaggle => "No Haggle",
            Self::AggressiveHaggle => "Aggressive Haggle",
        }
    }

    pub fn description(&self) -> &'static str {
        match self {
            Self::StingyHaggle => "Low retry, small deltas",
            Self::NoHaggle => "Single attempt, fixed price",
            Self::AggressiveHaggle => "High retry, large deltas",
        }
    }

    pub fn all() -> &'static [Self] {
        &[Self::StingyHaggle, Self::NoHaggle, Self::AggressiveHaggle]
    }

    pub fn to_haggle_method(&self) -> dsta::HaggleMethod {
        match self {
            Self::StingyHaggle => {
                dsta::HaggleMethod::VirtualMarketOrderWithLimitOrdThenConcede(
                    dsta::HaggleLimits {
                        retry_period: chrono::Duration::milliseconds(100),
                        delta_choice: 10.0,
                        delta_is_pct: true,
                        slippage_max: 25.0,
                    }
                )
            }
            Self::NoHaggle => {
                dsta::HaggleMethod::MinimumOfMidpointOrTheoreticalThenConcede(
                    dsta::HaggleLimits {
                        retry_period: chrono::Duration::milliseconds(50),
                        delta_choice: 0.0,
                        delta_is_pct: false,
                        slippage_max: 0.0,
                    }
                )
            }
            Self::AggressiveHaggle => {
                dsta::HaggleMethod::CheapLimitOrderThenIncrementDeltaUntilMax(
                    dsta::HaggleLimits {
                        retry_period: chrono::Duration::milliseconds(500),
                        delta_choice: 50.0,
                        delta_is_pct: true,
                        slippage_max: 100.0,
                    }
                )
            }
        }
    }
}

#[derive(Debug, Clone)]
pub enum Message {
    SelectPreset(SwatPresetOption),
}

#[derive(Debug, Clone)]
pub struct SwatPresets {
    pub selected: Option<SwatPresetOption>,
}

impl SwatPresets {
    pub fn new() -> Self {
        SwatPresets { selected: None }
    }

    pub fn update(&mut self, message: Message) {
        match message {
            Message::SelectPreset(preset) => {
                self.selected = Some(preset);
            }
        }
    }
}

impl Default for SwatPresets {
    fn default() -> Self {
        Self::new()
    }
}