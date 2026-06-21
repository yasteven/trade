// trade/vsta/src/base/v_combo_box/view.rs

use iced::widget::{combo_box, ComboBox};
use super::state::ComboBoxState; // Add this line instead

pub fn view<'a, T, MessageType>( // Added lifetime 'a here
    combobox: &'a ComboBoxState<T>,
    placeholder: &'a str, // And here
    map_message: impl Fn(T) -> MessageType + 'static,
) -> ComboBox<'a, T, MessageType> // And a' in the return type
where
    T: Clone + PartialEq + std::fmt::Debug + std::fmt::Display + 'static,
    MessageType: Clone + 'static,
{
    ComboBox::new(
        &combobox.state, // Also need to annotate the borrow here
        placeholder,
        combobox.selected.as_ref(),
        move |selection| map_message(selection.clone()),
    )
    .width(200)
}
