# Betting Service Contract - User Guide

## Accounts

### State Account
- Stores global settings and the list of admins.
- Address is derived using a program-specific seed.

### Market Account
- Represents a betting market.
- Contains details like title, description, dates, outcomes, and betting constraints.

### User Account (Optional)
- Used for tracking user-specific data (e.g., cumulative bets).
- Not explicitly shown in the provided code but often required for user-centric logic.

---

## Functions

### 1. Initialize
**Purpose:** Set up the program state and assign the first admin.

**Parameters:**
- `admin (Pubkey)`: The public key of the initial admin.

**Accounts:**
- `state`: Writable. The global state account.
- `caller`: Signer. The admin initializing the program.

**Usage:**  
Call this function after deploying the program. Only the first admin is assigned during initialization.

---

### 2. Add Admin
**Purpose:** Allow an existing admin to add a new admin.

**Parameters:**
- `new_admin (Pubkey)`: The public key of the new admin.

**Accounts:**
- `state`: Writable. Contains the list of admins.
- `caller`: Signer. Must be an existing admin.

**Usage:**  
Admins can call this function to expand the admin list and manage the program collaboratively.

---

### 3. Create Market
**Purpose:** Create a new betting market with defined constraints and details.

**Parameters:**
- `market_title (String)`: Name of the market.
- `market_description (String)`: Description of the market.
- `market_type (MarketType)`: Enum defining the type (e.g., binary, multi-outcome).
- `tokens (Option<[Pubkey; 2]>)`: Optional pair of tokens for betting.
- `opening_date (i64)`: Unix timestamp when betting opens.
- `closing_date (i64)`: Unix timestamp when betting closes.
- `settlement_date (i64)`: Unix timestamp when results are finalized.
- `commission_percentage (u8)`: Admin commission percentage.
- `min_bet (u64)`: Minimum bet amount.
- `max_bet (u64)`: Maximum bet amount.
- `max_cumulative_bet (u64)`: Maximum total bets allowed in the market.

**Accounts:**
- `state`: Read-only. Verifies the caller is an admin.
- `market`: Writable. The newly created market account.
- `admin`: Signer. The admin creating the market.

**Usage:**  
Admins use this function to launch new markets. Ensure dates and constraints are valid.

---

### 4. Get Admin Info
**Purpose:** Fetch information about an admin.

**Parameters:**
- `admin_pubkey (Pubkey)`: Public key of the admin to query.

**Accounts:**
- `state`: Read-only. Stores the admin list.

**Returns:**
- `AdminInfo (Struct)`: Contains `admin_pubkey` and `is_active` status.

**Usage:**  
This function helps identify if a given public key is an admin.

---

### 5. Place Bet
**Purpose:** Place a bet on a specific outcome in an open market.

**Parameters:**
- `outcome_index (u8)`: Index of the selected outcome.
- `amount (u64)`: Amount to bet.

**Accounts:**
- `market`: Writable. The market being bet on.
- `bettor`: Signer. The user placing the bet.

**Usage:**  
Users call this function during the market's open period to place bets. Ensure the bet amount satisfies the constraints.
