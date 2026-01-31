use anchor_lang::prelude::*;

declare_id!("3tVrEDczqBhY2uHvVVT9QZ88G7k19DVTo7wTZC5LC81B");

#[program]
pub mod vulnerable {
    use super::*;

    pub fn initialize(ctx: Context<Initialize>) -> Result<()> {
        let data = &mut ctx.accounts.data;
        // Critical: created_at is used for reward calculation (e.g., time-based accrual)
        data.created_at = Clock::get()?.unix_timestamp;
        data.points = 0;
        msg!("Initialized: created_at = {}, points = 0", data.created_at);
        Ok(())
    }

    pub fn claim_rewards(ctx: Context<Claim>) -> Result<()> {
        let data = &mut ctx.accounts.data;
        let now = Clock::get()?.unix_timestamp;

        let elapsed = (now - data.created_at) as u64;
        let new_rewards = elapsed * 100; // rate: 100 points per second
        data.points = data.points.checked_add(new_rewards).unwrap();

        msg!(
            "Claimed {} new points. Total points now: {}",
            new_rewards,
            data.points
        );

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
