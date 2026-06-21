
// trade/vsta/src/lens/v_stev/state.rs

#[derive(Debug, Clone)]
pub enum StevViewType
{ Nico
  , Amy
  , Future
}

impl std::fmt::Display for StevViewType
{ fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result
  { match self
    { StevViewType::Nico => write!(f, "Nico")
      , StevViewType::Amy => write!(f, "Amy")
      , StevViewType::Future => write!(f, "Future")
    }
  }
}

#[derive(Debug, Clone)]
pub enum Message
{ SwitchToNico
  , SwitchToAmy
  , SwitchToFuture
  , FromMakeNico(crate::form::v_nico::Message)
}

pub struct StevV {
    pub current_stev_view: StevViewType,
    pub my_nico: crate::form::v_nico::MakeNicoV,
    pub icon_nico: iced::advanced::image::Handle,
    pub icon_amy: iced::advanced::image::Handle,
    pub icon_future: iced::advanced::image::Handle,
}

impl StevV {
    pub fn new() -> Self {
        let icon_nico = iced::advanced::image::Handle::from_bytes(
            include_bytes!("../../../jpg/nico_icon.jpg").as_slice(),
        );
        let icon_amy = iced::advanced::image::Handle::from_bytes(
            include_bytes!("../../../jpg/amy_icon.jpg").as_slice(),
        );
        let icon_future = iced::advanced::image::Handle::from_bytes(
            include_bytes!("../../../jpg/future_icon.jpg").as_slice(),
        );

        StevV {
            current_stev_view: StevViewType::Nico,
            my_nico: crate::form::v_nico::MakeNicoV::new(),
            icon_nico,
            icon_amy,
            icon_future,
        }
    }

    pub fn with_aimd_backend(mut self, backend: std::sync::Arc<aimd::AllmBackend>) -> Self {
        self.my_nico = self.my_nico.with_aimd_backend(backend);
        self
    }

    pub fn update(&mut self, message: Message) -> iced::Task<Message> {
        match message {
            Message::SwitchToNico => {
                log::debug!("StevV::update switching to Nico");
                self.current_stev_view = StevViewType::Nico;
                iced::Task::none()
            }
            Message::SwitchToAmy => {
                log::debug!("StevV::update switching to Amy");
                self.current_stev_view = StevViewType::Amy;
                iced::Task::none()
            }
            Message::SwitchToFuture => {
                log::debug!("StevV::update switching to Future");
                self.current_stev_view = StevViewType::Future;
                iced::Task::none()
            }
            Message::FromMakeNico(nico_msg) => {
                log::debug!("StevV::update received FromMakeNico message: {:?}", nico_msg);
                self.my_nico.update(nico_msg).map(Message::FromMakeNico)
            }
        }
    }
}
