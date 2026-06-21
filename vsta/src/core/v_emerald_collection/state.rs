// trade/vsta/src/core/v_emerald_collection/state.rs
//
// CHANGED vs old version:
//   - Added `option_tick_index` field + rebuild_option_index() for O(1) option tick routing
//   - Added `current_push_result()` — single read path for views
//   - UpdateEmerald: no longer calls option_chain.update_from_emerald(); calls rebuild_option_index() instead
//   - UpdateTickers: two-phase lookup — underlying first, then option index
//   - SelectEmeraldType: calls option_chain.reset() instead of update_from_emerald()/clear()
//   - ClearEmerald: calls option_chain.reset() instead of clear()
//   - OptionChainMessage: passes current_push_result() into option_chain.update()
//   - get_selected_finass(): walks emeralds directly instead of via option_chain.get_order_leg_finass()
//   - option_chain.update() now takes (msg, data) — matches new state.rs signature

use crate::core::v_option_chain::{OptionChainControl, Message as OptionChainMessage};
use dsta::PushTickerResult;
use iced::widget::image;
use crate::dat::{EmeraldColor, EmeraldTypes};

// ============================================================
// Messages
// ============================================================

#[derive(Debug, Clone)]
pub enum Message {
    SelectEmeraldType(EmeraldTypes),
    OptionChainMessage(crate::core::v_option_chain::Message),
    UpdateTickers(dsta::Ticker),
    UpdateEmerald(PushTickerResult),
    ClearEmerald,
}

// ============================================================
// BotTypeContext
// ============================================================

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BotTypeContext {
    SwatBot,
    BuzzBot,
    StealthBot,
    SallyBot,
}

// ============================================================
// EmeraldCollectionControl
// ============================================================

#[derive(Debug, Clone)]
pub struct EmeraldCollectionControl {
    pub selected_type: Option<EmeraldTypes>,

    /// Maps base_ticker() string → (EmeraldTypes, Option<PushTickerResult>).
    ///   None  = slot pre-populated at startup, no data received yet
    ///   Some  = data received from backend, option chain available
    pub emeralds: std::collections::HashMap<String, (EmeraldTypes, Option<dsta::PushTickerResult>)>,

    /// Selection-state-only view for the option chain.
    pub option_chain: OptionChainControl,

    pub display_context: Option<BotTypeContext>,

    pub super_icon_handles: std::collections::HashMap<EmeraldColor, image::Handle>,
    pub chaos_icon_handles: std::collections::HashMap<EmeraldColor, image::Handle>,
    pub master_icon_handle: image::Handle,
    pub supers_icon_handle: image::Handle,
    pub chaoss_icon_handle: image::Handle,
    pub jewels_icon_handle: image::Handle,
    pub default_icon_handle: image::Handle,

    /// Last grid cell click (column_name, finass) — for parent to detect.
    pub last_grid_click: Option<(String, dsta::FinAss)>,

    /// O(1) reverse index: option_name → (emerald_key, deriv_idx, is_call, opt_idx).
    /// Rebuilt on every UpdateEmerald. Private — accessed only through UpdateTickers.
    option_tick_index: std::collections::HashMap<String, (String, usize, bool, usize)>,
}

impl EmeraldCollectionControl {
    pub fn new() -> Self {
        let mut emeralds = std::collections::HashMap::new();
        let mut super_icon_handles = std::collections::HashMap::new();
        let mut chaos_icon_handles = std::collections::HashMap::new();

        emeralds.insert("/ES".to_string(), (EmeraldTypes::MasterEmerald, None));

        for color in EmeraldColor::all_colors() {
            emeralds.insert(
                color.super_ticker().to_string(),
                (EmeraldTypes::SuperEmerald(*color), None),
            );
            emeralds.insert(
                color.chaos_ticker().to_string(),
                (EmeraldTypes::ChaosEmerald(*color), None),
            );
            super_icon_handles.insert(*color, color.super_icon_handle());
            chaos_icon_handles.insert(*color, color.chaos_icon_handle());
        }

        let master_icon_handle  = image::Handle::from_bytes(include_bytes!("../../../jpg/master_0ES.jpg").to_vec());
        let supers_icon_handle  = image::Handle::from_bytes(include_bytes!("../../../jpg/hidden_pallace.jpg").to_vec());
        let chaoss_icon_handle  = image::Handle::from_bytes(include_bytes!("../../../jpg/emerald_hill.jpg").to_vec());
        let jewels_icon_handle  = image::Handle::from_bytes(include_bytes!("../../../jpg/random_jewels.jpg").to_vec());
        let default_icon_handle = image::Handle::from_bytes(include_bytes!("../../../jpg/chaos_coral.jpg").to_vec());

        EmeraldCollectionControl {
            selected_type: Some(EmeraldTypes::MasterEmerald),
            emeralds,
            option_chain: OptionChainControl::new(),
            display_context: Some(BotTypeContext::SwatBot),
            super_icon_handles,
            chaos_icon_handles,
            master_icon_handle,
            supers_icon_handle,
            chaoss_icon_handle,
            jewels_icon_handle,
            default_icon_handle,
            last_grid_click: None,
            option_tick_index: std::collections::HashMap::new(),
        }
    }

    // --------------------------------------------------------
    // Display context / filter  (unchanged)
    // --------------------------------------------------------

    pub fn set_display_context(&mut self, context: BotTypeContext) {
        log::debug!("EmeraldCollectionControl: switching display context to {:?}", context);
        self.display_context = Some(context);
    }

    pub fn should_show_emerald_type(&self, et: &EmeraldTypes) -> bool {
        if let Some(dc) = self.display_context {
            if dc == BotTypeContext::SwatBot {
                return true;
            }
        }
        let ticker = Self::normalize_emerald_key(&et.base_ticker());
        self.emeralds
            .get(&ticker)
            .and_then(|(_, opt)| opt.as_ref())
            .is_some()
    }

    // --------------------------------------------------------
    // Accessors
    // --------------------------------------------------------

    pub fn current_ticker(&self) -> Option<String> {
        self.selected_type.as_ref().map(|et| et.base_ticker())
    }

    /// NEW: borrow the PushTickerResult for the currently selected emerald.
    /// This is the single read path used by the view layer and OptionChainMessage handler.
    pub fn current_push_result(&self) -> Option<&dsta::PushTickerResult> {
        self.selected_type.as_ref().and_then(|et| {
            let key = Self::normalize_emerald_key(&et.base_ticker());
            self.emeralds.get(&key).and_then(|(_, pr)| pr.as_ref())
        })
    }

    pub fn price_line_for(&self, et: &EmeraldTypes) -> String {
        let ticker = Self::normalize_emerald_key(&et.base_ticker());
        if let Some((_, Some(push_result))) = self.emeralds.get(&ticker) {
            if let Some(tk) = push_result.ass_deets.last_ticker() {
                return format!("B:{:.2} A:{:.2}", tk.bid, tk.ask);
            }
        }
        "—".to_string()
    }

    /// CHANGED: walks emeralds directly instead of delegating to
    /// option_chain.get_order_leg_finass() (which used the now-removed last_emerald field).
    pub fn get_selected_finass(&self) -> Option<dsta::FinAss> {
        log::trace!("[TVSCVECGSF] - v_emerald_collection::get_selected_finass called");

        // If a specific option row is selected, find its live copy from emeralds.
        if let Some(selected) = &self.option_chain.selected_option {
            if let Some(pr) = self.current_push_result() {
                for deriv in &pr.opt_chain {
                    for opt in deriv.option_calls.iter().chain(deriv.option_puts.iter()) {
                        if opt.option_name == selected.option_name {
                            return Some(dsta::FinAss::OptDeets(opt.clone()));
                        }
                    }
                }
            }
        }

        // Fallback: the underlying asset itself.
        self.current_push_result().map(|pr| pr.ass_deets.clone())
    }

    // --------------------------------------------------------
    // Internal helpers
    // --------------------------------------------------------

    fn normalize_emerald_key(ticker: &str) -> String {
        if ticker.starts_with("/ES") {
            "/ES".to_string()
        } else {
            ticker.to_string()
        }
    }

    /// NEW: rebuild option_name → (emerald_key, deriv_idx, is_call, opt_idx) index.
    /// Called once after every UpdateEmerald (amortised cost).
    fn rebuild_option_index(&mut self) {
        self.option_tick_index.clear();
        for (key, (_, maybe_result)) in &self.emeralds {
            if let Some(pr) = maybe_result {
                for (di, deriv) in pr.opt_chain.iter().enumerate() {
                    for (oi, opt) in deriv.option_calls.iter().enumerate() {
                        self.option_tick_index.insert(
                            opt.option_name.clone(),
                            (key.clone(), di, true, oi),
                        );
                    }
                    for (oi, opt) in deriv.option_puts.iter().enumerate() {
                        self.option_tick_index.insert(
                            opt.option_name.clone(),
                            (key.clone(), di, false, oi),
                        );
                    }
                }
            }
        }
        log::debug!(
            "EmeraldCollectionControl: option index rebuilt ({} entries)",
            self.option_tick_index.len()
        );
    }

    // --------------------------------------------------------
    // Update
    // --------------------------------------------------------

    pub fn update(&mut self, message: Message) -> iced::Task<Message> {
        match message {
            // CHANGED: reset() instead of update_from_emerald()/clear()
            Message::SelectEmeraldType(et) => {
                log::debug!("EmeraldCollectionControl: selected: {}", et.label());
                self.selected_type = Some(et);
                self.option_chain.reset();
                iced::Task::none()
            }

            // CHANGED: passes current_push_result() into option_chain.update()
            Message::OptionChainMessage(msg) => {
                match msg {
                    OptionChainMessage::GridCellClicked { ref column_name, ref finass } => {
                        log::info!(
                            "EmeraldCollectionControl: grid cell clicked - column={}, finass={}",
                            column_name,
                            finass.ticker_name()
                        );
                        self.last_grid_click = Some((column_name.clone(), finass.clone()));
                        let data = self.current_push_result();
                        // SAFETY: data borrow ends before we call update (immutable vs mutable),
                        // but we need to clone the data reference approach.
                        // Work around the borrow checker by cloning the key lookup:
                        let data_key = self.selected_type.as_ref()
                            .map(|et| Self::normalize_emerald_key(&et.base_ticker()));
                        // Re-borrow inside a block to satisfy the borrow checker:
                        let task = {
                            let data = data_key.as_deref().and_then(|k| {
                                self.emeralds.get(k).and_then(|(_, pr)| pr.as_ref())
                            });
                            self.option_chain.update(msg, data)
                        };
                        task.map(Message::OptionChainMessage)
                    }
                    other => {
                        let data_key = self.selected_type.as_ref()
                            .map(|et| Self::normalize_emerald_key(&et.base_ticker()));
                        let task = {
                            let data = data_key.as_deref().and_then(|k| {
                                self.emeralds.get(k).and_then(|(_, pr)| pr.as_ref())
                            });
                            self.option_chain.update(other, data)
                        };
                        task.map(Message::OptionChainMessage)
                    }
                }
            }

            // CHANGED: stores data + rebuilds index. No longer calls update_from_emerald().
            Message::UpdateEmerald(push_result) => {
                let ticker = push_result.ass_deets.ticker_name();
                let key = Self::normalize_emerald_key(&ticker);
                log::debug!("EmeraldCollectionControl: received data for '{}'", key);

                let et_tag = self
                    .emeralds
                    .get(&key)
                    .map(|(et, _)| et.clone())
                    .unwrap_or_else(|| EmeraldTypes::RandomJewels(key.clone()));

                self.emeralds.insert(key, (et_tag, Some(push_result)));
                self.rebuild_option_index();
                iced::Task::none()
            }

            // CHANGED: two-phase lookup — underlying first, then option index.
            Message::UpdateTickers(tick) => {
                let key = Self::normalize_emerald_key(&tick.name);

                // Phase 1: is this a ticker for an underlying we hold?
                if let Some((_, Some(pr))) = self.emeralds.get_mut(&key) {
                    pr.ass_deets.set_ticker(tick);
                    return iced::Task::none();
                }

                // Phase 2: is it an option we indexed?
                if let Some((emerald_key, di, is_call, oi)) =
                    self.option_tick_index.get(&tick.name).cloned()
                {
                    if let Some((_, Some(pr))) = self.emeralds.get_mut(&emerald_key) {
                        let opt = if is_call {
                            pr.opt_chain.get_mut(di).and_then(|d| d.option_calls.get_mut(oi))
                        } else {
                            pr.opt_chain.get_mut(di).and_then(|d| d.option_puts.get_mut(oi))
                        };
                        if let Some(opt) = opt {
                            opt.last_ticker = Some(tick);
                        }
                    }
                } else {
                    log::warn!(
                        "EmeraldCollectionControl: ticker update for '{}' but no slot or option found",
                        tick.name
                    );
                }

                iced::Task::none()
            }

            // CHANGED: reset() instead of clear()
            Message::ClearEmerald => {
                log::debug!("EmeraldCollectionControl: cleared selection");
                self.selected_type = None;
                self.option_chain.reset();
                iced::Task::none()
            }
        }
    }
}

impl Default for EmeraldCollectionControl {
    fn default() -> Self {
        Self::new()
    }
}