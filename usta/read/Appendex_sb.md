# Stealth Bot User Guide

## Description

The Stealth Bot is an automated options trading algorithm that executes **two-leg credit spread strategies** to convert profitable single-option positions into hedged, locked-in spreads. The bot monitors both long and short legs through multiple exit conditions and manages the complete lifecycle from initial entry through final position closure.

**Strategy goal**: Buy an out-of-the-money option → capture initial profit → sell a nearby in-the-money option to lock in gains → manage the resulting spread through one of several exit strategies.

---

## Architecture

### High-Level Flow

```
┌───────────────────────────────────────────────┐
│ INITIALIZATION (Fresh or Resumed)             │
│  • Request underlying asset & cash allocation │
│  • Calculate ATM & select option strikes      │
│  • Request ticker data for selected options   │
└────────────────┬──────────────────────────────┘
                 │
                 ▼
┌──────────────────────────────────────┐
│ ENTRY PHASE                          │
│  Begins → Check1 → Logic1            │
│  • Place BuyToOpen order (long leg)  │
│  • Wait for fill confirmation        │
│  • Monitor for profit or stop-loss   │
└────────────────┬─────────────────────┘
                 │
         ┌───────┴────────┐
         ▼                ▼
    ┌─────────┐      ┌──────────┐
    │ Profit? │      │ Stop?    │
    │ Spread! │      │ Exit     │
    └────┬────┘      └────┬─────┘
         │                │
         ▼                ▼
    ┌─────────┐      ┌──────────┐
    │ Check2  │      │ Check3   │
    │ Spread  │      │ Loss     │
    │  Fill   │      │ Exit     │
    └────┬────┘      └────┬─────┘
         │                │
         ▼                ▼
    ┌─────────┐      [ZOMBIE]
    │ Logic2  │       (End)
    │ Monitor │
    │ Spread  │
    └────┬────┘
         │
         ▼
    ┌──────────┐
    │ Exit OK? │
    │ Check4   │
    │ Close    │
    └────┬─────┘
         │
         ▼
     [ZOMBIE]
      (End)
```

### Core Components

| Component | Purpose |
|-----------|---------|
| **State Machine** | Sequential progression through 7 states with clear entry/exit conditions |
| **Ticker Manager** | Tracks 3 price feeds: underlying stock + 2 options |
| **Position Tracker** | Monitors owned quantities and average prices for all legs |
| **Exit Logic** | Checks multiple conditions simultaneously for early closure |
| **State Persistence** | Saves/loads brain state to/from JSON for resume capability |

---

## Construction

### Creating a Stealth Bot

```rust
use dsta::*;

let config = MakeStealthBot {
    // Account & identity
    my_accounting: CommonBotInfo {
        friendly_name: "my_stealthy_put_spread".to_string(),
        tracking_tick: "SPY".to_string(),
        haggle_action: HaggleAction::JustCancel(HaggleMethod::...),
        max_cash_risk: (true, 100.0, 5000.0),  // use max of 100% cash or $5k
    },
    
    // Capital allocation
    my_cash_alloc: 5000.0,
    
    // Direction & expiration
    stonk_feeling: MarketDirection::Corrects,  // bearish → put spreads
    option_expire: 30,                          // min 30 days to expiry
    
    // Strike selection
    option_bucket: 2,     // long leg: 2 strikes OTM from ATM
    spread_bucket: 1,     // short leg: 1 strike back toward ATM
    
    // Profit & loss targets
    exit_gain_pct: 50.0,   // lock in spread when 50% profit reached
    exit_loss_pct: -80.0,  // stop-loss at 80% loss
    
    // Exit strategies (any one triggers close)
    nice_exit_way: vec![
        StealthBotDoneNiceStyle::OtmShort,           // exit when short goes OTM
        StealthBotDoneNiceStyle::FinalPct(75.0),     // exit at 75% of max profit
    ],
    
    // Price selection
    use_theo_cost: false,  // false = use bid-ask midpoint
};
```

### Configuration Parameters Explained

#### Direction & Selection
- **`stonk_feeling`**: `GetStonk` (bullish/calls) or `Corrects` (bearish/puts)
- **`option_bucket`**: Distance from ATM for the **long leg** (0=ATM, 1=1 strike OTM, etc.)
- **`spread_bucket`**: Distance **back from long leg** to find short leg (typically 1-2)

**Example**: With SPY at $450:
- `option_bucket: 2` → long put at $446 strike
- `spread_bucket: 1` → short put at $448 strike (2-1=1 strike back)

#### Profit & Loss
- **`exit_gain_pct`**: Percentage gain on the long leg before triggering the spread
  - `50.0` means: "When long leg is up 50%, sell the short leg to lock it in"
  - Calculated as: `(current_price - entry_price) / entry_price * 100`
  
- **`exit_loss_pct`**: Maximum acceptable loss on the long leg
  - `-80.0` means: "Exit if down 80% from entry"
  - Converted to floor price: `entry_price × (1 + exit_loss_pct/100)`

#### Exit Conditions (`nice_exit_way`)
Provide a **vector of conditions** — any one can trigger the final close:

| Condition | Trigger |
|-----------|---------|
| `OtmShort` | Short leg goes out-of-the-money |
| `FinalPct(pct)` | Spread reaches target profit % |
| `PriorDay(time)` | N days before expiration at specific time |
| `GoExpire` | Hold until expiration (no auto-exit) |

#### Risk Rules
- **`max_cash_risk`**: `(use_max, pct_of_cash, dollar_amount)`
  - First bool: `true` = use MAX of the two, `false` = use MIN
  - Example: `(true, 100.0, 5000.0)` = "use whichever is larger: 100% of cash or $5000"

---

## Behavior

### State Machine Transitions

#### **Begins** — Entry Order Submission
**When**: Bot starts for the first time, or transitions from Logic1  
**Does**:
1. Calculates max position size: `qty = floor(available_cash / (target_price × 100))`
2. Submits a **`BuyToOpen`** limit order for the long leg
3. **Transitions to**: Check1

**Example**:
- Available cash: $5,000
- Target entry price: $2.50 per share
- Max contracts: `floor(5000 / (2.50 × 100))` = 20 contracts

---

#### **Check1** — Waiting for Entry Fill
**When**: After submitting the entry order  
**Does**:
1. Monitors account for the long option position
2. Validates fill matches expected quantity (tolerance: ±0.01 contracts)
3. Sets **stop-loss floor**: `entry_price × (1 + exit_loss_pct/100)`
4. Timeout: 20 seconds for partial fills

**Floor Calculation Example**:
- Entry price: $2.00
- exit_loss_pct: -80.0
- Floor: $2.00 × (1 - 0.80) = $0.40
  - If position drops below $0.40, triggers loss exit

**Transitions to**:
- `Logic1` (full fill detected)
- Stays in `Check1` (waiting for partial fill)

---

#### **Logic1** — Monitor & Trigger Spread Entry
**When**: After long leg fills  
**Does**:
1. **Stop-loss check**: If current price < floor → branches to loss exit (Check3)

2. **Trailing stop**: If profit > 50% of exit_gain_pct:
   - Raises floor by half the unrealized profit
   - Example: Entry $2.00, current $3.00, target 50% gain
     - Profit: $1.00 (50%)
     - Raise floor to: $2.00 + ($1.00/2) = $2.50

3. **Spread trigger calculation**: 
   - Determines the exact price at which to sell the short leg
   - **For puts** (bearish): `required_short_price = spread_width − entry_price × (1 − p)`
     - Where `p = exit_gain_pct/100` and `spread_width = short_strike − long_strike`
   - **For calls** (bullish): `required_short_price = entry_price × (1 + p) + spread_width`
   
4. If current short leg price ≥ required price → triggers spread entry (Check2)

**Transitions to**:
- `Check2` (spread trigger met)
- `Check3` (stop-loss triggered)
- Stays in `Logic1` (no conditions met)

---

#### **Check2** — Waiting for Spread Fill
**When**: After submitting the short leg (`SellToOpen`) order  
**Does**:
1. Monitors account for the short option position (short_puts or short_calls)
2. Validates fill with expected quantity
3. Timeout: 20 seconds for partial fills

**Transitions to**:
- `Logic2` (spread order filled)
- Stays in `Check2` (waiting for fill)

---

#### **Logic2** — Monitor Spread & Execute Nice Exit
**When**: After both long and short legs are filled  
**Does**:
1. **Calculates spread P&L**:
   - `credit_received = price_at_short_fill`
   - `current_spread_value = current_short_price − current_long_price`
   - `current_profit = credit_received − current_spread_value`
   - `profit_pct = (current_profit / max_profit) × 100`

2. **Checks all exit conditions from `nice_exit_way`**:
   - `OtmShort`: Is short strike profitable (OTM)?
   - `FinalPct(75)`: Have we hit 75% of max profit?
   - `PriorDay(time)`: Are we N days before expiry at this time?
   - `GoExpire`: (No auto-trigger)

3. If any condition met → submits two-leg close order (Check4)

**Example - OTM Exit**:
- Direction: `Corrects` (puts)
- Short strike: $450
- Current underlying: $455
- Since underlying > strike → short is OTM → exit condition met

**Transitions to**:
- `Check4` (exit condition triggered)
- Stays in `Logic2` (no conditions met)

---

#### **Check3** — Loss Exit Fill (Side Path)
**When**: Triggered from Logic1 by stop-loss  
**Does**:
1. Submits a **`SellToClose`** order for the long leg only
2. Monitors for position closing (approaching zero)
3. Timeout: 20 seconds

**Transitions to**:
- `Zombie` (loss exit complete, bot terminates)

---

#### **Check4** — Final Close Fill
**When**: After submitting the two-leg close order  
**Does**:
1. Monitors for both positions closing:
   - Long leg → `SellToClose`
   - Short leg → `BuyToClose`
2. Validates both near zero
3. Timeout: 20 seconds

**Transitions to**:
- `Zombie` (all positions closed, bot terminates)

---

### Initialization Flow

#### **Fresh Bot** (no prior save file)
```
1. Request underlying asset details
2. Request initial cash allocation
3. Wait for swap approval + account update
4. Request first underlying quote
5. Calculate ATM by finding closest strike
6. Select long leg: ATM + option_bucket strikes
7. Select short leg: long index − spread_bucket strikes
8. Request ticker data for both options
9. Transition to Begins state
```

#### **Resumed Bot** (loading from save)
```
1. Load brain state from ./dat/{friendly_name}.json
   (includes: current state, prices, positions, timestamps)
2. Request tickers for all tracked options
3. Continue from saved state
   (usually Logic1 or Logic2)
```

---

## Key Formulas

### Spread Entry Trigger
**For PUT spreads** (bearish):
```
spread_width = short_strike − long_strike
required_short_price = spread_width − entry_price × (1 − p)
where p = exit_gain_pct / 100

Example:
  Long strike:        $94
  Short strike:       $98
  Entry price:        $2.00
  exit_gain_pct:      50%
  
  spread_width = 98 − 94 = 4.00
  p = 0.50
  required_short_price = 4.00 − 2.00 × (1 − 0.50)
                       = 4.00 − 1.00
                       = 3.00
  
  Meaning: Sell the $98 put for at least $3.00 to lock in 50% profit
```

**For CALL spreads** (bullish):
```
spread_width = long_strike − short_strike
required_short_price = entry_price × (1 + p) + spread_width

Example:
  Long strike:        $105
  Short strike:       $100
  Entry price:        $2.00
  exit_gain_pct:      50%
  
  spread_width = 105 − 100 = 5.00
  p = 0.50
  required_short_price = 2.00 × (1 + 0.50) + 5.00
                       = 3.00 + 5.00
                       = 8.00
```

### Position Sizing
```
available_cash = min(cash_in_account, max_cash_risk)
target_price = theo_price OR (bid + 0.75×(ask−bid))
max_contracts = floor(available_cash / (target_price × 100))
order_quantity = max_contracts
order_cost = target_price × max_contracts × 100
```

### Floor Calculation (Stop-Loss)
```
floor = entry_price × (1 + exit_loss_pct/100)

Example:
  Entry: $2.00
  exit_loss_pct: -80.0
  floor = 2.00 × (1 + (−80.0)/100)
        = 2.00 × 0.20
        = 0.40
  
  Position exits if it drops below $0.40
```

---

## Timeout & Error Handling

| Event | Timeout | Action |
|-------|---------|--------|
| Ticker request | 5 seconds | Zombie (terminate) |
| Initial cash allocation | 2 seconds | Zombie (terminate) |
| Entry order fill | 20 seconds (partial) | Zombie (terminate) |
| Spread order fill | 20 seconds (partial) | Zombie (terminate) |
| Loss exit fill | 20 seconds (partial) | Zombie (terminate) |
| Final close fill | 20 seconds (partial) | Zombie (terminate) |

**Partial Fill**: If quantity filled is less than requested and doesn't increase within timeout period, bot terminates.

---

## State Persistence

### Saving
- **Trigger**: Bot transitions to `Stopped` mind state
- **File**: `./dat/{friendly_name}.json`
- **Content**: Complete `StealthBot` brain structure (all positions, prices, state)
- **Automatic**: Created on save; directory created if missing

### Loading
- **On startup**: Checks for `./dat/{friendly_name}.json`
- **If found**: Deserializes and resumes from that state
- **If not found**: Starts fresh with initialization flow

---

## Example Scenarios

### Scenario 1: Successful Put Credit Spread
```
Configuration:
  Direction:       Corrects (bearish)
  option_bucket:   1
  spread_bucket:   1
  exit_gain_pct:   50.0
  exit_loss_pct:   -80.0

Execution:
  1. SPY at $450 → select $448 put (long), $450 put (short)
  2. [Begins] Buy $448 put for $2.00 → State: Check1
  3. [Check1] Fill confirmed at $1.95 average → State: Logic1
  4. [Logic1] Underlying drops slightly, $448 put rises to $3.10
     → Required short price: $2.00 × (1−0.50) + 2 = $3.00
     → $3.10 ≥ $3.00 → Trigger spread
  5. [Check2] Sell $450 put for $3.10 → Fill confirmed → State: Logic2
  6. [Logic2] Monitor spread
     → Current: Long $2.80, Short $1.50
     → Profit: $3.10 − ($2.80 − $1.50) = $1.80
     → SPY rallies to $452 → $450 put goes OTM
     → OtmShort condition met
  7. [Check4] Close both legs
     → Sell $448 put for $0.05
     → Buy $450 put for $0.01
     → Net P&L: +$1.80 on initial $2.00 investment = 90% return
```

### Scenario 2: Stop-Loss Triggered
```
Same setup, but:
  
  2. [Begins] Buy $448 put for $2.00
  3. [Check1] Fill confirmed → State: Logic1
  4. [Logic1] Unexpected news → $448 put drops to $0.30
     → Floor: $2.00 × 0.20 = $0.40
     → $0.30 < $0.40 → Stop-loss triggered
  5. [Check3] Sell $448 put for $0.30 → Fill confirmed
  6. Bot terminates with loss: -$1.70 on $2.00 investment = -85% return
```

---

## Best Practices

### Safe Configuration
- **`option_bucket`**: 1-3 (avoid extremely OTM)
- **`spread_bucket`**: 1-2 (keep spread width reasonable)
- **`exit_gain_pct`**: 30-100% (realistic profit targets)
- **`exit_loss_pct`**: -50% to -100% (defined risk)
- **`option_expire`**: 30-60 days (sufficient time decay benefit)

### Exit Strategy Design
- Always include at least one exit condition
- Combine `OtmShort` with a percentage target for redundancy
- Use `PriorDay` to avoid overnight expiration risk
- `GoExpire` should be tested carefully with small positions

### Risk Management
- Start with `max_cash_risk: (false, 25.0, 1000.0)` (conservative)
- Trade one contract per position initially
- Monitor the saved state files for recovery testing
- Set `exit_loss_pct` to match your risk tolerance

---

## Troubleshooting

| Issue | Likely Cause | Solution |
|-------|--------------|----------|
| Bot stuck in Check1 | Partial fill not completing | Increase timeout or reduce position size |
| Bot terminates immediately | Ticker request failed | Verify tracking_tick symbol is valid |
| Spread never triggers | exit_gain_pct too high | Reduce target profit % |
| Unexpected zombies | Account sync issues | Check Dr. Robo account update flow |
| Save file not created | Permissions or path | Verify `./dat/` directory writable |

---

## Implementation Reference

### Key Files
- **Bot logic**: `trade/usta/src/stealth_bot.rs`
- **Data structures**: `dsta/src/lib.rs` (MakeStealthBot, StealthBotDoneNiceStyle)
- **Tests**: `stealth_bot_happy_path_*` test suite

### Channel Interface
The bot communicates via async tokio channels:
- **Ticker updates**: `Receiver<Ticker>`
- **Account updates**: `Receiver<Allocation>`
- **Trade submissions**: `Sender<Order>`
- **Status reports**: `Sender<Snively::SendLogNote>`

### Expected Account Updates
Dr. Robo provides periodic `Allocation` structs containing:
- `bot_safe.m_long_puts` / `m_long_calls` (owned positions)
- `bot_safe.m_short_puts` / `m_short_calls` (short positions)
- `bot_safe.m_cash` (available cash)
- `bot_mind` (Running, Trading, Stopped, Zombied, Seppuku)


older stuff:


//The design of this bot is the example to follow for a good bot implementation (sans haggle logic - for l8er)
// 1) reports zombied when we are done with all tasks
// 2) saves state (./dat/{fridendly_name}.json ) when transitioning to and from "stop"
// 3) checks for existing save at startup, loads first.
// 4) one function to create and launch one state machine thread
// 5) acquire (cash) assets only on first birth
// 6) request tracking tickers at state machine startup before infinite tokio select loop on our API 
// 7) seperate logic into ticker updates and mind transitions; 
// 8) we inform the frontend with f_report_* (sends a snively) during startup, and state machine state enum changes and order placement 

/*

  // What the stealth bot does, basically converts profitable long options into locked-in profit option spreads:

  option bucket = integer number of OTM strikes past ATM we chose for entry option
  spread bucket = integer number of ITM strikes past ATM we chose for exit spread option, spread bucket, exit loss, other exit criteria.
  always designated in the strkie low->high

  For example a PUT:

             option bucket         spread bucket
  LONG PUT | ------------- | ATM | ------------- | SHORT PUT 
  Low price                                        high price

  So the algo:

  we long the option at or within the option bucket with our cash allocation.
  we monitor for loss on that position for early exit, or monitor profit for
  opening spread. the 'monitor profit' looks at the locked in profit when we 
  sell the short side. 

  example:

    SPY @690 
    , MarketDirection::Corrects (bearish)
    , -6 option buck bucket = 684 put
    , -2 option buck bucket = 688 put

    we buy 684 put for X = $1 (itm @ 683). say we target a locked-in profit 
    of 50%, aka 684 put become worth $1.50, or when it's breakeven is 682.5
    Thus We look for the price of the 688 put to have the same breakeven at
    682.5, or when it is $5.50; selling that would lock in at least the 50% 
    gain in 684 put (and allow for spread gains).
    
  NOTE: as coded, we actually look at the spread bucket as relative to 
  the long option strike and not the ATM.

  begin_1 - 
    implictly waits for ticker data from all parts, then buys the options
    we should see our account information transition to trading to confirm

  check_1 - 
    wait for the order to complete - we should see the order get filled in
    our account information as well and transition from trading to running.
    fail (log error, report zombie and end task loop) if we wait too long.

    // NOTE: In the future we will do haggle logic here.

  logic_1 = 
    monitor the conditions for triggering the next actions to take:

      exit loss -> create exit order and execute, transition to check_2

      exit good -> create short option order and execute, using check_3

  check_2 = 
    wait for the exit order to complete - report zombie and end task loop
    if we wait too long fail like above.


  check_3 = 
    wait for the exit order to complete -  transition to logic_2
    if we wait too long fail like above.

  logic_2 = 

    monitor the conditions for triggering a final exit as specified
    by the construction parameters (e.g., short goes OTM, total profit, etc)
    since we don't care about trailing stops at this point

  check_4 = wait for the order to fill, commit seppuku
*/


// dsta shared objects:
/*
#[derive(Default, Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct CommonBotInfo 
{ // Bots assumed to run their trade algos multiple times.
  pub friendly_name: String,
  pub tracking_tick: String, // because all bots only follow 1 
  pub haggle_method: HaggleMethod, // how to enter trades
  pub haggle_action: HaggleAction, // what to do when we wait to long 
  pub haggle_number: u64, // Number of times we had to deal with a waiting order with resubmission
  pub max_cash_risk: (bool, f64, f64 ), // ( ? , %, $ ) bool == false => select min of next 2: % of total cash or set cash value; bool = true choose max. This specifies max $ to use in a bot's algo
  pub my_wins_count: u64, // total winning trades
  pub my_loss_count: u64, // number of losing trades
  pub my_wins_total: f64, // total dollars won
  pub my_loss_total: f64, // total dollars lost  (mark to market = won + lost)
  pub most_won_cash: (f64,f64), // (% won based on max cash risk, cash won) in a single trade => high score, max of below vector
  pub pl_performace: Vec<(chrono::NaiveTime, chrono::NaiveTime,f64,f64)> // (1st order complete time, exit positions order complete time, % won ^ , cash won), for each trade
}

#[derive(Default, Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct MakeStealthBot 
{ pub my_accounting: CommonBotInfo, // For Account managment
  pub my_cash_alloc: f64, // startup we request this. track updates from bot activity for future startups.
  pub stonk_feeling: MarketDirection, // for entry, we buying put/call?
  pub option_expire: u16, // minimum days for option expiry
  pub option_bucket: u8, // for entry, 0 => atm, +1 to go otm by 1, etc
  pub spread_bucket: u8, // for exit, number of buckets from entry itm direction
  pub exit_gain_pct: f64, // // 0 to +100% (or higher), as a function of target_spread
  pub exit_loss_pct: f64, // 0 to -100% but can go below -100% due to being a spread in a way...
  pub nice_exit_way: Vec<StealthBotDoneNiceStyle>, // Position Exit Strategy 
}

#[derive(Default, Clone, Debug, serde::Serialize, serde::Deserialize)]
pub enum StealthBotDoneNiceStyle 
{
  #[default] 
  OtmShort, // short leg goes OTM 
  FinalPct(f64), // short leg gain %
  FinalDay(UsaMarketTimes, f64), // exit on last day of trading
  PriorDay(UsaMarketTimes, f64, u64), // exit usize days before expiration 
  GoExpire, // let the contract expire
}
*/


Even more stuff:

// KEEP THE COMPACT STYLE YOU SEE AT THE BEGINNING OF THIS CODE
// nothing but whitespace preceeding a '{' 
// make sure that the closer will align } in the column
// 2 space tabs.
// Don't remove or alter comments.
// No '{' at the end of a line
// Put code or comment on same line as '{' like:
// example()
// { hello
// }

// Basically, what the buzz bomber does is sells OTM credit spreads that
// meet entry criteria when we have money, and monitors their prices for exits
// and then repeats that process indefintly

/* 

  logic_0 - 
    wait until the entry time window happens, re-request the stock deets
    which has the options tree. find the expiry date and select ~20 options
    to track and switch to logic_1

  logic_1 - 
    as ticker data comes in, we check adjacent strikes to see if their spread value matches entry criteria
    if ok, we submit the entry order & go to check_1

  check_1 - 
    wait for the order to complete - we should see the order get filled in
    our account information as well and transition from trading to running.
    fail (log error, report zombie and end task loop) if we wait too long.
    then we transition to cools_1
  
  cools_1 - 
    wait for cooldown period to expire, then transition to logic_2

  logic_2 - 
    check the exit strategy, if ok submit order & go to check_2
    (only kill / seppuku zombie reports end our flow here)

  check_2 - DONE 
    wait for the exit order to complete (no assets)
    if stone stuff says to repeat, go to logic_0.
    if we are 1-and-done, - raise error so we report zombie and end task.

  Like all bots:

    we shall do state machine processing on tickers

    we shall do mind machine processing on self updates

    we shall load at startup if existing, 

    we shall only request cash/asset resources at birth (non existing save)

    we shall save on transition to and from stopped

  Robo framework gaurentees:

    bots are told seppuku when they are told to die by dr_robo
    bots are also told "DIE" when they are killed by admin
    robo will deallocate resoruces from zombied bots automatically.
    when we send a snively report, it gets sent to the frontend user.

  Best strategy:

    launch main task at creation 

    main task does startup and runs tokio select loop on the 3 core api channels

    when a dispatched task fails, exit loop, report zombied, report completed, end function

    report errors to log::error and the frontend, like stealthbot's f_run_errors

    report startup / shutdown, major state changes and actions like the stealth bot

*/


// trade/usta/src/sbot/swat_bot.rs
//
// Internal-only bot, naming convention USER_[TICKER] or DEAD_[OLD_BOT_NAME]
// swat monitors its position(s), reporting tickers to the frontend (snd snively), and can exit positions.
// When it gets a Seppuku update, it shall create an exit limit order with haggle logic.
// When it gets an account update with empty assets, it will report Zombied.
// Persists state to disk on Stopped transitions and resumes on startup.
