
// trade/vsta/src/lens/v_stev/view.rs

use iced::widget::{button, column, container, row, text, Button, Column, Container, Row, Text};
use iced::{Element, Length, Alignment};
use crate::lens::v_stev::{StevV, Message, StevViewType};

fn view_nico(stev: &StevV)
->  Element<'_, crate::Message>
{ log::debug!("v_stev::view_nico BEGIN!");

  // Need to extract v_nico::Message from the crate::Message that comes back
  crate::form::v_nico::view(&stev.my_nico)
    .map(|top_level_msg| {
      // The incoming message is crate::Message, we need to extract the v_nico::Message from it
      // and wrap it in Stev(FromMakeNico(...))
      match top_level_msg {
        crate::Message::DrR(crate::lens::v_dr_r::Message::FromMakeNico(nico_msg)) => {
          crate::Message::Stev(crate::lens::v_stev::Message::FromMakeNico(nico_msg))
        }
        other => other  // Pass through any other messages unchanged
      }
    })
}

fn view_amy(_stev: &StevV)
->  Column<'_, crate::Message>
{ log::debug!("v_stev::view_amy BEGIN!");
  column!
  [ text("Amy Assistant Coming Soon").size(20)
  ]
  .padding(20)
}

fn view_future(_stev: &StevV)
->  Column<'_, crate::Message>
{ log::debug!("v_stev::view_future BEGIN!");
  column!
  [ text("Future Assistants Coming Soon").size(20)
  ]
  .padding(20)
}

pub fn view(stev: &StevV) -> Element<'_, crate::Message>
{ log::trace!("v_stev::view BEGIN! Rendering Stev UI...");

  let viewpage: Element<'_, crate::Message> = match &stev.current_stev_view
  { StevViewType::Nico =>
    { log::debug!("v_stev::view rendering Nico view");
      view_nico(stev).into()
    }
    , StevViewType::Amy =>
    { log::debug!("v_stev::view rendering Amy view");
      view_amy(stev).into()
    }
    , StevViewType::Future =>
    { log::debug!("v_stev::view rendering Future view");
      view_future(stev).into()
    }
  };

  let sidebar = Column::new()
    .width(Length::Fixed(120.0))
    .height(Length::Fill)
    .align_x(Alignment::Center)
    .spacing(10)
    .push
    ( Button::new
      ( iced::widget::image::Image::new
        ( stev.icon_nico.clone()
        ).width(Length::Fixed(100.0)).height(Length::Fixed(100.0))
      ).padding(10).on_press
      ( crate::Message::Stev(Message::SwitchToNico)
      )
    )
    .push
    ( Button::new
      ( iced::widget::image::Image::new
        ( stev.icon_amy.clone()
        ).width(Length::Fixed(100.0)).height(Length::Fixed(100.0))
      ).padding(10).on_press
      ( crate::Message::Stev(Message::SwitchToAmy)
      )
    )
    .push
    ( Button::new
      ( iced::widget::image::Image::new
        ( stev.icon_future.clone()
        ).width(Length::Fixed(100.0)).height(Length::Fixed(100.0))
      ).padding(10).on_press
      ( crate::Message::Stev(Message::SwitchToFuture)
      )
    )
    .push
    ( iced::widget::Space::new()
      .width(Length::Fill).height(Length::Fill)
    )
    .push
    ( Text::new("STEV")
      .size(14)
    );

  let main_row = row![sidebar, viewpage];

  Container::new(main_row)
    .width(Length::Fill)
    .height(Length::Fill)
    .into()
}