
// trade/vsta/src/lens/v_dr_r/view.rs

use iced::{
  widget::{Column, Container, Scrollable},
  Element, Length
};
use crate::lens::v_dr_r::{DrRobotnikV, Message};

fn view_make_none(dr_r: &DrRobotnikV) 
->  iced::Element<'_, crate::Message>
{ slog::debug!(1000,"view_make_none(dr_r: &DrRobotnikV)");

  let lv1_account_0_update = crate::form::v_ttai::v_account_0_update::view(&dr_r.mv_account_0_update)
      .map(|msg| crate::Message::DrR(msg));
  let lv2_all_my_positions = crate::form::v_ttai::v_all_my_positions::view(&dr_r.mv_all_my_positions)
      .map(|msg| crate::Message::DrR(msg));
  let lv3_every_sent_order = crate::form::v_ttai::v_every_sent_order::view(&dr_r.mv_every_sent_order)
      .map(|msg| crate::Message::DrR(msg));
  let lv4_order_update_vec = crate::form::v_ttai::v_order_update_vec::view(&dr_r.mv_order_update_vec)
      .map(|msg| crate::Message::DrR(msg));
  let lv5_all_push_tickers = crate::form::v_ttai::v_all_push_tickers::view(&dr_r.mv_all_push_tickers)
      .map(|msg| crate::Message::DrR(msg));
  
  let scroll_lv_1 = Scrollable::new(
    Container::new(lv1_account_0_update)
      .height(Length::Shrink)
      .max_height(400)
      .width(Length::Fill)
  ).height(Length::FillPortion(3));
    
  let scroll_lv_2 = Scrollable::new(
    Container::new(lv2_all_my_positions)
      .height(Length::Shrink)
      .max_height(200)
      .width(Length::Fill)
  ).height(Length::FillPortion(1));
    
  let scroll_lv_3 = Scrollable::new(
    Container::new(lv3_every_sent_order)
      .height(Length::Shrink)
      .max_height(200)
      .width(Length::Fill)
  ).height(Length::FillPortion(1));
    
  let scroll_lv_4 = Scrollable::new(
    Container::new(lv4_order_update_vec)
      .height(Length::Shrink)
      .max_height(200)
      .width(Length::Fill)
  ).height(Length::FillPortion(1));
    
  let scroll_lv_5 = Scrollable::new(
    Container::new(lv5_all_push_tickers)
      .height(Length::Shrink)
      .max_height(200)
      .width(Length::Fill)
  ).height(Length::FillPortion(1));
    
  Column::new()
    .width(Length::Fill)
    .height(Length::Fill)
    .push(scroll_lv_1)
    .push(scroll_lv_2)
    .push(scroll_lv_3)
    .push(scroll_lv_4)
    .push(scroll_lv_5)
    .into()
}


fn view_make_bot_buzz(dr_r: &DrRobotnikV)
->  iced::Element<'_, crate::Message>
{ crate::form::v_buzz_make::view(&dr_r.my_make_buzz_bomber)
}

fn view_make_bot_sally(dr_r: &DrRobotnikV)
->  iced::widget::Row<'_, crate::Message>
{ crate::form::v_sally_make::view
  ( &dr_r.my_make_sally_bot,
    &dr_r.my_emerald_collection_control,
  )
}

fn view_make_bot_stealth(dr_r: &DrRobotnikV)
->  iced::widget::Row<'_, crate::Message>
{ crate::form::v_stealth_make::view(&dr_r.my_make_stealth_bot)
}

fn view_make_bot_swat(dr_r: &DrRobotnikV)
-> iced::widget::Row<'_, crate::Message> 
{
  crate::form::v_swat_make::view(
    &dr_r.my_make_swat_bot,
    &dr_r.my_emerald_collection_control,
  )
}


pub fn view(dr_r: &DrRobotnikV) -> Element<'_, crate::Message> 
{ slog::debug!(1000,"v_dr_r::view BEGIN!");
  
  // Account Information or bot creation
  let column2 = match dr_r.mv_our_tool_display
  { crate::lens::v_dr_r::ViewBotMake::MakeNone =>
    { view_make_none(dr_r)
    },
    crate::lens::v_dr_r::ViewBotMake::MakeBuzz =>
    { iced::widget::column!
      [ view_make_bot_buzz(dr_r)
      ].height(Length::Fill).into()
    },
    crate::lens::v_dr_r::ViewBotMake::MakeStealth =>
    { iced::widget::column!
      [ view_make_bot_stealth(dr_r)
      ].height(Length::Fill).into()
    }
    , crate::lens::v_dr_r::ViewBotMake::MakeSally =>
    { iced::widget::column!
      [ view_make_bot_sally(dr_r)
      ].height(Length::Fill).into()
    }
    , crate::lens::v_dr_r::ViewBotMake::MakeSwat =>
    { iced::widget::column!
      [ view_make_bot_swat(dr_r)
      ].height(Length::Fill).into()
    }
  };

  // Bot Creation
  let column1 = 
    iced::widget::column!
    [ iced::widget::Button::new
      ( iced::widget::image::Image::new
        ( dr_r.icon_dr_r.clone()
        ).width(Length::Fill).height(Length::Fill)
      )
      .width(Length::Fixed(100.0))
      .height(Length::Fixed(100.0))
      .padding(2)
      .on_press
      ( crate::Message::DrR 
        ( crate::lens::v_dr_r::Message::ViewSwitcher(crate::lens::v_dr_r::ViewBotMake::MakeNone)
        )
      )
      ,
      iced::widget::Button::new
      ( iced::widget::image::Image::new
        ( dr_r.icon_buzz.clone()
        ).width(Length::Fill).height(Length::Fill)
      ).width(Length::Fixed(100.0)).height(Length::Fixed(100.0))
      .padding(2).on_press
      ( crate::Message::DrR
        ( crate::lens::v_dr_r::Message::ViewSwitcher
          ( crate::lens::v_dr_r::ViewBotMake::MakeBuzz
          )
        )
      )
      ,
      iced::widget::Button::new
      ( iced::widget::image::Image::new
        ( dr_r.icon_stealth.clone() 
        ).width(Length::Fill).height(Length::Fill)
      ).width(Length::Fixed(100.0)).height(Length::Fixed(100.0))
      .padding(2).on_press
      ( crate::Message::DrR
        ( crate::lens::v_dr_r::Message::ViewSwitcher
          ( crate::lens::v_dr_r::ViewBotMake::MakeStealth
          )
        )
      )
      ,
      iced::widget::Button::new
      ( iced::widget::image::Image::new
        ( dr_r.icon_sally.clone()
        ).width(Length::Fill).height(Length::Fill)
      ).width(Length::Fixed(100.0)).height(Length::Fixed(100.0))
      .padding(2).on_press
      ( crate::Message::DrR
        ( crate::lens::v_dr_r::Message::ViewSwitcher
          ( crate::lens::v_dr_r::ViewBotMake::MakeSally
          )
        )
      )
      ,
      iced::widget::Button::new
      ( iced::widget::image::Image::new
        ( dr_r.icon_swat.clone()
        ).width(Length::Fill).height(Length::Fill)
      ).width(Length::Fixed(100.0)).height(Length::Fixed(100.0)) 
      .padding(2).on_press
      ( crate::Message::DrR
        ( crate::lens::v_dr_r::Message::ViewSwitcher
          ( crate::lens::v_dr_r::ViewBotMake::MakeSwat
          )
        )
      )
      ,
    ]
    .spacing(8)
    //.height(Length::Fill)
    .push(iced::widget::Text::new("MAKE"))
    ;

  // Wrap column1 in a Scrollable
  let scrollable_column1 = Scrollable::new(column1)
  .height(Length::Fill);

  iced::widget::stack!
  [ iced::widget::image::Image::new
    ( match &dr_r.mv_our_tool_display
      { 
        crate::lens::ViewBotMake::MakeBuzz => {     dr_r.icon_buzz.clone() }
        crate::lens::ViewBotMake::MakeStealth => {  dr_r.icon_stealth.clone() }
        crate::lens::ViewBotMake::MakeSally => {    dr_r.icon_sally.clone() }
        crate::lens::ViewBotMake::MakeSwat => {     dr_r.icon_swat.clone() }
        crate::lens::ViewBotMake::MakeNone => {     dr_r.icon_dr_r.clone() }

      } 
    ).width(Length::Fill).height(Length::Fill)
  , Container::new
    ( iced::widget::column!
      [ { match &dr_r.mv_connection_state
          { crate::lens::v_dr_r::ViewBotTalk::Startup 
          => iced::widget::text!("{}",format!("Connection Startup!")) 
          , crate::lens::v_dr_r::ViewBotTalk::Connecting(who) 
          => iced::widget::text!("{}",format!("Connecting to {}", who)) 
          , crate::lens::v_dr_r::ViewBotTalk::Connected 
          => iced::widget::text!("{}",format!("CONNECTED!")) 
          , crate::lens::v_dr_r::ViewBotTalk::Failed(why)
          => iced::widget::text!("{}",format!("COMMS FAILURE: {}", why)) 
          }
        }
      , iced::widget::row![scrollable_column1, column2]
      ]
    ).width(Length::Fill).height(Length::Fill).padding(2)
  ].width(Length::Fill).height(Length::Fill)
  .into()
}


