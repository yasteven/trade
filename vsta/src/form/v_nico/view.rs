// trade/vsta/src/form/v_nico/view.rs 
use crate::form::v_nico::{MakeNicoV, Message, ChatDirection};
use crate::base::v_valid_any::view as validated_input_view;
use iced::widget::{button, column, container, row, scrollable, text, Column, Row, Text, TextInput, markdown};
use iced::{Element, Length, Alignment, Theme};

pub fn view(nico: &MakeNicoV) -> Element<'_, crate::Message>
{
  let mut scroll_content = Column::new()
    .width(Length::Fill)
    .spacing(10);
  
  for entry in nico.mv_chat_history.iter()
  { 
    // Use the pre-parsed markdown items from the state
    let chat_line = if entry.markdown_items.is_empty() {
      // If no items were parsed, display as plain text
      text(format!("{}", entry.content))
        .width(Length::Fill)
        .into()
    } else {
      markdown::view(&entry.markdown_items, Theme::Dark)
        .map(|_url| {
          // Handle link clicks if needed - for now just ignore them
          crate::Message::DrR(crate::lens::v_dr_r::Message::FromMakeNico(Message::ClearChat))
        })
    };
    
    scroll_content = scroll_content.push(chat_line);
  }
  
  let scrollable_chat = scrollable(scroll_content)
    .height(Length::Fill)
    .width(Length::Fill);
  
  // TextInput with on_submit for Enter key handling
  let message_input = TextInput::new(
    "Type your message here... (Shift+Enter for newline)",
    &nico.mv_message_input,
  )
  .on_input(move |s|
  { crate::Message::DrR
    ( crate::lens::v_dr_r::Message::FromMakeNico(Message::MessageInputChanged(s))
    )
  })
  .on_submit(
    crate::Message::DrR(crate::lens::v_dr_r::Message::FromMakeNico(Message::PressedSendMessage))
  )
  .padding(10);
  
  // Send button
  let send_button = button(text("Send"))
    .on_press(
      crate::Message::DrR(crate::lens::v_dr_r::Message::FromMakeNico(Message::PressedSendMessage))
    );
  
  // Copy all button
  let copy_button = button(text("Copy All"))
    .on_press(
      crate::Message::DrR(crate::lens::v_dr_r::Message::FromMakeNico(Message::CopyAllToClipboard))
    );
  
  // Clear chat button
  let clear_button = button(text("Clear Chat"))
    .on_press(
      crate::Message::DrR(crate::lens::v_dr_r::Message::FromMakeNico(Message::ClearChat))
    );
  
  // Input row with all buttons
  let input_row = Row::new()
    .push(message_input)
    .push(send_button)
    .push(copy_button)
    .push(clear_button)
    .spacing(10)
    .align_y(Alignment::Center);
  
  // Main column layout
  let main_column = Column::new()
    .width(Length::Fill)
    .height(Length::Fill)
    .spacing(10)
    .padding(20)
    .push(Text::new("NICO - AI Assistant Chat").size(24))
    .push(container(scrollable_chat).width(Length::Fill).height(Length::Fill).padding(10))
    .push(input_row);
  
  row![main_column].into()
}