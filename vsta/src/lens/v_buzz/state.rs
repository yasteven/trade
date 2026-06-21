
// trade/vsta/src/lens/v_buzz/state.rs

#[derive(Debug, Clone)]
pub enum StockDataType { Bid, Ask, Last}

#[derive(Debug, Clone)]
pub enum Message 
{ StockDataReceived((String, StockDataType, f32)) //  dsta::GotTick00) // all 01,... are the same
  , SwitchToDrR
  ,
}

pub struct BuzzBomberV 
{ pub stock_data: Vec<(u64, f32)>, // (timestamp, value) for bar chart
}

impl BuzzBomberV 
{ pub fn new ()-> Self 
  { BuzzBomberV
    { stock_data: Vec::new(),
      //sender,
      //receiver,
    }
  }

  pub fn update(&mut self, message: Message) 
  -> iced::Task<Message>
  {
    match message 
    { Message::StockDataReceived(data) => 
      { let ts = 0; // chrono::NaiveTime::now();
        log::debug!("BuzzBomberV received StockChartDataUpdate at timestamp {}", ts);
        self.stock_data.push((ts, data.2));
        // Keep only the last 50 data points for display
        if self.stock_data.len() > 50 {
          self.stock_data.drain(0..self.stock_data.len() - 50);
        }
      },

      Message::SwitchToDrR => 
      {
        log::debug!("Switching back to DrRobotnik view");
      },
    };

    iced::Task::none()
  }
}