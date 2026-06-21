
// trade/dsta/src/fun.rs

use crate::*;
use std::fmt;


// PUB HELPER THINGS

/// Computes market view once (context), then builds a brand-new initial order leg-by-leg
/// by creating modified copies of each original leg with haggle-aware prices.
pub fn create_initial_haggle_order
( hide_order: &Order,
  haggle_method: &HaggleMethod,
) -> (Order, HaggleContext) 
{ let dbgspt = format!("[MAKE_HAGGLE_ORD]");
  log::debug!("{} ENTRY! create_initial_haggle_order() ", dbgspt);

  // Build a fresh new order leg-by-leg ─────────────────────────
  let mut new_first_order = Order 
  { order_legs: Vec::new(),
    order_type: hide_order.order_type,          // copy top-level fields as needed
  };

  let mut combo_bid = 0.0_f64;
  let mut combo_ask = 0.0_f64;

  for original_leg in &hide_order.order_legs 
  { // Make a full copy of the original leg
    let mut new_leg = original_leg.clone();
    let t = new_leg.buy_what.last_ticker().unwrap();

    if new_leg.action.is_sell()
    { combo_bid += t.bid;
      combo_ask += t.ask;
    }
    else 
    { combo_bid -= t.ask;
      combo_ask -= t.bid;
    }

    match haggle_method
    { HaggleMethod::CheapLimitOrderThenIncrementDeltaUntilMax(_) =>
      { // i'm selling something, i want it high
        if new_leg.action.is_sell()
        { new_leg.price = t.ask + 0.01   
        }
        // im buying something, i want it cheap
        else
        { new_leg.price = t.bid - 0.01   
        }
      }

      HaggleMethod::VirtualMarketOrderWithLimitOrdThenConcede(_) =>
      { // i'm selling something, they want it low
        if new_leg.action.is_sell()
        { new_leg.price = t.bid 
        }
        // im buying something, they want it high
        else
        { new_leg.price = t.ask   
        }
      }

      HaggleMethod::MinimumOfMidpointOrTheoreticalThenConcede(_) =>
      { let mid = (t.bid + t.ask) / 2.0;
        new_leg.price = if let Some(theo_price) = t.theo_price
        { // i'm selling something, they want it low
          if new_leg.action.is_sell()
          { if mid > theo_price
            { mid
            }  
            else
            { theo_price
            }
          }
          else
          { if mid < theo_price
            { mid
            } 
            else
            { theo_price
            }
          }
        } else { mid };
      }
    }
    // Add the modified leg to the new order
    new_first_order.order_legs.push(new_leg);
  }

  let combo_mid = (combo_bid + combo_ask) / 2.0;
  let is_net_credit = combo_mid > 0.0;
  let (price_we_have, price_we_want) = if is_net_credit 
  { (combo_bid, combo_ask)
  } else 
  { (combo_ask, combo_bid)
  };

  let ctx = HaggleContext 
  { combo_mid,
    combo_bid,
    combo_ask,
    price_we_have,
    price_we_want,
    is_net_credit,
    initial_spread : combo_ask - combo_bid,
  };

  (new_first_order, ctx)
}

// STRUCTURE THINGS

impl BuyOrSell
{ pub fn is_sell(&self) -> bool
  { matches!
    ( *self,
      | BuyOrSell::SellToOpen 
      | BuyOrSell::SellToClose 
      | BuyOrSell::SellFutures 
    )
  }
  pub fn is_buy(&self) -> bool
  { !self.is_sell()
  }
}

impl HaggleAction
{ pub fn get_retry_period(&self) -> chrono::Duration
  { match *self 
    { HaggleAction::JustCancel(hm) =>hm.get_limits_clone()
    , HaggleAction::AnOyVeyJew(hm) =>hm.get_limits_clone()
    }
  }

  /// Returns absolute slippage introduced this step (combo-level concession).
  pub fn apply_haggle_changes
  ( &self,
    order: &mut Order,
    ctx: &HaggleContext,
  ) -> f64 
  { match *self
    { HaggleAction::AnOyVeyJew(h1) | HaggleAction::JustCancel(h1)
      => h1.apply_haggle_changes(order,ctx)
    }
  }

  pub fn get_slippage_max( &self ) -> f64 
  { match *self
    { HaggleAction::AnOyVeyJew(h1) | HaggleAction::JustCancel(h1)
      => h1.get_slippage_max()
    }
  }
}

impl HaggleMethod
{ pub fn get_limits_clone(&self)->chrono::TimeDelta
  { match *self
    { HaggleMethod::MinimumOfMidpointOrTheoreticalThenConcede(h1) |
      HaggleMethod::VirtualMarketOrderWithLimitOrdThenConcede(h1) |
      HaggleMethod::CheapLimitOrderThenIncrementDeltaUntilMax(h1) 
      => h1.retry_period.clone()
    }
  }

  pub fn get_slippage_max( &self ) -> f64
  { match *self
    { HaggleMethod::MinimumOfMidpointOrTheoreticalThenConcede(h1) |
      HaggleMethod::VirtualMarketOrderWithLimitOrdThenConcede(h1) |
      HaggleMethod::CheapLimitOrderThenIncrementDeltaUntilMax(h1) 
      => h1.slippage_max
    }
  }

  /// Returns absolute slippage introduced this step (combo-level concession).
  pub fn apply_haggle_changes(
      &self,
      order: &mut Order,
      ctx: &HaggleContext,
  ) -> f64 
  { match *self
    { HaggleMethod::MinimumOfMidpointOrTheoreticalThenConcede(h1) |
      HaggleMethod::VirtualMarketOrderWithLimitOrdThenConcede(h1) |
      HaggleMethod::CheapLimitOrderThenIncrementDeltaUntilMax(h1) 
      => h1.apply_haggle_changes(order,ctx)
    }
  }
}

impl HaggleLimits 
{


   /// Applies one concession step to the order for a credit/debit spread.
  /// 
  /// **New behavior:** Delta is distributed **evenly across all legs in atomic 0.01 increments**,
  /// using a priority order for remainder distribution:
  ///   1. Stock
  ///   2. Long Put
  ///   3. Short Put
  ///   4. Long Call
  ///   5. Short Call
  ///
  /// **Per-contract pricing:** Each leg receives an equal share of the delta, rounded to 0.01,
  /// which is then applied to the per-contract price (not total order cost).
  ///
  /// For credit spreads:
  ///   - short legs: price decreases (receive less premium)
  ///   - long legs:  price increases (pay more premium)
  ///
  /// For debit spreads:
  ///   - long legs: price increases (pay more to enter)
  ///   - short legs: price decreases (receive less on short)
  ///
  /// Returns absolute slippage introduced this step (per-contract concession).
  pub fn apply_haggle_changes
  ( &self,
    order: &mut Order,
    ctx: &HaggleContext,
  ) -> f64 
  { let dbgspt = "[HAGGLE_APPLY]";
    log::debug!("{} ENTRY - apply_haggle_changes(ctx.initial_spread={:.4})", dbgspt, ctx.initial_spread); 
    
    // ── Handle edge case: no legs ───────────────────────────────────────
    if order.order_legs.is_empty() {
      log::warn!("{} No legs in order, returning 0.0 slippage", dbgspt);
      return 0.0;
    }
    
    // ── Compute atomic delta for this haggle step ───────────────────────
    // Delta is either a fixed amount (delta_choice) or percentage of initial_spread
    let delta = if self.delta_is_pct 
    { ctx.initial_spread.max(0.01) * (self.delta_choice / 100.0)
    } else 
    { self.delta_choice
    };
    
    log::trace!("{} atomic delta for this step = {:.4} ({}% of spread {:.4})", 
      dbgspt, delta, self.delta_choice, ctx.initial_spread);
    
    // ── Distribute delta EVENLY across all legs (in 0.01 increments) ─────
    let num_legs = order.order_legs.len();
    let per_leg_ideal = delta / num_legs as f64;
    
    // Round each leg's ideal share to nearest 0.01 (atomic unit)
    let per_leg_rounded = (per_leg_ideal / 0.01).round() * 0.01;
    let mut adjustments = vec![per_leg_rounded; num_legs];
    
    // Calculate remainder after rounding all legs
    let total_allocated = per_leg_rounded * num_legs as f64;
    let mut remainder = delta - total_allocated;
    
    // Clean up floating point errors: round to nearest cent
    remainder = (remainder * 100.0).round() / 100.0;
    
    log::trace!("{} distribution: per_leg_ideal={:.4}, per_leg_rounded={:.4}, total_allocated={:.4}, remainder={:.4}",
      dbgspt, per_leg_ideal, per_leg_rounded, total_allocated, remainder);
    
    // ── Get leg priority indices and distribute remainder ─────────────────
    // Priority ensures consistent distribution across multiple haggle attempts
    let indices_by_priority = Self::get_priority_indices(order);
    let mut remainder_left = remainder.abs();
    let remainder_sign = if remainder < 0.0 { -1.0 } else { 1.0 };
    
    // Cycle through priority order, assigning 0.01 increments to remainder
    let mut priority_iter = indices_by_priority.iter().cycle();
    while remainder_left > 0.005 {  // 0.005 = half a tick (stop condition)
      if let Some(&idx) = priority_iter.next() {
        adjustments[idx] += 0.01 * remainder_sign;
        remainder_left -= 0.01;
        log::trace!("{} Remainder: added 0.01 to leg {} (priority), remaining={:.4}",
          dbgspt, idx, remainder_left);
      } else {
        break;  // Safety: shouldn't happen but prevent infinite loop
      }
    }
    
    // ── Apply adjustments to leg prices ──────────────────────────────────
    // Accumulate slippage as SUM OF ATOMIC DELTAS (not dollar impact)
    let mut total_atomic_slippage = 0.0;
    
    for (idx, leg) in order.order_legs.iter_mut().enumerate() {
        // Price adjustment logic:
        // - Selling: decrease price to haggle (receive less premium)
        // - Buying: increase price to haggle (pay more premium)
        let price_change = if leg.action.is_sell() { 
          -adjustments[idx]  // decrease: SellToOpen at lower price → receive less
        } else { 
          adjustments[idx]   // increase: BuyToOpen at higher price → pay more
        };
        
        let old_price = leg.price;
        leg.price += price_change;
        
        // Floor price at 0.01 minimum
        if leg.price < 0.01 {
            log::trace!("{} Leg {}: price floored from {:.4} to 0.01", dbgspt, idx, leg.price);
            leg.price = 0.01;
        }
        
        // Accumulate slippage as SUM OF ATOMIC ADJUSTMENTS (not affected by quantity)
        // This is: sum of the deltas applied, regardless of how many contracts
        // Allows predictable haggle tracking: max_slippage = 0.50 means "up to 50 cents of haggling"
        total_atomic_slippage += adjustments[idx].abs();
        
        let direction = if leg.action.is_sell() { "-" } else { "+" };
        log::trace!("{} Leg {}: adjustment={}{:+.4}, price {:.4}→{:.4}, qty={}, atomic_contribution={:.4}",
            dbgspt, idx, direction, adjustments[idx], old_price, leg.price, leg.quantity,
            adjustments[idx].abs()
        );
    }
    
    // ── Return sum of atomic deltas ──────────────────────────────────────
    // This is the sum of all price adjustments applied in this step
    // Accumulated across multiple haggle steps to track total concession
    // Example: 3-leg order with [0.02, 0.02, 0.01] → returns 0.05
    // NOT affected by quantity, so slippage_max is quantity-independent
    let slippage_this_step = total_atomic_slippage;
    
    log::info!("{} RESULT: delta={:.4}, num_legs={}, total_atomic_slippage={:.4}, haggle_count={:.0}",
        dbgspt, delta, num_legs, slippage_this_step, (slippage_this_step / 0.01).round());
    
    slippage_this_step
  }
  
  /// Applies one concession step to the order for a credit/debit spread.
  /// 
  /// **New behavior:** Delta is distributed **evenly across all legs in atomic 0.01 increments**,
  /// using a priority order for remainder distribution:
  ///   1. Stock
  ///   2. Long Put
  ///   3. Short Put
  ///   4. Long Call
  ///   5. Short Call
  ///
  /// **Per-contract pricing:** Each leg receives an equal share of the delta, rounded to 0.01,
  /// which is then applied to the per-contract price (not total order cost).
  ///
  /// For credit spreads:
  ///   - short legs: price decreases (receive less premium)
  ///   - long legs:  price increases (pay more premium)
  ///
  /// For debit spreads:
  ///   - long legs: price increases (pay more to enter)
  ///   - short legs: price decreases (receive less on short)
  ///
  /// Returns absolute slippage introduced this step (per-contract concession).
  pub fn apply_haggle_changes_sum_but_wrong_lol
  ( &self,
    order: &mut Order,
    ctx: &HaggleContext,
  ) -> f64 
  { let dbgspt = "[HAGGLE_APPLY]";
    log::debug!("{} ENTRY - apply_haggle_changes(ctx.initial_spread={:.4})", dbgspt, ctx.initial_spread); 
    
    // ── Handle edge case: no legs ───────────────────────────────────────
    if order.order_legs.is_empty() {
      log::warn!("{} No legs in order, returning 0.0 slippage", dbgspt);
      return 0.0;
    }
    
    // ── Compute atomic delta for this haggle step ───────────────────────
    // Delta is either a fixed amount (delta_choice) or percentage of initial_spread
    let delta = if self.delta_is_pct 
    { ctx.initial_spread.max(0.01) * (self.delta_choice / 100.0)
    } else 
    { self.delta_choice
    };
    
    log::trace!("{} atomic delta for this step = {:.4} ({}% of spread {:.4})", 
      dbgspt, delta, self.delta_choice, ctx.initial_spread);
    
    // ── Distribute delta EVENLY across all legs (in 0.01 increments) ─────
    let num_legs = order.order_legs.len();
    let per_leg_ideal = delta / num_legs as f64;
    
    // Round each leg's ideal share to nearest 0.01 (atomic unit)
    let per_leg_rounded = (per_leg_ideal / 0.01).round() * 0.01;
    let mut adjustments = vec![per_leg_rounded; num_legs];
    
    // Calculate remainder after rounding all legs
    let total_allocated = per_leg_rounded * num_legs as f64;
    let mut remainder = delta - total_allocated;
    
    // Clean up floating point errors: round to nearest cent
    remainder = (remainder * 100.0).round() / 100.0;
    
    log::trace!("{} distribution: per_leg_ideal={:.4}, per_leg_rounded={:.4}, total_allocated={:.4}, remainder={:.4}",
      dbgspt, per_leg_ideal, per_leg_rounded, total_allocated, remainder);
    
    // ── Get leg priority indices and distribute remainder ─────────────────
    // Priority ensures consistent distribution across multiple haggle attempts
    let indices_by_priority = Self::get_priority_indices(order);
    let mut remainder_left = remainder.abs();
    let remainder_sign = if remainder < 0.0 { -1.0 } else { 1.0 };
    
    // Cycle through priority order, assigning 0.01 increments to remainder
    let mut priority_iter = indices_by_priority.iter().cycle();
    while remainder_left > 0.005 {  // 0.005 = half a tick (stop condition)
      if let Some(&idx) = priority_iter.next() {
        adjustments[idx] += 0.01 * remainder_sign;
        remainder_left -= 0.01;
        log::trace!("{} Remainder: added 0.01 to leg {} (priority), remaining={:.4}",
          dbgspt, idx, remainder_left);
      } else {
        break;  // Safety: shouldn't happen but prevent infinite loop
      }
    }
    
    // ── Apply adjustments to leg prices ──────────────────────────────────
    // Accumulate slippage considering direction of each leg
    let mut combo_slippage = 0.0;
    
    for (idx, leg) in order.order_legs.iter_mut().enumerate() {
        let direction = if leg.action.is_sell() { 1.0 } else { -1.0 };  // +1 = we receive premium
        
        // Price adjustment logic:
        // - Selling: decrease price to haggle (receive less premium)
        // - Buying: increase price to haggle (pay more premium)
        let price_change = if leg.action.is_sell() { 
          -adjustments[idx]  // decrease: SellToOpen at lower price → receive less
        } else { 
          adjustments[idx]   // increase: BuyToOpen at higher price → pay more
        };
        
        let old_price = leg.price;
        leg.price += price_change;
        
        // Floor price at 0.01 minimum
        if leg.price < 0.01 {
            log::trace!("{} Leg {}: price floored from {:.4} to 0.01", dbgspt, idx, leg.price);
            leg.price = 0.01;
        }
        
        // Accumulate slippage: (direction * price_change) * quantity
        // This handles multi-contract orders correctly
        combo_slippage += direction * price_change * leg.quantity as f64;
        
        log::trace!("{} Leg {}: adjustment={:+.4}, price {:.4}→{:.4}, qty={}, contribution={:+.4}",
            dbgspt, idx, price_change, old_price, leg.price, leg.quantity,
            direction * price_change * leg.quantity as f64
        );
    }
    
    // ── Return absolute per-contract slippage ───────────────────────────
    // This represents the economic concession made per contract
    let slippage_this_step = combo_slippage.abs();
    
    log::info!("{} RESULT: delta={:.4}, num_legs={}, slippage={:.4}, per_contract={:.4}",
        dbgspt, delta, num_legs, slippage_this_step, slippage_this_step / 1.0);  // TODO: divide by qty for per-contract
    
    slippage_this_step
  }
  
  /// Get indices of order legs sorted by priority for remainder distribution.
  /// **Priority order:** Stock(1) > LongPut(2) > ShortPut(3) > LongCall(4) > ShortCall(5)
  /// 
  /// Lower number = higher priority. This ensures that when remainder increments (0.01)
  /// need to be distributed, they go to the most important leg first.
  fn get_priority_indices(order: &Order) -> Vec<usize> {
    let mut indexed: Vec<(usize, u8)> = order.order_legs.iter()
      .enumerate()
      .map(|(idx, leg)| {
        let priority = Self::classify_leg_priority(leg);
        (idx, priority)
      })
      .collect();
    
    // Sort by priority value (ascending = highest priority legs first)
    indexed.sort_by_key(|&(_, p)| p);
    
    // Return just the indices in priority order
    indexed.iter().map(|&(idx, _)| idx).collect()
  }
  
  /// Classify a leg's priority for 0.01 remainder distribution.
  /// Returns a priority number (lower = higher priority):
  ///   - Stock/Bond: 1
  ///   - Long Put: 2
  ///   - Short Put: 3
  ///   - Long Call: 4
  ///   - Short Call: 5
  fn classify_leg_priority(leg: &OrderLeg) -> u8 {
    match &leg.buy_what {
      FinAss::StkDeets(_) => 1,  // Stock: highest priority
      FinAss::BndDeets(_) => 1,  // Bond: treat like stock
      FinAss::OptDeets(opt) => {
        let is_short_leg = leg.action.is_sell();  // SellToOpen/SellToClose = short
        match opt.option_rite {
          OptRite::Put => {
            if is_short_leg { 3 } else { 2 }  // ShortPut(3) lower priority than LongPut(2)
          }
          OptRite::Call => {
            if is_short_leg { 5 } else { 4 }  // ShortCall(5) lowest priority, LongCall(4) above that
          }
        }
      }
    }
  }

  /// Applies one concession step to the order for a credit/debit spread.
  /// 
  /// Delta is distributed **proportionally by leg quantity**, with all price adjustments
  /// constrained to valid $0.01 ticks. Remainders are distributed sequentially across legs.
  ///
  /// For credit spreads:
  ///   - short legs: price decreases (receive less premium)
  ///   - long legs:  price increases (pay more premium)
  ///
  /// Returns absolute slippage introduced this step (combo-level concession).
  pub fn apply_haggle_changes_depreciated_does_sums
  ( &self,
    order: &mut Order,
    ctx: &HaggleContext,
  ) -> f64 
  { let dbgspt = "[HAGGLE_APPLY]";
    log::debug!("{} ENTRY - apply_haggle_changes(ctx = {:#?}", dbgspt, ctx); 
    
    // ── Handle edge case: no legs ───────────────────────────────────────
    if order.order_legs.is_empty() {
      log::warn!("{} No legs in order, returning 0.0 slippage", dbgspt);
      return 0.0;
    }
    
    // Compute total combo concession amount this step
    let total_delta = if self.delta_is_pct 
    { let tmp = ctx.initial_spread.max(0.01) * (self.delta_choice/100.0);
      log::trace!("{} total_delta is {}% of {} = ${:2}"
      , dbgspt, self.delta_choice, ctx.initial_spread, tmp
      );
      tmp
    } else 
    { let tmp = self.delta_choice;
      log::trace!("{} total_delta is solid ${:2}"
      , dbgspt, tmp
      );
      tmp
    };
    
    // ── Calculate total quantity across all legs ───────────────────────
    let total_quantity: f64 = order.order_legs.iter()
      .map(|leg| leg.quantity as f64)
      .sum();
    
    if total_quantity <= 0.0 {
      log::warn!("{} Total quantity is 0 or negative, returning 0.0 slippage", dbgspt);
      return 0.0;
    }
    
    // ── First pass: distribute delta proportionally, floored to 0.01 ticks ──
    let mut adjustments = vec![0.0; order.order_legs.len()];
    let mut remaining_delta = total_delta;
    
    log::trace!("{} First pass: distributing {} across {} legs (total qty={})"
    , dbgspt, total_delta, order.order_legs.len(), total_quantity
    );
    
    for (idx, leg) in order.order_legs.iter().enumerate() 
    {
      let weight = leg.quantity as f64 / total_quantity;
      let proportional = remaining_delta * weight;
      // Floor to nearest 0.01 tick (ensures no sub-tick adjustments)
      let floored = (proportional / 0.01).floor() * 0.01;
      
      adjustments[idx] = floored;
      remaining_delta -= floored;
      
      log::trace!("{} Leg {}: qty={}, weight={:.3}, proportional=${:.4}, floored=${:.2}, remaining=${:.4}"
      , dbgspt, idx, leg.quantity, weight, proportional, floored, remaining_delta
      );
    }
    
    // ── Second pass: distribute remainder sequentially (0.005 threshold) ──
    log::trace!("{} Second pass: distributing remainder of ${:.4}", dbgspt, remaining_delta);
    
    let mut leg_idx = 0;
    while remaining_delta > 0.005  // Stop when remainder < half a tick
    {
      // Find next leg with non-zero quantity
      let start_idx = leg_idx;
      loop 
      { if order.order_legs[leg_idx].quantity > 0.0
        { let add = 0.01_f64.min(remaining_delta);
          adjustments[leg_idx] += add;
          remaining_delta -= add;
          log::trace!("{} Remainder pass: added ${:.2} to leg {}, remaining=${:.4}"
          , dbgspt, add, leg_idx, remaining_delta
          );
          
          leg_idx = (leg_idx + 1) % order.order_legs.len();
          break;
        }
        leg_idx = (leg_idx + 1) % order.order_legs.len();
        
        // Safety: prevent infinite loop if all legs have qty=0
        if leg_idx == start_idx {
          log::warn!("{} No non-zero quantity legs found in remainder pass", dbgspt);
          break;
        }
      }
    }
    
    // ── Apply adjustments to leg prices and track combo values ──────────
    let mut new_combo_value = 0.0;
    let mut old_combo_value = 0.0;

    for (idx, leg) in order.order_legs.iter_mut().enumerate() {
        let direction = if leg.action.is_sell() { 1.0 } else { -1.0 };  // +1 = we receive premium

        old_combo_value += direction * leg.price * leg.quantity as f64;

        // Apply adjustment 
        let price_change = if leg.action.is_sell() { -adjustments[idx] } else { adjustments[idx] };
        leg.price += price_change;

        if leg.price.abs() < 0.01 {
            log::warn!("{} Price floor on leg {}: {} -> {}0.01", dbgspt, idx, leg.price, leg.price.signum());
            leg.price = leg.price.signum()*0.01;
        }

        new_combo_value += direction * leg.price * leg.quantity as f64;

        log::trace!("{} Leg {}: adjustment=${:+.2}, new_price=${:.2}, qty={}, combo_contribution=${:+.4}",
            dbgspt, idx, price_change, leg.price, leg.quantity,
            direction * leg.price * leg.quantity as f64
        );
    }
    
    // ── Validate and calculate slippage ─────────────────────────────────
    let combo_delta = if ctx.is_net_credit {
        old_combo_value - new_combo_value      // positive when credit decreases
    } else {
        new_combo_value - old_combo_value      // negative when debit increases (more negative)
    };

    let slippage_this_step = combo_delta.abs();   // always positive concession amount

    // Validation with slightly looser tolerance for fp math + rounding
    let validation_tolerance = 0.015;
    if (combo_delta.abs() - total_delta).abs() > validation_tolerance {
        log::warn!("{} VALIDATION MISMATCH: expected ~${:.3}, got ${:+.4} (diff {:+.4})",
            dbgspt, total_delta, combo_delta, combo_delta - total_delta);
    } else {
        log::trace!("{} VALIDATION PASSED: combo_delta=${:+.4} ≈ total_delta=${:.3}",
            dbgspt, combo_delta, total_delta);
    }

    log::debug!("{} RESULT: slippage=${:.4}, old_combo=${:+.4}, new_combo=${:+.4}",
        dbgspt, slippage_this_step, old_combo_value, new_combo_value);

    slippage_this_step
  }
}

// Fun Part A) Steven's functionality functions
impl BotActionTime
{
  pub fn is_entry_window
  ( &self
  , now: chrono::DateTime<chrono::Utc>
  , window: chrono::Duration
  ) -> bool
  { use chrono::Datelike;
    let ny = now.with_timezone(&chrono_tz::America::New_York);
    let dow = ny.weekday().num_days_from_sunday() as u16;
    if dow != self.relative_days
    { return false;
    }

    // Important: to_utc() returns NaiveTime, not NaiveDateTime!
    // We need to combine it with today's date in Eastern timezone.
    let target_naive_dt  = self.my_entry_time.to_utc_today();

    let window_start_naive = target_naive_dt + self.my_entry_wait;
    let window_end_naive   = window_start_naive + window;

    let ny_time = ny.time();   // NaiveTime in NY

    // Compare only the time parts (since we're already on the correct day)
    window_start_naive.time() <= ny_time && ny_time <= window_end_naive.time()
  }

  /// Returns the **exact UTC datetime** when the entry window should start
  /// (after applying my_entry_wait delay)
  pub fn get_entry_time(&self, reference: chrono::DateTime<chrono::Utc>) -> Option<chrono::DateTime<chrono::Utc>>
  { 
    use chrono::Datelike;
    //use chrono::TimeZone;

    let ny = reference.with_timezone(&chrono_tz::America::New_York);
    let today_eastern = ny.date_naive();

    // Are we even on the correct day of week?
    let nyw = ny.weekday();
    let dow = nyw.num_days_from_sunday() as u16;
    
    log::trace!("BotActionTime, nyw = {:#?}, dow = {}, entry dow = {}, \n    reference time = {:#?}, today eastern time = {:#?}, "
    , nyw, dow, self.relative_days, reference, today_eastern
    );

    if dow != self.relative_days
    { return None; // wrong day → no valid time on this reference date
    }

    // // Base target time-of-day in Eastern (naive)
    // let base_naive_time = self.my_entry_time.to_utc();                    // NaiveTime
    // // Combine with today's Eastern date → NaiveDateTime
    // let base_naive_dt = today_eastern.and_time(base_naive_time);
    
    // THis is actually a retular utc datetime
    let base_naive_dt = self.my_entry_time.to_utc_today();

    // Apply delay (can be negative → may cross day boundary)
    Some(base_naive_dt + self.my_entry_wait)
  }

  /// Returns the **exact UTC datetime** when the entry window should start
  /// assuming the wait delay is zero
  pub fn get_exit_time(&self, reference: chrono::DateTime<chrono::Utc>) -> Option<chrono::DateTime<chrono::Utc>>
  { 
    use chrono::Datelike;
    //use chrono::TimeZone;

    let ny = reference.with_timezone(&chrono_tz::America::New_York);
    let today_eastern = ny.date_naive();

    // Are we even on the correct day of week?
    let nyw = ny.weekday();
    
    log::trace!("BotActionTime, nyw = {:#?}, relative_days = {}, \n    reference time = {:#?}, today eastern time = {:#?}, "
    , nyw, self.relative_days, reference, today_eastern
    );

    // Combine with today's Eastern date → NaiveDateTime
    let base_naive_dt = self.my_entry_time.to_utc_today();

    // Apply delay (can be negative → may cross day boundary)
    Some(base_naive_dt + self.my_entry_wait)
  }
  pub fn has_exit_time_passed(&self, now: chrono::DateTime<chrono::Utc>) -> bool
  { 
    let Some(target) = self.get_exit_time(now) else { return true; }; // wrong day → treat as passed
    now > target
  }

  /// Convenience: is now past (or at) the calculated entry time on the correct day?
  /// Returns false if we're on the wrong day of week.
  pub fn is_entry_time(&self, now: chrono::DateTime<chrono::Utc>) -> bool
  { 
    let Some(target) = self.get_entry_time(now) else { return false; };
    now >= target
  }

  /// Bonus helper: has the entry window already passed (useful for skipping old days)
  pub fn has_entry_time_passed(&self, now: chrono::DateTime<chrono::Utc>) -> bool
  { 
    let Some(target) = self.get_entry_time(now) else { return true; }; // wrong day → treat as passed
    now > target
  }
}


impl UsaMarketTimes 
{ /// Returns the **NaiveTime** (time-of-day only) in Eastern Time
  pub fn to_et_naive(&self) -> chrono::NaiveTime 
  { match self 
    { UsaMarketTimes::ResetTtai  => chrono::NaiveTime::from_hms_opt(17, 20, 0).unwrap(),
      UsaMarketTimes::ItsClosed  => chrono::NaiveTime::from_hms_opt(15, 59, 59).unwrap(),
      UsaMarketTimes::Hurry5sec  => chrono::NaiveTime::from_hms_opt(15, 59, 55).unwrap(),
      UsaMarketTimes::TminusTen  => chrono::NaiveTime::from_hms_opt(15, 59, 50).unwrap(),
      UsaMarketTimes::Tminus30s  => chrono::NaiveTime::from_hms_opt(15, 59, 30).unwrap(),
      UsaMarketTimes::Tminus60s  => chrono::NaiveTime::from_hms_opt(15, 59, 0).unwrap(),
      UsaMarketTimes::PowerEnds  => chrono::NaiveTime::from_hms_opt(15, 50, 0).unwrap(),
      UsaMarketTimes::DeadShort  => chrono::NaiveTime::from_hms_opt(15, 45, 0).unwrap(),
      UsaMarketTimes::PowerFive  => chrono::NaiveTime::from_hms_opt(15, 55, 0).unwrap(),
      UsaMarketTimes::PowerEasy  => chrono::NaiveTime::from_hms_opt(15, 30, 0).unwrap(),
      UsaMarketTimes::PowerHour  => chrono::NaiveTime::from_hms_opt(15, 0, 0).unwrap(),
      UsaMarketTimes::FedSpeach  => chrono::NaiveTime::from_hms_opt(14, 30, 0).unwrap(),
      UsaMarketTimes::FedMinute  => chrono::NaiveTime::from_hms_opt(14, 0, 0).unwrap(),
      UsaMarketTimes::TradeTime  => chrono::NaiveTime::from_hms_opt(13, 0, 0).unwrap(),
      UsaMarketTimes::LunchTime  => chrono::NaiveTime::from_hms_opt(12, 0, 0).unwrap(),
      UsaMarketTimes::EuroClose  => chrono::NaiveTime::from_hms_opt(10, 30, 0).unwrap(),
      UsaMarketTimes::DoneTrend  => chrono::NaiveTime::from_hms_opt(10, 0, 0).unwrap(),
      UsaMarketTimes::OpenChaos  => chrono::NaiveTime::from_hms_opt(9, 30, 0).unwrap(),
    }
  }

    /// Returns the **UTC timestamp** for this market time on a date that is
    /// `day_offset` days from today (in New York local time).
    ///
    /// - `day_offset = 0`  → today
    /// - `day_offset = -1` → yesterday
    /// - `day_offset = 1`  → tomorrow
    /// - etc.
    pub fn to_utc_any(&self, day_offset: i64) -> chrono::DateTime<chrono::Utc> 
    { use chrono::TimeZone;
      let eastern_time = self.to_et_naive();
      let ny_tz = chrono_tz::America::New_York;
      let ny_now = chrono::Utc::now().with_timezone(&ny_tz);
      let ny_date = ny_now.date_naive();
      let targetd = ny_date + chrono::Duration::days(day_offset);
      let ny_datetime = ny_tz
          .from_local_datetime(&targetd.and_time(eastern_time))
          .single()
          .unwrap_or_else(|| {
              ny_tz
                  .from_local_datetime(&targetd.and_time(eastern_time))
                  .earliest()
                  .unwrap()
          });
      ny_datetime.with_timezone(&chrono::Utc)
    }

    /// Convenience: today's chrono::Utc timestamp for this market time
    /// (equivalent to `to_utc_any(0)`)
    pub fn to_utc_today(&self) -> chrono::DateTime<chrono::Utc> {
        self.to_utc_any(0)
    }

    /// How long from now until/since this market time on the given day offset
    pub fn countdown_on(&self, day_offset: i64) -> chrono::Duration {
        let target = self.to_utc_any(day_offset);
        let now = chrono::Utc::now();
        target.signed_duration_since(now)
    }

    /// Countdown for today's instance
    pub fn countdown_today(&self) -> chrono::Duration {
        self.countdown_on(0)
    }
}
impl PartialOrd for UsaMarketTimes 
{ fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> 
  { Some(self.cmp(other))
  }
}
impl Ord for UsaMarketTimes 
{ fn cmp(&self, other: &Self) -> std::cmp::Ordering 
  { self.to_utc_today().cmp(&other.to_utc_today())
  }
}
impl Eq for UsaMarketTimes {}










pub(crate) fn get_tick_for_price(price: f64, tiers: &[crate::TickTier]) -> f64 {
    // Iterate in reverse (highest tier first)
    for tier in tiers.iter().rev() {
        if let Some(th) = tier.threshold {
            if price >= th {
                return tier.value;
            }
        } else {
            // No threshold = default / highest tier
            return tier.value;
        }
    }
    // Fallback to lowest tier or default
    tiers.first().map(|t| t.value).unwrap_or(0.01)
}


impl FinAss
{ 

  // FinAss self API funcitons
  pub fn ticker_name(&self) -> String
  { match self
    { FinAss::StkDeets(stk) =>
      { stk.ticker_name.clone()
      }
      FinAss::BndDeets(bnd) =>
      { bnd.ticker_name.clone()
      }
      FinAss::OptDeets(opt) =>
      { opt.ticker_name.clone()
      }
    }
  }
  pub fn order_name(&self)-> String
  { match self
    { FinAss::StkDeets(stk) =>
      { stk.ticker_name.clone()
      }
      FinAss::BndDeets(bnd) =>
      { bnd.ticker_name.clone()
      }
      FinAss::OptDeets(opt) =>
      { opt.option_name.clone()
      }
    }
  }
  pub fn set_ticker(&mut self, tick : Ticker)
  { match self
    { FinAss::StkDeets(stk) =>
      { stk.last_ticker = Some(tick);
      }
      FinAss::BndDeets(bnd) =>
      { bnd.last_ticker = Some(tick);
      }
      FinAss::OptDeets(opt) =>
      { opt.last_ticker = Some(tick);
      }
    }
  }

  /// Returns the tick rules (tiered tick sizes) for this asset.
  /// 
  /// - If the asset has stored tick_rulers → returns them
  /// - Otherwise falls back to sensible defaults (0.01 underlying/spread, 0.05/0.10 for options)
  /// 
  /// This method never returns hardcoded rules unless no real data exists.
  pub fn tick_rulers(&self) -> TickRules 
  {
    // Sensible fallback defaults when no specific rules are stored
    let fallback = TickRules {
        underlying_tiers: vec![TickTier { threshold: None, value: 0.01 }],
        option_tiers: vec![TickTier { threshold: None, value: 0.01 }],
        // I want to assume 0.01 always, not this:
        // option_tiers: vec![
        //     TickTier { threshold: Some(3.0), value: 0.05 },
        //     TickTier { threshold: None,      value: 0.10 },
        // ],
        spread_tiers: vec![TickTier { threshold: None, value: 0.01 }],
    };

        match self {
            FinAss::StkDeets(stk) => {
                stk.tick_rulers.clone().unwrap_or(fallback)
            }
            FinAss::OptDeets(opt) => {
                // For options: prefer stored rules
                // If missing → use fallback (standard penny-pilot rules)
                opt.tick_rulers.clone().unwrap_or(fallback)
            }
            FinAss::BndDeets(bnd) => {
                // Bonds usually don't have complex tiers — just use underlying default
                bnd.tick_rulers.clone().unwrap_or_else(|| TickRules {
                    underlying_tiers: vec![TickTier { threshold: None, value: 0.01 }],
                    option_tiers: vec![],
                    spread_tiers: vec![],
                })
            }
        }
  }

  pub fn last_ticker(&self) -> Option<Ticker>
  { match self
    { FinAss::StkDeets(stk) =>
      { stk.last_ticker.clone()
      }
      FinAss::BndDeets(bnd) =>
      { bnd.last_ticker.clone()
      }
      FinAss::OptDeets(opt) =>
      { opt.last_ticker.clone()
      }
    }
  }


  // FinAss Helpers

  /// Returns the contract multiplier (notional value per point / per contract).
  /// This is used to compute actual dollar exposure / cash effect:
  ///   - For futures: price * multiplier
  ///   - For options on futures: premium * multiplier
  ///   - For stock options: usually 100
  ///   - For stocks: 1
  pub fn get_multiplier(&self) -> f64 
  { let underlying = self.order_name();  // e.g. "/ES", "/MES", "/NQ", "AAPL", etc.
    // Order Name:
    /* Tastytrade symbology deduction:
       order_name starts with "./" => Futuers Option       
       order_name starts with "/" => futures
    */
    if underlying.starts_with("./") || underlying.starts_with("/")
    { get_futures_multiplier(&underlying)
    } 
    else 
    { // Non-futures (stocks, stock options, bonds)
      match self 
      { FinAss::StkDeets(_)     => 1.0,
        FinAss::OptDeets(_)     => 100.0,  // standard equity/index options
        FinAss::BndDeets(_)     => 1.0,    // or 100/1000 depending on convention
      }
    }
  }

  /// Returns actual cash impact for **one contract / one share** at given price.
  /// Examples:
  ///   - /ES future at 5000.00 → 50 * 5000 = $250,000 notional
  ///   - /ES option premium 5.00 → 50 * 5.00 = $250 cash per contract
  ///   - AAPL stock at 200.00 → 1 * 200 = $200
  ///   - AAPL option at 3.50 → 100 * 3.50 = $350
  pub fn get_normalized_cost(&self, price: f64) -> f64 
  { price * self.get_multiplier()
  }


    /// Returns tick size and snapped values for the given price.
    /// 
    /// - Uses tiered rules if available (option_tiers / spread_tiers / underlying_tiers)
    /// - Determines correct tick size based on price level
    /// - Computes closest valid tick (tick_snap)
    /// - Computes the "pass" value (floor or ceiling depending on snap direction)
  pub fn get_tick_and_snap(
        &self,
        price: f64,
        is_spread_context: bool,  // true = use spread/combo rules
    ) -> TickSnapResult 
  {
        let rules = self.tick_rulers();  // assumes you have this method

        // Select the correct tier list
        let tiers = if is_spread_context {
            &rules.spread_tiers
        } else if matches!(self, FinAss::OptDeets(_)) {
            &rules.option_tiers
        } else {
            &rules.underlying_tiers
        };

        // Get the applicable tick size for this price
        let tick_size = get_tick_for_price(price, tiers);

        // Safety fallback
        let tick_size = if tick_size <= 0.0 { 0.01 } else { tick_size };

        // Compute snapped value (closest)
        let snapped = (price / tick_size).round() * tick_size;
        let snapped = snapped.max(0.01);

        // Determine direction and compute tick_pass
        let diff = snapped - price;
        let tick_pass = if diff.abs() < 1e-9 {
            // No adjustment
            None
        } else if diff > 0.0 {
            // Snapped UP → pass = floor(original)
            Some((price / tick_size).floor() * tick_size)
        } else {
            // Snapped DOWN → pass = ceil(original)
            Some((price / tick_size).ceil() * tick_size)
        };

        TickSnapResult {
            tick_size,
            tick_snap: snapped,
            tick_pass,
        }
  }


   

}
// Define a list of (root_symbol, multiplier) pairs
const FUTURES_ROOT_SYMBOLS: &[(&str, f64)] = &[
    ("ES", 50.0),
    ("MES", 5.0),
    ("NQ", 20.0),
    ("MNQ", 2.0),
    ("YM", 5.0),
    ("MYM", 0.5),
    ("RTY", 50.0),
    ("M2K", 5.0),
    ("CL", 1000.0),
    ("QM", 500.0),
    ("MCL", 100.0),
    ("NG", 10000.0),
    ("QG", 2500.0),
    ("RB", 42000.0),
    ("HO", 42000.0),
    ("GC", 100.0),
    ("MGC", 10.0),
    ("SI", 5000.0),
    ("SIL", 1000.0),
    ("HG", 25000.0),
    ("MHG", 2500.0),
    ("6E", 125000.0),
    ("M6E", 12500.0),
    ("6A", 100000.0),
    ("M6A", 10000.0),
    ("6B", 62500.0),
    ("M6B", 6250.0),
    ("6C", 100000.0),
    ("MCD", 10000.0),
    ("6J", 12500000.0),
    ("6M", 500000.0),
    ("BTC", 5.0),
    ("MBT", 0.1),
    ("ETH", 50.0),
    ("MET", 0.1),
    ("ZB", 1000.0),
    ("ZF", 1000.0),
    ("ZN", 1000.0),
    ("ZT", 2000.0),
    ("UB", 1000.0),
    ("TN", 1000.0),
    ("SR3", 2500.0),
    ("GE", 2500.0),
    ("ZC", 50.0),
    ("XC", 10.0),
    ("ZS", 50.0),
    ("XK", 10.0),
    ("ZW", 50.0),
    ("XW", 10.0),
    ("HE", 400.0),
    ("LE", 400.0),
    ("VX", 1000.0),
    ("VXM", 100.0),
    ("MNG", 1000.0),
    ("FBT", 0.1),
    ("FET", 0.1),
    ("30Y", 1000.0),
    ("10Y", 1000.0),
    ("5YY", 1000.0),
    ("2YY", 1000.0),
    ("S2Y", 100.0),
    ("S10Y", 100.0),
    ("S30Y", 100.0),
    ("SFX", 100.0),
    ("SMES", 100.0),
];
fn extract_futures_root_symbol(underlying: &str) -> Option<&'static str> {
    if !underlying.starts_with("./") {
        return None; // Not a futures option
    }

    // Extract the part after `./` (e.g., "ESM5" from "./ESM5")
    let root_part = &underlying[2..];

    // Check if the first 2-4 characters match a known root symbol
    for (symbol, _) in FUTURES_ROOT_SYMBOLS {
        if root_part.starts_with(symbol) {
            return Some(*symbol);
        }
    }

    None // Unknown futures root symbol
}
fn get_futures_multiplier(underlying: &str) -> f64 {
    if let Some(root_symbol) = extract_futures_root_symbol(underlying) {
        for (symbol, multiplier) in FUTURES_ROOT_SYMBOLS {
            if *symbol == root_symbol {
                return *multiplier;
            }
        }
    }

    // Default multiplier for unknown futures or non-futures
    100.0
}


// ==================================================================
// Fun Part B) unfun impls for standard things
// ==================================================================
// todo: move to nuf.rs // boilerplate

// ==================================================================
// Default implementations for structs
// ==================================================================

impl Default for HaggleAction
{ fn default() -> Self
  { Self::JustCancel(HaggleMethod::default())
  }
}

impl Default for HaggleLimits
{ fn default() -> Self
  { HaggleLimits
    { retry_period : chrono::TimeDelta::seconds(10)
    , delta_choice : 0.5
    , delta_is_pct : false
    , slippage_max : 1.0
    }
  }
}

impl Default for HaggleMethod
{ fn default() -> Self
  { Self::MinimumOfMidpointOrTheoreticalThenConcede
    ( HaggleLimits::default()
    )
  }
}

impl Default for FinAss 
{ fn default() -> Self 
  { FinAss::StkDeets(Stk::default())
  }
}

impl Default for AccountUpdate 
{ fn default() -> AccountUpdate 
  { AccountUpdate 
    { ttai_uid: format!("NONE"),
      long_usd: 0.0,
      can_sell: 0.0,
      long_stk: 0.0,
      sell_stk: 0.0,
      long_opt: 0.0,
      sell_opt: 0.0,
      long_fut: 0.0,
      sell_fut: 0.0,
      long_fop: 0.0,
      sell_fop: 0.0,
      long_xxx: 0.0,
      sell_xxx: 0.0,
      long_bnd: 0.0,
      next_usd: 0.0,
      summings: 0.0,
      cash_out: 0.0,
    }
  }
}


impl Default for Ticker 
{ fn default() -> Self 
  { Ticker 
    { name: format!("NONE"),
      bid: 0.0,
      bid_size: 0.0,
      ask: 0.0,
      ask_size: 0.0,
      time: chrono::Utc::now(),
      theo_price: None,
    }
  }
}



// ==================================================================
// Debug + Display implementations for structs
// ==================================================================

impl std::fmt::Debug for PushTickerResult 
{ fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result 
  { 
    let x = if self.opt_chain.is_empty() 
    { format!("empty")
    } else 
    { format!
      ( "{{ {}, {} ... {} }} = {} dates",
          self.opt_chain
              .first()
              .map(|d| d.expire_time.format("%Y-%m-%d").to_string())
              .unwrap_or_default(),
          self.opt_chain
              .get(1)
              .map(|d| d.expire_time.format("%Y-%m-%d").to_string())
              .unwrap_or_default(),
          self.opt_chain
              .last()
              .map(|d| d.expire_time.format("%Y-%m-%d").to_string())
              .unwrap_or_default(),
          self.opt_chain.len()
      )
    }
    ;
    f.debug_struct("PushTickerResult")
    .field("ass_deets", &self.ass_deets)
    .field("opt_chain", &x)
    .finish()
  }
}



impl fmt::Display for OptRite 
{ fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            OptRite::Put => write!(f, "Put"),
            OptRite::Call => write!(f, "Call"),
        }
    }
}

impl fmt::Display for OptStyle {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            OptStyle::Usa => write!(f, "American"),
            OptStyle::Eur => write!(f, "European"),
            OptStyle::Etc => write!(f, "Other"),
        }
    }
}

impl fmt::Display for BuyOrSell {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            BuyOrSell::BuyToOpen => write!(f, "Buy to Open"),
            BuyOrSell::SellToOpen => write!(f, "Sell to Open"),
            BuyOrSell::BuyToClose => write!(f, "Buy to Close"),
            BuyOrSell::SellToClose => write!(f, "Sell to Close"),
            BuyOrSell::BuyFutures => write!(f, "Buy Futures"),
            BuyOrSell::SellFutures => write!(f, "Sell Futures"),
        }
    }
}

impl fmt::Display for MarketOrLimit {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            MarketOrLimit::Market => write!(f, "Market"),
            MarketOrLimit::Limit => write!(f, "Limit"),
        }
    }
}

impl fmt::Display for OrderFill {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            OrderFill::Live => write!(f, "Live"),
            OrderFill::Done => write!(f, "Filled"),
        }
    }
}

impl fmt::Display for StealthBotDoneNiceStyle {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            StealthBotDoneNiceStyle::OtmShort => write!(f, "OTM Short"),
            StealthBotDoneNiceStyle::FinalPct(p) => write!(f, "Final {}%", p),
            StealthBotDoneNiceStyle::PriorDay(p) => write!(f, "Exit {} days before exp", p.relative_days),
            StealthBotDoneNiceStyle::GoExpire => write!(f, "Expire"),
        }
    }
}

impl fmt::Display for HaggleAction 
{ fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result 
  { match self 
    { HaggleAction::JustCancel(how) => write!(f, "Cancel at retry: {}", how),
      HaggleAction::AnOyVeyJew(how) => write!(f, "Haggle at retry: {}", how),
    }
  }
}

impl fmt::Display for HaggleLimits 
{ fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result 
  { let rp = &self.retry_period;
    let dc = &self.delta_choice;
    let dp = &self.delta_is_pct;
    let sm = &self.slippage_max;
    write!
    ( f
    , "Haggle Limits: max slip = {}, retry {}, delta = {}"
    , sm
    , rp
    , if *dp { format!("{:1} % spread, ",dc)} else { format!("$ {:2}", dc)}
    )
  }
}

impl fmt::Display for HaggleMethod 
{ fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result 
  { match self 
    { HaggleMethod::MinimumOfMidpointOrTheoreticalThenConcede(how) => 
      { write!
        ( f, "Order Fair Prices - {}", how
        )
      }
      HaggleMethod::VirtualMarketOrderWithLimitOrdThenConcede(how) => 
      { write!
        ( f, "Order Virtual Mkt - {}", how
        )
      }
      HaggleMethod::CheapLimitOrderThenIncrementDeltaUntilMax(how) => 
      { write!
        ( f, "Bargin Like a Jew - {}", how
        )
      }
    }
  }
}


impl fmt::Display for SallyAction {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            SallyAction::SubmitRightAwayButLikeOnlyUseForTest => write!(f, "Submit Immediately (Test)"),
            SallyAction::SubmitWhenBidAskMidAttainsInputPrice(p) => write!(f, "Submit when Mid ≥ {:.2}", p),
            SallyAction::SubmitWhenMarketSideReachesThisPrice(p) => write!(f, "Submit when Market Side hits {:.2}", p),
            SallyAction::SubmitAtThisTimeUnlessItReachesPrice(p, shift) => write!(f, "Submit at time unless price {:.2} is worse than {:.2}", p, shift),
        }
    }
}

impl fmt::Display for Ticker {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{} | Bid {:.2}×{:.0}  Ask {:.2}×{:.0}  @ {}  Theo: {:?}",
            self.name,
            self.bid,
            self.bid_size,
            self.ask,
            self.ask_size,
            self.time,
            self.theo_price
        )
    }
}

impl fmt::Display for Opt {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{} ({}) {} {:.2} exp {} style {}",
            self.option_name,
            self.ticker_name,
            self.option_rite,
            self.strike_spot,
            self.expire_date.date_naive(),
            self.option_type
        )
    }
}

impl fmt::Display for Bnd {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{} ({}) coupon {:.2}% exp {}",
            self.ticker_name,
            self.ticker_info,
            self.coupon_rate,
            self.expire_date.date_naive()
        )
    }
}

impl fmt::Display for Stk {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} ({})", self.ticker_name, self.ticker_info)
    }
}

impl fmt::Display for FinAss {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            FinAss::StkDeets(s) => write!(f, "Stock: {}", s),
            FinAss::BndDeets(b) => write!(f, "Bond: {}", b),
            FinAss::OptDeets(o) => write!(f, "Option: {}", o),
        }
    }
}


impl fmt::Display for PositionsList {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.positions.is_empty() {
            return write!(f, "<no positions>");
        }
        writeln!(f, "Positions:")?;
        for pos in self.positions.values() {
            writeln!(f, "  {:#?}", pos)?;
        }
        Ok(())
    }
}

impl fmt::Display for OrderLeg {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{} {} qty {} (rem {}) @ {:.4}",
            self.action,
            self.buy_what.ticker_name(),
            self.quantity,
            self.remaining,
            self.price
        )
    }
}

impl fmt::Display for Order {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} order", self.order_type)?;
        writeln!(f, ":")?;
        for leg in &self.order_legs {
            writeln!(f, "  {}", leg)?;
        }
        Ok(())
    }
}

impl fmt::Display for PlacedOrder {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Order #{} status {}", self.order_hash, self.order_fill)?;
        for leg in &self.legs_fills {
            write!(f, "\n  {}", leg)?;
        }
        Ok(())
    }
}

impl fmt::Display for AccountUpdate {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Account {} | Cash {:.2} | NextUSD {:.2} | Summings {:.2}",
            self.ttai_uid, self.long_usd, self.next_usd, self.summings
        )
    }
}

impl fmt::Display for CommonBotInfo {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{} tracking {}",
            self.friendly_name,
            self.tracking_tick,
        )
    }
}

// Display for BotActionTime
impl fmt::Display for BotActionTime {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Day: {}, Time: {:?}, Wait: {}s",
            self.relative_days,
            self.my_entry_time,
            self.my_entry_wait.num_seconds()
        )
    }
}

// Display for BuzzBotDoneNiceStyle
impl fmt::Display for BuzzBotDoneNiceStyle {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            BuzzBotDoneNiceStyle::SpreadValueGain(credit) =>
                write!(f, "Profit target: ${:.2}", credit * 100.0),
            BuzzBotDoneNiceStyle::SpreadValueLoss(credit) =>
                write!(f, "Stop loss: ${:.2}", credit * 100.0),
            BuzzBotDoneNiceStyle::SpreadValueTime(time) =>
                write!(f, "Time-based exit: {}", time),
        }
    }
}

// Display for MakeBuzzBomber
impl fmt::Display for MakeBuzzBomber {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(
            f,
            "BuzzBomber: {}",
            self.my_accounting.friendly_name
        )?;
        writeln!(
            f,
            "  Allocation: ${:.2} | Feeling: {:?} | Min DTE: {}",
            self.my_cash_alloc,
            self.stonk_feeling,
            self.option_expire
        )?;
        writeln!(
            f,
            "  Target Spread: ${:.2} | Cooldown: {}s | Repeat: {}",
            self.target_spread * 100.0,
            self.algo_cooldown.num_seconds(),
            if self.bombs_forever { "Forever" } else { "Once" }
        )?;
        writeln!(
            f,
            "  Entry Timing: {}",
            self.time_to_order
        )?;
        writeln!(
            f,
            "  Exit Rules:"
        )?;
        for (i, rule) in self.follow_a_exit.iter().enumerate() {
            writeln!(
                f,
                "    {}. {}",
                i + 1,
                rule
            )?;
        }
        Ok(())
    }
}


impl fmt::Display for MakeStealthBot {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "StealthBot {} | Alloc {:.2} | Feeling {} | Exp ≥ {}d | Exit style {:?}",
            self.my_accounting.friendly_name,
            self.my_cash_alloc,
            self.stonk_feeling,
            self.option_expire,
            self.nice_exit_way
        )
    }
}


impl std::fmt::Display for UsaMarketTimes {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self {
            Self::ResetTtai => "5:20 pm ~ markets closed",
            Self::PowerEnds => "3:50 pm",
            Self::ItsClosed => "3:59:59 pm",
            Self::Hurry5sec => "3:59:55 pm",
            Self::TminusTen => "3:59:50 pm",
            Self::Tminus30s => "3:59:30 pm",
            Self::Tminus60s => "3:59:00 pm",
            Self::DeadShort => "3:45 pm",
            Self::PowerFive => "3:55 pm",
            Self::PowerEasy => "3:30 pm",
            Self::PowerHour => "3:00 pm",
            Self::FedSpeach => "2:30 pm",
            Self::FedMinute => "2:00 pm",
            Self::TradeTime => "1:00 pm",
            Self::LunchTime => "12:00 pm",
            Self::EuroClose => "10:30 am",
            Self::DoneTrend => "10:00 am",
            Self::OpenChaos => "9:30 am",
        })
    }
}

impl std::fmt::Display for MarketDirection {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self {
            Self::GetStonk => "BULL MARKET",
            Self::Sideways => "LULL MARKET",
            Self::Corrects => "BEAR MARKET",
        })
    }
}


#[cfg(test)]
mod tests {
    use super::*; // assuming the code is in the same module or appropriately imported


    fn stuffmap() ->  std::collections::HashMap<String,crate::FinAss>
    {
      let expiry = chrono::Utc::now() + chrono::Duration::days(35);
      let mut stuff: std::collections::HashMap<String,crate::FinAss> = HashMap::new();
      let long_put = Opt
      { ticker_name:".FAKE 95_P_35".to_string(),
        option_name:"FAKE 95_P_35".to_string(),
        option_rite:OptRite::Put,
        strike_spot:95.0,
        expire_date:expiry,
        option_type:OptStyle::Usa,
        last_ticker:None,
        tick_rulers:None,
      };
      let short_put = Opt
      { ticker_name:".FAKE 100_P_35".to_string(),
        option_name:"FAKE 100_P_35".to_string(),
        option_rite:OptRite::Put,
        strike_spot:100.0,
        expire_date:expiry,
        option_type:OptStyle::Usa,
        last_ticker:None,
        tick_rulers:None,
      };
      stuff.insert(long_put.option_name.clone(),FinAss::OptDeets(long_put));
      stuff.insert(short_put.option_name.clone(),FinAss::OptDeets(short_put));
      stuff
    }


    fn approx_eq(a: f64, b: f64, tol: f64) -> bool {
        (a - b).abs() < tol
    }

    // Helper to create a simple 1-leg order (easier to debug)
    fn single_leg_debit_order(price: f64, qty: f64) -> Order {
      let stuff = stuffmap();

        Order {
            order_legs: vec![OrderLeg {
                buy_what: stuff.get("FAKE 100_P_35").unwrap().clone(),
                quantity: qty,
                remaining: qty,
                action: crate::BuyOrSell::BuyToOpen,
                price,
            }],
            order_type: crate::MarketOrLimit::Limit,
        }
    }

    fn single_leg_credit_order(price: f64, qty: f64) -> Order {
      let stuff = stuffmap();
        Order {
            order_legs: vec![OrderLeg {
                buy_what: stuff.get("FAKE 100_P_35").unwrap().clone(),
                quantity: qty,
                remaining: qty,
                action: crate::BuyOrSell::SellToOpen,
                price,
            }],
            order_type: crate::MarketOrLimit::Limit,
        }
    }

    fn ctx_for_debit(spread: f64) -> HaggleContext {
        HaggleContext {
            combo_mid: -2.0,
            combo_bid: - (2.0 + spread / 2.0),
            combo_ask: - (2.0 - spread / 2.0),
            price_we_have: -(2.0 + spread / 2.0),
            price_we_want: -(2.0 - spread / 2.0),
            is_net_credit: false,
            initial_spread: spread,
        }
    }

    fn ctx_for_credit(spread: f64) -> HaggleContext {
        HaggleContext {
            combo_mid: 2.0,
            combo_bid: 2.0 - spread / 2.0,
            combo_ask: 2.0 + spread / 2.0,
            price_we_have: 2.0 - spread / 2.0,
            price_we_want: 2.0 + spread / 2.0,
            is_net_credit: true,
            initial_spread: spread,
        }
    }

    #[test]
    fn test_debit_concession_increases_debit() {
        let limits = HaggleLimits {
            retry_period: chrono::Duration::zero(),
            delta_choice: 33.0,
            delta_is_pct: true,
            slippage_max: 10.0,
        };

        let mut order = single_leg_debit_order(2.00, 1.0); // 1 contract for simplicity
        let ctx = ctx_for_debit(0.20);

        let slippage = limits.apply_haggle_changes(&mut order, &ctx);

        let new_price = order.order_legs[0].price;
        assert!(new_price > 2.00, "Debit price should increase when conceding");
        assert!(approx_eq(new_price, 2.066, 0.005), "Expected ~ +0.066");

        // The real economic concession is how much more we pay
        let extra_paid = (new_price - 2.00) * 1.0;
        assert!(approx_eq(extra_paid, 0.066, 0.005));

        // After fix, slippage should be positive and ≈ total_delta
        assert!(slippage > 0.0);
        assert!(approx_eq(slippage, 0.066, 0.01), "Slippage should match concession");
    }

    #[test]
    fn test_credit_concession_decreases_credit() {
        let limits = HaggleLimits {
            retry_period: chrono::Duration::zero(),
            delta_choice: 33.0,
            delta_is_pct: true,
            slippage_max: 10.0,
        };

        let mut order = single_leg_credit_order(2.00, 1.0);
        let ctx = ctx_for_credit(0.20);

        let slippage = limits.apply_haggle_changes(&mut order, &ctx);

        let new_price = order.order_legs[0].price;
        assert!(new_price < 2.00, "Credit price should decrease when conceding");
        assert!(approx_eq(new_price, 1.934, 0.005), "Expected ~ -0.066");

        let less_received = (2.00 - new_price) * 1.0;
        assert!(approx_eq(less_received, 0.066, 0.005));

        assert!(slippage > 0.0);
        assert!(approx_eq(slippage, 0.066, 0.01));
    }

    #[test]
    fn test_combo_delta_sign_correct_for_debit() {
        let limits = HaggleLimits {
            delta_choice: 0.10,
            delta_is_pct: false, // absolute $0.10
            ..Default::default()
        };
        let mut order = single_leg_debit_order(2.00, 10.0);
        let ctx = ctx_for_debit(0.40);

        let slippage = limits.apply_haggle_changes(&mut order, &ctx);

        // After fix: we want slippage ≈ total_delta > 0
        assert!(slippage > 0.12); // at least some positive value
        assert!(!approx_eq(slippage, 0.0, 0.001), "Should not clamp to zero");
    }

    #[test]
    fn test_remainder_does_not_break_sign() {
        // Very small delta + qty not divisible nicely
        let limits = HaggleLimits {
            delta_choice: 0.10,
            delta_is_pct: false, // absolute $0.10
            ..Default::default()
        };

        let mut order = single_leg_debit_order(1.50, 7.0); // awkward qty
        let ctx = ctx_for_debit(1.0);

        let _ = limits.apply_haggle_changes(&mut order, &ctx);

        assert!(order.order_legs[0].price > 1.50);
        // no negative price, no inversion
        assert!(order.order_legs[0].price >= 0.01);
    }

    // You can add more:
    // - multi-leg credit spread
    // - multi-leg debit spread
    // - qty=0 leg ignored
    // - price floored at 0.01
}