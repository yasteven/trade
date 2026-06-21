
// trade/vsta/src/base/v_usa_times/state.rs

use super::super::v_combo_box::ComboBoxState;
use dsta::UsaMarketTimes;

#[derive(Debug, Clone)]
pub struct UsaMarketTimesComboBox {
    pub combobox: ComboBoxState<UsaMarketTimes>,
}

impl UsaMarketTimesComboBox {
    pub fn new() -> Self {
        let options = vec![
            UsaMarketTimes::PowerEnds,
            UsaMarketTimes::ItsClosed,
            UsaMarketTimes::Hurry5sec,
            UsaMarketTimes::TminusTen,
            UsaMarketTimes::Tminus30s,
            UsaMarketTimes::Tminus60s,
            UsaMarketTimes::DeadShort,
            UsaMarketTimes::PowerFive,
            UsaMarketTimes::PowerEasy,
            UsaMarketTimes::PowerHour,
            UsaMarketTimes::FedSpeach,
            UsaMarketTimes::FedMinute,
            UsaMarketTimes::TradeTime,
            UsaMarketTimes::LunchTime,
            UsaMarketTimes::EuroClose,
            UsaMarketTimes::DoneTrend,
            UsaMarketTimes::OpenChaos,
            UsaMarketTimes::ResetTtai,
        ];

        UsaMarketTimesComboBox {
            combobox: ComboBoxState::new(options),
        }
    }

    pub fn update(&mut self, selected: UsaMarketTimes) {
        self.combobox.update(selected);
    }
}
