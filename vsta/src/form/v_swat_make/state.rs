// trade/vsta/src/form/v_swat_make/state.rs - SIMPLIFIED

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ControlTab {
    Assets,
    SwatSettings,
}

#[derive(Debug, Clone)]
pub enum Message 
{ PressedCreateBot,
  ProcessCreateBot(dsta::MakeSwatBotsGo),
  CommonBotInfoMessage(crate::core::v_bot_common_info::Message),
  SwatPresetSelected(crate::core::v_swat_preset::SwatPresetOption),
  SwitchTab(ControlTab),
  EmeraldCollectionMessage
  ( crate::core::v_emerald_collection::Message
  ),
}

use crate::dat::*;

#[derive(Debug, Clone)]
pub struct MakeSwatV {
    pub common_bot_info_control: crate::core::CommonBotInfoControl,
    pub swat_presets: crate::core::v_swat_preset::SwatPresets,
    pub current_control_tab: ControlTab,
    pub selected_assets: SelectedAssets,
}

impl MakeSwatV {
    pub fn new() -> Self {
        MakeSwatV {
            common_bot_info_control: crate::core::CommonBotInfoControl::new(),
            swat_presets: crate::core::v_swat_preset::SwatPresets::new(),
            current_control_tab: ControlTab::Assets,
            selected_assets: SelectedAssets::default(),
        }
    }

    pub fn update(&mut self, message: Message) -> iced::Task<Message> {
        match message {
            Message::PressedCreateBot => {
                log::info!("pressed create swat bot");
                iced::Task::done(Message::ProcessCreateBot(
                    dsta::MakeSwatBotsGo {
                        my_accounting: self.common_bot_info_control.common_bot_info.clone(),
                    }
                ))
            }

            Message::ProcessCreateBot(_) => {
                log::error!("Unreachable - ProcessCreateBot");
                iced::Task::none()
            }

            Message::CommonBotInfoMessage(msg) => {
                self.common_bot_info_control.update(msg);
                iced::Task::none()
            }

            Message::SwatPresetSelected(preset) => {
                log::info!("Applying swat preset: {:?}", preset);
                self.swat_presets.update(
                    crate::core::v_swat_preset::Message::SelectPreset(preset)
                );

                // Update haggle settings
                let haggle_method = preset.to_haggle_method();
                let haggle_action = dsta::HaggleAction::AnOyVeyJew(haggle_method);
                self.common_bot_info_control.common_bot_info.haggle_action = haggle_action;

                iced::Task::none()
            }

            Message::SwitchTab(tab) => {
                log::debug!("Switching to tab: {:?}", tab);
                self.current_control_tab = tab;
                iced::Task::none()
            }

            Message::EmeraldCollectionMessage(msg) => {
                log::debug!("MakeSwatV: received EmeraldCollectionMessage (forwarded by parent)");
                // Parent (v_dr_r) already handled the update to emerald control
                // and synced to this view via set_selected_emerald/set_selected_finass
                // So we just return none here - the view will be updated by parent
                iced::Task::none()
            }
        }
    }

    pub fn subscription(&self) -> iced::Subscription<Message> {
        iced::Subscription::none()
    }

    /// Called by parent (v_dr_r) when an emerald type is selected.
    pub fn set_selected_emerald(&mut self, et: crate::dat::EmeraldTypes) 
    {
      log::debug!("MakeSwatV: emerald selected: {}", et.label());
      self.selected_assets.selected_emerald_type = Some(et.clone());
      self.common_bot_info_control.common_bot_info.friendly_name =
          format!("SB.{}", et.label());
      self.common_bot_info_control.common_bot_info.tracking_tick =
          et.base_ticker();
      self.common_bot_info_control.sync_from_common_bot_info();
    }

    /// Called by parent (v_dr_r) when a FinAss (option chain row) is selected.
    pub fn set_selected_finass(&mut self, finass: dsta::FinAss) {
        log::debug!("MakeSwatV: finass selected: {}", finass.ticker_name());
        self.selected_assets.selected_finass = Some(finass.clone());
        self.common_bot_info_control.common_bot_info.friendly_name =
            format!("SB.E.{}", finass.ticker_name());
        self.common_bot_info_control.common_bot_info.tracking_tick =
            finass.ticker_name();
        self.common_bot_info_control.sync_from_common_bot_info();
    }


    /// Called by parent (v_dr_r) when a preset is applied
    pub fn apply_swat_preset(&mut self, preset: crate::core::v_swat_preset::SwatPresetOption) {
        log::info!("MakeSwatV: applying preset: {:?}", preset);
        let haggle_method = preset.to_haggle_method();
        let haggle_action = dsta::HaggleAction::AnOyVeyJew(haggle_method);
        self.common_bot_info_control.common_bot_info.haggle_action = haggle_action;
        self.common_bot_info_control.haggle_action_control.sync_from_dsta(&haggle_action);
    }
}