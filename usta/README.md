// README.md
 
# USTA - ultra simple trade automation
 
The main backend component of the RUST trade prototype. 

---

## Table of contents:

1) PART 1: Overview 
2) PART 2: Bot List
3) PART 3: Unit andIntegation Test Results for peace of mind.

---
# PART 1: Overview
---

This library is the backend component for a rapid prototype RUST-
only automated trading system. It uses an actor pattern and tokio
mpsc async channels to make a internal communication mesh. Single
respsibility actor modules creates the asset trading system thus:

1) dr_robo - main actor process that manages all the other actors
2) dr_seek - subordinate actor that eables frontend communication  
3) dr_ttai - subordinate actor for communicating to online broker
4) dr_bota : an internal interface definition for any trading bot 
5) dr_bot0 - trading bot 0 acts as an proxy for trading with ttai
6) dr_bot1 - user bot for frontend ticker views, managment, info!
7) dr_bot2 - taxman for saving chunks of account for taxes. TODO!

Essentially, we take a tastytrade account we control and put what
is basically a virtual account manager over it, so it can finance
robot accounts internally with a restricted set of availabe stuff
a robot account can have, allowing for bot position margin checks
and allowing them trade amongst themselves, to create a orchestra
of interacting & usefull trading bots, to ideally generate money.

for more information, see:
## Dr Robo: ./read/Appendex_dr.md (dr_robo) 
## Dr Seek: ./read/Appendex_ds.md (dr_seek) 
## Dr Ttai: ./read/Appendex_tt.md (dr_ttai) 
## Dr Bota: ./read/Appendex_ba.md (dr_bota) 
## Dr Bot0: ./read/Appendex_b0.md (dr_bot0) 
## Dr Bot1: ./read/Appendex_b1.md (dr_bot1) 
## Dr Bot2: ./read/Appendex_b2.md (dr_bot2) 
  
---
# PART 2: Runtime User CTOR Bot List:
---

Messages and processes within the usta system are context-aware of 
the bot type and source. 

There are two flavors of bots:

  OM - order making bots
  VB - virual activities bot

Please see the deatils for each in ./read directory.

OM:

## YouSwatBot: ./read/Appendex_ys.md 
  owns one thing (or more if failed to deallocate), made when other bots die
## StealthBot: ./read/Appendex_sb.md
  assists in maximizing profit / minimizing losses from risky options trades
## BuzzBomber: ./read/Appendex_bb.md
  automatically sells options spreads based upon varied entry/exit critera 
## Coconuts01: 
  never sells, never transfers out more than 50% of its max ever owned after it owns the first 200.

  example: bot put in 200.001, first max. i take out 100, max allowed to remove. say bot put in 300 to get to 400, new max. half max is now 200, but i have already taken out 100, so i can only take out 100. that puts coconust down to 300, since the total i took out was 200. i'd have a rule that payouts can only be requested in chunks of 100 so 200 needs to be gained first at least every time i guess  

## SallyBot03:
  places orders from human with an 3 types of extra layers of submission logic based on price
  1)
## Muttski101:
  places orders from human, with an 3 types of extra layers of submission logic based on time. {1:avoid-midday (makes puts buy between 0.5-hr and 1-minute before the close, makes longs buy around 10:30am-11:00 am or at a drop before then), 2:dollar-cost-avg frequency + amplitude, 3:delay a certian time and only get if on discount or if at the close. }




 
# bot combo's in the future:

  1) UncleChuck: has a fake dr robotnik api layer used to control other bots:

    runs a bot that submits orders

    when we get an order, stop that bot, run another bot that we pipe orders into  like the GoSallyBot.
 
    pipe those new bot orders to the real dr_robotnik

    basically a robotnik proxy, treated as a OM
 
VBs:

  2) Centipedes: tracks assets histories & greeks & stuff, reports a Snively to Dr R to raise
  simulated ticker data with the ticker SIMULATED_/WHAT-And-VALUE/

  3) BinaryBats: tracks simulated tickers, creates essentially arbitrary entry / exit signals / flags / etc.

OMs:
  4) DragonBots:
    kicks off a binary bat or subscribes to one 
    uses binary bat to generate a one-and-done buzzbomber or stealthbot or uncle chuck. this allows, e.g., buzz bomber to sell based upon moving averages instead of just, e.g., time of day/week/etc.

VB (special)
  5) Scorpion04: // bot2 for you2
    follows bots and does botswap requests reallocating for arbitrary reasons
      (pass in list of bots, and like an enum to say to do a thing like reallocate cash to best )
    // keep taxes, bonus, costs, etc., the core things, as bot2 repsonibility
    right after a bot exits trading. 
    // Put all bots into a 1s cooldown after trade confirmed; scorpion made at same time or before every bot sees the udpate, calculates profit / loss / costs, and reallocate from it. ignores trades etc after reported seppuku/zombie; self destructs.       

--- 
# PART 3: Next TODO's
---

stealth,buzz: vsta make finish

vsta improvments:

re-order the buttons:
swat on top

sally : btc preset!


VERIFY:
  // need to verify we update theo prices!

FINAL BATCH: - necessary for prototype testable
1) // need to ensure order flow works right, dead with assets transfer to swat, swat die sells agressively. // need seppuku vs zombied difference hashed out for swat.

PROTO BATCH - necessary for prototype finished
1) // need saving and loading and ttrs reset perfect
2) // need backend to drop frontend pipes when a new one connects
3) ttrs - need to be able to cancel a ticker (at tastytrade's thing)
3) ttai - need to cancel ticker when no bots exist for it
4) swat - stores history - <24 gb per stock for 10.5 years of 1 second bars of (64 bit each)
  // [bid bsize ask asize hi lo time theo(none<=>0.0))]
  // => 769 GB for the 32 emeralds
  // => raise alert at 420 gb (~5 years)
  // => 12 TB for 10 years of S&P 500 1 second bars; so the 
  // 24gb is 10.5 years, so we need a 10 year alert!
  // Say we have 16 colors, then 





CORE BACKEND WORK: SAVING + LOADING:
  bots shall skip their allocation requests when they're loaded from file (dr robo will send an account update to them)

    9) implement saving / loading:
       SAVE:
      a)  bot mind set to stopped by robo.
      TODO b) bot save's its state (using the stopped mind state)
      TODO c) the canonical botaaccount for the bots is saved by robo (using the prior-to-stopped mind state). 
      TODO d) bot mind reported stopped by bot => save/load OK.
               
     At startup and shutdown of dr_robo, which can be done as a reset option from frontend snively
     and have reset of dr_robo At a random interval from 5:30-5:40pm,  
     have it stop all bots, wait for them to all report stopped, then store the dr_robo bota info

   at robo startup  it checks for save state, if exists, it loads from there.
   at any bot startup (all startup stopped ?),, it checks for save state based on its own name, if exists, it loads from there.
   and it reports it (transitions state to.e.g, running if thats what it was doing before stopped ). 

   => Robo save: for each account, save it, grab state, change to stop & report; change to state & report.

  PRIME BOT, BOT 2: force-saving and loading - 
    we can make robo do a reboot at 5:20 pm (right now we do 10 minutes after starting with debug)
    this can be used to calculate drift in cash reserves (e.g., slippage, broker fees), daily P&L, 



FUTURE BATCH: add ORDER_END 

TESTS:

cycle through all of /ES options  - request ticker, wait for the ticker, cancel the ticker,
// time each request->cancel, keep the full price info in the dr_robo's asset deets.
// print the timing info, store historical data and print timing of the save, tell the size,
//tell the expected memory needed for 1 year etc based on stuff
// make it a test for the master emerald thing, // make  a tool to 'history' it, e..g, eggrobo s&k, select to make a thing ythat does history stuff.

CORE BACKEND WORK - proper paper trading to test robo

  need to upgrade to new order update system,
  need to make semi-determinisitc assets & tickers - at startup of the paper, actually request real ttrs info,
  need to make "FAKE [json function args - we can make market models with parameters ]"

  which we can use to feed to the paper-engine - set-seed random % / tick etc market model (so all runs are same only during a single day);
  so we can alert tickers at a variable preset frequency (for stress testing).

  we'll create a run which does a sin-wave like behavior for the % changes on underlying tickers throughout the day 
  // E.g., open - to close = 0% gain, assume 1 second ticks: 9:30am -> 4:00pm = 6.5 hrs = 390 minutes, 
  // =>  0.75% total gain at 97 minutes, 0% gain at 195 minutes, -0.75 total gain at 293 minutes ;
  // have this be for the midpoint, and then make the spread on each side of the midpoint the seeded pesudo-random change.
  // later we'll hook up a black scholes to make the options prices do their thing based on the underlyings ^ 
  E.g., FAKE_SIN

  we'll greate a run which does weiner process
  E.g., FAKE_WIN

  we'll greate a run which does historical prices if available from the swat's save.
  E.g., FAKE_BIN

CORE FRONTEND + BACKEND WORK - bot message discrimination
  add a to snively: 
  enum BotMessage{ FromSwat(BotAlert), FromSally(BotAlert), ... }
  BotAlert
  { bot_stone : String
  , bot_brain : String
  , bot_stuff : String
  , bot_alert : String 
  }

  on frontend
  as we rx snively BotMessage, add to a collection of the bots - one collection per bot type runtime view
  when a bot is reported "KILLED", we may still get updates so instead
  we flip a bit to allow the user to delete &&|| save the bot output and remove from frontend.

  get the keepalive parsewarnings passed to the frontend to display connection health (1m timeout on frontend, 30s keep alive)

CORE BACKEND WORK: DONE DR_ROBO FOR BETTER ORDER CASH MANAGMENT:

    // TODO: in the future - make a new swatbot to transfer CASH to: TRANSACTION_ORDER_[BOT_NAME]_[ORDER_ID|"BITCOIN"] + ($2 * number of contracts in case tastytade is lying )
    // THEN, after the order is finished, do we get the final transaction fees cost and delete that from the temp bot above 
    // and transfer back any cash to USER_CASH, if its negative (tastytrade severly underestimated transction fees, ), remove from user cash and Raise frontend Snively; if user_cash goes negative, kill all bots. 

CORE BACKEND WORK: CASH DEPOSITS / WITHDRAWLS

  allow a mechanism for cash deposit with bot0, if we have a "check deposits" button on the frontend,
  that looks at the transactions since the last time it checked transactions, adds / removes cash when there
  is differences (e.g., bot0 needs to remember its first time it got the account information.

CORE MISC WORK: 

  eliminate warnings (do more than just logging to silence unused, unless we log why it's unused! )

  documentation: when we finalize a v 1.0, update the documentations in READ.

  SEE_IMAGE_PART_=./jpg/whatver.jpg => gets compiled to base64 for the inclusion, put .md renderer in the vsta 
  so that we have information for each button etc.

  //TODO: future: rename:
  // pub mod v_retry_act  <- v_haggle_action;
  // pub mod v_retry_lim v_haggle_limits;
  // pub mod v_retry_how v_haggle_method;

CORE FUTURE WORK: 

  hook up nicole with snively messages, allow it to send emails or whatever:


TESTING IRL:

  ensure we get bots dying right -> transfer to swat bot
                                 -> delete bot completey

  ensure we get futures pricing & accont effects right

  Easy: sally bot should seppuku after she's done can do anytime


#[derive(Default, Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct CommonBotInfo 
{ pub friendly_name: String,
  pub tracking_tick: String, // because all bots only follow 1 
  pub haggle_action: HaggleAction, // what to do when we wait to long 
  pub max_cash_risk: (bool, f64, f64 ), // ( ? , %, $ ) bool == false => select min of next 2: % of total cash or set cash value; bool = true choose max. This specifies max $ to use in a bot's algo
}


COOLDOWN:


Finished is when I can comfortably add 1 new bot per week while debugging older ones running irl.
 While also fixing 1 bug or updating file per day (12-15 files!) 
 (1) dry-run @ entry/exit and debug stops at bot order placement  
 (2) remove debug stop 1 to allow entry
 (3) verify entry for real (keep debug stop on the exit)

// Then we can do these polishing:
 (1/8) more glossary less magic 
 (1/4) robo logic verification and tests - Eliminate panics
 (1/2) add email/text capability alert for order entry 
 (A) robo needs to have try_send for all sends for pure non-blocking (rust allm project does it slightly better, but this should work for now)
 (A.a) insert timing profiling information (use logs)
 (B) no "_ = " for await stuff anywhere (except for the bota things, like do tells, not process)
 (9) add cancel order to the bot0 bot
 (10) Expand tests - make current bot tests parametric, define a higher order set of test coverages

11) Documentation - have a md with a steve tag TRADE_VSTA_IMAGE=[path] which will let us
// run a startup script to embed markdown into the frontend, allowing form+ visuals to have help buttons that load the Appendex's for them.

Current State (as of iced ~0.13 / recent versions)
The iced::widget::markdown widget supports images, but only via:

http(s) URLs → loads them asynchronously over the network (works fine)
Local file paths (e.g. images/screenshot.png or ./logo.png) → only works reliably when your app is run from the project directory (i.e. cargo run), because iced resolves relative paths against the current working directory (std::env::current_dir()), not relative to the markdown file's original location.


When you embed the markdown with include_str! and ship a binary, relative image paths almost always break — the images won't be found.
Recommended Solutions (from easiest → most integrated)
1. Easiest: Convert images → data: URLs (base64) before embedding
This is the most reliable way when embedding everything into the binary.
Steps:

Write a tiny build script or one-time tool that:
Reads your info/myfile.md
Finds every ![alt](path.png)
Loads the image file
Base64-encodes it
Replaces with ![alt](data:image/png;base64,ABC123...)
Writes the result to e.g. src/embedded_info.md

