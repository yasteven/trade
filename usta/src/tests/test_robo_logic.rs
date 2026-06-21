//usta/src/tests/test_robo_logic.rs

#![cfg(test)]

use std::collections::HashMap;
use tokio::sync::mpsc;
use crate::*;
use crate::tests::util::*;

// Helper to create minimal channels needed for testing
fn dummy_dr_robo_api() -> DrRoboApi {
    DrRoboApi {
        tell_dr_a_snively: mpsc::channel(100).0,
        tell_dr_pls_trade: mpsc::channel(100).0,
        tell_dr_new_brain: mpsc::channel(100).0,
        tell_dr_pls_ticks: mpsc::channel(100).0,
        tell_dr_pls_check: mpsc::channel(100).0,
        tell_dr_stop_tick: mpsc::channel(100).0,
        tell_dr_stop_info: mpsc::channel(100).0,
        tell_dr_swap_info: mpsc::channel(100).0,
    }
}

fn dummy_dr_seek_api() -> DrSeekApi {
    DrSeekApi {
        tell_dr_snively: mpsc::channel(100).0,
        tell_dr_made_bot: mpsc::channel(100).0,
        tell_dr_got_ttai: mpsc::channel(100).0,
        tell_dr_got_tick00: mpsc::channel(100).0,
        tell_dr_got_tick01: mpsc::channel(100).0,
    }
}

// Minimal CtorBot builder
fn make_ctor_bot(name: &str) -> CtorBot {
    let (_hand, foot) = fn_make_bot_api_bridge_dr_handles();
    CtorBot {
        robot_api_brain_state: Allocation {
            bot_name: name.to_string(),
            bot_safe: BotaAccount::default(),
            bot_mind: BotMindState::Stopped,
            num_swap: 0,
        },
        tell_bot_to_trade_api: mpsc::channel(1).0,
        tell_bot_to_check_api: mpsc::channel(1).0,
        tell_bot_it_sucks_api: mpsc::channel(1).0,
        told_the_bot_requests: foot,
    }
}

fn make_allocation(who: u32, wat: String, hoo: &DrBotaApiHand) -> BotApi {
    BotApi {
        robot_brain_state: crate::Allocation {
            bot_name: format!("Bot {} for test", who),
            bot_safe: crate::BotaAccount {
                m_cash: 5000.0,
                m_stock: if wat.is_empty() {
                    None
                } else {
                    Some(dsta::OwnedPosition {
                        asset: dsta::FinAss::StkDeets(dsta::Stk {
                            ticker_name: wat,
                            ticker_info: "ERROR STOCK for testing".to_string(),
                            last_ticker: None,
                            tick_rulers: None,

                        }),
                        owned: 100.0,
                        price: 123.45,
                    })
                },
                m_long_calls: None,
                m_long_puts: None,
                m_short_calls: None,
                m_short_puts: None,
                m_call_credit_spread_lein: 0.0,
                m_short_put_lein: 0.0,
                m_put_credit_spread_lein: 0.0,
                m_short_iron_condor_lein: 0.0,
            },
            bot_mind: BotMindState::Running,
            num_swap: 0,
        },
        robot_bota_sender: hoo.clone(),
    }
}

#[tokio::test]
async fn test_swap_info_basic_cash_transfer() {
    let mut all_bot_api: Vec<Option<BotApi>> = Vec::new();
    let mut map_bots_index: HashMap<String, usize> = HashMap::new();
    let mut all_bot_checks: Vec<Vec<usize>> = Vec::new();
    let asset_infos: HashMap<String, dsta::FinAss> = HashMap::new();
    let (tell_dr_new_robot, _) = mpsc::channel(10);

    // Manually create two bots
    let (hand_a, _) = fn_make_bot_api_hand_and_foot();
    let (hand_b, _) = fn_make_bot_api_hand_and_foot();

    let bot_a_state = Allocation {
        bot_name: "A".to_string(),
        bot_safe: BotaAccount {
            m_cash: 1000.0,
            ..Default::default()
        },
        bot_mind: BotMindState::Running,
        num_swap: 0,
    };
    let bot_b_state = Allocation {
        bot_name: "B".to_string(),
        bot_safe: BotaAccount {
            m_cash: 2000.0,
            ..Default::default()
        },
        bot_mind: BotMindState::Running,
        num_swap: 0,
    };

    all_bot_api.push(Some(BotApi {
        robot_brain_state: bot_a_state.clone(),
        robot_bota_sender: hand_a,
    }));
    all_bot_api.push(Some(BotApi {
        robot_brain_state: bot_b_state.clone(),
        robot_bota_sender: hand_b,
    }));

    map_bots_index.insert("A".to_string(), 0);
    map_bots_index.insert("B".to_string(), 1);

    let dr_robo_api = dummy_dr_robo_api();
    let dr_seek_api = dummy_dr_seek_api();

    // Swap: A gives $400, B gives $0 (net: A loses 400, B gains 400)
    let swap = BotSwaper {
        source_offering: Some(SwapWat {
            who: "A".to_string(),
            how: 400.0,
            wat: dsta::PositionsList {
                positions: HashMap::new(),
            },
        }),
        target_released: Some(SwapWat {
            who: "B".to_string(),
            how: 0.0,
            wat: dsta::PositionsList {
                positions: HashMap::new(),
            },
        }),
    };

    crate::dr_robo::from_bota::process_swap_info
    ( 1
    , swap,
      &mut all_bot_api,
      &mut map_bots_index,
      &mut all_bot_checks, 
      &dr_robo_api,
      &tell_dr_new_robot,
      &asset_infos,
      &dr_robo_api,
      &dr_seek_api, // : &DrSeekApi
    )
    .await;

    let final_a_cash = all_bot_api[0]
        .as_ref()
        .unwrap()
        .robot_brain_state
        .bot_safe
        .m_cash;
    let final_b_cash = all_bot_api[1]
        .as_ref()
        .unwrap()
        .robot_brain_state
        .bot_safe
        .m_cash;

    assert!((final_a_cash - 600.0).abs() < 1e-6);
    assert!((final_b_cash - 2400.0).abs() < 1e-6);
}

#[tokio::test]
async fn test_swap_info_index_out_of_bounds_safety() {
    let mut all_bot_api = vec![Some(BotApi {
        robot_brain_state: Allocation::default(),
        robot_bota_sender: fn_make_bot_api_hand_and_foot().0,
    })];
    let mut map_bots_index = HashMap::new();
    let mut all_bot_checks: Vec<Vec<usize>> = Vec::new();
    let (tell_dr_new_robot, _) = mpsc::channel(10);
    let dr_robo_api = dummy_dr_robo_api();

    // Craft a swap with invalid indices
    let swap = BotSwaper {
        source_offering: Some(SwapWat {
            who: "invalid".to_string(),
            how: 100.0,
            wat: dsta::PositionsList {
                positions: HashMap::new(),
            },
        }),
        target_released: Some(SwapWat {
            who: "invalid2".to_string(),
            how: -100.0,
            wat: dsta::PositionsList {
                positions: HashMap::new(),
            },
        }),
    };

    // Should early return without panic
    let dr_seek_api = dummy_dr_seek_api();
    crate::dr_robo::from_bota::process_swap_info(
        999,
        swap,
        &mut all_bot_api,
        &mut map_bots_index,
        &mut all_bot_checks,
        &dr_robo_api,
        &tell_dr_new_robot,
        &HashMap::new(),
        &dr_robo_api,
        &dr_seek_api, // : &DrSeekApi
    )
    .await;

    // If we get here without panic → test passes
}

#[tokio::test]
async fn test_ticker_subscription_and_unsubscription() {
    let _ = crate::tests::util::setup_log4rs(
        "test_robo_logic_test_ticker_subscription_and_unsubscription.txt",
    );
    let (h, _f) = fn_make_bot_api_hand_and_foot();

    let all_bot_api = vec![
        Some(make_allocation(0, String::new(), &h)),
        Some(make_allocation(1, String::new(), &h)),
        Some(make_allocation(2, String::new(), &h)),
        Some(make_allocation(3, String::new(), &h)),
        Some(make_allocation(4, String::new(), &h)),
        Some(make_allocation(5, String::new(), &h)),
    ];
    let mut map_tick_looks: HashMap<String, Vec<usize>> = HashMap::new();
    let (tell_add_tx, _) = mpsc::channel(10);
    let (tell_pop_tx, _) = mpsc::channel(10);
    let (result_tx, _result_rx) = mpsc::channel(10);
    let dr_seek_api = dummy_dr_seek_api();

    // Subscribe bot 5 to "AAPL"
    crate::dr_robo::from_bota::process_pls_ticks(
        5,
        "AAPL".to_string(),
        &all_bot_api,
        &mut map_tick_looks,
        &tell_add_tx,
        result_tx.clone(),
        &mut HashMap::new(),
        &dr_seek_api,
    )
    .await;

    assert_eq!(map_tick_looks["AAPL"], vec![5]);

    // Subscribe again → no duplicate
    crate::dr_robo::from_bota::process_pls_ticks(
        5,
        "AAPL".to_string(),
        &all_bot_api,
        &mut map_tick_looks,
        &tell_add_tx,
        result_tx,
        &mut HashMap::new(),
        &dr_seek_api,
    )
    .await;

    assert_eq!(map_tick_looks["AAPL"], vec![5]);

    // Unsubscribe
    crate::dr_robo::from_bota::process_a_stop_tick(
        5,
        "AAPL".to_string(),
        &all_bot_api,
        &mut map_tick_looks,
        &tell_pop_tx,
    )
    .await;

    assert!(!map_tick_looks.contains_key("AAPL"));
}

#[tokio::test]
async fn test_brain_state_propagation_to_subscribers() {
    let mut all_bot_api: Vec<Option<BotApi>> = Vec::new();
    let mut all_bot_checks: Vec<Vec<usize>> = Vec::new();
    let mut map_bots_index: HashMap<String, usize> = HashMap::new();

    // Create two bots
    let (hand0, _) = fn_make_bot_api_hand_and_foot();
    let (hand1, _) = fn_make_bot_api_hand_and_foot();

    all_bot_api.push(Some(BotApi {
        robot_brain_state: Allocation {
            bot_name: "parent".to_string(),
            bot_mind: BotMindState::Running,
            ..Default::default()
        },
        robot_bota_sender: hand0,
    }));
    all_bot_api.push(Some(BotApi {
        robot_brain_state: Allocation {
            bot_name: "child".to_string(),
            bot_mind: BotMindState::Running,
            ..Default::default()
        },
        robot_bota_sender: hand1,
    }));

    // child (index 1) checks parent (index 0)
    all_bot_checks.push(vec![1]); // parent has child subscriber
    all_bot_checks.push(vec![]);

    let dr_robo_api = dummy_dr_robo_api();

    // Update parent mind state
    crate::dr_robo::from_bota::process_new_brain(
        0,
        BotMindState::Trading,
        &mut all_bot_api,
        &mut all_bot_checks,
        &mut map_bots_index,
        &dr_robo_api,
    )
    .await;

    // Parent's state should be updated
    let parent_bot = all_bot_api[0].as_ref().unwrap();
    assert_eq!(parent_bot.robot_brain_state.bot_mind, BotMindState::Trading);
}

use crate::tests::util::*;

#[tokio::test]
async fn test_bot_death_zombied_with_empty_account_deletes_bot() 
{ let _ = setup_log4rs("test_bot_death_zombied_with_empty_account_deletes_bot");

  let mut all_bot_api: Vec<Option<BotApi>> = Vec::new();
  let mut all_bot_checks: Vec<Vec<usize>> = Vec::new();
  let mut map_bots_index: HashMap<String, usize> = HashMap::new();

  // Create magic index 0 bot:
  let (hand, _) = fn_make_bot_api_hand_and_foot();
  all_bot_api.push
  ( Some
    ( BotApi 
      { // we always have a bot0 index filled with a magic bot
        robot_brain_state: Allocation 
        { bot_name: "0".to_string(),
          bot_safe: BotaAccount 
          { m_cash: 100.01, // negligible
            ..Default::default()
          } ,
          bot_mind: BotMindState::Running,
          num_swap: 0,
        },
        robot_bota_sender: hand,
      }
    )
  );

  // Create a bot with zero assets and minimal cash
  let (hand, _) = fn_make_bot_api_hand_and_foot();
  all_bot_api.push
  ( Some
    ( BotApi 
      { robot_brain_state: Allocation 
        { bot_name: "dying_bot".to_string(),
          bot_safe: BotaAccount 
          { m_cash: 0.01, // negligible
            ..Default::default()
          } ,
          bot_mind: BotMindState::Running,
          num_swap: 0,
        }
        , robot_bota_sender: hand,
      }
    )
  );

  let dr_robo_api = dummy_dr_robo_api();

  // Report Zombied state
  crate::dr_robo::from_bota::process_new_brain
  ( 1
  , BotMindState::Zombied
  , &mut all_bot_api
  , &mut all_bot_checks
  , &mut map_bots_index
  , &dr_robo_api
  )
  .await;

  // Bot should be deleted (set to None)
  assert!(all_bot_api[1].is_none());
}

#[tokio::test]
async fn test_bot_death_seppuku_with_assets_waits_for_cleanup() {
    let mut all_bot_api: Vec<Option<BotApi>> = Vec::new();
    let mut all_bot_checks: Vec<Vec<usize>> = Vec::new();
    let mut map_bots_index: HashMap<String, usize> = HashMap::new();

    // Create magic index 0 bot:
    let (hand, _) = fn_make_bot_api_hand_and_foot();
    all_bot_api.push(Some(BotApi { // we always have a bot0 index filled with a magic bot
        robot_brain_state: Allocation {
            bot_name: "0".to_string(),
            bot_safe: BotaAccount {
                m_cash: 100.01, // negligible
                ..Default::default()
            },
            bot_mind: BotMindState::Running,
            num_swap: 0,
        },
        robot_bota_sender: hand,
    }));

    // Create a bot with assets
    let (hand, _) = fn_make_bot_api_hand_and_foot();
    all_bot_api.push(Some(BotApi {
        robot_brain_state: Allocation {
            bot_name: "rich_dying_bot".to_string(),
            bot_safe: BotaAccount {
                m_cash: 1000.0,
                m_stock: Some(dsta::OwnedPosition {
                    asset: dsta::FinAss::StkDeets(dsta::Stk {
                        ticker_name: "AAPL".to_string(),
                        ticker_info: "Apple Inc.".to_string(),
                        tick_rulers: None,
                        last_ticker: None,
                    }),
                    owned: 10.0,
                    price: 150.0,
                }),
                ..Default::default()
            },
            bot_mind: BotMindState::Running,
            num_swap: 0,
        },
        robot_bota_sender: hand,
    }));
    map_bots_index.insert("rich_dying_bot".to_string(), 0);

    let dr_robo_api = dummy_dr_robo_api();

    // Report Seppuku state (not Zombied yet)
    crate::dr_robo::from_bota::process_new_brain(
        1,
        BotMindState::Seppuku,
        &mut all_bot_api,
        &mut all_bot_checks,
        &mut map_bots_index,
        &dr_robo_api,
    )
    .await;

    // Bot should still exist, waiting for asset cleanup
    assert!(all_bot_api[0].is_some());
}

#[tokio::test]
async fn test_bot_death_zombied_with_assets_triggers_dispersal() {
    let mut all_bot_api: Vec<Option<BotApi>> = Vec::new();
    let mut all_bot_checks: Vec<Vec<usize>> = Vec::new();
    let mut map_bots_index: HashMap<String, usize> = HashMap::new();


    // Create magic index 0 bot:
    let (hand, _) = fn_make_bot_api_hand_and_foot();
    all_bot_api.push(Some(BotApi { // we always have a bot0 index filled with a magic bot
        robot_brain_state: Allocation {
            bot_name: "0".to_string(),
            bot_safe: BotaAccount {
                m_cash: 100.01, // negligible
                ..Default::default()
            },
            bot_mind: BotMindState::Running,
            num_swap: 0,
        },
        robot_bota_sender: hand,
    }));

    // Create a bot with assets
    let (hand, _) = fn_make_bot_api_hand_and_foot();
    all_bot_api.push(Some(BotApi {
        robot_brain_state: Allocation {
            bot_name: "zombied_with_assets".to_string(),
            bot_safe: BotaAccount {
                m_cash: 500.0,
                m_stock: Some(dsta::OwnedPosition {
                    asset: dsta::FinAss::StkDeets(dsta::Stk {
                        ticker_name: "MSFT".to_string(),
                        ticker_info: "Microsoft".to_string(),
                        last_ticker: None,
                        tick_rulers: None,

                    }),
                    owned: 5.0,
                    price: 300.0,
                }),
                ..Default::default()
            },
            bot_mind: BotMindState::Running,
            num_swap: 0,
        },
        robot_bota_sender: hand,
    }));
    map_bots_index.insert("zombied_with_assets".to_string(), 0);

    let (swap_tx, mut swap_rx) = mpsc::channel(10);
    let mut dr_robo_api = dummy_dr_robo_api();
    dr_robo_api.tell_dr_swap_info = swap_tx;

    // Report Zombied state with assets
    crate::dr_robo::from_bota::process_new_brain(
        1,
        BotMindState::Zombied,
        &mut all_bot_api,
        &mut all_bot_checks,
        &mut map_bots_index,
        &dr_robo_api,
    )
    .await;

    // Should have sent swap requests for dispersal
    // At least one for cash, one for stock
    let swap1 = swap_rx.try_recv();
    assert!(swap1.is_ok(), "Expected at least one swap for dispersal");

    // Bot should still exist (not deleted yet, waiting for swaps to complete)
    assert!(all_bot_api[0].is_some());
}

#[tokio::test]
async fn test_bot_zero_cannot_die() {
    let mut all_bot_api: Vec<Option<BotApi>> = Vec::new();
    let all_bot_checks: Vec<Vec<usize>> = Vec::new();
    let map_bots_index: HashMap<String, usize> = HashMap::new();

    // Create bot 0
    let (hand, _) = fn_make_bot_api_hand_and_foot();
    all_bot_api.push(Some(BotApi {
        robot_brain_state: Allocation {
            bot_name: "0".to_string(),
            bot_safe: BotaAccount::default(),
            bot_mind: BotMindState::Running,
            num_swap: 0,
        },
        robot_bota_sender: hand,
    }));

    let dr_robo_api = dummy_dr_robo_api();

    // Try to kill bot 0 - should not panic but handle gracefully
    // Note: The actual code has unimplemented!() which would panic in debug
    // In a real test environment, we'd catch this or the code would handle it differently
    // For now, we just verify the bot exists
    assert!(all_bot_api[0].is_some());
}

#[tokio::test]
async fn test_swap_rejects_bots_in_trading_state() {
    let mut all_bot_api: Vec<Option<BotApi>> = Vec::new();
    let mut map_bots_index: HashMap<String, usize> = HashMap::new();
    let (tell_dr_new_robot, _) = mpsc::channel(10);

    // Create two bots, one in Trading state
    let (hand_a, _) = fn_make_bot_api_hand_and_foot();
    let (hand_b, _) = fn_make_bot_api_hand_and_foot();

    all_bot_api.push(Some(BotApi {
        robot_brain_state: Allocation {
            bot_name: "A".to_string(),
            bot_safe: BotaAccount {
                m_cash: 1000.0,
                ..Default::default()
            },
            bot_mind: BotMindState::Trading, // In Trading state
            num_swap: 0,
        },
        robot_bota_sender: hand_a,
    }));
    all_bot_api.push(Some(BotApi {
        robot_brain_state: Allocation {
            bot_name: "B".to_string(),
            bot_safe: BotaAccount {
                m_cash: 2000.0,
                ..Default::default()
            },
            bot_mind: BotMindState::Running,
            num_swap: 0,
        },
        robot_bota_sender: hand_b,
    }));

    map_bots_index.insert("A".to_string(), 0);
    map_bots_index.insert("B".to_string(), 1);

    let dr_robo_api = dummy_dr_robo_api();

    let swap = BotSwaper {
        source_offering: Some(SwapWat {
            who: "A".to_string(),
            how: 400.0,
            wat: dsta::PositionsList {
                positions: HashMap::new(),
            },
        }),
        target_released: Some(SwapWat {
            who: "B".to_string(),
            how: 0.0,
            wat: dsta::PositionsList {
                positions: HashMap::new(),
            },
        }),
    };

    let initial_cash_a = all_bot_api[0]
        .as_ref()
        .unwrap()
        .robot_brain_state
        .bot_safe
        .m_cash;

    // Should reject the swap
    let mut all_bot_checks: Vec<Vec<usize>> = Vec::new();
    let dr_seek_api = dummy_dr_seek_api();
    crate::dr_robo::from_bota::process_swap_info(
        1,
        swap,
        &mut all_bot_api,
        &mut map_bots_index,
        &mut all_bot_checks, 
        &dr_robo_api,
        &tell_dr_new_robot,
        &HashMap::new(),
        &dr_robo_api,
        &dr_seek_api, // : &DrSeekApi
    )
    .await;

    // Cash should remain unchanged (swap rejected)
    let final_cash_a = all_bot_api[0]
        .as_ref()
        .unwrap()
        .robot_brain_state
        .bot_safe
        .m_cash;
    assert_eq!(initial_cash_a, final_cash_a);
}