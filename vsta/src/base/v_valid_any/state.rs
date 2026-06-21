// trade/vsta/src/base/v_valid_any/state.rs

use iced::widget::text_input;

#[derive(Debug, Clone)]
pub enum Message<T> {
    InputChanged(String),
    InputValidated(T),
    InputError,
    FlashError(iced::time::Instant),
}

// Manual Clone implementation since Box<dyn Fn> isn't Clone
pub struct ValidatedInput<T> {
    pub value: String,
    pub parsed_value: Option<T>,
    pub is_valid: bool,
    pub error_flash: Option<f64>,
    validator: std::rc::Rc<dyn Fn(&str) -> Option<T>>,
}
impl<T: std::fmt::Debug>  std::fmt::Debug for ValidatedInput<T>
{ fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result
  { f.debug_struct("ValidatedInput")
      .field("value", &self.value)
      .field("parsed_value", &self.parsed_value)
      .field("is_valid", &self.is_valid)
      .field("error_flash", &self.value)
       .finish()
  }
}

impl<T> Clone for ValidatedInput<T> 
where T: Clone 
{
    fn clone(&self) -> Self {
        ValidatedInput {
            value: self.value.clone(),
            parsed_value: self.parsed_value.clone(),
            is_valid: self.is_valid,
            error_flash: self.error_flash,
            validator: self.validator.clone(),
        }
    }
}

impl<T : std::fmt::Display> ValidatedInput<T> 
{ pub fn new(validator: Box<dyn Fn(&str) -> Option<T>>) -> Self 
  {
        ValidatedInput {
            value: String::new(),
            parsed_value: None,
            is_valid: true,
            error_flash: None,
            validator: std::rc::Rc::from(validator),
        }
  }

  pub fn old(value : T , validator: Box<dyn Fn(&str) -> Option<T>>) -> Self 
  { ValidatedInput 
    { value: format!("{}",value),
            parsed_value: Some(value),
            is_valid: true,
            error_flash: None,
            validator: std::rc::Rc::from(validator),
        }
    }


    pub fn update(&mut self, message: Message<T>) {
        match message {
            Message::InputChanged(input) => {
                self.value = input.clone();
                self.parsed_value = (self.validator)(&input);
                self.is_valid = self.parsed_value.is_some();
                if !self.is_valid {
                    self.error_flash = Some(0.0);
                }
                else
                { self.error_flash = None;
                }
                // else i need to set the color back to original
            }
            Message::InputValidated(_) => {
                self.error_flash = None;
                self.is_valid = true;
            }
            Message::InputError => {
                self.is_valid = false;
                self.error_flash = Some(0.0);
            }
            Message::FlashError(_elapsed) => {
                if let Some(progress) = self.error_flash.as_mut() {
                    *progress += 0.016;
                    if *progress >= 1.0 {
                        self.error_flash = None;
                    }
                }
            }
        }
    }
}