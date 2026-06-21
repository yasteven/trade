
// trade/vsta/src/lens/v_buzz/view.rs

use iced::{
  widget::{button, Column, Container},
  Element, Length
};

use crate::lens::v_buzz::{BuzzBomberV, Message};

pub fn view(buzz: &BuzzBomberV) -> Element<'_, Message> {
  let controls = Column::new()
    .push(
      button::Button::new("Back to Dr. Robotnik")
        .on_press(Message::SwitchToDrR)
    )
    .spacing(10)
    .padding(10);
  /*
  let chart = ChartWidget::new(StockChart { buzz })
    .width(Length::Fill)
    .height(Length::FillPortion(3));
  */


  Container::new(
    Column::new()
      .push(controls)
//      .push(chart)
      .spacing(10)
  )
  .width(Length::Fill)
  .height(Length::Fill)
  .padding(10)
  .into()
}

struct StockChart<'a> {
  buzz: &'a BuzzBomberV,
}

/*
impl<'a> Chart<Message> for StockChart<'a> {
  type State = ();

  fn build_chart<DB: DrawingBackend>(&self, _state: &Self::State, mut builder: ChartBuilder<DB>) {
    if self.buzz.stock_data.is_empty() {
      return;
    }

    let min_time = self.buzz.stock_data.iter().map(|&(t, _)| t).min().unwrap();
    let max_time = self.buzz.stock_data.iter().map(|&(t, _)| t).max().unwrap();
    let min_value = self.buzz.stock_data.iter().map(|&(_, v)| v).fold(f32::INFINITY, f32::min);
    let max_value = self.buzz.stock_data.iter().map(|&(_, v)| v).fold(f32::NEG_INFINITY, f32::max);

    let mut chart = builder
      .caption("BuzzBomber Stock Chart", ("sans-serif", 20).into_font())
      .x_label_area_size(30)
      .y_label_area_size(40)
      .build_cartesian_2d(min_time as f32..max_time as f32, min_value..max_value)
      .unwrap();

    chart.configure_mesh().draw().unwrap();

    chart.draw_series(
      self.buzz.stock_data.iter().map(|&(time, value)| {
        let bar_width = (max_time - min_time) as f32 / self.buzz.stock_data.len() as f32 * 0.8;
        Rectangle::new(
          [(time as f32 - bar_width / 2.0, 0.0), (time as f32 + bar_width / 2.0, value)],
          RED.filled(),
        )
      })
    ).unwrap();
  }
}
*/