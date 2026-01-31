use anchor_lang::prelude::*;

declare_id!("9VyazGUeXCWsvgb1nqB8Vj52cwmNKHaMZC9fdX7vh18U");

#[program]
pub mod fixed {
    use super::*;

    pub fn initialize(ctx: Context<Initialize>) -> Result<()> {
        let data = &mut ctx.accounts.data;

        // Fix: prevent reinitialization
        if data.created_at != 0 {
            return err!(CustomError::AlreadyInitialized);
        }

        data.created_at = Clock::get()?.unix_timestamp;
        data.points = 0;
        msg!("Initialized (one-time only): created_at = {}, points = 0", data.created_at);
        Ok(())
    }

    pub fn claim_rewards(ctx: Context<Claim>) -> Result<()> {
        let data = &mut ctx.accounts.data;
        let now = Clock::get()?.unix_timestamp;
        let elapsed = now - data.created_at;
        let reward = elapsed as u64 * 100;
        data.points += reward;
        msg!("Claimed {} points", reward);
        Ok(())
    }
}

#[derive(Accounts)]
pub struct Initialize<'info> {
    #[account(
        init_if_needed,
        payer = user,
        space = 8 + 8 + 8,
        seeds = [b"rewards", user.key().as_ref()],
        bump,
    )]
    pub data: Account<'info, RewardData>,
    #[account(mut)]
    pub user: Signer<'info>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct Claim<'info> {
    #[account(mut)]
    pub data: Account<'info, RewardData>,
    pub user: Signer<'info>,
}

#[account]
pub struct RewardData {
    pub created_at: i64,
    pub points: u64,
}

#[error_code]
pub enum CustomError {
    #[msg("Account already initialized - cannot reinitialize")]
    AlreadyInitialized,
}
