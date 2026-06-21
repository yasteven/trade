// trade/vsta/src/base/v_combo_box/state.rs

use iced::widget::combo_box;

#[derive(Debug, Clone)]
pub struct ComboBoxState<T> // Renamed from ComboBox<T> to ComboBoxState<T>
where
    T: Clone + PartialEq + std::fmt::Debug + std::fmt::Display + 'static,
{
    pub state: combo_box::State<T>,
    pub selected: Option<T>,
    pub options: Vec<T>,
}

impl<T> ComboBoxState<T> // Renamed impl block
where
    T: Clone + PartialEq + std::fmt::Debug + std::fmt::Display + 'static,
{
    pub fn new(options: Vec<T>) -> Self {
        ComboBoxState { // Renamed constructor
            state: combo_box::State::new(options.clone()),
            selected: options.first().cloned(),
            options,
        }
    }

    pub fn update(&mut self, selected: T) {
        self.selected = Some(selected);
    }
}
