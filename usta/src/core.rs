// trade/usta/src/core.rs 
use crate::*;

// =================================================================
// CORE HELPER FUNCTIONS - 
// =================================================================

/// Returns true if the account can be safely broken into independent pieces
/// and dispersed to simple SwatBots without losing margin benefits or violating rules.
pub(crate) fn can_safely_disperse(account: &BotaAccount) -> bool {
    // If any spread/condor leins exist → entangled
    if account.m_call_credit_spread_lein > 0.01
        || account.m_put_credit_spread_lein > 0.01
        || account.m_short_iron_condor_lein > 0.01
        || account.m_short_put_lein > 0.01
    {
        return false;
    }

    // If short call exists and stock exists → likely covered call, keep together
    if account.m_short_calls.is_some() && account.m_stock.is_some() {
        return false;
    }

    // Otherwise: only naked longs, or isolated short puts (but short_put_lein already blocked above)
    true
}

/// Determines if a 2-leg order is closing an existing spread
fn is_closing_spread
( order: &dsta::Order,
  account: &BotaAccount,
  stuff: &std::collections::HashMap<String, dsta::FinAss>,
) -> bool 
{ if order.order_legs.len() != 2 
  { return false;
  }

  let leg1 = &order.order_legs[0];
  let leg2 = &order.order_legs[1];

  // Check if both legs are closing actions
  let both_closing = matches!(leg1.action, dsta::BuyOrSell::SellToClose | dsta::BuyOrSell::BuyToClose)
    && matches!(leg2.action, dsta::BuyOrSell::SellToClose | dsta::BuyOrSell::BuyToClose);

  if !both_closing 
  { return false;
  }

  // Determine if it's a call or put spread being closed
  let is_call_spread = order.order_legs.iter().all
  ( |leg| 
    { if let Some(dsta::FinAss::OptDeets(o)) = stuff.get(&leg.buy_what.order_name()) 
      { matches!(o.option_rite, dsta::OptRite::Call)
      } else 
      { false
      }
    }
  );

  let is_put_spread = order.order_legs.iter().all
  ( |leg| 
    { if let Some(dsta::FinAss::OptDeets(o)) = stuff.get(&leg.buy_what.order_name())
      { matches!(o.option_rite, dsta::OptRite::Put)
      } else 
      { false
      }
    }
  );

  if !is_call_spread && !is_put_spread 
  { return false;
  }

  // Check if account has matching spread positions
  if is_call_spread 
  { account.m_long_calls.is_some() && account.m_short_calls.is_some()
  } else 
  { account.m_long_puts.is_some() && account.m_short_puts.is_some()
  }
}

// Higher level Virtual layer Account management (for Dr Robo)
/*

I need to update my check legality and other functions, since i changed the OrderStructure to not include the cost; and I updated my FinAss structure to be cognizant of its ticker pricing scheme, so 

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct OrderLeg 
{ pub buy_what: FinAss,
  pub quantity: f64,
  pub remaining: f64,
  pub action: BuyOrSell,
  pub price: f64,
}

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct Order 
{ pub order_legs: Vec<OrderLeg>,
  pub order_type: MarketOrLimit,
  // pub order_cost: Option<f64>, // ELIMINATED - calculated elsewhere
}

impl FinAss
{ pub fn ticker_name(&self) -> String
  pub fn order_name(&self)-> String
  pub fn set_ticker(&mut self, tick : Ticker)
  pub fn last_ticker(&self) -> Option<Ticker>
  /// Returns the contract multiplier (notional value per point / per contract).
  /// This is used to compute actual dollar exposure / cash effect:
  ///   - For futures: price * multiplier
  ///   - For options on futures: premium * multiplier
  ///   - For stock options: usually 100
  ///   - For stocks: 1
  pub fn get_multiplier(&self) -> f64 
  /// Returns actual cash impact for **one contract / one share** at given price.
  /// Examples:
  ///   - /ES future at 5000.00 → 50 * 5000 = $250,000 notional
  ///   - /ES option premium 5.00 → 50 * 5.00 = $250 cash per contract
  ///   - AAPL stock at 200.00 → 1 * 200 = $200
  ///   - AAPL option at 3.50 → 100 * 3.50 = $350
  pub fn get_normalized_cost(&self, price: f64) -> f64 
}

so my core checks / cost calculations need to be updated.
  RULES FOR TRADING SYSTEM: 
  1) a bota account, and thus a bot, may only track one kind of underlying that would match the stock if it owns a stock.
  2) naked short calls and naked short stocks are not allowed to be entered, but a broker action or early exercise may create a short stock position; the system takes care of it eslewhere, thus we enforce this..
  3) only a finite set of positions are therefore allowed; aside from pure longs, we have this which require collateral or a lein:
  
  The task is to update the order‑legality and cash/margin logic so it no longer depends on order_cost and instead uses each FinAss’s normalized cash effect (via get_multiplier / get_normalized_cost), while still enforcing the one‑underlying rule and the constrained set of allowed positions (no naked short calls/stock; only specific spread/condor structures).

High‑level changes needed

Move all cost/margin reasoning off order.order_cost and onto per‑leg normalized cash impact computed from FinAss::get_normalized_cost(leg.price) (or price * get_multiplier).

Ensure spread type detection (credit vs debit) and margin sizing use normalized notional (strike differences × multipliers × quantity), not sign/size of a global order cost.

Keep the structural rules:
  Single underlying per BotaAccount and per order (aside from cash-only).
  No SellToOpen stock; short stock may only appear via external/broker events, handled elsewhere.
  etc. 

Concrete code changes

0. Data structures / helpers:
  Confirm Order has no order_cost field anymore (already done):
  Order now has pub fn fn_calculate_order_cost(self : &dsta::Order)-> Result<f64, String> (already done) 

1. fn_check_bota_legality changes
  Remove all uses of order.order_cost from this function.
  Keep The “legless order with cost” branch; it is essential for robo
  Underlying consistency:

  Keep get_und and the logic that:

  Uses account.m_stock’s underlying if present.
  Otherwise infers from the first leg’s buy_what.order_name().

  For each leg, continue validating all legs share the same underlying.

  Stock rules:
    Keep the enforcement:
    SellToOpen on stock → error.
    SellToClose/BuyToClose require an existing stock position with matching ticker and sufficient owned quantity.
    Ensure you do not rely on order_cost when checking stock anymore; only quantities and actions; we use the leg's normaized_cost

  Option rules (single legs):
    Keep the logic that:

  BuyToOpen/SellToOpen create or modify positions subject to:
    No second put/call series (must either reduce or increase an existing short or match that series).
    Closing actions check FinAss::order_name() equality and sufficient owned absolute quantity.
    Add a sanity check on leg cost sign:
    For each leg, given unit_cash = get_normalized_cost(leg.price):
    Buy* actions should have unit_cash > 0 (cash outflow per contract); effective cash change is -unit_cash * qty.
    Sell* actions should also have unit_cash > 0, but the order‑side sign comes from the action.
    If a leg’s pricing is “nonsensical” (e.g, buytoopen with positive price), reject.

  Spread detection and classification in legality check
  Keep the structural detection for 2‑leg call or put spreads using stuff to confirm option type.
  Replace is_credit_spread = order.order_cost.map_or(false, |cost| cost >= 0.0) with:
  Compute leg‑level normalized cash effects (as in the helper above) and sum to order_cash_net.
  Set:
    is_credit_spread = order_cash_net > 0.0.
    Debit spread if order_cash_net <= 0.0.

  Strike validation:
    Keep current strike ordering logic (buy vs sell leg, credit vs debit) but no longer depend on order_cost.
    Only use strike relationships and is_credit_spread flag determined from normalized cash.

  Spread margin check:
    Replace:
    spread_margin = (buy_opt.strike_spot - sell_opt.strike_spot).abs() * 100.0 * buy_leg.quantity
    with:
    spread_width = (buy_opt.strike_spot - sell_opt.strike_spot).abs().
    mult = buy_leg.buy_what.get_multiplier()
    spread_margin = spread_width * mult * buy_leg.quantity.
  
  Check account.m_cash >= spread_margin as before.
  since we calculate the normalized order cost total (todo) 

Where logic uses order.order_cost to decide whether this is credit / debit, use the normalized cost instead


2. apply_order_to_account changes
  This function sadly still uses order.order_cost heavily to adjust stock cost basis and to infer debit/credit for spreads.
  Introduce a computed trade_net:
  trade_net = order.fn_calculate_order_cost();

  Replace uses of order.order_cost in this function:

  For determining is_credit:

    Replace let is_credit = order.order_cost.map_or(false, |cost| cost >= 0.0); with let is_credit = trade_net > 0.0. 

  For stock price/basis updates:

  When updating or creating a stock position, compute total dollar traded in that leg set:
    For each stock leg, compute leg_cash = +/- get_normalized_cost(leg.price) * qty depending on action.
    Combine with order_stock_change to derive an average price per share for basis.
    Replace uses of order.order_cost.expect("ORDER COST FAIL") in price/basis calculations with a dedicated stock_gross value computed only from the stock legs.
    For options, the netting logic between long/short maps can remain, but any place where order_cost is used to detect credit vs debit or to compute something monetary should use normalized cash.

3. calculate_option_margin changes

  Core margin logic is already in terms of strikes, quantities, and a 100‑share multiplier.
  Replace hard‑coded 100.0 multipliers with get_multiplier() to support non‑equity underlyings:
  For iron condor margin: amount = max(call_width, put_width) * mult * condor_qty, where mult is from one of the option FinAsses.
  For call and put credit spreads: (strike_diff) * mult * spread_qty.
  For short puts: required cash = ps_strike * mult * remaining_short_qty.
  Ensure all quantities use absolute contract counts (-cs_qty, -ps_qty, etc.) as now, but scale with multipliers instead of assuming 100.

*/
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum LeinType 
{ // Stock bots never enter  
    CallCreditSpread
  , ShortPut
  , PutCreditSpread
  , IronCondor,
}

#[derive(Debug, Clone)]
pub struct AccountCashChanges 
{  // what gets added to each lien from the cash pile
    pub cash_transfer_to_leins: Vec<(LeinType, f64)>
  , pub cash_gain_from_order: f64 // 
}

/// Performs basic legality-of-order checks for an account given the account
///
/// # Arguments
/// * `account` - Current Account Information
/// * `order` - Requested Order to check
/// * `stuff` - HashMap of FinAss details keyed by ticker name
///
/// # Checks:
/// * Options must have same underlying
/// * Stock cannot SellToOpen (No Shorting!)
/// * Buy/Sell ToClose must have enough abs(quantity) available
/// * Validates call and put credit and debit spreads
/// * Uses normalized cost (per-leg FinAss multiplier) for cash/margin calculations
pub fn fn_check_bota_legality
( account: &BotaAccount
, order: &dsta::Order
, stuff: &std::collections::HashMap<String, dsta::FinAss>
) -> Result<(),String> 
{ let dbgspt = format!("[{}]", crate::glossary::LOG_PREFIX_BOTA_ACCOUNT_OK);
  log::debug!("{} [ENTRY!] = fn_check_bota_legality with account: {:#?}, order: {:#?} ", dbgspt, account, order);
  
  if order.order_type != dsta::MarketOrLimit::Limit 
  { log::debug!("{} [EXIT!] : Order type is not Limit; checked order_type: {:?}", dbgspt, order.order_type);
    return Err(format!("{} Orders must all be LIMIT; checked order_type: {:?}", dbgspt, order.order_type));
  }
  
  fn get_und(wat : &str) -> Result<String,String>
  { let dbgspt = format!("[{}]", crate::glossary::LOG_PREFIX_BOTA_ACCOUNT_OK);
    let fw = wat.split_whitespace().next();
    match fw
    { Some(word) => { Ok(format!("{}",word)) }
    , None => 
      { Err(format!("{} Can't get Underlying : IMPOSSIBLE TO HANDLE NAME STRING = [{}]", dbgspt, wat))
      }
    }
  }

  // Get underlying
  let ulying = 
  { if let Some(pos) = &account.m_stock
    { format!("{}", pos.asset.order_name())
    }
    else if order.order_legs.len() > 0
    { get_und(&order.order_legs[0].buy_what.order_name())? // only setter for underlying
    }
    else if order.order_legs.len() == 0
    { log::warn!("{} [OK EXIT!] EMPTY ORDER being allowed (cash flows)", dbgspt);
      return Ok(());
    }
    else
    { let errmsg = format!("{} - {}{}", dbgspt
      , format!(" undefined underlying : account = {:?}" , account)
      , format!("\norder={:#?}", order)
      );
      log::error!("{}",errmsg);
      log::debug!("{} [BAD EXIT!] BAD ORDER!", dbgspt);
      return Err(errmsg);
    }
  };
  
  // Check underlyings in order
  for leg in &order.order_legs 
  { let und = get_und(&leg.buy_what.order_name())?;
    if und != ulying
    { let erm = format!
      ( "{} [BAD EXIT!]: Different underlyings in order legs; existing: {}, bad: {}", dbgspt
      , ulying
      , und
      );
      log::error!("{}", erm);
      return Err(erm);
    }
  }
  
  // Calculate normalized order cash to determine spread type
  let order_cash_net = order.order_legs.iter().try_fold(0.0, |acc, leg| -> Result<f64, String>
  { let unit_cash = leg.buy_what.get_normalized_cost(leg.price);
    let leg_cash = match leg.action
    { dsta::BuyOrSell::BuyToOpen | dsta::BuyOrSell::BuyToClose => -unit_cash * leg.quantity
    , dsta::BuyOrSell::SellToOpen | dsta::BuyOrSell::SellToClose => unit_cash * leg.quantity
    , _ => return Err(format!("{} Invalid action in leg", dbgspt))
    };
    Ok(acc + leg_cash)
  })?;
  
  let is_credit_spread = order_cash_net > 0.0;
  
  // Check for call or put spread validity
  let is_call_spread = order.order_legs.len() == 2
  && order.order_legs.iter().all
  ( |leg| 
    { if let Some(dsta::FinAss::OptDeets(o)) = stuff.get(&leg.buy_what.order_name())
      { matches!(o.option_rite, dsta::OptRite::Call)
      } 
      else 
      { false
      }
    }
  );
  
  let is_put_spread = order.order_legs.len() == 2
  && order.order_legs.iter().all
  ( |leg| 
    { if let Some(dsta::FinAss::OptDeets(o)) = stuff.get(&leg.buy_what.order_name())
      { matches!(o.option_rite, dsta::OptRite::Put)
      } 
      else 
      { false
      }
    }
  );
  
  // Check if this is closing an existing spread
  if (is_call_spread || is_put_spread) && is_closing_spread(order, account, stuff) 
  { log::warn!
    ( "{} Closing existing {} spread, skipping initial spread validation"
    , dbgspt, if is_call_spread { "call" } else { "put" }
    );
    // Fall through to single-leg validation which will verify positions exist
    // Don't return here - let it validate each leg can be closed
  }
  else if is_call_spread || is_put_spread 
  { let leg1 = &order.order_legs[0];
    let leg2 = &order.order_legs[1];
    
    let (buy_leg, sell_leg) = if is_credit_spread 
    { // Credit spread: SellToOpen is the primary action
      if matches!(leg1.action, dsta::BuyOrSell::SellToOpen) 
      { (leg2, leg1)
      } 
      else if matches!(leg2.action, dsta::BuyOrSell::SellToOpen) 
      { (leg1, leg2)
      } 
      else 
      { let em = format!("{} [BAD EXIT!] fn_check_bota_legality: Invalid credit spread actions; leg1: {:?}, leg2: {:?}"
        , dbgspt, leg1.action, leg2.action
        );
        log::error!("{}", em);
        return Err(em)
      }
    } 
    else 
    { // Debit spread: BuyToOpen is the primary action
      if matches!(leg1.action, dsta::BuyOrSell::BuyToOpen) 
      { (leg1, leg2)
      } 
      else if matches!(leg2.action, dsta::BuyOrSell::BuyToOpen) 
      { (leg2, leg1)
      } 
      else 
      { let em = format!("{} [BAD EXIT!] : Invalid debit spread actions; leg1: {:?}, leg2: {:?}"
        , dbgspt, leg1.action, leg2.action);
        log::error!("{}", em);
        return Err(em)
      }
    };

    let buy_asset = stuff.get(&buy_leg.buy_what.order_name());
    let sell_asset = stuff.get(&sell_leg.buy_what.order_name());
    
    let buy_opt = if let Some(dsta::FinAss::OptDeets(o)) = &buy_asset { o } else { 
        let em = format!("{} [BAD EXIT!] : Invalid buy leg asset for spread", dbgspt);
        log::error!("{}",em);
        return Err(em); 
    };
    
    let sell_opt = if let Some(dsta::FinAss::OptDeets(o)) = &sell_asset { o } else { 
        let em = format!("{} [BAD EXIT!] : Invalid sell leg asset for spread", dbgspt);
        log::error!("{}",em);
        return Err(em); 
    };

    // Validate strike configuration
    if is_call_spread {
        if is_credit_spread && buy_opt.strike_spot <= sell_opt.strike_spot {
            let em = format!("{} [BAD EXIT!] : Invalid call credit spread, buy strike {} <= sell strike {}", dbgspt, buy_opt.strike_spot, sell_opt.strike_spot);
            log::error!("{}",em);
            return Err(em);
        }
        if !is_credit_spread && buy_opt.strike_spot >= sell_opt.strike_spot {
            let em = format!("{} [BAD EXIT!] : Invalid call debit spread, buy strike {} >= sell strike {}", dbgspt, buy_opt.strike_spot, sell_opt.strike_spot);
            log::error!("{}",em);
            return Err(em);
        }
    }
    
    if is_put_spread {
        if is_credit_spread && buy_opt.strike_spot >= sell_opt.strike_spot {
            let em = format!("{} [BAD EXIT!] : Invalid put credit spread, buy strike {} >= sell strike {}", dbgspt, buy_opt.strike_spot, sell_opt.strike_spot);
            log::error!("{}",em);
            return Err(em);
        }
        if !is_credit_spread && buy_opt.strike_spot <= sell_opt.strike_spot {
            let em = format!("{} [BAD EXIT!] : Invalid put debit spread, buy strike {} <= sell strike {}", dbgspt, buy_opt.strike_spot, sell_opt.strike_spot);
            log::error!("{}",em);
            return Err(em);
        }
    }

    // ★★★ NEW: Calculate actual margin requirement accounting for net cash flow ★★★
    let spread_width = (buy_opt.strike_spot - sell_opt.strike_spot).abs();
    let mult = buy_leg.buy_what.get_multiplier();
    let max_loss = spread_width * mult * buy_leg.quantity as f64;
    
    let required_margin = if is_credit_spread {
        // Premium collected reduces margin requirement
        (max_loss - order_cash_net).max(0.0)  // Can't be negative
    } else {
        // Debit spread requires paying the net cost
        order_cash_net
    };
    
    if account.m_cash < required_margin {
        let em = format!(
            "{} [BAD EXIT!] : Insufficient cash for {} spread; required: {}, available: {}", dbgspt,
            if is_credit_spread { "credit" } else { "debit" },
            required_margin,
            account.m_cash
        );
        log::error!("{}",em);
        return Err(em);
    }

    // Check existing positions for compatibility
    let (short_pos, long_pos) = if is_call_spread {
        (&account.m_short_calls, &account.m_long_calls)
    } else {
        (&account.m_short_puts, &account.m_long_puts)
    };

    // Check if account has an existing credit spread
    if is_credit_spread {
        if let (Some(short_opt), Some(long_opt)) = (short_pos, long_pos) {
            if let (dsta::FinAss::OptDeets(existing_short), dsta::FinAss::OptDeets(existing_long)) = (&short_opt.asset, &long_opt.asset) {
                let is_existing_spread = if is_call_spread {
                    existing_long.strike_spot > existing_short.strike_spot
                } else {
                    existing_long.strike_spot < existing_short.strike_spot
                };
                if is_existing_spread && existing_long.expire_date == existing_short.expire_date {
                    // Check if the sell leg reduces the existing long position
                    if existing_long.option_name == sell_opt.option_name && sell_leg.quantity as f64 >= long_opt.owned {
                        log::info!("{} [OK EXIT!] : {} credit spread allowed, sell leg reduces existing long position {}", dbgspt, 
                            if is_call_spread { "Call" } else { "Put" }, sell_opt.option_name);
                        return Ok(());
                    }
                    // Check if the buy leg reduces the existing short position
                    if existing_short.option_name == buy_opt.option_name && buy_leg.quantity as f64 >= short_opt.owned.abs() {
                        log::info!("{} [OK EXIT!] : {} credit spread allowed, buy leg reduces existing short position {}", dbgspt, 
                            if is_call_spread { "Call" } else { "Put" }, buy_opt.option_name);
                        return Ok(());
                    }
                    // Allow if new spread matches existing spread exactly
                    if existing_short.option_name == sell_opt.option_name && existing_long.option_name == buy_opt.option_name {
                        log::info!("{} [OK EXIT!] : New {} credit spread matches existing spread, allowing increase", dbgspt, 
                            if is_call_spread { "call" } else { "put" });
                        return Ok(());
                    }
                    let em = format!("{} [BAD EXIT!] : New {} credit spread does not match existing spread, does not reduce existing short or long; existing short: {}, new short: {}, existing long: {}, new long: {}", dbgspt, 
                        if is_call_spread { "call" } else { "put" }, existing_short.option_name, sell_opt.option_name, existing_long.option_name, buy_opt.option_name);
                    log::error!("{}", em);
                    return Err(em);
                }
            }
        }
    }

    // For new spreads, check cash coverage and position conflicts
    if is_call_spread { 
        // Check if the new long call conflicts with an existing long call
        if let Some(existing_long) = &account.m_long_calls {
            if let dsta::FinAss::OptDeets(long_opt) = &existing_long.asset {
                if long_opt.option_name != buy_opt.option_name {
                    let reduces_existing_long = order.order_legs.iter().any(|l| {
                        if matches!(l.action, dsta::BuyOrSell::SellToOpen | dsta::BuyOrSell::SellToClose) {
                            if let Some(dsta::FinAss::OptDeets(opt)) = stuff.get(&l.buy_what.order_name()) {
                                opt.option_name == long_opt.option_name && l.quantity as f64 >= existing_long.owned
                            } else {
                                false
                            }
                        } else {
                            false
                        }
                    });
                    if !reduces_existing_long {
                        let em = format!(
                            "{} [BAD EXIT!] : New call {} spread creates conflicting long call; existing long: {}, new long: {}", dbgspt,
                            if is_credit_spread { "credit" } else { "debit" },
                            long_opt.option_name,
                            buy_opt.option_name
                        );
                        log::error!("{}", em);
                        return Err(em);
                    }
                }
            }
        }
        
        // Check if the new short call conflicts with an existing short call
        if let Some(existing_short) = &account.m_short_calls {
            if let dsta::FinAss::OptDeets(short_opt) = &existing_short.asset {
                if short_opt.option_name != sell_opt.option_name {
                    let reduces_existing_short = order.order_legs.iter().any(|l| {
                        if l.action.is_buy() 
                        { if let Some(dsta::FinAss::OptDeets(opt)) = stuff.get(&l.buy_what.order_name()) 
                          { opt.option_name == short_opt.option_name && l.quantity as f64 >= existing_short.owned.abs()
                          } else 
                          { false
                          }
                        } else {
                            false
                        }
                    });
                    if !reduces_existing_short {
                        let em = format!(
                            "{} [BAD EXIT!] : New call {} spread creates conflicting short call; existing short: {}, new short: {}", dbgspt,
                            if is_credit_spread { "credit" } else { "debit" },
                            short_opt.option_name,
                            sell_opt.option_name
                        );
                        log::error!("{}",em);
                        return Err(em);
                    }
                }
            }
        }
        
        log::info!(
            "{} [OK EXIT!] : Call {} spread allowed, sufficient cash coverage (required: {}, available: {})", dbgspt,
            if is_credit_spread { "credit" } else { "debit" },
            required_margin,
            account.m_cash
        );
        return Ok(());
    }

    if is_put_spread {
        // Check if the new long put conflicts with an existing long put
        if let Some(existing_long) = &account.m_long_puts {
            if let dsta::FinAss::OptDeets(long_opt) = &existing_long.asset {
                if long_opt.option_name != buy_opt.option_name {
                    let reduces_existing_long = order.order_legs.iter().any(|l| {
                        if matches!(l.action, dsta::BuyOrSell::SellToOpen | dsta::BuyOrSell::SellToClose) {
                            if let Some(dsta::FinAss::OptDeets(opt)) = stuff.get(&l.buy_what.order_name()) {
                                opt.option_name == long_opt.option_name && l.quantity as f64 >= existing_long.owned
                            } else {
                                false
                            }
                        } else {
                            false
                        }
                    });
                    if !reduces_existing_long {
                        let em = format!(
                            "{} [BAD EXIT!] : New put {} spread creates conflicting long put; existing long: {}, new long: {}", dbgspt,
                            if is_credit_spread { "credit" } else { "debit" },
                            long_opt.option_name,
                            buy_opt.option_name
                        );
                        log::error!("{}",em);
                        return Err(em);
                    }
                }
            }
        }
        
        // Check if the new short put conflicts with an existing short put
        if let Some(existing_short) = &account.m_short_puts {
            if let dsta::FinAss::OptDeets(short_opt) = &existing_short.asset {
                if short_opt.option_name != sell_opt.option_name {
                    let reduces_existing_short = order.order_legs.iter().any(|l| {
                        if matches!(l.action, dsta::BuyOrSell::BuyToOpen | dsta::BuyOrSell::BuyToClose) {
                            if let Some(dsta::FinAss::OptDeets(opt)) = stuff.get(&l.buy_what.order_name()) {
                                opt.option_name == short_opt.option_name && l.quantity as f64 >= existing_short.owned.abs()
                            } else {
                                false
                            }
                        } else {
                            false
                        }
                    });
                    if !reduces_existing_short {
                        let em = format!(
                            "{} [BAD EXIT!] : New put {} spread creates conflicting short put; existing short: {}, new short: {}", dbgspt,
                            if is_credit_spread { "credit" } else { "debit" },
                            short_opt.option_name,
                            sell_opt.option_name
                        );
                        log::error!("{}",em);
                        return Err(em);
                    }
                }
            }
        }
        
        log::info!(
            "{} [OK EXIT!] : Put {} spread allowed, sufficient cash coverage (required: {}, available: {})", dbgspt,
            if is_credit_spread { "credit" } else { "debit" },
            required_margin,
            account.m_cash
        );
        return Ok(());
    }
  }

  // Handle single-leg orders
  for leg in &order.order_legs {
    let bwon = leg.buy_what.order_name();
    let asset = match stuff.get(&bwon) {
        Some(a) => a,
        None => {
            log::error!("Couldn't find {} in {:#?}", bwon, stuff);
            return Err(format!("Single leg order has unknown asset for ticker name"))
        }
    };
    
    log::debug!("Checking asset for {}; asset = {:#?}", leg.buy_what.order_name(), asset);
    match &asset {
        dsta::FinAss::StkDeets(_sasset) => {
            log::debug!("Processing stock leg: action {:?}", leg.action);
            match &leg.action {
                dsta::BuyOrSell::BuyToOpen => {
                    if let Some(pos) = &account.m_stock {
                        if let dsta::FinAss::StkDeets(_pass) = &pos.asset {
                            let ton1 = pos.asset.order_name();
                            let ton2 = asset.order_name();
                            if ton1 != ton2 {
                                let em = format!(
                                    "{} [BAD EXIT!] : Stock asset mismatch for BuyToOpen; existing: {}, new: {}",
                                    dbgspt, ton1, ton2
                                );
                                log::error!("{}", em);
                                return Err(em);
                            }
                        }
                    }
                }
                dsta::BuyOrSell::SellToClose => {
                    if let Some(pos) = &account.m_stock {
                        let existing_name = pos.asset.order_name();
                        let new_name = asset.order_name();
                        if existing_name != new_name {
                            let em = format!(
                                "{} [BAD EXIT!] : Names MISMATCH; existing: {}, new: {}",
                                dbgspt, existing_name, new_name
                            );
                            log::error!("{}", em);
                            return Err(em);
                        }
                        if pos.owned < leg.quantity as f64 {
                            let em = format!(
                                "{} [BAD EXIT!] : Insufficient long stock for SellToClose; owned: {}, quantity: {}",
                                dbgspt, pos.owned, leg.quantity
                            );
                            log::error!("{}", em);
                            return Err(em);
                        }
                    } else {
                        let em = format!(
                            "{} BAD EXIT! : {}",
                            dbgspt,
                            glossary::ERROR_NO_STOCK_POSITION_TO_CLOSE
                        );
                        log::error!("{}", em);
                        return Err(em);
                    }
                }
                dsta::BuyOrSell::BuyToClose => {
                    if let Some(pos) = &account.m_stock {
                        let existing_name = pos.asset.order_name();
                        let new_name = asset.order_name();
                        if existing_name != new_name || pos.owned >= -(leg.quantity as f64) {
                            let em = format!(
                                "{} [BAD EXIT!] : Insufficient short stock or name mismatch for BuyToClose; owned: {}, quantity: {}, existing: {}, new: {}",
                                dbgspt, pos.owned, leg.quantity, existing_name, new_name
                            );
                            log::error!("{}", em);
                            return Err(em);
                        }
                    } else {
                        let em = format!("{} BAD EXIT! : No stock position for BuyToClose", dbgspt);
                        log::error!("{}", em);
                        return Err(em);
                    }
                }
                dsta::BuyOrSell::SellToOpen => {
                    let em = format!("{} BAD EXIT! : SellToOpen stock is illegal", dbgspt);
                    log::error!("{}", em);
                    return Err(em);
                }
                _ => {
                    let em = format!("{} [BAD EXIT!] : Invalid action for stock: {:?}", dbgspt, leg.action);
                    log::error!("{}", em);
                    return Err(em);
                }
            }
        }
        dsta::FinAss::OptDeets(o) => {
            let is_call = matches!(o.option_rite, dsta::OptRite::Call);
            log::debug!("Processing option leg: is_call: {}, action: {:?}", is_call, leg.action);
            
            let pos = if matches!(leg.action, dsta::BuyOrSell::BuyToOpen | dsta::BuyOrSell::SellToClose) {
                if is_call { &account.m_long_calls } else { &account.m_long_puts }
            } else {
                if is_call { &account.m_short_calls } else { &account.m_short_puts }
            };

            if matches!(leg.action, dsta::BuyOrSell::BuyToOpen | dsta::BuyOrSell::SellToOpen) {
                if is_call && matches!(leg.action, dsta::BuyOrSell::SellToOpen) {
                    // Check if the sell leg closes an existing long call
                    if let Some(existing_long) = &account.m_long_calls {
                        if let dsta::FinAss::OptDeets(long_opt) = &existing_long.asset {
                            if long_opt.option_name == o.option_name && leg.quantity as f64 >= existing_long.owned {
                                log::info!(
                                    "{} [OK EXIT!]: SellToOpen call allowed, closes existing long call {}"
                                    , dbgspt, o.option_name
                                );
                                return Ok(());
                            }
                        }
                    }
                    // Check for conflicting short call
                    if let Some(existing_short) = &account.m_short_calls {
                        if let dsta::FinAss::OptDeets(existing) = &existing_short.asset {
                            if existing.option_name != o.option_name {
                                let reduces_existing_short = order.order_legs.iter().any(|l| {
                                    if matches!(l.action, dsta::BuyOrSell::BuyToClose | dsta::BuyOrSell::BuyToOpen) {
                                        if let Some(dsta::FinAss::OptDeets(opt)) = stuff.get(&l.buy_what.order_name()) {
                                            opt.option_name == existing.option_name && l.quantity as f64 >= existing_short.owned.abs()
                                        } else {
                                            false
                                        }
                                    } else {
                                        false
                                    }
                                });
                                if !reduces_existing_short {
                                    let em = format!(
                                        "{} [BAD EXIT!] : SellToOpen call would create second short call; existing: {}, new: {}", dbgspt,
                                        existing.option_name, o.option_name
                                    );
                                    log::error!("{}", em);
                                    return Err(em);
                                }
                            }
                        }
                    }
                    log::info!(
                        "{} [OK EXIT!] : SellToOpen call allowed for {}, no conflicting short call or matches existing", dbgspt,
                        o.option_name
                    );
                    return Ok(());
                }
                // Handle BuyToOpen or non-call SellToOpen
                if let Some(p) = pos {
                    if let dsta::FinAss::OptDeets(pass) = &p.asset {
                        if pass.option_name != o.option_name {
                            log::info!(
                                "{} [OK EXIT!] : Option leg check on {} with action {:?}, different ticker than existing {}, allowing new position", dbgspt,
                                o.option_name, leg.action, pass.option_name
                            );
                            return Ok(());
                        }
                    }
                }
                log::info!(
                    "{} [OK EXIT!] : Option leg check on {} with action {:?}, opening new position", dbgspt,
                    o.option_name, leg.action
                );
                return Ok(());
            } else {
                if let Some(p) = pos {
                    if let dsta::FinAss::OptDeets(pass) = &p.asset {
                        if pass.option_name != o.option_name {
                            let em = format!(
                                "{} [BAD EXIT!] : {} = pass.option_name != o.option_name = {}, ", dbgspt,
                                pass.option_name, o.option_name
                            );
                            log::error!("{}",em);
                            return Err(em);
                        }
                        if p.owned.abs() < leg.quantity as f64 {
                            let em = format!(
                                "{} [BAD EXIT!] : {}; ticker: {}, owned: {}, quantity: {}",
                                dbgspt,
                                crate::glossary::ERROR_INSUFFICIENT_OPTION_POSITION_FOR_CLOSING,
                                pass.option_name, p.owned.abs(), leg.quantity
                            );
                            log::error!("{}",em);
                            return Err(em);
                        }
                    }
                } else {
                    let em = format!(
                        "{} [BAD EXIT!] :{} ticker: {}, action: {:?}",
                        dbgspt,
                        crate::glossary::ERROR_NO_OPTION_POSITION_TO_CLOSE,
                        o.option_name, leg.action
                    );
                    log::error!("{}",em);
                    return Err(em);
                }
            }
        }
        _ => {
            let em = format!("{} [BAD EXIT!] : Invalid asset type for ticker: {}", dbgspt, leg.buy_what.order_name());
            log::error!("{}",em);
            return Err(em);
        }
    }
  }

  log::info!("{} OK EXIT! : Order is valid", dbgspt);
  Ok(())
}


pub(crate) fn apply_order_to_account
( account: &mut BotaAccount
, order: &dsta::Order
, stuff: &std::collections::HashMap<String, dsta::FinAss>
) -> Result<(), String> 
{ let dbgspt = "[CORE_ORD_TO_ACT]";
  log::trace!("{} ENTRY apply_order_to_account, order = {:?}", dbgspt, order);

  // Calculate total net cash effect of order using normalized costs
  let trade_net = order.order_legs.iter().try_fold(0.0, |acc, leg| -> Result<f64, String>
  { let unit_cash = leg.buy_what.get_normalized_cost(leg.price);
    let leg_cash = match leg.action
    { dsta::BuyOrSell::BuyToOpen | dsta::BuyOrSell::BuyToClose => -unit_cash * leg.quantity
    , dsta::BuyOrSell::SellToOpen | dsta::BuyOrSell::SellToClose => unit_cash * leg.quantity
    , _ => return Err(format!("{} Invalid action in leg", dbgspt))
    };
    Ok(acc + leg_cash)
  })?;

  let is_credit = trade_net > 0.0;

  // Extract existing positions
  let existing_long_calls = if let Some(pos) = &account.m_long_calls {
      if let dsta::FinAss::OptDeets(opt) = &pos.asset {
          Some((opt.option_name.clone(), opt.strike_spot, pos.owned, pos.price))
      } else {
          None
      }
  } else {
      None
  };

  let existing_short_calls = if let Some(pos) = &account.m_short_calls {
      if let dsta::FinAss::OptDeets(opt) = &pos.asset {
          Some((opt.option_name.clone(), opt.strike_spot, pos.owned.abs(), pos.price))
      } else {
          None
      }
  } else {
      None
  };

  let existing_long_puts = if let Some(pos) = &account.m_long_puts {
      if let dsta::FinAss::OptDeets(opt) = &pos.asset {
          Some((opt.option_name.clone(), opt.strike_spot, pos.owned, pos.price))
      } else {
          None
      }
  } else {
      None
  };

  let existing_short_puts = if let Some(pos) = &account.m_short_puts {
      if let dsta::FinAss::OptDeets(opt) = &pos.asset {
          Some((opt.option_name.clone(), opt.strike_spot, pos.owned.abs(), pos.price))
      } else {
          None
      }
  } else {
      None
  };

  let mut new_stock = 
  dsta::Stk 
  { ticker_name: format!(""),
    ticker_info: format!(""),
    last_ticker: None,
    tick_rulers: None,
  };

  // Calculate order effects by option type and ticker
  let mut order_long_calls: std::collections::HashMap<String, (f64, f64)> = std::collections::HashMap::new();
  let mut order_short_calls: std::collections::HashMap<String, (f64, f64)> = std::collections::HashMap::new();
  let mut order_long_puts: std::collections::HashMap<String, (f64, f64)> = std::collections::HashMap::new();
  let mut order_short_puts: std::collections::HashMap<String, (f64, f64)> = std::collections::HashMap::new();
  let mut order_stock_change = 0.0;
  let mut stock_leg_cash = 0.0;

  // Validate spread or condor configurations
  let is_call_spread = order.order_legs.len() == 2
      && order.order_legs.iter().all(|leg| {
          if let Some(dsta::FinAss::OptDeets(o)) = stuff.get(&leg.buy_what.order_name()) {
              matches!(o.option_rite, dsta::OptRite::Call)
          } else {
              false
          }
      });

  let is_put_spread = order.order_legs.len() == 2
      && order.order_legs.iter().all(|leg| {
          if let Some(dsta::FinAss::OptDeets(o)) = stuff.get(&leg.buy_what.order_name()) {
              matches!(o.option_rite, dsta::OptRite::Put)
          } else {
              false
          }
      });

  let is_iron_condor = order.order_legs.len() == 4
      && order.order_legs.iter().filter(|leg| {
          if let Some(dsta::FinAss::OptDeets(o)) = stuff.get(&leg.buy_what.order_name()) {
              matches!(o.option_rite, dsta::OptRite::Call)
          } else {
              false
          }
      }).count() == 2
      && order.order_legs.iter().filter(|leg| {
          if let Some(dsta::FinAss::OptDeets(o)) = stuff.get(&leg.buy_what.order_name()) {
              matches!(o.option_rite, dsta::OptRite::Put)
          } else {
              false
          }
      }).count() == 2;

  if is_call_spread || is_put_spread || is_iron_condor {
      let mut call_strikes = Vec::new();
      let mut put_strikes = Vec::new();

      for leg in &order.order_legs {
          if let Some(dsta::FinAss::OptDeets(opt)) = stuff.get(&leg.buy_what.order_name()) {
              let strike = opt.strike_spot;
              if matches!(opt.option_rite, dsta::OptRite::Call) {
                  call_strikes.push((strike, leg.action));
              } else {
                  put_strikes.push((strike, leg.action));
              }
          }
      }

      if is_call_spread || is_iron_condor {
          if call_strikes.len() == 2 {
              let (buy_strike, sell_strike) = if is_credit {
                  let buy = call_strikes.iter().find(|(_, action)| matches!(action, dsta::BuyOrSell::BuyToOpen));
                  let sell = call_strikes.iter().find(|(_, action)| matches!(action, dsta::BuyOrSell::SellToOpen));
                  (buy.map(|(s, _)| s).copied(), sell.map(|(s, _)| s).copied())
              } else {
                  let buy = call_strikes.iter().find(|(_, action)| matches!(action, dsta::BuyOrSell::BuyToOpen));
                  let sell = call_strikes.iter().find(|(_, action)| matches!(action, dsta::BuyOrSell::SellToOpen));
                  (buy.map(|(s, _)| s).copied(), sell.map(|(s, _)| s).copied())
              };
              if let (Some(buy), Some(sell)) = (buy_strike, sell_strike) {
                  if is_credit && buy <= sell {
                      return Err(format!("Invalid call credit spread: buy strike {} <= sell strike {}", buy, sell));
                  }
                  if !is_credit && buy >= sell {
                      return Err(format!("Invalid call debit spread: buy strike {} >= sell strike {}", buy, sell));
                  }
              }
          }
      }

      if is_put_spread || is_iron_condor {
          if put_strikes.len() == 2 {
              let (buy_strike, sell_strike) = if is_credit {
                  let buy = put_strikes.iter().find(|(_, action)| matches!(action, dsta::BuyOrSell::BuyToOpen));
                  let sell = put_strikes.iter().find(|(_, action)| matches!(action, dsta::BuyOrSell::SellToOpen));
                  (buy.map(|(s, _)| s).copied(), sell.map(|(s, _)| s).copied())
              } else {
                  let buy = put_strikes.iter().find(|(_, action)| matches!(action, dsta::BuyOrSell::BuyToOpen));
                  let sell = put_strikes.iter().find(|(_, action)| matches!(action, dsta::BuyOrSell::SellToOpen));
                  (buy.map(|(s, _)| s).copied(), sell.map(|(s, _)| s).copied())
              };
              if let (Some(buy), Some(sell)) = (buy_strike, sell_strike) {
                  if is_credit && buy >= sell {
                      return Err(format!("Invalid put credit spread: buy strike {} >= sell strike {}", buy, sell));
                  }
                  if !is_credit && buy <= sell {
                      return Err(format!("Invalid put debit spread: buy strike {} <= sell strike {}", buy, sell));
                  }
              }
          }
      }
  }

  for leg in &order.order_legs {
      let asset = match stuff.get(&leg.buy_what.order_name()) {
          Some(finass) => finass,
          None => {
              return Err(format!("{}",crate::glossary::ERROR_NO_ASSET_INFO));
          }
      };
      let qty = leg.quantity as f64;
      
      match &asset {
          dsta::FinAss::StkDeets(stk) => {
              new_stock = stk.clone();
              let unit_cash = leg.buy_what.get_normalized_cost(leg.price);
              
              match leg.action {
                  dsta::BuyOrSell::BuyToOpen | dsta::BuyOrSell::BuyToClose => {
                      log::info!("apply_order_to_account: Increasing stock position by {}", qty);
                      order_stock_change += qty;
                      stock_leg_cash -= unit_cash * qty;
                  }
                  dsta::BuyOrSell::SellToOpen | dsta::BuyOrSell::SellToClose => {
                      log::info!("apply_order_to_account: Decreasing stock position by {}", qty);
                      order_stock_change -= qty;
                      stock_leg_cash += unit_cash * qty;
                  }
                  _ => return Err("Invalid stock action".to_string()),
              }
          }
          dsta::FinAss::OptDeets(opt) => {
              let ticker = opt.option_name.clone();
              let is_call = matches!(opt.option_rite, dsta::OptRite::Call);

              match leg.action {
                  dsta::BuyOrSell::BuyToOpen => {
                      log::info!("apply_order_to_account: Handling BuyToOpen for {}", ticker);
                      if is_call {
                          order_long_calls.entry(ticker.clone()).or_insert((0.0, leg.price)).0 += qty;
                      } else {
                          order_long_puts.entry(ticker.clone()).or_insert((0.0, leg.price)).0 += qty;
                      }
                  }
                  dsta::BuyOrSell::BuyToClose => {
                      log::info!("apply_order_to_account: Handling BuyToClose for {}", ticker);
                      if is_call {
                          order_long_calls.entry(ticker.clone()).or_insert((0.0, leg.price)).0 += qty;
                      } else {
                          order_long_puts.entry(ticker.clone()).or_insert((0.0, leg.price)).0 += qty;
                      }
                  }
                  dsta::BuyOrSell::SellToOpen => {
                      log::info!("apply_order_to_account: Handling SellToOpen for {}", ticker);
                      if is_call {
                          order_short_calls.entry(ticker.clone()).or_insert((0.0, leg.price)).0 += qty;
                      } else {
                          order_short_puts.entry(ticker.clone()).or_insert((0.0, leg.price)).0 += qty;
                      }
                  }
                  dsta::BuyOrSell::SellToClose => {
                      log::info!("apply_order_to_account: Handling SellToClose for {}", ticker);
                      if is_call {
                          order_short_calls.entry(ticker.clone()).or_insert((0.0, leg.price)).0 += qty;
                      } else {
                          order_short_puts.entry(ticker.clone()).or_insert((0.0, leg.price)).0 += qty;
                      }
                  }
                  _ => return Err("Invalid option action".to_string()),
              }
          }
          _ => return Err("Invalid asset type".to_string()),
      }
  }

  // Calculate resulting positions
  let mut resulting_long_calls: std::collections::HashMap<String, (f64, f64)> = std::collections::HashMap::new();
  let mut resulting_short_calls: std::collections::HashMap<String, (f64, f64)> = std::collections::HashMap::new();
  let mut resulting_long_puts: std::collections::HashMap<String, (f64, f64)> = std::collections::HashMap::new();
  let mut resulting_short_puts: std::collections::HashMap<String, (f64, f64)> = std::collections::HashMap::new();

  // Add existing positions to result maps
  if let Some((ticker, _, qty, price)) = existing_long_calls {
      resulting_long_calls.insert(ticker, (qty, price));
  }
  if let Some((ticker, _, qty, price)) = existing_short_calls {
      resulting_short_calls.insert(ticker, (qty, price));
  }
  if let Some((ticker, _, qty, price)) = existing_long_puts {
      resulting_long_puts.insert(ticker, (qty, price));
  }
  if let Some((ticker, _, qty, price)) = existing_short_puts {
      resulting_short_puts.insert(ticker, (qty, price));
  }

  // Apply order effects - handle netting between long and short
  for (ticker, (order_long_qty, price)) in order_long_calls {
      let existing_short = resulting_short_calls.get(&ticker).map(|(qty, _)| *qty).unwrap_or(0.0);
      if order_long_qty <= existing_short {
          resulting_short_calls.insert(ticker.clone(), (existing_short - order_long_qty, price));
          log::info!("Closed {} contracts of short call position {} via Buy", order_long_qty, ticker);
      } else {
          resulting_short_calls.insert(ticker.clone(), (0.0, price));
          let remaining_long = order_long_qty - existing_short;
          let existing_long = resulting_long_calls.get(&ticker).map(|(qty, _)| *qty).unwrap_or(0.0);
          resulting_long_calls.insert(ticker.clone(), (existing_long + remaining_long, price));
          log::info!("Increased long call position {} by {}", ticker, remaining_long);
      }
  }

  for (ticker, (order_short_qty, price)) in order_short_calls {
      let existing_long = resulting_long_calls.get(&ticker).map(|(qty, _)| *qty).unwrap_or(0.0);
      if order_short_qty <= existing_long {
          resulting_long_calls.insert(ticker.clone(), (existing_long - order_short_qty, price));
          log::info!("Closed {} contracts of long call position {} via Sell", order_short_qty, ticker);
      } else {
          resulting_long_calls.insert(ticker.clone(), (0.0, price));
          let remaining_short = order_short_qty - existing_long;
          let existing_short = resulting_short_calls.get(&ticker).map(|(qty, _)| *qty).unwrap_or(0.0);
          resulting_short_calls.insert(ticker.clone(), (existing_short + remaining_short, price));
          log::info!("Increased short call position {} by {}", ticker, remaining_short);
      }
  }

  for (ticker, (order_long_qty, price)) in order_long_puts {
      let existing_short = resulting_short_puts.get(&ticker).map(|(qty, _)| *qty).unwrap_or(0.0);
      if order_long_qty <= existing_short {
          resulting_short_puts.insert(ticker.clone(), (existing_short - order_long_qty, price));
          log::info!("Closed {} contracts of short put position {} via Buy", order_long_qty, ticker);
      } else {
          resulting_short_puts.insert(ticker.clone(), (0.0, price));
          let remaining_long = order_long_qty - existing_short;
          let existing_long = resulting_long_puts.get(&ticker).map(|(qty, _)| *qty).unwrap_or(0.0);
          resulting_long_puts.insert(ticker.clone(), (existing_long + remaining_long, price));
          log::info!("Increased long put position {} by {}", ticker, remaining_long);
      }
  }

  for (ticker, (order_short_qty, price)) in order_short_puts {
      let existing_long = resulting_long_puts.get(&ticker).map(|(qty, _)| *qty).unwrap_or(0.0);
      if order_short_qty <= existing_long {
          resulting_long_puts.insert(ticker.clone(), (existing_long - order_short_qty, price));
          log::info!("Closed {} contracts of long put position {} via Sell", order_short_qty, ticker);
      } else {
          resulting_long_puts.insert(ticker.clone(), (0.0, price));
          let remaining_short = order_short_qty - existing_long;
          let existing_short = resulting_short_puts.get(&ticker).map(|(qty, _)| *qty).unwrap_or(0.0);
          resulting_short_puts.insert(ticker.clone(), (existing_short + remaining_short, price));
          log::info!("Increased short put position {} by {}", ticker, remaining_short);
      }
  }

  // Remove zero positions
  resulting_long_calls.retain(|_, (qty, _)| *qty > 0.0);
  resulting_short_calls.retain(|_, (qty, _)| *qty > 0.0);
  resulting_long_puts.retain(|_, (qty, _)| *qty > 0.0);
  resulting_short_puts.retain(|_, (qty, _)| *qty > 0.0);

  // Validate account constraints (only one position per type)
  if resulting_long_calls.len() > 1 {
      return Err("Multiple long call positions not supported".to_string());
  }
  if resulting_short_calls.len() > 1 {
      return Err("Multiple short call positions not supported".to_string());
  }
  if resulting_long_puts.len() > 1 {
      return Err("Multiple long put positions not supported".to_string());
  }
  if resulting_short_puts.len() > 1 {
      return Err("Multiple short put positions not supported".to_string());
  }

  let mut resulting_stock = -1.0;
  if order_stock_change != 0.0 {
      if let Some(stk) = account.m_stock.as_mut() {
          let price = if stk.owned != 0.0 {
              (stk.price * stk.owned + stock_leg_cash.abs()) / (stk.owned + order_stock_change)
          } else {
              stock_leg_cash.abs() / order_stock_change
          };
          stk.price = price;
          stk.owned += order_stock_change;
          resulting_stock = stk.owned;
      } else if new_stock.ticker_name != "" {
          log::debug!("Detected new stock position opening!");
          let price = stock_leg_cash.abs() / order_stock_change;
          account.m_stock = Some(
              dsta::OwnedPosition {
                  owned: order_stock_change,
                  asset: dsta::FinAss::StkDeets(new_stock),
                  price,
              }
          );
          resulting_stock = order_stock_change;
      } else {
          return Err("Stock position handling failed".to_string());
      }
  }

  // Update account with results
  if resulting_stock == 0.0 {
      account.m_stock = None;
  }

  // Update option positions
  account.m_long_calls = resulting_long_calls.into_iter().next().map(|(ticker, (qty, price))| {
      if let Some(dsta::FinAss::OptDeets(opt)) = stuff.get(&ticker) {
          log::info!("Updated long call position for {}: {} contracts at strike {}", ticker, qty, opt.strike_spot);
          dsta::OwnedPosition {
              owned: qty,
              asset: dsta::FinAss::OptDeets(opt.clone()),
              price,
          }
      } else {
          panic!("Invalid asset for ticker {}", ticker);
      }
  });

  account.m_short_calls = resulting_short_calls.into_iter().next().map(|(ticker, (qty, price))| {
      if let Some(dsta::FinAss::OptDeets(opt)) = stuff.get(&ticker) {
          log::info!("Updated short call position for {}: {} contracts at strike {}", ticker, qty, opt.strike_spot);
          dsta::OwnedPosition {
              owned: -qty,
              asset: dsta::FinAss::OptDeets(opt.clone()),
              price,
          }
      } else {
          panic!("Invalid asset for ticker {}", ticker);
      }
  });

  account.m_long_puts = resulting_long_puts.into_iter().next().map(|(ticker, (qty, price))| {
      if let Some(dsta::FinAss::OptDeets(opt)) = stuff.get(&ticker) {
          log::info!("Updated long put position for {}: {} contracts at strike {}", ticker, qty, opt.strike_spot);
          dsta::OwnedPosition {
              owned: qty,
              asset: dsta::FinAss::OptDeets(opt.clone()),
              price,
          }
      } else {
          panic!("Invalid asset for ticker {}", ticker);
      }
  });

  account.m_short_puts = resulting_short_puts.into_iter().next().map(|(ticker, (qty, price))| {
      if let Some(dsta::FinAss::OptDeets(opt)) = stuff.get(&ticker) {
          log::info!("Updated short put position for {}: {} contracts at strike {}", ticker, qty, opt.strike_spot);
          dsta::OwnedPosition {
              owned: -qty,
              asset: dsta::FinAss::OptDeets(opt.clone()),
              price,
          }
      } else {
          panic!("Invalid asset for ticker {}", ticker);
      }
  });

  log::info!("EXIT apply_order_to_account: Successfully processed order");
  Ok(())
}

pub(crate) fn calculate_option_margin(account: &BotaAccount) -> Result<Vec<(LeinType, f64)>, String> 
{
  log::trace!("ENTRY - calculate_option_margin Calculating option margin for account: {:?}", account);
  let mut leins = Vec::new();
  let stock = account.m_stock.as_ref(); 
  
  // Calculate stock covered contracts using the multiplier from the stock asset
  let stock_covered_contracts = if let Some(s) = stock {
      if let dsta::FinAss::StkDeets(_) = &s.asset {
          let mult = s.asset.get_multiplier();
          (s.owned / (100.0 * mult)).floor()
      } else {
          0.0
      }
  } else {
      0.0
  };

  let call_short = account.m_short_calls.as_ref();
  let call_long = account.m_long_calls.as_ref();
  let put_short = account.m_short_puts.as_ref();
  let put_long = account.m_long_puts.as_ref();

  let has_call_short = call_short.is_some();
  let has_put_short = put_short.is_some();
  let has_call_long = call_long.is_some();
  let has_put_long = put_long.is_some();

  let mut call_covered_qty = 0.0;
  let mut put_covered_qty = 0.0;

  // Check Iron Condors first
  if has_call_short && has_put_short && has_call_long && has_put_long {
      let cs = call_short.unwrap();
      let ps = put_short.unwrap();
      let cl = call_long.unwrap();
      let pl = put_long.unwrap();
      
      let cs_qty = cs.owned;
      let ps_qty = ps.owned;
      let cl_qty = cl.owned;
      let pl_qty = pl.owned;
      
      let condor_qty = (-cs_qty).min(-ps_qty).min(cl_qty).min(pl_qty);
      
      log::trace!("calculate_option_margin detects potential iron condor with qty: {}", condor_qty);
      
      let cs_o = if let dsta::FinAss::OptDeets(o) = &cs.asset { o } else {
          let ems = "calculate_option_margin detects invalid asset in short call".to_string();
          log::error!("EXIT - {}", ems);
          return Err(ems);
      };
      
      let cl_o = if let dsta::FinAss::OptDeets(o) = &cl.asset { o } else {
          let ems = "calculate_option_margin detects invalid asset in long call".to_string();
          log::error!("EXIT - {}", ems);
          return Err(ems);
      };
      
      let ps_o = if let dsta::FinAss::OptDeets(o) = &ps.asset { o } else {
          let ems = "calculate_option_margin detects invalid asset in short put".to_string();
          log::error!("EXIT - {}", ems);
          return Err(ems);
      };
      
      let pl_o = if let dsta::FinAss::OptDeets(o) = &pl.asset { o } else {
          let ems = "calculate_option_margin detects invalid asset in long put".to_string();
          log::error!("EXIT - {}", ems);
          return Err(ems);
      };
      
      if cs_o.expire_date > cl_o.expire_date || ps_o.expire_date > pl_o.expire_date {
          let ems = "calculate_option_margin detects short expires after long for condor".to_string();
          log::debug!("EXIT - {}", ems);
          return Err(ems);
      }
      
      if cl_o.strike_spot <= cs_o.strike_spot || pl_o.strike_spot >= ps_o.strike_spot {
          let ems = "calculate_option_margin detects not credit config for condor".to_string();
          log::info!("ABORT IF STATEMENT - (lein free please...) {} (leins = {:#?})", ems, leins);
      } else {
          let call_width = cl_o.strike_spot - cs_o.strike_spot;
          let put_width = ps_o.strike_spot - pl_o.strike_spot;
          let call_mult = cs.asset.get_multiplier();
          let amount = call_width.max(put_width) * call_mult * condor_qty;
          leins.push((LeinType::IronCondor, amount));
          call_covered_qty = condor_qty;
          put_covered_qty = condor_qty;
          log::debug!("calculate_option_margin computed iron condor margin: {} for qty: {}", amount, condor_qty);
      }
  }

  // Handle remaining short calls
  if let Some(cs) = call_short {
      log::trace!("calculate_option_margin detects SHORT CALL...");
      let cs_qty = cs.owned;
      if cs_qty >= 0.0 {
          let ems = format!("calculate_option_margin detects illegal non-negative quantity of the short call position (={})", cs_qty);
          log::error!("EXIT - {}", ems);
          return Err(ems);
      }
      
      let cs_o = if let dsta::FinAss::OptDeets(o) = &cs.asset { o } else {
          let ems = "calculate_option_margin detects illegal asset type in the short call (not a call)".to_string();
          log::error!("EXIT - {}", ems);
          return Err(ems);
      };
      
      let expire = cs_o.expire_date;
      let cs_strike = cs_o.strike_spot;
      let cs_mult = cs.asset.get_multiplier();
      let mut call_lein = 0.0;
      let mut call_type: Option<LeinType> = None;
      let mut spread_covered_qty = 0.0;
      let remaining_short_qty = -cs_qty - call_covered_qty;
      
      if remaining_short_qty > 0.0 {
          // Check for call spreads with remaining long calls
          if let Some(cl) = call_long {
              let cl_qty = cl.owned;
              let cl_o = if let dsta::FinAss::OptDeets(o) = &cl.asset { o } else {
                  let ems = "calculate_option_margin detects invalid asset in long call".to_string();
                  log::error!("EXIT - {}", ems);
                  return Err(ems);
              };
              
              if expire > cl_o.expire_date {
                  let ems = "calculate_option_margin detects short call expires after long".to_string();
                  log::warn!("NOTICE - {}", ems);
              } else {
                  let cl_strike = cl_o.strike_spot;
                  let spread_qty = (cl_qty - call_covered_qty).min(remaining_short_qty);
                  if spread_qty > 0.0 {
                      if cl_strike > cs_strike {
                          call_lein = (cl_strike - cs_strike) * cs_mult * spread_qty;
                          call_type = Some(LeinType::CallCreditSpread);
                      } else if cl_strike < cs_strike {
                          call_lein = 0.0;
                      } else {
                          let ems = "calculate_option_margin detects same strike for call spread".to_string();
                          log::error!("EXIT - {}", ems);
                          return Err(ems);
                      }
                  }
                  spread_covered_qty = spread_qty;
              }
          }
          
          // Check stock coverage for remaining short calls
          let remaining_short_qty_after_spread = -cs_qty - call_covered_qty - spread_covered_qty;
          log::trace!("calculate_option_margin: remaining short calls after spread coverage: {}", remaining_short_qty_after_spread);
          
          if remaining_short_qty_after_spread > 0.0 {
              log::trace!("calculate_option_margin: checking stock coverage for {} remaining short calls", remaining_short_qty_after_spread);
              log::trace!("calculate_option_margin: stock_covered_contracts = {}", stock_covered_contracts);
              
              if stock_covered_contracts >= remaining_short_qty_after_spread {
                  log::trace!("calculate_option_margin: short calls fully covered by stock, no additional lein");
              } else {
                  let ems = format!("calculate_option_margin detects uncovered short call: need {} contracts, stock covers {}", 
                                   remaining_short_qty_after_spread, stock_covered_contracts);
                  log::error!("EXIT - {}", ems);
                  return Err(ems);
              }
          }
      }
      
      if call_lein > 0.0 {
          leins.push((call_type.unwrap(), call_lein));
      }
      log::debug!("calculate_option_margin computed call margin: {}", call_lein);
  }

  // Handle remaining short puts
  if let Some(ps) = put_short {
      log::trace!("calculate_option_margin detects SHORT PUT...");
      let ps_qty = ps.owned;
      if ps_qty >= 0.0 {
          let ems = format!("calculate_option_margin detects illegal non-negative quantity of the short put position (={})", ps_qty);
          log::error!("EXIT - {}", ems);
          return Err(ems);
      }

      let ps_o = if let dsta::FinAss::OptDeets(o) = &ps.asset { o } else {
          let ems = "calculate_option_margin detects illegal asset type in the short put (not a put)".to_string();
          log::error!("EXIT - {}", ems);
          return Err(ems);
      };

      let expire = ps_o.expire_date;
      let ps_strike = ps_o.strike_spot;
      let ps_mult = ps.asset.get_multiplier();
      let mut put_lein = 0.0;
      let mut put_type: Option<LeinType> = None;
      let mut spread_covered_qty = 0.0;
      let remaining_short_qty = -ps_qty - put_covered_qty;
      
      if remaining_short_qty > 0.0 {
          // Check for put spreads with remaining long puts
          if let Some(pl) = put_long {
              let pl_qty = pl.owned;
              let pl_o = if let dsta::FinAss::OptDeets(o) = &pl.asset { o } else {
                  let ems = "calculate_option_margin detects invalid asset in long put".to_string();
                  log::error!("EXIT - {}", ems);
                  return Err(ems);
              };
              
              if expire > pl_o.expire_date {
                  let ems = "calculate_option_margin detects short put expires after long".to_string();
                  log::warn!("NOTICE! - {}", ems);
              } else {
                  let pl_strike = pl_o.strike_spot;
                  let spread_qty = (pl_qty - put_covered_qty).min(remaining_short_qty);
                  if spread_qty > 0.0 {
                      if pl_strike < ps_strike {
                          put_lein = (ps_strike - pl_strike) * ps_mult * spread_qty;
                          put_type = Some(LeinType::PutCreditSpread);
                      } else if pl_strike > ps_strike {
                          put_lein = 0.0;
                      } else {
                          let ems = "calculate_option_margin detects same strike for put spread".to_string();
                          log::error!("EXIT - {}", ems);
                          return Err(ems);
                      }
                  }
                  spread_covered_qty = spread_qty;
              }
          }

          // Check cash coverage for remaining short puts
          let remaining_short_qty = -ps_qty - put_covered_qty - spread_covered_qty;
          if remaining_short_qty > 0.0 {
              let cash = account.m_cash;
              let required_cash = ps_strike * ps_mult * remaining_short_qty;
              if cash >= required_cash {
                  put_lein = required_cash;
                  put_type = Some(LeinType::ShortPut);
              } else {
                  let ems = "calculate_option_margin detects uncovered short put".to_string();
                  log::error!("EXIT - {}", ems);
                  return Err(ems);
              }
          }
      }
      
      if put_lein > 0.0 {
          leins.push((put_type.unwrap(), put_lein));
      }
      log::debug!("calculate_option_margin computed put margin: {} (condor_covered: {}, spread_covered: {}, remaining: {})", 
                  put_lein, put_covered_qty, put_covered_qty, remaining_short_qty);
  }

  Ok(leins)
}

// Gets a set of total cash and the leins which get distributed.
// The AccountCashChanges will work like this later: add the indicated cash, then distribute cash to leins as described 
pub fn fn_get_account_shennanagans
( order: &dsta::Order
, account: &BotaAccount
, stuff: &std::collections::HashMap<String, dsta::FinAss>
) -> Result<AccountCashChanges, String> {
    log::debug!("ENTRY! fn_get_account_shennanagans with order: {:?}", order);

    // Step 0: Check order legality
    match fn_check_bota_legality(account, order, stuff) 
    { Ok(_) => {}
    , Err(e) =>
      {
        log::debug!("EXIT! fn_get_account_shennanagans - Bota legality failed: {:#?}", e);
        return Err("Bota legality failed".to_string());
      }
    }
    log::debug!("Trade Legality confirmed!");

    // Step 0: Create account clone and set all leins to zero
    let mut new_account = account.clone();
    new_account.m_cash += new_account.m_call_credit_spread_lein;
    new_account.m_cash += new_account.m_short_put_lein;
    new_account.m_cash += new_account.m_put_credit_spread_lein;
    new_account.m_cash += new_account.m_short_iron_condor_lein;
    new_account.m_call_credit_spread_lein = 0.0;
    new_account.m_short_put_lein = 0.0;
    new_account.m_put_credit_spread_lein = 0.0;
    new_account.m_short_iron_condor_lein = 0.0;
    log::debug!("Cloned account and reset all leins to zero: {:?}", new_account);
    // Step 0: Check order legality
    // Step 1: Add order proceeds to cash
    let trade_net = match fn_calculate_order_cost(order) 
    { Ok(x) => {x}
    , Err(e) =>
      {
        log::debug!("EXIT! fn_get_account_shennanagans+fn_calculate_order_cost failed : {:#?}", e);
        return Err("Bota legality failed".to_string());
      }
    };

    log::debug!("Trade Legality confirmed!");


    new_account.m_cash += trade_net;
    log::debug!("Applied order proceeds to cash: trade_net = {}, new cash = {}", trade_net, new_account.m_cash);

    // Step 2: Apply order to account clone
    apply_order_to_account(&mut new_account, order, stuff)?;
    log::debug!("Applied order to account clone: {:?}", new_account);

    // Step 3: Calculate option margin on account clone
    let new_leins = calculate_option_margin(&new_account)?;
    log::debug!("Calculated new leins: {:?}", new_leins);

    // Construct AccountCashChanges
    let changes = AccountCashChanges {
        cash_transfer_to_leins: new_leins,
        cash_gain_from_order: trade_net,
    };

    // Check cash sufficiency
    let total_lein_increase: f64 = changes.cash_transfer_to_leins.iter().map(|(_, amount)| *amount).sum();
    if new_account.m_cash < total_lein_increase 
    { log::debug!
      ( "Insufficient cash: current {}, required lein {}"
      , new_account.m_cash
      , total_lein_increase
      );
      return Err(format!("{}", crate::glossary::ERROR_INSUFFICIENT_LEIN_CASH));
    }

    log::debug!("EXIT! fn_get_account_shennanagans - Success, changes: {:?}", changes);
    Ok(changes)
}

pub fn apply_account_cash_and_collateral_changes
( account: &mut BotaAccount
, changes: &AccountCashChanges
) 
{ log::debug!("Entering apply_account_cash_and_collateral_changes with changes: {:?}", changes);

    // Step 1: Set all leins to zero
    account.m_cash += account.m_call_credit_spread_lein;
    account.m_cash += account.m_short_put_lein;
    account.m_cash += account.m_put_credit_spread_lein;
    account.m_cash += account.m_short_iron_condor_lein;
    account.m_call_credit_spread_lein = 0.0;
    account.m_short_put_lein = 0.0;
    account.m_put_credit_spread_lein = 0.0;
    account.m_short_iron_condor_lein = 0.0;
    log::debug!("Reset all leins to zero: {:?}", account);

    // Step 2: Add the indicated cash to the account
    account.m_cash += changes.cash_gain_from_order;
    log::debug!("Applied cash gain: {}, new cash: {}", changes.cash_gain_from_order, account.m_cash);

    // Step 3: Distribute cash to the leins
    for (lein_type, amount) in &changes.cash_transfer_to_leins {
        log::debug!("Transferring from cash to lein: {:?}, amount: {}", lein_type, amount);
        if *amount < 0.0 {
            log::error!("Invalid lein amount: {} for {:?}", amount, lein_type);
            panic!("Invalid lein amount: {} for {:?}", amount, lein_type);
        }
        account.m_cash -= *amount;
        match lein_type {
            LeinType::CallCreditSpread => account.m_call_credit_spread_lein += *amount,
            LeinType::ShortPut => account.m_short_put_lein += *amount,
            LeinType::PutCreditSpread => account.m_put_credit_spread_lein += *amount,
            LeinType::IronCondor => account.m_short_iron_condor_lein += *amount,
        }
    }

    log::debug!("Exiting apply_account_cash_and_collateral_changes, updated account: {:?}", account);
}


/// Returns the total cost/proceeds:
/// - Negative value: you PAY cash (debit/buy)
/// - Positive value: you RECEIVE cash (credit/sell)
/// # Arguments
/// * `order` - The order to calculate cost for (MUST be Limit order type)
/// # Returns
/// * `Result<f64, String>` - The calculated order cost, or error if not limit or missing price
pub fn fn_calculate_order_cost(
    order: &dsta::Order,
) -> Result<f64, String> {
    let dbgspt = format!("[{}]", crate::glossary::LOG_PREFIX_BOTA_ACCOUNT_OK);
    
    // MUST be limit order
    if order.order_type != dsta::MarketOrLimit::Limit {
        let em = format!(
            "{} [BAD EXIT!] fn_calculate_order_cost requires Limit order; got {:?}",
            dbgspt, order.order_type
        );
        log::error!("{}", em);
        return Err(em);
    }
    
    if order.order_legs.is_empty() {
        let em = format!("{} [BAD EXIT!] Order has no legs", dbgspt);
        log::error!("{}", em);
        return Err(em);
    }

    let mut total_cost = 0.0;

    for leg in &order.order_legs 
    { let qty = leg.quantity;
      let price = leg.price;
      // Determine sign based on action
      // Buy = negative (you pay), Sell = positive (you receive)
      let sign = if leg.action.is_buy() { -1.0 } else { 1.0 };
      // Infer multiplier from ticker format
      // Options tickers typically contain spaces (e.g., "AAPL 150C") or special chars
      // Stocks are just the ticker with no spaces
      let contract_multiplier = leg.buy_what.get_multiplier();
      let leg_cost = sign * qty * price * contract_multiplier;
      log::trace!
      ( "{} Leg buy {:#?}: action={:?}, qty={}, price={}, multiplier={}, leg_cost={}",
          dbgspt, leg.buy_what, leg.action, qty, price, contract_multiplier, leg_cost
      );
      total_cost += leg_cost;
    }

    log::debug!("{} [OK] Calculated order cost for limit order: {}", dbgspt, total_cost);
    Ok(total_cost)
}