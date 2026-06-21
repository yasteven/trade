// trade/vsta/src/core/v_option_chain/state.rs
//
// OptionChainControl: selection state ONLY.
// No data ownership — PushTickerResult lives in EmeraldCollectionControl::emeralds.
// View receives Option<&dsta::PushTickerResult> at render time.
//
// REMOVED vs old version:
//   - expiry_levels: Vec<ExpiryLevel>
//   - underlying_ticker: String
//   - last_emerald: Option<dsta::PushTickerResult>
//   - update_from_emerald()
//   - clear()  →  reset()
//   - select_call() / select_put()
//   - get_order_leg_finass()
//   - current_expiry() / visible_expiries()
//   - Message::UpdateChain / Message::ClearChain
//
// update() now takes `data: Option<&dsta::PushTickerResult>` instead of zero data args.
// ExpiryLevel stays but is a view-local helper built on the fly during render.

#[derive(Debug, Clone, PartialEq)]
pub enum OptionSide {
    Call,
    Put,
}

#[derive(Debug, Clone)]
pub struct SelectedOption {
    pub underlying_ticker: String,
    pub option_name: String,
    pub strike: f64,
    pub side: OptionSide,
    /// Only stores non-LAST ticks (where bid != ask).
    /// LAST ticks (bid == ask) are used only for display.
    pub last_ticker: Option<dsta::Ticker>,
}

#[derive(Debug, Clone)]
pub enum Message {
    SelectExpiry(usize),
    GridCellClicked {
        column_name: String,
        finass: dsta::FinAss,
    },
    ScrollExpiryUp,
    ScrollExpiryDown,
}

/// View-local helper: one expiry level built on the fly from &dsta::Derivatives.
/// NOT stored in control state — built during render only.
#[derive(Debug, Clone)]
pub struct ExpiryLevel {
    pub expire_date: chrono::DateTime<chrono::Utc>,
    pub calls: Vec<dsta::Opt>,
    pub puts: Vec<dsta::Opt>,
}

impl ExpiryLevel {
    pub fn from_derivatives(derivatives: &dsta::Derivatives) -> Self {
        ExpiryLevel {
            expire_date: derivatives.expire_time,
            calls: derivatives.option_calls.clone(),
            puts: derivatives.option_puts.clone(),
        }
    }

    pub fn strikes(&self) -> Vec<f64> {
        let mut strikes = std::collections::HashSet::new();
        for o in self.calls.iter().chain(self.puts.iter()) {
            strikes.insert((o.strike_spot * 1000.0).round() as i64);
        }
        let mut sorted: Vec<f64> = strikes.into_iter().map(|s| s as f64 / 1000.0).collect();
        sorted.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
        sorted
    }

    pub fn call_at_strike(&self, strike: f64) -> Option<&dsta::Opt> {
        self.calls.iter().find(|o| (o.strike_spot - strike).abs() < 0.001)
    }

    pub fn put_at_strike(&self, strike: f64) -> Option<&dsta::Opt> {
        self.puts.iter().find(|o| (o.strike_spot - strike).abs() < 0.001)
    }
}

/// Selection state only. Does NOT own chain data.
#[derive(Debug, Clone)]
pub struct OptionChainControl {
    pub selected_expiry_index: usize,
    pub selected_option: Option<SelectedOption>,
    pub expiry_scroll_offset: usize,
    pub expiry_visible_count: usize,
}

impl OptionChainControl {
    pub fn new() -> Self {
        OptionChainControl {
            selected_expiry_index: 0,
            selected_option: None,
            expiry_scroll_offset: 0,
            expiry_visible_count: 5,
        }
    }

    /// Replaces the old clear() — resets selection state only.
    pub fn reset(&mut self) {
        self.selected_expiry_index = 0;
        self.selected_option = None;
        self.expiry_scroll_offset = 0;
    }

    pub fn update(
        &mut self,
        message: Message,
        data: Option<&dsta::PushTickerResult>,
    ) -> iced::Task<Message> {
        match message {
            Message::SelectExpiry(index) => {
                let count = data.map(|d| d.opt_chain.len()).unwrap_or(0);
                if index < count {
                    self.selected_expiry_index = index;
                    self.selected_option = None;
                    log::debug!("Selected expiry index {}", index);
                }
                iced::Task::none()
            }

            Message::GridCellClicked { column_name, finass } => {
                log::debug!(
                    "Grid cell clicked: column={}, finass={}",
                    column_name,
                    finass.ticker_name()
                );

                if let dsta::FinAss::OptDeets(opt) = &finass {
                    let side = if column_name.starts_with("CALL") {
                        OptionSide::Call
                    } else {
                        OptionSide::Put
                    };

                    // Only store non-LAST ticks (bid != ask).
                    let last_ticker = opt.last_ticker.clone().and_then(|t| {
                        if (t.bid - t.ask).abs() > f64::EPSILON { Some(t) } else { None }
                    });

                    self.selected_option = Some(SelectedOption {
                        underlying_ticker: data
                            .map(|d| d.ass_deets.ticker_name())
                            .unwrap_or_default(),
                        option_name: opt.option_name.clone(),
                        strike: opt.strike_spot,
                        side,
                        last_ticker,
                    });

                    log::debug!(
                        "selected_option is now: {:?}",
                        self.selected_option.as_ref().map(|s| &s.option_name)
                    );
                }

                iced::Task::none()
            }

            Message::ScrollExpiryUp => {
                if self.expiry_scroll_offset > 0 {
                    self.expiry_scroll_offset -= 1;
                }
                iced::Task::none()
            }

            Message::ScrollExpiryDown => {
                let count = data.map(|d| d.opt_chain.len()).unwrap_or(0);
                let max = count.saturating_sub(self.expiry_visible_count);
                if self.expiry_scroll_offset < max {
                    self.expiry_scroll_offset += 1;
                }
                iced::Task::none()
            }
        }
    }
}

impl Default for OptionChainControl {
    fn default() -> Self {
        Self::new()
    }
}