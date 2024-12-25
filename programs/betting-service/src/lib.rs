use anchor_lang::prelude::*;
use std::collections::HashMap;

declare_id!("4rwXZbzucJ3oDz9qL5Lp2e53SFLYxdLdqrWAkcoBhPAA");

#[program]
pub mod betting_service {
    use super::*;

    pub fn initialize(ctx: Context<Initialize>, admin: Pubkey) -> Result<()> {
        let state = &mut ctx.accounts.state;
        state.admins.push(admin);
        Ok(())
    }

    pub fn add_admin(ctx: Context<ModifyAdmins>, new_admin: Pubkey) -> Result<()> {
        let state = &mut ctx.accounts.state;

        // Only an existing admin can add a new admin
        require!(
            state.admins.contains(&ctx.accounts.caller.key()),
            ErrorCode::Unauthorized
        );

        state.admins.push(new_admin);
        Ok(())
    }

    pub fn create_market(
        ctx: Context<CreateMarket>,
        market_title: String,
        market_description: String,
        market_type: MarketType,
        tokens: Option<[Pubkey; 2]>,
        opening_date: i64,
        closing_date: i64,
        settlement_date: i64,
        commission_percentage: u8,
        min_bet: u64,
        max_bet: u64,
        max_cumulative_bet: u64,
    ) -> Result<()> {
        let state = &ctx.accounts.state;

        // Only admins can create markets
        require!(
            state.admins.contains(&ctx.accounts.admin.key()),
            ErrorCode::Unauthorized
        );

        let market = &mut ctx.accounts.market;

        market.admin = *ctx.accounts.admin.key;
        market.market_title = market_title;
        market.market_description = market_description;
        market.market_type = market_type;
        market.tokens = tokens;
        market.opening_date = opening_date;
        market.closing_date = closing_date;
        market.settlement_date = settlement_date;
        market.commission_percentage = commission_percentage;
        market.min_bet = min_bet;
        market.max_bet = max_bet;
        market.max_cumulative_bet = max_cumulative_bet;
        market.total_bets = 0;
        market.status = MarketStatus::Opened;
        market.outcomes = vec![];

        Ok(())
    }

    pub fn get_admin_info(ctx: Context<GetAdminInfo>, admin_pubkey: Pubkey) -> Result<AdminInfo> {
        let state = &ctx.accounts.state;

        // Check if the specified admin exists
        require!(
            state.admins.contains(&admin_pubkey),
            ErrorCode::AdminNotFound
        );

        // Retrieve admin-specific information
        let admin_info = AdminInfo {
            admin_pubkey,
            is_active: true, // Placeholder: add logic if you track admin status
        };

        Ok(admin_info)
    }

    pub fn place_bet(ctx: Context<PlaceBet>, outcome_index: u8, amount: u64) -> Result<()> {
        let market = &mut ctx.accounts.market;

        // Validate market status and dates
        require!(
            market.status == MarketStatus::Opened,
            ErrorCode::MarketClosed
        );
        let now = Clock::get()?.unix_timestamp;
        require!(
            now >= market.opening_date && now <= market.closing_date,
            ErrorCode::BettingPeriodOver
        );

        // Validate bet amount
        require!(amount >= market.min_bet, ErrorCode::BetTooSmall);
        require!(amount <= market.max_bet, ErrorCode::BetTooLarge);

        // Update user bet totals and ensure max cumulative bet is not exceeded
        let total_user_bet = market.user_bets.entry(*ctx.accounts.user.key).or_insert(0);
        *total_user_bet += amount;
        require!(
            *total_user_bet <= market.max_cumulative_bet,
            ErrorCode::MaxCumulativeBetExceeded
        );

        // Update outcome bet pool
        if (outcome_index as usize) >= market.outcomes.len() {
            return Err(ErrorCode::InvalidOutcomeIndex.into());
        }
        market.outcomes[outcome_index as usize].total_bets += amount;

        // Transfer SOL from user to market pool
        **ctx.accounts.user.lamports.borrow_mut() -= amount;
        **ctx.accounts.market_pool.lamports.borrow_mut() += amount;

        Ok(())
    }

    pub fn settle_market(ctx: Context<SettleMarket>, winning_outcome: u8) -> Result<()> {
        let market = &mut ctx.accounts.market;

        // Only admins can settle the market
        require!(
            market.admin == *ctx.accounts.admin.key,
            ErrorCode::Unauthorized
        );

        // Validate market status
        require!(
            market.status == MarketStatus::Closed,
            ErrorCode::MarketNotClosed
        );

        // Update market status and set the winning outcome
        market.status = MarketStatus::Settled;
        market.winning_outcome = Some(winning_outcome);

        Ok(())
    }

    pub fn claim_reward(ctx: Context<ClaimReward>) -> Result<()> {
        let market = &mut ctx.accounts.market;
        let user = &ctx.accounts.user;
        let market_pool = &mut ctx.accounts.market_pool;

        // Ensure the market is settled
        require!(
            market.status == MarketStatus::Settled,
            ErrorCode::MarketNotSettled
        );

        // Ensure the user bet on the winning outcome
        let _user_bet = market
            .user_bets
            .get(&user.key())
            .ok_or(ErrorCode::NoBetPlaced)?;
        let winning_outcome = market.winning_outcome.ok_or(ErrorCode::NoWinnerSet)?;
        let outcome = &market.outcomes[winning_outcome as usize];
        let user_bet_on_winning = outcome.user_bets.get(&user.key()).copied().unwrap_or(0);

        require!(user_bet_on_winning > 0, ErrorCode::NotEligibleForReward);

        // Calculate user's reward
        let total_losing_bets: u64 = market
            .outcomes
            .iter()
            .filter(|out| out.index != winning_outcome)
            .map(|out| out.total_bets)
            .sum();

        let user_share =
            user_bet_on_winning as u128 * total_losing_bets as u128 / outcome.total_bets as u128;
        let reward = user_share as u64 + user_bet_on_winning;

        // Deduct commission fee
        let commission = (reward * market.commission_percentage as u64) / 100;
        let net_reward = reward - commission;

        // Transfer rewards
        **market_pool.lamports.borrow_mut() -= reward;
        **ctx.accounts.fee_collector.lamports.borrow_mut() += commission;
        **user.lamports.borrow_mut() += net_reward;

        // Remove user's bet record to prevent double claiming
        market.user_bets.remove(&user.key());

        Ok(())
    }
}

#[account]
pub struct State {
    pub admins: Vec<Pubkey>,
}

#[derive(Accounts)]
pub struct Initialize<'info> {
    #[account(init, payer = payer, space = 8 + 32 * 10)] // Max 10 admins, adjust space as needed
    pub state: Account<'info, State>,
    #[account(mut)]
    pub payer: Signer<'info>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct ModifyAdmins<'info> {
    #[account(mut)]
    pub state: Account<'info, State>,
    #[account(signer)]
    pub caller: Signer<'info>,
}

#[derive(Accounts)]
pub struct CreateMarket<'info> {
    #[account(
        init,
        payer = admin,
        space = 8 + Market::MAX_SIZE
    )]
    pub market: Account<'info, Market>,
    #[account(mut)]
    pub admin: Signer<'info>,
    #[account(mut)]
    pub state: Account<'info, State>,
    #[account(
        mut,
        seeds = [market.key().as_ref(), b"pool"],
        bump
    )]
    pub market_pool: SystemAccount<'info>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct PlaceBet<'info> {
    #[account(mut)]
    pub market: Account<'info, Market>,
    #[account(mut)]
    pub user: Signer<'info>,
    #[account(mut)]
    pub market_pool: SystemAccount<'info>,
}

#[derive(Accounts)]
pub struct SettleMarket<'info> {
    #[account(mut)]
    pub market: Account<'info, Market>,
    #[account(mut)]
    pub admin: Signer<'info>,
}

#[derive(Accounts)]
pub struct ClaimReward<'info> {
    #[account(mut)]
    pub market: Account<'info, Market>,
    #[account(mut)]
    pub user: Signer<'info>,
    #[account(mut)]
    pub market_pool: SystemAccount<'info>,
    #[account(mut)]
    pub fee_collector: SystemAccount<'info>,
}

#[derive(Accounts)]
pub struct GetAdminInfo<'info> {
    pub state: Account<'info, State>,
}

#[derive(Debug, Clone, AnchorSerialize, AnchorDeserialize)]
pub struct AdminInfo {
    pub admin_pubkey: Pubkey,
    pub is_active: bool,
}


#[account]
pub struct Market {
    pub admin: Pubkey,                   // 32 bytes
    pub market_title: String,            // 4 bytes (length) + max length
    pub market_description: String,      // 4 bytes (length) + max length
    pub market_type: MarketType,         // 1 byte
    pub tokens: Option<[Pubkey; 2]>,     // 1 byte (discriminator) + 64 bytes (2 Pubkeys)
    pub opening_date: i64,               // 8 bytes
    pub closing_date: i64,               // 8 bytes
    pub settlement_date: i64,            // 8 bytes
    pub commission_percentage: u8,       // 1 byte
    pub min_bet: u64,                    // 8 bytes
    pub max_bet: u64,                    // 8 bytes
    pub max_cumulative_bet: u64,         // 8 bytes
    pub total_bets: u64,                 // 8 bytes
    pub status: MarketStatus,            // 1 byte
    pub winning_outcome: Option<u8>,     // 1 byte (discriminator) + 1 byte
    pub outcomes: Vec<Outcome>,          // 4 bytes (length) + max outcomes * outcome size
    pub user_bets: HashMap<Pubkey, u64>, // 4 bytes (length) + max bets * entry size
}

impl Market {
    pub const MAX_TITLE_LENGTH: usize = 64; // Example max title length
    pub const MAX_DESC_LENGTH: usize = 256; // Example max description length
    pub const MAX_OUTCOMES: usize = 4; // Maximum number of outcomes
    pub const MAX_BETS: usize = 100; // Maximum number of user bets

    pub const MAX_SIZE: usize = 32    // admin
        + 4 + Self::MAX_TITLE_LENGTH  // market_title
        + 4 + Self::MAX_DESC_LENGTH   // market_description
        + 1                           // market_type
        + 1 + 64                      // tokens (Option<[Pubkey; 2]>)
        + 8 + 8 + 8                   // opening_date, closing_date, settlement_date
        + 1                           // commission_percentage
        + 8 + 8 + 8                   // min_bet, max_bet, max_cumulative_bet
        + 8                           // total_bets
        + 1                           // status
        + 1 + 1                       // winning_outcome (Option<u8>)
        + 4 + (Self::MAX_OUTCOMES * Outcome::MAX_SIZE) // outcomes
        + 4 + (Self::MAX_BETS * (32 + 8)); // user_bets (Pubkey + u64)
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone)]
pub struct Outcome {
    pub index: u8,                       // 1 byte
    pub total_bets: u64,                 // 8 bytes
    pub user_bets: HashMap<Pubkey, u64>, // 4 + max_entries * entry_size
}

impl Outcome {
    pub const MAX_USER_BETS: usize = 100; // Example max user bets per outcome
    pub const MAX_SIZE: usize = 1        // index
        + 8                              // total_bets
        + 4 + (Self::MAX_USER_BETS * (32 + 8)); // user_bets (Pubkey + u64)
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, PartialEq, Eq)]
pub enum MarketStatus {
    Opened,
    Closed,
    Settled,
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, PartialEq, Eq)]
pub enum MarketType {
    Hilo,
    TokenFight,
    Custom,
}

#[error_code]
pub enum ErrorCode {
    #[msg("Market is already closed.")]
    MarketClosed,
    #[msg("Betting period is over.")]
    BettingPeriodOver,
    #[msg("Bet amount is too small.")]
    BetTooSmall,
    #[msg("Bet amount is too large.")]
    BetTooLarge,
    #[msg("Maximum cumulative bet exceeded.")]
    MaxCumulativeBetExceeded,
    #[msg("Invalid outcome index.")]
    InvalidOutcomeIndex,
    #[msg("Unauthorized.")]
    Unauthorized,
    #[msg("Market is not closed.")]
    MarketNotClosed,
    #[msg("Market is not settled.")]
    MarketNotSettled,
    #[msg("No bet placed.")]
    NoBetPlaced,
    #[msg("No winner set.")]
    NoWinnerSet,
    #[msg("Not eligible for reward.")]
    NotEligibleForReward,
    #[msg("Admin not found.")]
    AdminNotFound,
}
