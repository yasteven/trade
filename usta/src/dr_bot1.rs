//dr_bot1.rs

// Bot 1: User Input Bot (Placeholder)
pub mod dr_bot1
{ 
  use crate::*;
      
  pub async fn f_run_main_dr_bot1
  (   mut told_input_data: tokio::sync::mpsc::Receiver<dsta::Ticker>
    , bot_api_input_bot: BotApi
    , our_dr_r_ask_robo: DrRoboApi
    , dr_ttai_api: DrTtaiApi
  ) 
  { log::debug!("ENTRY: f_loop_bot_1_dr_r_input_state [FLBIS] -> Handle Inputs");
    loop 
    { tokio::select! 
      { res = told_input_data.recv() => 
        { if let Some(ticker) = res 
          { log::info!("Bot 1 processing input ticker: {}", ticker.name);
            let _ = our_dr_r_ask_robo.tell_dr_pls_ticks.send((1, ticker.name.clone()));
          } else 
          { log::error!("[FLBIS] told_input_data channel closed");
            break;
          }
        },
      }
    }
  } 
}
