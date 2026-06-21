// usta/src/tests/test_robo_accounts_comprehensive.rs
use crate::*;
use crate::core::*;
use crate::dr_robo::*;
use crate::tests::util::*;
use dsta::*;
use std::collections::HashMap;

fn setup() {
    // setup_log4rs("comprehensive-tests");
    setup_logenv();
}

// ============================================================================
// HELPER FUNCTIONS (updated to use buy_what: FinAss)
// ============================================================================

fn buy_stock(qty: f64, limit_price: f64) -> Order {
    let mut stk = dsta::Stk::default();
    stk.ticker_name = "STK".to_string();

    Order {
        order_legs: vec![OrderLeg {
            buy_what: dsta::FinAss::StkDeets(stk),
            quantity: qty,
            remaining: qty,
            action: BuyOrSell::BuyToOpen,
            price: limit_price,
        }],
        order_type: MarketOrLimit::Limit,
        //order_cost: Some(-limit_price * qty * 100.0),
    }
}

fn sell_stock(qty: f64, limit_price: f64) -> Order {
    let mut stk = dsta::Stk::default();
    stk.ticker_name = "STK".to_string();

    Order {
        order_legs: vec![OrderLeg {
            buy_what: dsta::FinAss::StkDeets(stk),
            quantity: qty,
            remaining: qty,
            action: BuyOrSell::SellToClose,
            price: limit_price,
        }],
        order_type: MarketOrLimit::Limit,
        //order_cost: Some(limit_price * qty * 100.0),
    }
}

fn buy_put(strike: f64, days: f64, qty: f64, premium: f64) -> Order {
    let mut opt = dsta::Opt::default();
    opt.ticker_name = format!(".STK {} P {}", strike, days);
    opt.option_name = format!("STK {} P {}", strike, days);
    opt.strike_spot = strike;
    opt.option_rite = dsta::OptRite::Put;

    Order {
        order_legs: vec![OrderLeg {
            buy_what: dsta::FinAss::OptDeets(opt),
            quantity: qty,
            remaining: qty,
            action: BuyOrSell::BuyToOpen,
            price: premium,
        }],
        order_type: MarketOrLimit::Limit,
        //order_cost: Some(-premium * qty * 100.0),
    }
}

fn sell_put(strike: f64, days: f64, qty: f64, premium: f64) -> Order {
    let mut opt = dsta::Opt::default();
    opt.ticker_name = format!(".STK {} P {}", strike, days);
    opt.option_name = format!("STK {} P {}", strike, days);
    opt.strike_spot = strike;
    opt.option_rite = dsta::OptRite::Put;

    Order {
        order_legs: vec![OrderLeg {
            buy_what: dsta::FinAss::OptDeets(opt),
            quantity: qty,
            remaining: qty,
            action: BuyOrSell::SellToOpen,
            price: premium,
        }],
        order_type: MarketOrLimit::Limit,
        //order_cost: Some(premium * qty * 100.0),
    }
}

fn buy_call(strike: f64, days: f64, qty: f64, premium: f64) -> Order {
    let mut opt = dsta::Opt::default();
    opt.ticker_name = format!(".STK {} C {}", strike, days);
    opt.option_name = format!("STK {} C {}", strike, days);
    opt.strike_spot = strike;
    opt.option_rite = dsta::OptRite::Call;

    Order {
        order_legs: vec![OrderLeg {
            buy_what: dsta::FinAss::OptDeets(opt),
            quantity: qty,
            remaining: qty,
            action: BuyOrSell::BuyToOpen,
            price: premium,
        }],
        order_type: MarketOrLimit::Limit,
        //order_cost: Some(-premium * qty * 100.0),
    }
}

fn sell_call(strike: f64, days: f64, qty: f64, premium: f64) -> Order {
    let mut opt = dsta::Opt::default();
    opt.ticker_name = format!(".STK {} C {}", strike, days);
    opt.option_name = format!("STK {} C {}", strike, days);
    opt.strike_spot = strike;
    opt.option_rite = dsta::OptRite::Call;

    Order {
        order_legs: vec![OrderLeg {
            buy_what: dsta::FinAss::OptDeets(opt),
            quantity: qty,
            remaining: qty,
            action: BuyOrSell::SellToOpen,
            price: premium,
        }],
        order_type: MarketOrLimit::Limit,
        //order_cost: Some(premium * qty * 100.0),
    }
}

fn close_long_put(strike: f64, days: f64, qty: f64, premium: f64) -> Order 
{ 
  
    let mut opt = dsta::Opt::default();
    opt.ticker_name = format!(".STK {} P {}", strike, days);
    opt.option_name = format!("STK {} P {}", strike, days);
    opt.strike_spot = strike;
    opt.option_rite = dsta::OptRite::Put;

    Order {
        order_legs: vec![OrderLeg {
            buy_what: dsta::FinAss::OptDeets(opt),
            quantity: qty,
            remaining: qty,
            action: BuyOrSell::SellToClose,
            price: premium,
        }],
        order_type: MarketOrLimit::Limit,
        //order_cost: Some(premium * qty * 100.0),
    }
}

fn close_short_put(strike: f64, days: f64, qty: f64, premium: f64) -> Order {
    let mut opt = dsta::Opt::default();
    opt.ticker_name = format!(".STK {} P {}", strike, days);
    opt.option_name = format!("STK {} P {}", strike, days);
    opt.strike_spot = strike;
    opt.option_rite = dsta::OptRite::Put;

    Order {
        order_legs: vec![OrderLeg {
            buy_what: dsta::FinAss::OptDeets(opt),
            quantity: qty,
            remaining: qty,
            action: BuyOrSell::BuyToClose,
            price: premium,
        }],
        order_type: MarketOrLimit::Limit,
        //order_cost: Some(-premium * qty * 100.0),
    }
}

fn close_long_call(strike: f64, days: f64, qty: f64, premium: f64) -> Order {
    let mut opt = dsta::Opt::default();
    opt.ticker_name = format!(".STK {} C {}", strike, days);
    opt.option_name = format!("STK {} C {}", strike, days);
    opt.strike_spot = strike;
    opt.option_rite = dsta::OptRite::Call;

    Order {
        order_legs: vec![OrderLeg {
            buy_what: dsta::FinAss::OptDeets(opt),
            quantity: qty,
            remaining: qty,
            action: BuyOrSell::SellToClose,
            price: premium,
        }],
        order_type: MarketOrLimit::Limit,
        //order_cost: Some(premium * qty * 100.0),
    }
}

fn close_short_call(strike: f64, days: f64, qty: f64, premium: f64) -> Order {
    let mut opt = dsta::Opt::default();
    opt.ticker_name = format!(".STK {} C {}", strike, days);
    opt.option_name = format!("STK {} C {}", strike, days);
    opt.strike_spot = strike;
    opt.option_rite = dsta::OptRite::Call;

    Order {
        order_legs: vec![OrderLeg {
            buy_what: dsta::FinAss::OptDeets(opt),
            quantity: qty,
            remaining: qty,
            action: BuyOrSell::BuyToClose,
            price: premium,
        }],
        order_type: MarketOrLimit::Limit,
        //order_cost: Some(-premium * qty * 100.0),
    }
}

fn put_credit_spread(short_strike: f64, long_strike: f64, days: f64, qty: f64, credit: f64) -> Order {
    let mut short_opt = dsta::Opt::default();
    short_opt.ticker_name = format!(".STK {} P {}", short_strike, days);
    short_opt.option_name = format!("STK {} P {}", short_strike, days);
    short_opt.strike_spot = short_strike;
    short_opt.option_rite = dsta::OptRite::Put;

    let mut long_opt = dsta::Opt::default();
    long_opt.ticker_name = format!(".STK {} P {}", long_strike, days);
    long_opt.option_name = format!("STK {} P {}", long_strike, days);
    long_opt.strike_spot = long_strike;
    long_opt.option_rite = dsta::OptRite::Put;

    Order {
        order_legs: vec![
            OrderLeg {
                buy_what: dsta::FinAss::OptDeets(short_opt),
                quantity: qty,
                remaining: qty,
                action: BuyOrSell::SellToOpen,
                price: credit + 0.5,
            },
            OrderLeg {
                buy_what: dsta::FinAss::OptDeets(long_opt),
                quantity: qty,
                remaining: qty,
                action: BuyOrSell::BuyToOpen,
                price: 0.5,
            },
        ],
        order_type: MarketOrLimit::Limit,
        //order_cost: Some(credit * qty * 100.0),
    }
}

fn call_credit_spread(short_strike: f64, long_strike: f64, days: f64, qty: f64, credit: f64) -> Order {
    let mut short_opt = dsta::Opt::default();
    short_opt.ticker_name = format!(".STK {} C {}", short_strike, days);
    short_opt.option_name = format!("STK {} C {}", short_strike, days);
    short_opt.strike_spot = short_strike;
    short_opt.option_rite = dsta::OptRite::Call;

    let mut long_opt = dsta::Opt::default();
    long_opt.ticker_name = format!(".STK {} C {}", long_strike, days);
    long_opt.option_name = format!("STK {} C {}", long_strike, days);
    long_opt.strike_spot = long_strike;
    long_opt.option_rite = dsta::OptRite::Call;

    Order {
        order_legs: vec![
            OrderLeg {
                buy_what: dsta::FinAss::OptDeets(short_opt),
                quantity: qty,
                remaining: qty,
                action: BuyOrSell::SellToOpen,
                price: credit + 0.5,
            },
            OrderLeg {
                buy_what: dsta::FinAss::OptDeets(long_opt),
                quantity: qty,
                remaining: qty,
                action: BuyOrSell::BuyToOpen,
                price: 0.5,
            },
        ],
        order_type: MarketOrLimit::Limit,
        //order_cost: Some(credit * qty * 100.0),
    }
}

fn close_put_credit_spread(short_strike: f64, long_strike: f64, days: f64, qty: f64, debit: f64) -> Order {
    let mut short_opt = dsta::Opt::default();
    short_opt.ticker_name = format!(".STK {} P {}", short_strike, days);
    short_opt.option_name = format!("STK {} P {}", short_strike, days);
    short_opt.strike_spot = short_strike;
    short_opt.option_rite = dsta::OptRite::Put;

    let mut long_opt = dsta::Opt::default();
    long_opt.ticker_name = format!(".STK {} P {}", long_strike, days);
    long_opt.option_name = format!("STK {} P {}", long_strike, days);
    long_opt.strike_spot = long_strike;
    long_opt.option_rite = dsta::OptRite::Put;

    Order {
        order_legs: vec![
            OrderLeg {
                buy_what: dsta::FinAss::OptDeets(long_opt),
                quantity: qty,
                remaining: qty,
                action: BuyOrSell::SellToClose,
                price: 0.3,
            },
            OrderLeg {
                buy_what: dsta::FinAss::OptDeets(short_opt),
                quantity: qty,
                remaining: qty,
                action: BuyOrSell::BuyToClose,
                price: debit + 0.3,
            },
        ],
        order_type: MarketOrLimit::Limit,
        //order_cost: Some(-debit * qty * 100.0),
    }
}

fn close_call_credit_spread(short_strike: f64, long_strike: f64, days: f64, qty: f64, debit: f64) -> Order {
    let mut short_opt = dsta::Opt::default();
    short_opt.ticker_name = format!(".STK {} C {}", short_strike, days);
    short_opt.option_name = format!("STK {} C {}", short_strike, days);
    short_opt.strike_spot = short_strike;
    short_opt.option_rite = dsta::OptRite::Call;

    let mut long_opt = dsta::Opt::default();
    long_opt.ticker_name = format!(".STK {} C {}", long_strike, days);
    long_opt.option_name = format!("STK {} C {}", long_strike, days);
    long_opt.strike_spot = long_strike;
    long_opt.option_rite = dsta::OptRite::Call;

    Order {
        order_legs: vec![
            OrderLeg {
                buy_what: dsta::FinAss::OptDeets(long_opt),
                quantity: qty,
                remaining: qty,
                action: BuyOrSell::SellToClose,
                price: 0.3,
            },
            OrderLeg {
                buy_what: dsta::FinAss::OptDeets(short_opt),
                quantity: qty,
                remaining: qty,
                action: BuyOrSell::BuyToClose,
                price: debit + 0.3,
            },
        ],
        order_type: MarketOrLimit::Limit,
        //order_cost: Some(-debit * qty * 100.0),
    }
}

// ============================================================================
// BASIC LEGALITY TESTS (updated to match new helpers)
// ============================================================================

#[test]
fn legality_empty_order_allowed() {
    setup();
    let account = BotaAccount::new().with_cash(10000.0);
    let order = Order {
        order_legs: vec![],
        order_type: MarketOrLimit::Limit,
        //order_cost: Some(0.0),
    };
    let stuff = make_stuff_with_tickers(&[]);
    assert!(fn_check_bota_legality(&account, &order, &stuff).is_ok());
}

#[test]
fn legality_market_order_fails() {
    setup();
    let account = BotaAccount::new().with_cash(10000.0);
    let mut order = buy_stock(1.0, 50.0);
    order.order_type = MarketOrLimit::Market;
    let stuff = make_stuff_with_tickers(&["STK"]);
    assert!(fn_check_bota_legality(&account, &order, &stuff).is_err());
}

#[test]
fn legality_different_underlyings_fails() {
    setup();
    let account = BotaAccount::new().with_cash(10000.0);
    let order = Order {
        order_legs: vec![
            OrderLeg {
                buy_what: dsta::FinAss::OptDeets(dsta::Opt {
                    ticker_name: ".SPY 450 P 30".to_string(),
                    option_name: "SPY 450 P 30".to_string(),
                    ..Default::default()
                }),
                quantity: 1.0,
                remaining: 1.0,
                action: BuyOrSell::BuyToOpen,
                price: 2.0,
            },
            OrderLeg {
                buy_what: dsta::FinAss::OptDeets(dsta::Opt {
                    ticker_name: ".QQQ 350 P 30".to_string(),
                    option_name: "QQQ 350 P 30".to_string(),
                    ..Default::default()
                }),
                quantity: 1.0,
                remaining: 1.0,
                action: BuyOrSell::SellToOpen,
                price: 3.0,
            },
        ],
        order_type: MarketOrLimit::Limit,
        //order_cost: Some(100.0),
    };
    let stuff = make_stuff_with_tickers(&["SPY 450 P 30", "QQQ 350 P 30"]);
    assert!(fn_check_bota_legality(&account, &order, &stuff).is_err());
}
// ============================================================================
// STOCK LEGALITY TESTS
// ============================================================================

#[test]
fn legality_buy_stock_ok() {
    setup();
    let account = BotaAccount::new().with_cash(10000.0);
    let order = buy_stock(1.0, 50.0);
    let stuff = make_stuff_with_tickers(&["STK"]);
    
    assert!(fn_check_bota_legality(&account, &order, &stuff).is_ok());
}

#[test]
fn legality_sell_to_open_stock_fails() {
    setup();
       let mut stk = dsta::Stk::default();
          stk.ticker_name = format!("STK");
    let account = BotaAccount::new().with_cash(10000.0);
    let order = Order {
        order_legs: vec![OrderLeg {
                buy_what: dsta::FinAss::StkDeets(stk),
            quantity: 1.0,
            remaining: 1.0,
            action: BuyOrSell::SellToOpen,
            price: 50.0,
        }],
        order_type: MarketOrLimit::Limit,
        //order_cost: Some(5000.0),
    };
    let stuff = make_stuff_with_tickers(&["STK"]);
    
    assert!(fn_check_bota_legality(&account, &order, &stuff).is_err());
}

#[test]
fn legality_sell_to_close_without_position_fails() {
    setup();
    let account = BotaAccount::new().with_cash(10000.0);
    let order = sell_stock(1.0, 50.0);
    let stuff = make_stuff_with_tickers(&["STK"]);
    
    assert!(fn_check_bota_legality(&account, &order, &stuff).is_err());
}

#[test]
fn legality_sell_to_close_insufficient_shares_fails() {
    setup();
    let account = BotaAccount::new()
        .with_cash(10000.0)
        .with_stock(50.0, 1.0, 50.0);
    let order = sell_stock(2.0, 50.0);
    let stuff = make_stuff_with_tickers(&["STK"]);
    
    assert!(fn_check_bota_legality(&account, &order, &stuff).is_err());
}

#[test]
fn legality_sell_to_close_with_position_ok() {
    setup();
    let account = BotaAccount::new()
        .with_cash(10000.0)
        .with_stock(50.0, 2.0, 50.0);
    let order = sell_stock(1.0, 55.0);
    let stuff = make_stuff_with_tickers(&["STK"]);
    
    assert!(fn_check_bota_legality(&account, &order, &stuff).is_ok());
}

// ============================================================================
// SINGLE OPTION LEGALITY TESTS
// ============================================================================

#[test]
fn legality_buy_long_put_ok() {
    setup();
    let account = BotaAccount::new().with_cash(10000.0);
    let order = buy_put(50.0, 30.0, 1.0, 2.0);
    let stuff = make_stuff_with_tickers(&["STK 50 P 30"]);
    
    assert!(fn_check_bota_legality(&account, &order, &stuff).is_ok());
}

#[test]
fn legality_buy_long_call_ok() {
    setup();
    let account = BotaAccount::new().with_cash(10000.0);
    let order = buy_call(50.0, 30.0, 1.0, 2.0);
    let stuff = make_stuff_with_tickers(&["STK 50 C 30"]);
    
    assert!(fn_check_bota_legality(&account, &order, &stuff).is_ok());
}

#[test]
fn legality_sell_short_put_ok() {
    setup();
    let account = BotaAccount::new().with_cash(10000.0);
    let order = sell_put(50.0, 30.0, 1.0, 2.0);
    let stuff = make_stuff_with_tickers(&["STK 50 P 30"]);
    
    assert!(fn_check_bota_legality(&account, &order, &stuff).is_ok());
}

#[test]
fn legality_close_long_put_without_position_fails() {
    setup();
    let account = BotaAccount::new().with_cash(10000.0);
    let order = close_long_put(50.0, 30.0, 1.0, 1.0);
    let stuff = make_stuff_with_tickers(&["STK 50 P 30"]);
    
    assert!(fn_check_bota_legality(&account, &order, &stuff).is_err());
}

#[test]
fn legality_close_long_put_insufficient_quantity_fails() {
    setup();
    let account = BotaAccount::new()
        .with_cash(10000.0)
        .with_long_put(50.0, 1.0, 30.0, 2.0);
    let order = close_long_put(50.0, 30.0, 2.0, 1.0);
    let stuff = make_stuff_with_tickers(&["STK 50 P 30"]);
    
    assert!(fn_check_bota_legality(&account, &order, &stuff).is_err());
}

#[test]
fn legality_close_short_put_without_position_fails() {
    setup();
    let account = BotaAccount::new().with_cash(10000.0);
    let order = close_short_put(50.0, 30.0, 1.0, 1.5);
    let stuff = make_stuff_with_tickers(&["STK 50 P 30"]);
    
    assert!(fn_check_bota_legality(&account, &order, &stuff).is_err());
}

#[test]
fn legality_close_long_call_with_position_ok() {
    setup();
    let account = BotaAccount::new()
        .with_cash(10000.0)
        .with_long_call(55.0, 2.0, 30.0, 3.0);
    let order = close_long_call(55.0, 30.0, 1.0, 4.0);
    let stuff = make_stuff_with_tickers(&["STK 55 C 30"]);
    
    assert!(fn_check_bota_legality(&account, &order, &stuff).is_ok());
}

// ============================================================================
// PUT CREDIT SPREAD TESTS
// ============================================================================

#[test]
fn legality_put_credit_spread_opening_ok() {
    setup();
    let account = BotaAccount::new().with_cash(10000.0);
    let order = put_credit_spread(50.0, 45.0, 30.0, 1.0, 1.5);
    let stuff = make_stuff_with_tickers(&["STK 50 P 30", "STK 45 P 30"]);
    
    assert!(fn_check_bota_legality(&account, &order, &stuff).is_ok());
}

#[test]
fn legality_put_credit_spread_invalid_strikes_fails() {
    setup();
    let account = BotaAccount::new().with_cash(10000.0);
    // Short strike should be > long strike for credit spread
    let order = put_credit_spread(45.0, 50.0, 30.0, 1.0, 1.5);
    let stuff = make_stuff_with_tickers(&["STK 45 P 30", "STK 50 P 30"]);
    
    assert!(fn_check_bota_legality(&account, &order, &stuff).is_err());
}

#[test]
fn legality_put_credit_spread_insufficient_cash_fails() {
    setup();
    let account = BotaAccount::new().with_cash(349.999);
    let order = put_credit_spread(50.0, 45.0, 30.0, 1.0, 1.5);
    let stuff = make_stuff_with_tickers(&["STK 50 P 30", "STK 45 P 30"]);
    
    assert!(fn_check_bota_legality(&account, &order, &stuff).is_err());

    let account = BotaAccount::new().with_cash(350.000);
    let order = put_credit_spread(50.0, 45.0, 30.0, 1.0, 1.5);
    let stuff = make_stuff_with_tickers(&["STK 50 P 30", "STK 45 P 30"]);
    
    assert!(fn_check_bota_legality(&account, &order, &stuff).is_ok());
}

#[test]
fn legality_close_put_credit_spread_ok() {
    setup();
    let account = BotaAccount::new()
        .with_cash(7000.0)
        .with_short_put(50.0, -1.0, 30.0, 2.5)
        .with_long_put(45.0, 1.0, 30.0, 1.0);
    let order = close_put_credit_spread(50.0, 45.0, 30.0, 1.0, 1.0);
    let stuff = make_stuff_with_tickers(&["STK 50 P 30", "STK 45 P 30"]);
    
    assert!(fn_check_bota_legality(&account, &order, &stuff).is_ok());
}

#[test]
fn legality_put_credit_spread_matching_existing_ok() {
    setup();
    let account = BotaAccount::new()
        .with_cash(7000.0)
        .with_short_put(50.0, -1.0, 30.0, 2.5)
        .with_long_put(45.0, 1.0, 30.0, 1.0);
    let order = put_credit_spread(50.0, 45.0, 30.0, 1.0, 1.5);
    let stuff = make_stuff_with_tickers(&["STK 50 P 30", "STK 45 P 30"]);
    
    assert!(fn_check_bota_legality(&account, &order, &stuff).is_ok());
}

#[test]
fn legality_put_credit_spread_conflicting_existing_fails() {
    setup();
    let account = BotaAccount::new()
        .with_cash(7000.0)
        .with_short_put(50.0, -1.0, 30.0, 2.5)
        .with_long_put(45.0, 1.0, 30.0, 1.0);
    // Try to open different spread
    let order = put_credit_spread(55.0, 50.0, 30.0, 1.0, 1.5);
    let stuff = make_stuff_with_tickers(&["STK 55 P 30", "STK 50 P 30", "STK 45 P 30"]);
    
    assert!(fn_check_bota_legality(&account, &order, &stuff).is_err());
}

// ============================================================================
// CALL CREDIT SPREAD TESTS
// ============================================================================

#[test]
fn legality_call_credit_spread_opening_ok() {
    setup();
    let account = BotaAccount::new().with_cash(10000.0);
    let order = call_credit_spread(55.0, 60.0, 30.0, 1.0, 1.5);
    let stuff = make_stuff_with_tickers(&["STK 55 C 30", "STK 60 C 30"]);
    
    assert!(fn_check_bota_legality(&account, &order, &stuff).is_ok());
}

#[test]
fn legality_call_credit_spread_invalid_strikes_fails() {
    setup();
    let account = BotaAccount::new().with_cash(10000.0);
    // Long strike should be > short strike for credit spread
    let order = call_credit_spread(60.0, 55.0, 30.0, 1.0, 1.5);
    let stuff = make_stuff_with_tickers(&["STK 60 C 30", "STK 55 C 30"]);
    
    assert!(fn_check_bota_legality(&account, &order, &stuff).is_err());
}

#[test]
fn legality_close_call_credit_spread_ok() {
    setup();
    let account = BotaAccount::new()
        .with_cash(7000.0)
        .with_short_call(55.0, -1.0, 30.0, 3.0)
        .with_long_call(60.0, 1.0, 30.0, 1.0);
    let order = close_call_credit_spread(55.0, 60.0, 30.0, 1.0, 1.0);
    let stuff = make_stuff_with_tickers(&["STK 55 C 30", "STK 60 C 30"]);
    
    assert!(fn_check_bota_legality(&account, &order, &stuff).is_ok());
}

#[test]
fn legality_call_credit_spread_conflicting_short_fails() {
    setup();
    let account = BotaAccount::new()
        .with_cash(7000.0)
        .with_short_call(55.0, -1.0, 30.0, 3.0)
        .with_long_call(60.0, 1.0, 30.0, 1.0);
    // Try to open different spread
    let order = call_credit_spread(50.0, 55.0, 30.0, 1.0, 1.5);
    let stuff = make_stuff_with_tickers(&["STK 50 C 30", "STK 55 C 30", "STK 60 C 30"]);
    
    assert!(fn_check_bota_legality(&account, &order, &stuff).is_err());
}

// ============================================================================
// MARGIN CALCULATION TESTS
// ============================================================================

#[test]
fn margin_no_positions_no_leins() {
    setup();
    let account = BotaAccount::new().with_cash(10000.0);
    let leins = calculate_option_margin(&account).unwrap();
    
    assert_eq!(leins.len(), 0);
}

#[test]
fn margin_naked_short_put_requires_strike_cash() {
    setup();
    let account = BotaAccount::new()
        .with_cash(10000.0)
        .with_short_put(50.0, -1.0, 30.0, 2.0);
    
    let leins = calculate_option_margin(&account).unwrap();
    assert_eq!(leins.len(), 1);
    assert_eq!(leins[0].0, LeinType::ShortPut);
    assert_eq!(leins[0].1, 5000.0);
}

#[test]
#[should_panic(expected = "uncovered short put")]
fn margin_naked_short_put_insufficient_cash_fails() {
    setup();
    let account = BotaAccount::new()
        .with_cash(3000.0)
        .with_short_put(50.0, -1.0, 30.0, 2.0);
    
    let _ = calculate_option_margin(&account).unwrap();
}

#[test]
fn margin_put_credit_spread() {
    setup();
    let account = BotaAccount::new()
        .with_cash(10000.0)
        .with_short_put(50.0, -2.0, 30.0, 2.5)
        .with_long_put(45.0, 2.0, 30.0, 0.8);
    
    let leins = calculate_option_margin(&account).unwrap();
    assert_eq!(leins.len(), 1);
    assert_eq!(leins[0].0, LeinType::PutCreditSpread);
    assert_eq!(leins[0].1, 1000.0); // (50-45) * 100 * 2 contracts
}

#[test]
fn margin_call_credit_spread() {
    setup();
    let account = BotaAccount::new()
        .with_cash(10000.0)
        .with_short_call(55.0, -1.0, 30.0, 3.0)
        .with_long_call(60.0, 1.0, 30.0, 1.0);
    
    let leins = calculate_option_margin(&account).unwrap();
    assert_eq!(leins.len(), 1);
    assert_eq!(leins[0].0, LeinType::CallCreditSpread);
    assert_eq!(leins[0].1, 500.0); // (60-55) * 100 * 1 contract
}

#[test]
fn margin_iron_condor_uses_max_width() {
    setup();
    let account = BotaAccount::new()
        .with_cash(20000.0)
        .with_short_call(55.0, -1.0, 30.0, 2.0)
        .with_long_call(60.0, 1.0, 30.0, 0.5)
        .with_short_put(45.0, -1.0, 30.0, 1.8)
        .with_long_put(40.0, 1.0, 30.0, 0.4);
    
    let leins = calculate_option_margin(&account).unwrap();
    assert_eq!(leins.len(), 1);
    assert_eq!(leins[0].0, LeinType::IronCondor);
    // Max width: max(60-55, 45-40) = 5
    assert_eq!(leins[0].1, 500.0);
}

#[test]
fn margin_stock_covered_call() {
    setup();
    let account = BotaAccount::new()
        .with_cash(10000.0)
        .with_stock(50.0, 100.0, 50.0) // need 100 to cover a call
        .with_short_call(55.0, -1.0, 30.0, 2.0);
    
    let leins = calculate_option_margin(&account).unwrap();
    // Stock covers the short call, no lein required
    assert_eq!(leins.len(), 0);
}

#[test]
#[should_panic(expected = "uncovered short call")]
fn margin_uncovered_short_call_fails() {
    setup();
    let account = BotaAccount::new()
        .with_cash(10000.0)
        .with_short_call(55.0, -1.0, 30.0, 2.0);
    
    let _ = calculate_option_margin(&account).unwrap();
}
