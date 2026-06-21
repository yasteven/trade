
// trade/dsta/src/convert.rs
  use crate::*;
    use ttrs::{Balance, Position, OrderResponse, 
  StreamQuote, StreamTrade, OptionChainData, OptionChainRoot, Strike
    };
    //use serde_json::json;


    /// Convert Tastytrade's tiered tick size arrays into our TickRules
fn instrument_tick_rulers_to_dsta(
    tick_sizes: &Option<Vec<ttrs::TickSize>>,
    option_tick_sizes: &Option<Vec<ttrs::TickSize>>,
    spread_tick_sizes: &Option<Vec<ttrs::SpreadTickSize>>,
) -> TickRules {
    fn parse_tiers<T>(
        tiers_opt: &Option<Vec<T>>,
        val_fn: impl Fn(&T) -> Option<String>,
        thresh_fn: impl Fn(&T) -> Option<String>,
    ) -> Vec<TickTier> {
        let mut v = vec![];
        if let Some(list) = tiers_opt {
            for t in list {
                let value = val_fn(&t).and_then(|s| s.parse().ok()).unwrap_or(0.01);
                let threshold = thresh_fn(&t).and_then(|s| s.parse().ok());
                v.push(TickTier { threshold, value });
            }
        }
        if v.is_empty() {
            vec![TickTier { threshold: None, value: 0.01 }]
        } else {
            v.sort_by(|a, b| {
                let a_th = a.threshold.unwrap_or(f64::INFINITY);
                let b_th = b.threshold.unwrap_or(f64::INFINITY);
                a_th.total_cmp(&b_th)
            });
            v
        }
    }

    TickRules {
        underlying_tiers: parse_tiers(tick_sizes, |t| t.value.clone(), |t| t.threshold.clone()),
        option_tiers: parse_tiers(option_tick_sizes, |t| t.value.clone(), |t| t.threshold.clone()),
        spread_tiers: parse_tiers(spread_tick_sizes, |t| t.value.clone(), |_| None),
    }
}


/// Convert ttrs::StreamQuote => dsta::Ticker
pub fn stream_quote_to_ticker(quote: &StreamQuote) -> Ticker {
    let bid = quote.bid_price.unwrap_or(0.0);
    let bid_size = if quote.bid_price.is_none() { 0.0 } else { quote.bid_size.unwrap_or(0.0) };
    
    let ask = quote.ask_price.unwrap_or(0.0);
    let ask_size = if quote.ask_price.is_none() { 0.0 } else { quote.ask_size.unwrap_or(0.0) };
    
    Ticker {
        name: quote.symbol.clone(),
        bid,
        bid_size,
        ask,
        ask_size,
        time: chrono::Utc::now(),
        theo_price:None,
    }
}

/// Convert ttrs::StreamTrade => dsta::Ticker
pub fn stream_greek_to_ticker(greek: &ttrs::dat::StreamGreeks) -> Ticker 
{
    let price = greek.theo_price.unwrap_or(0.0);
    let size = 1.0;

  log::warn!
  ( "todo: we need to  put the ticker time to whativer the gree time is: {:#?}"
  , greek.event_time
  );
    // TODO: in the future, we can make nd raise multiple tickers for each greek.
    Ticker {
        name: greek.symbol.clone(),
        bid: price,
        bid_size: size,
        ask: price,
        ask_size: size,
        time: chrono::Utc::now(), // TODO: replace with tie from greek
        theo_price:Some(price),
    }
}



/// Convert ttrs::StreamTrade => dsta::Ticker
pub fn stream_trade_to_ticker(trade: &StreamTrade) -> Ticker {
    let price = trade.price;
    let size = if price == 0.0 && trade.size.is_none() { 0.0 } else { trade.size.unwrap_or(0.0) };
    
    Ticker {
        name: trade.symbol.clone(),
        bid: price,
        bid_size: size,
        ask: price,
        ask_size: size,
        time: chrono::Utc::now(),
        theo_price:None,
    }
}

/// Convert ttrs::OptionChainData => Vec<dsta::Derivatives>
pub fn option_chain_data_to_derivatives(chain_data: &OptionChainData) -> Vec<Derivatives> {
    chain_data
        .items
        .iter()
        .flat_map(option_chain_root_to_derivatives)
        .collect()
}

fn option_chain_root_to_derivatives(root: &OptionChainRoot) -> Vec<Derivatives> {
  // Get or derive the underlying's tick rules
  //  // For now: use root-level data as proxy for underlying
  //  // In production: fetch the actual underlying Instrument and convert
  //  let underlying_rules = instrument_tick_rulers_to_dsta(
  //      &Some(root.tick_sizes.clone()),  // wrap Vec in Some
  //      &None,                            // no option-specific here
  //      &None,                            // spread_tick_sizes doesn't exist on root
  //  );
  //  root.expirations
  //      .iter()
  //      .map(|expiration| {
  //          let (puts, calls) = strikes_to_options(
  //              &expiration.strikes,
  //              expiration.expiration_date.clone(),
  //              underlying_rules.clone(),  // ← pass the rules down
  //          );
  //          Derivatives {
  //              expire_time: parse_expiration_date(&expiration.expiration_date),
  //              option_puts: puts,
  //              option_calls: calls,
  //          }
  //      })
  //      .collect()

  root.expirations
  .iter()
  .map(|expiration| {
      // Try expiration-level tick_sizes first (futures), fall back to root (equity)
      let tick_sizes_to_use = if !expiration.tick_sizes.is_empty() 
      { expiration.tick_sizes.clone()  // Futures: use expiration's tiers
      } 
      else 
      { root.tick_sizes.clone()  // Equity: use root's tiers
      };

      let expiration_rules = instrument_tick_rulers_to_dsta
      ( &None,  // ← underlying_tiers is None
        &if tick_sizes_to_use.is_empty() { None } else { Some(tick_sizes_to_use) },  // ← option_tick_sizes
        &None,  // ← spread_tick_sizes is None (for now)
      );

      let (puts, calls) = strikes_to_options
      (
          &expiration.strikes,
          expiration.expiration_date.clone(),
          expiration_rules,
      );
      
      Derivatives {
          expire_time: parse_expiration_date(&expiration.expiration_date),
          option_puts: puts,
          option_calls: calls,
      }
  })
  .collect()
}

/// Helper: Parse expiration date string to chrono::DateTime<Utc>
/// Assumes format like "2025-12-19" or similar ISO date
fn parse_expiration_date(date_str: &str) -> chrono::DateTime<chrono::Utc> {
    chrono::DateTime::parse_from_rfc3339(format!("{}T16:00:00Z", date_str).as_str())
        .map(|dt| dt.with_timezone(&chrono::Utc))
        .unwrap_or_else(|_| chrono::Utc::now())
}

/// Helper: Convert strikes to separate puts and calls vectors
fn strikes_to_options(
    strikes: &[Strike],
    expiration_date: String,
    underlying_tick_rulers: TickRules,  // ← NEW: pass the underlying's rules here
) -> (Vec<Opt>, Vec<Opt>) {
    let expire_time = parse_expiration_date(&expiration_date);

    let mut puts = Vec::new();
    let mut calls = Vec::new();

    for strike in strikes {
        let strike_price: f64 = strike.strike_price.parse().unwrap_or(0.0);

        // Use the EXACT same TickRules as the underlying
        let rules = Some(underlying_tick_rulers.clone());

        puts.push(Opt {
            ticker_name: strike.put_streamer_symbol.to_string(),
            option_name: strike.put.clone(),
            option_rite: OptRite::Put,
            strike_spot: strike_price,
            expire_date: expire_time,
            option_type: OptStyle::Usa,
            tick_rulers: rules.clone(),  // ← copy from underlying
            last_ticker: None,
        });

        calls.push(Opt {
            ticker_name: strike.call_streamer_symbol.to_string(),
            option_name: strike.call.clone(),
            option_rite: OptRite::Call,
            strike_spot: strike_price,
            expire_date: expire_time,
            option_type: OptStyle::Usa,
            tick_rulers: rules,  // ← copy from underlying
            last_ticker: None,
        });
    }

    (puts, calls)
}

/// Format dsta::BuyOrSell to ttrs action string
fn format_order_action(action: &BuyOrSell) -> String {
    log::trace!("Order Action to convert to string: {:#?}",action);
    match action {
        BuyOrSell::BuyToOpen => "Buy to Open".to_string(),
        BuyOrSell::SellToOpen => "Sell to Open".to_string(),
        BuyOrSell::BuyToClose => "Buy to Close".to_string(),
        BuyOrSell::SellToClose => "Sell to Close".to_string(),
        BuyOrSell::BuyFutures => "Buy".to_string(),
        BuyOrSell::SellFutures => "Sell".to_string(),
    }
}

/// Helper: Snap a final price using the correct tier (spread or single option)
fn snap_final_price(
    price: f64,
    is_spread: bool,
    tick_rulers: &crate::TickRules,
) -> f64 {
    let tiers = if is_spread {
        &tick_rulers.spread_tiers
    } else {
        &tick_rulers.option_tiers
    };

    if tiers.is_empty() {
        return price.max(0.01);
    }

    let tick_size = crate::get_tick_for_price(price, tiers);
    let snapped = (price / tick_size).round() * tick_size;
    snapped.max(0.01)
}

/// Convert dsta::Order => ttrs::OrderRequest
pub fn order_to_order_request(order: &crate::Order) -> ttrs::OrderRequest {
    let is_spread = order.order_legs.len() > 1;
    let is_crypto = order
        .order_legs
        .iter()
        .any(|leg| {
            let tn = leg.buy_what.ticker_name();
            tn.contains('/') && tn.contains("USD") && (tn.contains("BTC") || tn.contains("ETH"))
        });

    if is_crypto && order.order_legs.len() != 1 {
        panic!("Cryptocurrency orders must have exactly one leg. Found {}", order.order_legs.len());
    }

    // Find tick_rulers from any option leg (all should have the same copy)
    let tick_rulers = order.order_legs.iter().find_map(|leg| {
        if let crate::FinAss::OptDeets(opt) = &leg.buy_what {
            opt.tick_rulers.clone()
        } else {
            None
        }
    }).unwrap_or_else(|| crate::TickRules {
        underlying_tiers: vec![crate::TickTier { threshold: None, value: 0.01 }],
        option_tiers: vec![crate::TickTier { threshold: None, value: 0.01 }],
        spread_tiers: vec![crate::TickTier { threshold: None, value: 0.01 }],
    });

    // Build legs
    let legs: Vec<ttrs::OrderLeg> = order
        .order_legs
        .iter()
        .map(|leg| {
            let tn = leg.buy_what.ticker_name();
            let tno = leg.buy_what.order_name();
            let instrument_type = if tn.starts_with('.') {
                "Equity Option".to_string()
            } else if tn.starts_with('/') {
                "Future".to_string()
            } else if is_crypto {
                "Cryptocurrency".to_string()
            } else {
                "Equity".to_string()
            };

            let action = format_order_action(&leg.action);

            if is_crypto {
                ttrs::OrderLeg {
                    symbol: tno,
                    action,
                    quantity: None,
                    instrument_type: Some(instrument_type),
                    extra: std::collections::HashMap::new(),
                }
            } else {
                ttrs::OrderLeg {
                    symbol: tno,
                    action,
                    quantity: if leg.quantity != 0.0 { Some(leg.quantity) } else { None },
                    instrument_type: Some(instrument_type),
                    extra: std::collections::HashMap::new(),
                }
            }
        })
        .collect();

    if is_crypto {
        let leg = &order.order_legs[0];
        let notional_abs = if leg.quantity != 0.0 && leg.price != 0.0 {
            Some(leg.quantity.abs() * leg.price)
        } else {
            None
        };
        let is_buy = format_order_action(&leg.action).contains("Buy");
        let value_effect = if is_buy { Some("Debit".to_string()) } else { Some("Credit".to_string()) };

        return ttrs::OrderRequest {
            time_in_force: "IOC".to_string(),
            order_type: "Notional Market".to_string(),
            price: None,
            price_effect: None,
            value: notional_abs,
            value_effect,
            legs,
            extra: std::collections::HashMap::new(),
        };
    }

    // Non-crypto: Limit order with snapped price
    let net_per_contract: f64 = order
        .order_legs
        .iter()
        .fold(0.0, |acc, leg| {
            let signed = if matches!(leg.action, crate::BuyOrSell::SellToOpen | crate::BuyOrSell::SellToClose) {
                leg.price
            } else {
                -leg.price
            };
            acc + signed * leg.quantity
        })
        / order.order_legs[0].quantity;

    let price_to_snap = net_per_contract.abs();
    let snapped_price = snap_final_price(price_to_snap, is_spread, &tick_rulers);

    if (snapped_price - price_to_snap).abs() > 1e-9 {
        log::info!(
            "Final order price snapped: {:.4} → {:.4} (spread={}, tick_rulers from underlying)",
            price_to_snap,
            snapped_price,
            is_spread
        );
    }

    let price = Some(snapped_price);
    let price_effect = Some(if net_per_contract > 0.0 {
        "Credit".to_string()
    } else {
        "Debit".to_string()
    });

    ttrs::OrderRequest {
        time_in_force: "Day".to_string(),
        order_type: "Limit".to_string(),
        price,
        price_effect,
        value: None,
        value_effect: None,
        legs,
        extra: std::collections::HashMap::new(),
    }
}

/// Convert ttrs::OrderResponse => dsta::PlacedOrder (existing, kept for reference)
pub fn order_response_to_placed_order(order: &OrderResponse) -> PlacedOrder 
{ let order_fill = match order.status.as_str() 
  { "FILLED" => OrderFill::Done,
    "Filled" => OrderFill::Done,
    "LIVE" | "RECEIVED" | "IN_FLIGHT" | "ROUTED" => OrderFill::Live,
    "Live" | "Received" | "In Flight" | "Routed" => OrderFill::Live,
    _ => 
    { log::warn!("Unparsed order fill = {} is mared as DONE", order.status);
      OrderFill::Done
    } 
  };

  let legs_fills: Vec<OrderLeg> = order.legs.iter()
  .map
  ( |leg| 
    { let price = if !leg.fills.is_empty() 
      { leg.fills.iter().map(|f| f.quantity).sum::<f64>()
      } 
      else 
      { 0.0
      };

      let quantity = if let Some(x) = leg.quantity { x } else 
      { log::warn!("no quant => Potential Notional order conversion! we may need to calc quantity ourselves. (ttai or bot0 can-do)");
        0.0 
      }; 

      let remquantity = if let Some(x) = leg.remaining_quantity { x } else 
      { log::warn!("no remaingin => Potential Notional order conversion! we may need to calc quantity ourselves. (ttai or bot0 can-do)");
        0.0 
      }; 


      let finass = if leg.symbol.len() >= 20
      { // 0-5 root
        // 6-11 expire
        // 12 P/C
        // 13-20 strike
        // TODO: convert option name to tickername: // probably format(".{}12-20 ", root.trim())
        crate::FinAss::OptDeets
        ( crate::Opt
          {   ticker_name : format!("") // String, (we should be able to deduce it )
            , option_name : format!("{}", leg.symbol) // String,
            , option_rite : if leg.symbol.chars().nth(12).map_or( false, |c| c == 'P') 
              { OptRite::Put
              } else 
              { OptRite::Call
              }
            , strike_spot : 0.0 // f64,
            , expire_date : chrono::DateTime::<chrono::Utc>::default() // chrono::DateTime<chrono::Utc>,
            , option_type : OptStyle::Usa
            , tick_rulers : None
            , last_ticker : None
          }
        )
      }
      else
      { crate::FinAss::StkDeets
        ( crate::Stk
          {   ticker_name : format!("{}", leg.symbol) // String,
            , ticker_info : format!("")
            , last_ticker : None
            , tick_rulers : None
          }
        )
      };

      OrderLeg 
      { buy_what: finass,
        quantity: quantity,
        remaining: remquantity,
        action: parse_order_action(&leg.action),
        price,
      }
    }
  )
  .collect();

  PlacedOrder 
  { order_hash: if let Some(x) = order.id {x} else { 0 },
    legs_fills,
    order_fill,
  }
}

/// Helper: Convert string action to BuyOrSell enum
fn parse_order_action(action: &str) -> BuyOrSell {
    match action.to_uppercase().as_str() {
        "BUY_TO_OPEN" => BuyOrSell::BuyToOpen,
        "BUY TO OPEN" => BuyOrSell::BuyToOpen,
        "Buy to Open" => BuyOrSell::BuyToOpen, // Actual value
        "SELL_TO_OPEN" => BuyOrSell::SellToOpen,
        "SELL TO OPEN" => BuyOrSell::SellToOpen,
        "Sell to Open" => BuyOrSell::SellToOpen,
        "BUY_TO_CLOSE" => BuyOrSell::BuyToClose,
        "BUY TO CLOSE" => BuyOrSell::BuyToClose,
        "Buy to Close" => BuyOrSell::BuyToClose,
        "SELL_TO_CLOSE" => BuyOrSell::SellToClose,
        "SELL TO CLOSE" => BuyOrSell::SellToClose,
        "Sell to Close" => BuyOrSell::SellToClose,
        "BUY" => BuyOrSell::BuyFutures,
        "Buy" => BuyOrSell::BuyFutures,
        "SELL" => BuyOrSell::SellFutures,
        "Sell" => BuyOrSell::SellFutures,
        _ => BuyOrSell::BuyToOpen,
    }
}

/// Convert ttrs::Balance => dsta::AccountUpdate
pub fn balance_to_account_update(balance: &Balance) -> AccountUpdate {
    AccountUpdate {
        ttai_uid: balance.account_number.clone(),
        long_usd: balance.cash_balance,
        can_sell: balance.derivative_buying_power,
        long_stk: balance.long_equity_value,
        sell_stk: balance.short_equity_value,
        long_opt: balance.long_derivative_value,
        sell_opt: balance.short_derivative_value,
        long_fut: 0.0, // Not in Balance struct
        sell_fut: 0.0,
        long_fop: 0.0,
        sell_fop: 0.0,
        long_xxx: balance.long_cryptocurrency_value,
        sell_xxx: balance.short_cryptocurrency_value,
        long_bnd: 0.0, // Not directly in Balance
        next_usd: balance.pending_cash,
        summings: balance.net_liquidating_value,
        cash_out: balance.cash_available_to_withdraw,
    }
}

/// Convert Vec<ttrs::Position> => dsta::PositionsList
pub fn positions_to_positions_list
( positions: &[Position]) -> PositionsList 
  { let mut pos_map = HashMap::new();
    for pos in positions 
    { let mut quantity = pos.quantity;
      if pos.quantity_direction.to_lowercase() == "short" && quantity > 0.0 
      { quantity = -quantity;
      }
      let finass = if pos.symbol.len() >= 20
      { // 0-5 root
        // 6-11 expire
        // 12 P/C
        // 13-20 strike
        // TODO: convert option name to tickername: // probably format(".{}12-20 ", root.trim())
        crate::FinAss::OptDeets
        ( crate::Opt
          {   ticker_name : format!("") // String, (we should be able to deduce it )
            , option_name : format!("{}", pos.symbol) // String,
            , option_rite : if pos.symbol.chars().nth(12).map_or( false, |c| c == 'P') 
              { OptRite::Put
              } else 
              { OptRite::Call
              }
            , strike_spot : 0.0 // f64,
            , expire_date : chrono::DateTime::<chrono::Utc>::default() // chrono::DateTime<chrono::Utc>,
            , option_type : OptStyle::Usa
            , tick_rulers : None
            , last_ticker : None
          }
        )
      }
      else
      { crate::FinAss::StkDeets
        ( crate::Stk
          {   ticker_name : format!("{}", pos.symbol) // String,
            , ticker_info : format!("")
            , last_ticker : None
            , tick_rulers : None
          }
        )
      };

      let current_pos = OwnedPosition 
      { asset : finass,
        owned : quantity,
        price: pos.average_open_price,
      };
      pos_map.insert(pos.symbol.clone(), current_pos);
    }

    PositionsList 
    { positions: pos_map,
    }
}

/// Convert Vec<ttrs::OrderResponse> => Vec<dsta::PlacedOrder>
pub fn order_responses_to_placed_orders(orders: &[OrderResponse]) -> Vec<PlacedOrder> {
    orders.iter().map(order_response_to_placed_order).collect()
}

/// Create a dsta::Ticker from symbol and price (for ticker streaming)
pub fn create_ticker(symbol: &str, bid: f64, ask: f64) -> Ticker {
    Ticker {
        name: symbol.to_string(),
        bid,
        bid_size: 0.0,
        ask,
        ask_size: 0.0,
        time: chrono::Utc::now(),
        theo_price:None,
    }
}

/// Create a stock dsta::FinAss from symbol and optional ticker
pub fn create_stock_fin_ass(symbol: &str, ticker: Option<Ticker>) -> FinAss {
    FinAss::StkDeets(Stk {
        ticker_name: symbol.to_string(),
        ticker_info: symbol.to_string(),
        last_ticker: ticker,
        tick_rulers: None,
    })
}

/// Create an option dsta::FinAss
pub fn create_option_fin_ass(
    ticker_name: &str,
    option_name: &str,
    strike: f64,
    expire_date: chrono::DateTime<chrono::Utc>,
    is_call: bool,
) -> FinAss {
    FinAss::OptDeets(Opt {
        ticker_name: ticker_name.to_string(),
        option_name: option_name.to_string(),
        option_rite: if is_call { OptRite::Call } else { OptRite::Put },
        strike_spot: strike,
        expire_date,
        option_type: OptStyle::Usa,
        tick_rulers: None,
        last_ticker: None,
    })
}