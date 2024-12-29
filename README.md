# Betting Contract System

## Overview

This contract system allows administrators to create betting markets, manage users' bets, and settle the outcomes of markets. It supports the following features:
- Administrators can manage users and create new betting markets.
- Users can place bets on various outcomes of the market.
- Market administrators can close and settle markets, determining winners.
- Users can claim rewards based on the outcome of the market.

## Functions

### 1. `initialize`

**Purpose:** Initializes the contract state and adds the first administrator.

**Parameters:**
- `admin: Pubkey` — The public key of the first administrator.

**Action:** 
- Creates the state account.
- Adds the first administrator to the list of administrators.

### 2. `add_admin`

**Purpose:** Adds a new administrator to the system.

**Parameters:**
- `new_admin: Pubkey` — The public key of the new administrator.

**Action:**
- Ensures that the caller is an existing administrator.
- Adds the new administrator to the list.

### 3. `create_market`

**Purpose:** Creates a new betting market.

**Parameters:**
- Various details about the market, including:
  - `title: String` — The title of the market.
  - `description: String` — A description of the market.
  - `token: Option<[Pubkey; 2]>` — Optional tokens for betting.
  - `opening_date: i64`, `closing_date: i64` — The opening and closing dates for the market.
  - `commission_percentage: u8` — The commission for the market.
  - `min_bet: u64`, `max_bet: u64`, `max_cumulative_bet: u64` — The betting limits.

**Action:**
- Ensures the caller is an administrator.
- Creates a new market and adds it to the state.

### 4. `get_market_info`

**Purpose:** Retrieves information about a specific market.

**Parameters:**
- `market_pubkey: Pubkey` — The public key of the market.

**Action:**
- Ensures the market exists.
- Returns the full details of the market.

### 5. `get_admin_info`

**Purpose:** Retrieves information about an administrator.

**Parameters:**
- `admin_pubkey: Pubkey` — The public key of the administrator.

**Action:**
- Ensures the administrator exists.
- Returns the administrator's status.

### 6. `place_bet`

**Purpose:** Allows a user to place a bet on a market outcome.

**Parameters:**
- `outcome_index: u8` — The index of the outcome to bet on.
- `amount: u64` — The amount to bet.

**Action:**
- Ensures the market is open for betting and the current date is within the betting period.
- Ensures the bet amount is within the defined limits.
- Updates the market and user data, including the total bet amount.

### 7. `get_user_info`

**Purpose:** Retrieves information about a user’s bets.

**Parameters:**
- `user_address: Pubkey` — The public key of the user.

**Action:**
- Ensures the user exists.
- Returns the user’s betting history, total bets, and total amount bet.

### 8. `settle_market`

**Purpose:** Settles a market by determining the winning outcome.

**Parameters:**
- `winning_outcome: u8` — The index of the winning outcome.

**Action:**
- Ensures the caller is the market administrator.
- Sets the market’s status as settled and records the winning outcome.

### 9. `claim_reward`

**Purpose:** Allows a user to claim their winnings after a market has been settled.

**Action:**
- Ensures the market has been settled.
- Verifies the user’s bet is on the winning outcome.
- Calculates the reward, deducting the commission, and transfers it to the user.

## Structures

### 1. `State`

**Description:** Stores the global state of the contract.

**Fields:**
- `admins: Vec<Pubkey>` — A list of administrators.
- `markets: Vec<Pubkey>` — A list of market public keys.

### 2. `Market`

**Description:** Represents a betting market.

**Fields:**
- `admin: Pubkey` — The administrator of the market.
- `market_title: String` — The market's title.
- `market_description: String` — The market's description.
- `tokens: Option<[Pubkey; 2]>` — Optional tokens for betting.
- `opening_date: i64`, `closing_date: i64`, `settlement_date: i64` — Market dates.
- `commission_percentage: u8` — The market's commission rate.
- `min_bet: u64`, `max_bet: u64`, `max_cumulative_bet: u64` — Betting limits.
- `total_bets: u64` — The total amount bet on the market.
- `status: MarketStatus` — The current status of the market.
- `outcomes: Vec<Outcome>` — The list of possible outcomes.

### 3. `User`

**Description:** Stores user data.

**Fields:**
- `address: Pubkey` — The user's public key.
- `total_bets: u64` — The number of bets placed.
- `total_amount_bet: u64` — The total amount the user has bet.
- `bets: Vec<UserBet>` — The list of bets placed by the user.

### 4. `MarketStatus`

**Description:** The status of a market.

Possible values:
- `Open`
- `Closed`
- `Settled`

### 5. `Outcome`

**Description:** Represents an outcome in the market.

**Fields:**
- `index: u8` — The outcome’s index.
- `total_bets: u64` — The total amount of bets placed on this outcome.

### 6. `UserBet`

**Description:** Information about a user's bet.

**Fields:**
- `amount: u64` — The amount of the bet.
- `outcome_index: u8` — The index of the outcome on which the bet was placed.

## Errors

The contract uses the following error codes:

- `Unauthorized` — The caller does not have the required permissions.
- `MarketNotFound` — The requested market does not exist.
- `MarketClosed` — The market is closed for betting.
- `BettingPeriodOver` — The betting period has ended.
- `BetTooSmall` — The bet is below the minimum allowed amount.
- `BetTooLarge` — The bet exceeds the maximum allowed amount.
- `MaxCumulativeBetExceeded` — The user’s total bets exceed the allowed limit.
- `NoBetPlaced` — No bet was placed by the user.
- `NoWinnerSet` — No winner has been set for the market.
- `NotEligibleForReward` — The user is not eligible for a reward.


