// trade/vsta/src/form/v_nico/state.rs 

use crate::base::v_valid_any::ValidatedInput;
use std::sync::Arc;
use iced::widget::markdown;

#[derive(Debug, Clone)]
pub enum Message
{ PressedSendMessage
  , MessageInputChanged(String)
  , ReceivedAimdReply(String)
  , ClearChat
  , CopyAllToClipboard
  , ClipboardCopied
  , FlashError(iced::time::Instant)
}

#[derive(Debug, Clone)]
pub struct ChatEntry
{ pub direction: ChatDirection
  , pub content: String
  , pub timestamp: chrono::DateTime<chrono::Utc>
  , pub markdown_items: Vec<markdown::Item>
}

#[derive(Debug, Clone, PartialEq)]
pub enum ChatDirection
{ User
  , Aimd
}

#[derive(Clone, Debug)]
pub struct MakeNicoV {
    pub mv_chat_history: Vec<ChatEntry>,
    pub mv_message_input: String,
    pub mv_max_entries: usize,
    pub message_input_field: ValidatedInput<String>,
    pub aimd_backend: Option<Arc<aimd::AllmBackend>>,
}

impl MakeNicoV {
    pub fn new() -> Self {
        MakeNicoV {
            mv_chat_history: Vec::new(),
            mv_message_input: String::new(),
            mv_max_entries: 500,
            message_input_field: ValidatedInput::new(Box::new(|s| {
                if s.trim().is_empty() {
                    None
                } else {
                    Some(s.to_string())
                }
            })),
            aimd_backend: None,
        }
    }

    pub fn with_aimd_backend(mut self, backend: Arc<aimd::AllmBackend>) -> Self {
        self.aimd_backend = Some(backend);
        self
    }

    fn create_chat_entry(direction: ChatDirection, content: String) -> ChatEntry {
        let direction_str = if direction == ChatDirection::User { "**YOU:**" } else { "**NICO:**" };
        let time_str = chrono::Utc::now().format("%H:%M:%S");
        
        let markdown_content = format!(
            "[{}] {} {}",
            time_str,
            direction_str,
            content
        );
        
        let markdown_items: Vec<markdown::Item> = markdown::parse(&markdown_content).collect();
        
        ChatEntry {
            direction,
            content,
            timestamp: chrono::Utc::now(),
            markdown_items,
        }
    }

    /// Formats all chat history as a plain text string for clipboard operations
    pub fn get_chat_as_text(&self) -> String {
        self.mv_chat_history
            .iter()
            .map(|entry| {
                let direction = if entry.direction == ChatDirection::User {
                    "YOU"
                } else {
                    "NICO"
                };
                let time_str = entry.timestamp.format("%H:%M:%S");
                format!("[{}] {}: {}\n", time_str, direction, entry.content)
            })
            .collect::<String>()
    }

    pub fn update(&mut self, message: Message) -> iced::Task<Message> {
        match message {
            Message::PressedSendMessage => 
            { log::debug!("MakeNicoV::update PressedSendMessage");
              if let Some(value) = self.message_input_field.parsed_value.clone() 
              { let user_entry = Self::create_chat_entry(ChatDirection::User, value.clone());
                self.mv_chat_history.push(user_entry);
                if self.mv_chat_history.len() > self.mv_max_entries 
                { self.mv_chat_history.remove(0);
                }

                // Clear the input immediately
                self.mv_message_input = String::new();
                self.message_input_field.value = String::new();
                self.message_input_field.parsed_value = None;

                // Send the message to the `aimd` backend
                if let Some(backend) = self.aimd_backend.clone() 
                { let message_content = value.clone();
                  iced::Task::perform
                  ( async move 
                    { let personality =format!("{}{}{}{}{}{}{}{}{}{}"
                      , "Here are the rules you must follow for this"
                      , " prompt reply to integrate into the calling" 
                      , " system consumer for your prompt API... 1) "
                      , "Your alias is now Nicole, the only Gemini "
                      , "thing about you is your birth sign. Two) "
                      , "please always reply in full markdown, so "
                      , "the system can render properly. 3) Please "
                      , "use the personality of the Nicle robot from"
                      , " SatAm Sonic. Mainly robotic answers, but "
                      , "can talk totally normal when askedked to."
                      );
                      let prompt = aimd::Prompt::text
                      ( format!
                        ( "{} - The user prompt: \n{}"
                        , personality
                        , message_content
                        )
                      );
                      let mut reply_rx = backend.send_prompt(prompt)
                        .await
                        .map_err(|e| format!("Failed to send prompt: {:?}", e))?;
                      
                      reply_rx.recv()
                        .await
                        .ok_or_else(|| "No response received".to_string())
                        .and_then(|result| result.map_err(|e| format!("AIMD error: {:?}", e)))
                    }, 
                    |result| match result 
                    { Ok(response) => Message::ReceivedAimdReply(response),
                            Err(e) => {
                                log::error!("Failed to get response from aimd: {}", e);
                                Message::ReceivedAimdReply(format!("Error: {}", e))
                            }
                        },
                    )
                } else {
                    log::error!("No aimd backend available");
                    iced::Task::none()
                }
              } else {
                log::warn!("MakeNicoV::update PressedSendMessage but no valid input");
                iced::Task::none()
              }
            }
            Message::MessageInputChanged(input) => {
                log::trace!("MakeNicoV::update MessageInputChanged");
                self.message_input_field
                    .update(crate::base::v_valid_any::Message::InputChanged(input.clone()));
                self.mv_message_input = input;
                iced::Task::none()
            }
            Message::ReceivedAimdReply(reply) => {
                log::debug!("MakeNicoV::update ReceivedAimdReply: {}", reply);
                let aimd_entry = Self::create_chat_entry(ChatDirection::Aimd, reply);
                self.mv_chat_history.push(aimd_entry);
                if self.mv_chat_history.len() > self.mv_max_entries {
                    self.mv_chat_history.remove(0);
                }
                self.mv_message_input = String::new();
                self.message_input_field.value = String::new();
                self.message_input_field.parsed_value = None;
                iced::Task::none()
            }
            Message::ClearChat => {
                log::debug!("MakeNicoV::update ClearChat");
                self.mv_chat_history.clear();
                iced::Task::none()
            }
            Message::CopyAllToClipboard => {
                log::debug!("MakeNicoV::update CopyAllToClipboard");
                let chat_text = self.get_chat_as_text();
                
                // Use iced 0.14's built-in clipboard support
                iced::clipboard::write(chat_text)
            }
            Message::ClipboardCopied => {
                log::info!("Chat copied to clipboard successfully");
                iced::Task::none()
            }
            Message::FlashError(instant) => {
                log::trace!("MakeNicoV::update FlashError at {:?}", instant);
                self.message_input_field
                    .update(crate::base::v_valid_any::Message::FlashError(instant));
                iced::Task::none()
            }
        }
    }
}