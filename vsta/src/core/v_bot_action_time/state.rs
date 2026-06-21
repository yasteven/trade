
// trade/vsta/src/core/v_bot_action_time/state.rs (UPDATED)

use crate::base::v_time_span::{TimeSpanControl, Message as TimeSpanMessage};
use crate::base::v_usa_times::UsaMarketTimesComboBox;
use dsta::{BotActionTime, UsaMarketTimes};

#[derive(Debug, Clone)]
pub enum Message {
    EntryDayNumChanged(String),
    MyEntryTimeChanged(UsaMarketTimes),
    MyEntryWaitMessage(TimeSpanMessage),
    FlashError(iced::time::Instant),
}

#[derive(Debug, Clone)]
pub struct BotActionTimeControl {
    pub buzz_time_to_order: BotActionTime,
    pub relative_days_input: crate::base::v_valid_any::ValidatedInput<u16>,
    pub my_entry_time_combobox: UsaMarketTimesComboBox,
    pub my_entry_wait_control: TimeSpanControl,
}

impl BotActionTimeControl {
    pub fn new() -> Self {
        BotActionTimeControl {
            buzz_time_to_order: BotActionTime {
                relative_days: 0,
                my_entry_time: UsaMarketTimes::default(),
                my_entry_wait: chrono::Duration::seconds(0),
            },
            relative_days_input: crate::base::v_valid_any::ValidatedInput::new(
                Box::new(|s| s.parse::<u16>().ok())
            ),
            my_entry_time_combobox: UsaMarketTimesComboBox::new(),
            my_entry_wait_control: TimeSpanControl::new(),
        }
    }

    pub fn update(&mut self, message: Message) {
        match message {
            Message::EntryDayNumChanged(input) => {
                self.relative_days_input.update(
                    crate::base::v_valid_any::Message::InputChanged(input)
                );
                if let Some(value) = self.relative_days_input.parsed_value {
                    self.buzz_time_to_order.relative_days = value;
                }
            }
            Message::MyEntryTimeChanged(selected) => {
                self.my_entry_time_combobox.update(selected);
                self.buzz_time_to_order.my_entry_time = selected;
            }
            Message::MyEntryWaitMessage(msg) => {
                self.my_entry_wait_control.update(msg);
                self.buzz_time_to_order.my_entry_wait = 
                    self.my_entry_wait_control.time_span.to_duration();
            }
            Message::FlashError(elapsed) => {
                self.relative_days_input.update(
                    crate::base::v_valid_any::Message::FlashError(elapsed)
                );
            }
        }
    }
}
