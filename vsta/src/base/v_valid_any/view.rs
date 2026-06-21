
// trade/vsta/src/base/v_valid_any/view.rs

use iced::widget::{text_input, TextInput};
use super::state::{ValidatedInput, Message};

pub fn view<'a, T, MessageType>(
    input: &'a ValidatedInput<T>,
    placeholder: &str,
    map_message: impl Fn(Message<T>) -> MessageType + 'a,
) -> TextInput<'a, MessageType>
where
    T: 'static,
    MessageType: 'a + Clone,
{
    let background_color = match input.error_flash {
        Some(progress) => {
            let t = progress.min(1.0); // Clamp to [0, 1]
            iced::Color {
                r: (1.0 * (1.0 - t)) as f32, // Fade from red (1.0) to original (e.g., 0.9 for light gray)
                g: (0.1 * (1.0 - t) + 0.9 * t) as f32, // Fade to gray
                b: (0.1 * (1.0 - t) + 0.9 * t) as f32, // Fade to gray
                a: 1.0,
            }
        }
        None => iced::Color::from_rgb(0.9, 0.9, 0.9), // Default background (light gray)
    };

    text_input(placeholder, &input.value)
        .padding(10)
        .style(move |_theme: &iced::Theme, _status| {
            text_input::Style {
                background: iced::Background::Color(background_color),
                border: iced::Border {
                    color: iced::Color::BLACK, // Default border color
                    width: 1.0,                // Default border width
                    radius: 4.0.into(),        // Default border radius
                },
                icon: iced::Color::BLACK,                       // Default icon color
                placeholder: iced::Color::from_rgb(0.3, 0.3, 0.3), // Default placeholder color
                value: iced::Color::BLACK,                      // Default text color
                selection: iced::Color::from_rgb(0.7, 0.7, 1.0), // Default selection color
            }
        })
        .on_input(move |s| map_message(Message::InputChanged(s)))
}