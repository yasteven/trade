
// trade/vsta/src/core/v_haggle_method/state.rs

use dsta::HaggleMethod;
use crate::core::v_haggle_limits::{HaggleLimitsControl, Message as HaggleLimitsMessage};

#[derive(Debug, Clone)]
pub enum Message {
    MethodVariantSelected(HaggleMethodType),
    LimitsMessage(HaggleLimitsMessage),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HaggleMethodType {
    MinimumOfMidpointOrTheoreticalThenConcede,
    VirtualMarketOrderWithLimitOrdThenConcede,
    CheapLimitOrderThenIncrementDeltaUntilMax,
}

impl std::fmt::Display for HaggleMethodType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            HaggleMethodType::MinimumOfMidpointOrTheoreticalThenConcede => {
                write!(f, "Min Midpoint/Theoretical + Concede")
            }
            HaggleMethodType::VirtualMarketOrderWithLimitOrdThenConcede => {
                write!(f, "Virtual Market + Limit + Concede")
            }
            HaggleMethodType::CheapLimitOrderThenIncrementDeltaUntilMax => {
                write!(f, "Cheap Limit + Increment Delta")
            }
        }
    }
}

#[derive(Debug, Clone)]
pub struct HaggleMethodControl {
    pub method_type: HaggleMethodType,
    pub limits_control: HaggleLimitsControl,
}

impl HaggleMethodControl {
    pub fn new() -> Self {
        HaggleMethodControl {
            method_type: HaggleMethodType::MinimumOfMidpointOrTheoreticalThenConcede,
            limits_control: HaggleLimitsControl::new(),
        }
    }

    pub fn from_dsta(method: &HaggleMethod) -> Self {
        let method_type = match method {
            HaggleMethod::MinimumOfMidpointOrTheoreticalThenConcede(_) => {
                HaggleMethodType::MinimumOfMidpointOrTheoreticalThenConcede
            }
            HaggleMethod::VirtualMarketOrderWithLimitOrdThenConcede(_) => {
                HaggleMethodType::VirtualMarketOrderWithLimitOrdThenConcede
            }
            HaggleMethod::CheapLimitOrderThenIncrementDeltaUntilMax(_) => {
                HaggleMethodType::CheapLimitOrderThenIncrementDeltaUntilMax
            }
        };

        let limits = match method {
            HaggleMethod::MinimumOfMidpointOrTheoreticalThenConcede(l) => l,
            HaggleMethod::VirtualMarketOrderWithLimitOrdThenConcede(l) => l,
            HaggleMethod::CheapLimitOrderThenIncrementDeltaUntilMax(l) => l,
        };

        HaggleMethodControl {
            method_type,
            limits_control: HaggleLimitsControl::from_dsta(limits),
        }
    }


    pub fn to_dsta(&self) -> HaggleMethod {
        let limits = self.limits_control.to_dsta();
        match self.method_type {
            HaggleMethodType::MinimumOfMidpointOrTheoreticalThenConcede => {
                HaggleMethod::MinimumOfMidpointOrTheoreticalThenConcede(limits)
            }
            HaggleMethodType::VirtualMarketOrderWithLimitOrdThenConcede => {
                HaggleMethod::VirtualMarketOrderWithLimitOrdThenConcede(limits)
            }
            HaggleMethodType::CheapLimitOrderThenIncrementDeltaUntilMax => {
                HaggleMethod::CheapLimitOrderThenIncrementDeltaUntilMax(limits)
            }
        }
    }

    pub fn sync_from_dsta(&mut self, method: &HaggleMethod) {
        // Update method type
        self.method_type = match method {
            HaggleMethod::MinimumOfMidpointOrTheoreticalThenConcede(_) => {
                HaggleMethodType::MinimumOfMidpointOrTheoreticalThenConcede
            }
            HaggleMethod::VirtualMarketOrderWithLimitOrdThenConcede(_) => {
                HaggleMethodType::VirtualMarketOrderWithLimitOrdThenConcede
            }
            HaggleMethod::CheapLimitOrderThenIncrementDeltaUntilMax(_) => {
                HaggleMethodType::CheapLimitOrderThenIncrementDeltaUntilMax
            }
        };
        
        // Extract limits and sync limits control
        let limits = match method {
            HaggleMethod::MinimumOfMidpointOrTheoreticalThenConcede(l) => l,
            HaggleMethod::VirtualMarketOrderWithLimitOrdThenConcede(l) => l,
            HaggleMethod::CheapLimitOrderThenIncrementDeltaUntilMax(l) => l,
        };
        self.limits_control.sync_from_dsta(limits);
    }
    

    pub fn update(&mut self, message: Message) {
        match message {
            Message::MethodVariantSelected(variant) => {
                self.method_type = variant;
            }
            Message::LimitsMessage(msg) => {
                self.limits_control.update(msg);
            }
        }
    }
}
