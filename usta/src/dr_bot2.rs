//dr_bot2.rs

// Bot 2: Reserved (Placeholder)
// Bot 2: Taxman 
// bots are coded to send their calculated capital gains over to dr_bot2
// Once A quarter these funds have to be allocated to Bot 3: USA (cash for taxes)
// 60/40 for the spx xps, etc index options
// 60 long term => VTEB or MUB or PZA, also for states vflqx, dpenx, fcotx, vnytx, cmf, LTCG=~15~30%
// 40 short term => BIL or PZA , STGC=37~5
// Vteb MUB vnytx


// Bot 4 shall be the square even prime bot, it analyzes the accounts of all the bots by 
// subscribing to all of them, and then checking that the cash allocation sum 
// of them all is less than or equal to the most recent ttai account cash.
// If they aren't, the bots need to be deallocated: user bot first. 
// If there's more cash in the account than the sum, also allocate to user bot.
pub mod dr_bot2
{
  use crate::*;
      
  async fn f_loop_bot_2_dr_r_reserved
  ( bot_api_reserved_bot: BotApi
  , our_dr_r_ask_robo: DrRoboApi
  )
  { log::debug!("ENTRY: f_loop_bot_2_dr_r_reserved [FLBRS] -> Reserved for Future Use");
    loop 
    { tokio::time::sleep(std::time::Duration::from_secs(3600)).await; // Sleep indefinitely
    }
  }
}

// Bot 5 shall be the SteveMan, which analizes other bots, and then uses the taxman collateral to place trades, e.g., 1-2 day otm call credit spreads on never-sell stock positions. ( if they expire at a loss, we need to allocate tax debt - always be tax positive by allocating cash to steve to begin with to cover that 1st spread $100 debt.)
// Bot 6 is the Mark to market bot, which has its 