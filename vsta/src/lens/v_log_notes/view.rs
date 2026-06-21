
// trade/vsta/src/lens/v_log_notes/view.rs

use chrono::prelude::*;
use iced::widget::{button, column, container, row, scrollable, text, Column, Row, Space, Button};

pub fn view(log_notes: &crate::lens::v_log_notes::LogNotesV)
->  iced::Element<'_, crate::Message>
{ slog::debug!(1000,"view_log_notes(log_notes: &LogNotesV)");

  let mut scroll_content = Column::new()
    .width(iced::Length::Fill)
    .spacing(4);

  // Newest on top — remove .rev() if you want oldest first
  for entry in log_notes.mv_log_entries.iter().rev()
  { let direction_str = if entry.direction == crate::lens::v_log_notes::LogDirection::Sent
    { "▶ SENT"
    }
    else
    { "◀ RECV"
    };
    let time_str = entry.timestamp.format("%H:%M:%S");
    let log_line = format!("[{}] {} | {}", time_str, direction_str, entry.note);

    scroll_content = scroll_content.push
    ( text(log_line)
        .size(15)
    );
  }

  let clear_button = Button::new
  ( text("Clear Log")
  )
  .on_press
  ( crate::Message::DrR
    ( crate::lens::v_dr_r::Message::ClearLogNotes
    )
  );

  iced::widget::row!
  [   Space::new().width(iced::Length::Fixed(20.0))
    , column!
      [ Space::new().height(iced::Length::Fixed(10.0))
        , Row::new()
          .width(iced::Length::Fill)
          .push(text("SNIVELY MESSAGE LOG").size(20))
          .push(Space::new().width(iced::Length::Fill))
          .push(clear_button)
        , Space::new().height(iced::Length::Fixed(10.0))
        , container
          ( scrollable
            ( scroll_content
            )
            .height(iced::Length::Fill)
            .width(iced::Length::Fill)
          )
          .width(iced::Length::Fill)
          .height(iced::Length::Fill)
          .padding(10)
          .style(iced::widget::container::bordered_box)
        , Space::new().height(iced::Length::Fixed(20.0))
      ]
  ].into()
}