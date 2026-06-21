
// trade/vsta/src/lens/v_log_notes/state.rs

#[derive(Debug, Clone)]
pub enum Message 
{   AddSentNote(String)
  , AddRecvNote(String)
  , ClearAllNotes
}

#[derive(Debug, Clone)]
pub struct LogEntry
{   pub direction: LogDirection
  , pub note: String
  , pub timestamp: chrono::DateTime<chrono::Utc>
}

#[derive(Debug, Clone, PartialEq)]
pub enum LogDirection
{   Sent
  , Received
}

pub struct LogNotesV
{   pub mv_log_entries: Vec<LogEntry>
  , pub mv_max_entries: usize
}

impl LogNotesV
{
  pub fn new() -> Self 
  { LogNotesV 
    {   mv_log_entries: Vec::new()
      , mv_max_entries: 500
    }
  }
  
  pub fn update(&mut self, message: crate::lens::v_log_notes::Message) -> iced::Task<crate::lens::v_log_notes::Message> 
  { match message 
    { Message::AddSentNote(note) =>
      { log::debug!("LogNotesV::update AddSentNote: {}", note);
        let entry = LogEntry
        {   direction: LogDirection::Sent
          , note
          , timestamp: chrono::Utc::now()
        };
        self.mv_log_entries.push(entry);
        if self.mv_log_entries.len() > self.mv_max_entries
        { self.mv_log_entries.remove(0);
        }
        iced::Task::none()
      }
      , Message::AddRecvNote(note) =>
      { log::debug!("LogNotesV::update AddRecvNote: {}", note);
        let entry = LogEntry
        {   direction: LogDirection::Received
          , note
          , timestamp: chrono::Utc::now()
        };
        self.mv_log_entries.push(entry);
        if self.mv_log_entries.len() > self.mv_max_entries
        { self.mv_log_entries.remove(0);
        }
        iced::Task::none()
      }
      , Message::ClearAllNotes =>
      { log::debug!("LogNotesV::update ClearAllNotes");
        self.mv_log_entries.clear();
        iced::Task::none()
      }
    }
  }

  pub fn subscription(&self) -> iced::Subscription<Message> 
  { iced::Subscription::none()
  }
}
