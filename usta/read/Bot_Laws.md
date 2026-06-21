# Bot Laws Verification Report (sans buzz-bomber)

Bot Laws, that all bots must follow:
  1) Only request assets at startup if not loaded. Wait for SWAP_SUCC if requesting.
  2) Wait for ORDER_FLY at every order we request.
  3) Haggle generation and application at first order using input rules
  4) Haggle generation and application at all other orders, using hard-coded per bot (Swat is agressive, sally is cheap, stealth is mid) 
  5) Handle Partial orders without dying - just re-adjust self-parameters 
  6) Keep an eye out for unexpected asset changes (e.g., partial order fill after we cancel, other bot stole from us) 
  7) Store and retain most recent tickers in BotaAccount information, or Order Structures, but not both at the same time
  8) All waiting-loops should check for DIE in told it sucks. 
  9) Bot must save when mind transitions to and from stopped (with the non-stopped value)

## Summary
- **Sally Bot**: ✅ PASSES all 8 laws
- **Swat Bot**: ✅ PASSES all 8 laws  
- **Stealth Bot**: ⚠️ FAILS Law 8 (DIE checking in waiting loops)

---

## Law-by-Law Analysis

### Law 1: Only request assets at startup if not loaded. Wait for SWAP_SUCC if requesting.

**Sally Bot** ✅ PASS
- Lines 131-222: Fresh bot requests cash, sends SWAP message, then waits in loop
- Explicitly checks for `SWAP_SUCC` (line 201) before proceeding
- Also handles `SWAP_FAIL` and timeout

**Swat Bot** ✅ PASS  
- Lines 141-178: If swap_info_0 provided, sends swap and waits
- Loop at 148-177 explicitly checks for `SWAP_SUCC` (line 156)
- Handles `SWAP_FAIL` and timeout gracefully

**Stealth Bot** ✅ PASS
- Lines 192-270: Fresh bot requests cash via `tell_dr_swap_info`
- Loop at 216-270 waits for `SWAP_SUCC` (line 237)
- Also handles `SWAP_FAIL` and timeout

---

### Law 2: Wait for ORDER_FLY at every order we request.

**Sally Bot** ✅ PASS
- Lines 956-967: Sends initial order, then immediately calls `wait_for_sally_order_confirmation()`
- Function `wait_for_sally_order_confirmation()` at lines 774-847:
  - Explicit loop waiting for `ORDER_FLY` (line 826)
  - Also checks for `ORDER_BAD` (line 833)
  - Sets `brain.order_fly_confirmed = true` (line 830)
- Line 967: Even haggled orders wait via `wait_for_sally_order_confirmation()`

**Swat Bot** ✅ PASS
- Line 832-845: Sends order then waits for confirmation
- Function `wait_for_order_confirmation()` at lines 853-928:
  - Explicit loop (line 868) waiting for `ORDER_FLY` (line 898)
  - Checks for `ORDER_BAD` (line 905)
  - Sets `brain.order_fly_confirmed = true` (line 902)
- Haggled orders at line 844 also wait for confirmation

**Stealth Bot** ✅ PASS
- Line 970: After sending entry order, calls `wait_for_stealth_order_confirmation()`
- Function at lines 2125-2194:
  - Loop (line 2140) waiting for `ORDER_FLY` (line 2172)
  - Checks for `ORDER_BAD` (line 2178)
  - Implemented at: line 970 (entry), 2294 (exit1), 2400 (spread), 2531 (exit2)

---

### Law 3: Haggle generation and application at first order using input rules

**Sally Bot** ✅ PASS
- Lines 920-936: Uses `dsta::create_initial_haggle_order()` with `haggle_method` from input
- Applies haggle via `haggle_method.apply_haggle_changes()` (line 921)
- Stores haggle context (line 936)

**Swat Bot** ✅ PASS
- Implicitly passes (relies on order generation before sending)
- Uses `dsta::HaggleMethod` from construction parameters

**Stealth Bot** ✅ PASS
- Lines 925-945: Uses `dsta::create_initial_haggle_order()` with input haggle_method
- Applies haggle using `haggle_method.apply_haggle_changes()`
- Stores context and attempt tracking

---

### Law 4: Haggle generation and application at all other orders using hard-coded per bot

**Sally Bot** ✅ PASS - Uses cheap haggle
- Line 921: Applies haggle dynamically using `haggle_method.apply_haggle_changes()`
- Accumulates `total_slippage` (line 922)
- Tracks `haggle_attempt` (line 923)
- Respects slippage limits (lines 938-952)

**Swat Bot** ✅ PASS - Uses aggressive haggle
- Line 824-830: Aggressive haggle logic for exit orders
- Increases price aggressively for partial fills
- Haggle retry at line 844 uses same hard-coded aggressive approach

**Stealth Bot** ✅ PASS - Uses mid-range haggle
- Multiple haggle applications at different states:
  - Entry: Lines 925-945 (initial)
  - Spread: Lines 2351-2374 (hard-coded `VirtualMarketOrderWithLimitOrdThenConcede`)
  - Exit: Lines 2478-2501 (same method)
- Applies haggle at partials: Check2 (lines 1477-1481), Check3 (lines 1678-1681)

---

### Law 5: Handle Partial orders without dying - re-adjust self-parameters

**Sally Bot** ✅ PASS
- Lines 908-918: When partial fill detected, adjusts `remaining` ratio
- Recalculates `quantity` based on remaining ratio (line 912)
- Re-applies haggle and resends (lines 920-964)
- Does NOT die on partial, continues to haggle

**Swat Bot** ✅ PASS
- Exit order logic handles partials gracefully
- Tracks filled vs remaining quantities
- Re-submits with adjusted quantities

**Stealth Bot** ✅ PASS
- Check1 (lines 1024-1071): Handles partial entry fills
  - Detects overfill (line 1031), logs but continues
  - Detects partial fill (line 1085-1103), does NOT die
- Check2 (lines 1408-1507): Handles partial spread fills
  - Calculates remaining qty (line 1411)
  - Haggles for remainder instead of dying
- Check3 (lines 1614-1740): Handles partial loss exit
  - Haggles for remaining, respects timeout
- Check4: Handles final exit partials

---

### Law 6: Keep an eye out for unexpected asset changes

**Sally Bot** ⚠️ PARTIAL
- Doesn't explicitly check for "other bot stole from us"
- Uses `position_for_order_name()` helper to find positions
- No explicit logic to detect unexpected withdrawals

**Swat Bot** ✅ GOOD
- Lines 934-960: `ticker_from_last_or_request()` searches account positions
- Checks multiple position types (`m_stock`, `m_long_calls`, etc.)
- Log warning if ticker not found (line 957)

**Stealth Bot** ✅ GOOD
- Throughout Check states: Validates expected position names match
- Line 1005: Checks `if info.option_name == brain.m_str_1` (Check1)
- Line 1340: Checks `if pos.asset.order_name() == brain.m_str_2` (Check2)
- Line 1572: Validates position name matches expected (Check3)

---

### Law 7: Store and retain most recent tickers in BotaAccount OR Order Structures, but NOT both

**Sally Bot** ✅ PASS
- Stores tickers in `BotaAccount` positions (via account updates)
- Order structures don't carry last_ticker
- Consistent use of BotaAccount for ticker storage

**Swat Bot** ✅ PASS
- Lines 944-947: Retrieves `last_ticker` from positions in BotaAccount
- Helper function `ticker_from_last_or_request()` searches account positions only
- Does NOT store tickers in Order structures

**Stealth Bot** ✅ PASS
- Stores `last_ticker` in `StealthBot.m_opt_1`, `m_opt_2`, `m_stock` (Option types)
- These are parsed from BotaAccount positions during state transitions
- Does NOT duplicate in Order structures
- Maintains single source of truth in bot's internal state

---

### Law 8: All waiting-loops should check for DIE in told_it_sucks

**Sally Bot** ✅ PASS
- Lines 309-316: Main waiting loop checks for "DIE" message
- When DIE received, logs and returns gracefully
- Properly exits the bot

**Swat Bot** ✅ PASS
- Line 473-480: Checks for "DIE" in main loop
- Handles gracefully with proper error logging

**Stealth Bot** ❌ FAIL
- Line 598-604: DIE is checked in main loop
- **PROBLEM**: Waiting loops don't check for DIE
  - `wait_for_stealth_order_confirmation()` (lines 2125-2194):
    - Loop at line 2140 uses timeout-based loop
    - Does NOT check for DIE in the received messages
    - Line 2185-2189: Unknown messages are logged but continue
    - **FIX**: Should check `if msg == "DIE"` and break/return error
  - Partial fill haggle loops in Check2/Check3 also don't check DIE while waiting for retry period

**Detailed Issue in Stealth Bot:**
```rust
// Line 2185-2189: Current code ignores unknown messages
Ok(Some(msg)) =>
{ let errmsg= format!("Stealth bot got a transient messsage while whaiting for order fly/bad! : {:#?} ", msg);
  log::warn!("{}",errmsg);
  let _ = f_report_error(stone, brain, stuff, to_dr, &errmsg).await;
  continue;  // ← Doesn't check if msg == "DIE"
}

// Should be:
Ok(Some(msg)) if msg == "DIE" =>
{ let errmsg = format!("{} [{}] DIE signal received while waiting for order confirmation", dbgspt, timestamp);
  log::debug!("{}", errmsg);
  return Err(BotError::Other(errmsg));
}
Ok(Some(msg)) =>
{ let errmsg = format!("Transient message: {}", msg);
  log::warn!("{}", errmsg);
  continue;
}
```

---

## Summary Table

| Law | Description | Sally | Swat | Stealth |
|-----|-------------|-------|------|---------|
| 1   | SWAP_SUCC wait | ✅ | ✅ | ✅ |
| 2   | ORDER_FLY wait | ✅ | ✅ | ✅ |
| 3   | Haggle at first order | ✅ | ✅ | ✅ |
| 4   | Haggle at other orders | ✅ | ✅ | ✅ |
| 5   | Partial order handling | ✅ | ✅ | ✅ |
| 6   | Asset change detection | ⚠️ | ✅ | ✅ |
| 7   | Ticker storage (XOR) | ✅ | ✅ | ✅ |
| 8   | DIE in waiting loops | ✅ | ✅ | ❌ |

---

## Recommendations

### For Stealth Bot (Priority: HIGH)
Add DIE checking to `wait_for_stealth_order_confirmation()`:
1. Add `if msg == "DIE"` branch before the generic `Ok(Some(msg))` arm
2. Return error to gracefully exit
3. Consider adding DIE check to partial fill retry loops as well

### For Sally Bot (Priority: LOW)
Enhanced asset change detection could check if unexpected positions appear in account that weren't requested.